use actix_web::{web, HttpResponse, Responder};
use bitcoincore_rpc::RpcApi;
use serde::{Deserialize, Serialize};
use crate::configuration::Network;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SwitchNetworkRequest {
    pub network: String,
}

#[derive(Debug, Serialize)]
pub struct NetworkInfo {
    pub current_network: String,
    pub available_networks: Vec<String>,
}

pub async fn switch_network(
    state: web::Data<AppState>,
    req: web::Json<SwitchNetworkRequest>,
) -> impl Responder {
    match Network::from_str(&req.network) {
        Some(network) => {
            state.node_manager.set_current_network(network);
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": format!("Switched to {} network", network),
                "network": network.as_str()
            }))
        }
        None => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": format!("Invalid network: {}. Available: regtest, testnet, mainnet", req.network)
            }))
        }
    }
}

pub async fn get_current_network(state: web::Data<AppState>) -> impl Responder {
    let current = state.node_manager.get_current_network();

    HttpResponse::Ok().json(NetworkInfo {
        current_network: current.as_str().to_string(),
        available_networks: vec![
            "regtest".to_string(),
            "testnet".to_string(),
            "mainnet".to_string(),
        ],
    })
}

pub async fn get_network_info(state: web::Data<AppState>) -> impl Responder {
    let current = state.node_manager.get_current_network();

    let blockchain_info = match state.node_manager.execute_rpc("", |client| {
        client.get_blockchain_info()
    }).await {
        Ok(info) => Some(serde_json::json!({
            "blocks": info.blocks,
            "headers": info.headers,
            "chain": info.chain,
            "difficulty": info.difficulty,
        })),
        Err(e) => {
            eprintln!("Failed to get blockchain info: {}", e);
            None
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "current_network": current.as_str(),
        "available_networks": ["regtest", "testnet", "mainnet"],
        "blockchain_info": blockchain_info,
    }))
}