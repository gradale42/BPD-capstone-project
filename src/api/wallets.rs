use actix_web::{web, HttpResponse, Responder};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::api::AppState;

#[derive(Debug, serde::Serialize)]
pub struct WalletInfo {
    pub name: String,
    pub balance: f64,
    pub address_count: usize,
    pub addresses: Vec<AddressInfo>,
}

#[derive(Debug, serde::Serialize)]
pub struct AddressInfo {
    pub address: String,
    pub balance: f64,
    pub label: Option<String>,
    pub received: f64,
    pub spent: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct DescriptorInfo {
    pub descriptor: String,
    pub active: bool,
    pub range: Option<Vec<u64>>,
    pub timestamp: u64,
}

pub async fn list_wallets(state: web::Data<AppState>) -> impl Responder {
    match Client::new(&state.rpc_url, state.rpc_auth.clone()) {
        Ok(client) => {
            match client.list_wallets() {
                Ok(wallets) => {
                    let mut wallet_infos = Vec::new();

                    for wallet_name in wallets {
                        let wallet_url = format!("{}/wallet/{}", state.rpc_url, wallet_name);
                        if let Ok(wallet_client) = Client::new(&wallet_url, state.rpc_auth.clone()) {
                            match get_wallet_details(&wallet_client, &wallet_name) {
                                Ok(info) => wallet_infos.push(info),
                                Err(e) => {
                                    eprintln!("Error getting details for wallet {}: {}", wallet_name, e);
                                    wallet_infos.push(WalletInfo {
                                        name: wallet_name,
                                        balance: 0.0,
                                        address_count: 0,
                                        addresses: vec![],
                                    });
                                }
                            }
                        } else {
                            wallet_infos.push(WalletInfo {
                                name: wallet_name,
                                balance: 0.0,
                                address_count: 0,
                                addresses: vec![],
                            });
                        }
                    }

                    HttpResponse::Ok().json(json!({
                        "status": "success",
                        "wallets": wallet_infos
                    }))
                }
                Err(e) => HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Failed to list wallets: {}", e)
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Failed to connect to RPC: {}", e)
        }))
    }
}

pub async fn get_wallet_details_handler(
    state: web::Data<AppState>,
    wallet_name: web::Path<String>,
) -> impl Responder {
    let wallet_url = format!("{}/wallet/{}", state.rpc_url, wallet_name);

    match Client::new(&wallet_url, state.rpc_auth.clone()) {
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
    let wallet_url = format!("{}/wallet/{}", state.rpc_url, wallet_name);

    match Client::new(&wallet_url, state.rpc_auth.clone()) {
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