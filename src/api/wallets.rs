use crate::domain::address::{AddressInfo, DescriptorInfo};
use crate::domain::wallet::WalletInfo;
use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
use bitcoincore_rpc::{Client, RpcApi};
use serde_json::{json, Value};

pub async fn list_wallets(state: web::Data<AppState>) -> impl Responder {
    // 1. Get the list of names using the default client
    let names_result = web::block({
        let state = state.clone();
        move || {
            let self1 = &state.node_manager;
            let client_arc = self1.get_default_current_client();
            let client = client_arc.lock().map_err(|_| "Lock error".to_string())?;
            // Convert bitcoincore_rpc::Error to String immediately
            client.list_wallets().map_err(|e| e.to_string())
        }
    }).await;

    let wallet_names = match names_result {
        Ok(Ok(names)) => names,
        _ => return HttpResponse::InternalServerError().json(json!({"error": "Failed to list wallets"})),
    };

    // 2. Fetch details for each wallet
    let mut wallet_infos = Vec::new();

    for name in wallet_names {
        let name_for_err = name.clone();
        let state_clone = state.clone();

        let info = web::block(move || {
            let self1 = &state_clone.node_manager;
            let client_arc = self1.get_current_client(&name);
            let client = client_arc.lock().map_err(|_| "Lock error".to_string())?;

            get_wallet_details(&client, &name).map_err(|e| e.to_string())
        })
        .await
        .map(|res| res.unwrap_or_else(|e| {
            eprintln!("Error for wallet {}: {}", name_for_err, e);
            WalletInfo::default_with_name(name_for_err.clone())
        }))
        .unwrap_or_else(|_| WalletInfo::default_with_name(name_for_err));

        wallet_infos.push(info);
    }

    HttpResponse::Ok().json(json!({
        "status": "success",
        "wallets": wallet_infos
    }))
}



pub async fn get_wallet_details_handler(
    state: web::Data<AppState>,
    wallet_name: web::Path<String>,
) -> impl Responder {
    let self1 = &state.node_manager;
    match self1.get_current_client(&wallet_name).lock() {
        Ok(wallet_client) => {
            match get_wallet_details(&wallet_client, &wallet_name) {
                Ok(info) => HttpResponse::Ok().json(json!({
                    "status": "success",
                    "wallet": info
                })),
                Err(e) => HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Failed to get wallet details: {}", e)
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Failed to connect to wallet: {}", e)
        }))
    }
}

pub async fn get_descriptors_handler(
    state: web::Data<AppState>,
    wallet_name: web::Path<String>,
) -> impl Responder {
    let self1 = &state.node_manager;
    match self1.get_current_client(&wallet_name).lock() {
        Ok(wallet_client) => {
            let params: Vec<Value> = vec![json!(false)];

            match wallet_client.call::<Value>("listdescriptors", &params) {
                Ok(response) => {
                    if let Some(descriptors_array) = response.get("descriptors").and_then(|d| d.as_array()) {
                        let descriptor_infos: Vec<DescriptorInfo> = descriptors_array
                            .iter()
                            .filter_map(|desc| {
                                let descriptor = desc.get("desc")
                                    .and_then(|d| d.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                if descriptor.is_empty() {
                                    return None;
                                }

                                let active = desc.get("active")
                                    .and_then(|a| a.as_bool())
                                    .unwrap_or(false);

                                let timestamp = desc.get("timestamp")
                                    .and_then(|t| t.as_u64())
                                    .unwrap_or(0);

                                let range = desc.get("range")
                                    .and_then(|r| r.as_array())
                                    .map(|arr| arr.iter()
                                        .filter_map(|v| v.as_u64())
                                        .collect::<Vec<u64>>());

                                Some(DescriptorInfo {
                                    descriptor,
                                    active,
                                    range,
                                    timestamp,
                                })
                            })
                            .collect();

                        HttpResponse::Ok().json(json!({
                            "status": "success",
                            "descriptors": descriptor_infos
                        }))
                    } else {
                        HttpResponse::Ok().json(json!({
                            "status": "success",
                            "descriptors": []
                        }))
                    }
                }
                Err(e) => {
                    eprintln!("Error calling listdescriptors: {}", e);
                    HttpResponse::Ok().json(json!({
                        "status": "success",
                        "descriptors": [],
                        "message": "No descriptors found or wallet doesn't support descriptors"
                    }))
                }
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Failed to connect to wallet: {}", e)
        }))
    }
}

fn get_wallet_details(wallet_client: &Client, name: &str) -> Result<WalletInfo, Box<dyn std::error::Error>> {
    let balance = wallet_client.get_balance(None, None)?.to_btc();
    let addresses = get_wallet_addresses(wallet_client)?;

    Ok(WalletInfo {
        name: name.to_string(),
        balance,
        address_count: addresses.len(),
        addresses,
    })
}

fn get_wallet_addresses(wallet_client: &Client) -> Result<Vec<AddressInfo>, Box<dyn std::error::Error>> {
    let mut addresses = Vec::new();

    //  list_unspent for addresses with balance
    match wallet_client.list_unspent(None, None, None, None, None) {
        Ok(unspent) => {
            use std::collections::HashMap;
            let mut address_map: HashMap<String, f64> = HashMap::new();

            for utxo in unspent {
                if let Some(address) = utxo.address {
                    let address_str = format!("{:?}", address)
                        .trim_matches('"')
                        .to_string();

                    *address_map.entry(address_str).or_insert(0.0) += utxo.amount.to_btc();
                }
            }

            for (address, balance) in address_map {
                addresses.push(AddressInfo {
                    address,
                    balance,
                    label: None,
                    received: balance,
                    spent: 0.0,
                });
            }
        }
        Err(e) => {
            eprintln!("Error getting unspent outputs: {}", e);
        }
    }

    // at least one address if no UTXO 
    if addresses.is_empty() {
        match wallet_client.get_new_address(None, None) {
            Ok(address) => {
                let address_str = format!("{:?}", address)
                    .trim_matches('"')
                    .to_string();

                addresses.push(AddressInfo {
                    address: address_str,
                    balance: 0.0,
                    label: None,
                    received: 0.0,
                    spent: 0.0,
                });
            }
            Err(e) => {
                eprintln!("Error getting new address: {}", e);
            }
        }
    }

    Ok(addresses)
}