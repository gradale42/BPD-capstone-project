use actix_web::{get, web, HttpResponse, Responder};
use bitcoincore_rpc::RpcApi;
use serde_json::json;
use std::fs;
use bpd_capstone_project::AppState;

#[derive(Debug, serde::Serialize)]
pub struct DashboardStats {
    pub last_block: u64,
    pub mempool_count: usize,
    pub peer_count: usize,
    pub hashrate: f64,
}

pub async fn get_mock_stats() -> impl Responder {
    match fs::read_to_string("resources/stats.json") {
        Ok(content) => HttpResponse::Ok().json(json!(content)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error reading file: {}", e)),
    }
}

pub async fn get_stats(state: web::Data<AppState>) -> impl Responder {
    match state.get_default_bitcoin_client().lock() {
        Ok(client) => {

            let blockchain_info = match client.get_blockchain_info() {
                Ok(info) => info,
                Err(e) => {
                    eprintln!("Error getting blockchain info: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to get blockchain info: {}", e)
                    }));
                }
            };

            let mempool_info = match client.get_mempool_info() {
                Ok(info) => info,
                Err(e) => {
                    eprintln!("Error getting mempool info: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to get mempool info: {}", e)
                    }));
                }
            };

            let peers = match client.get_peer_info() {
                Ok(peers) => peers,
                Err(e) => {
                    eprintln!("Error getting peer info: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to get peer info: {}", e)
                    }));
                }
            };

            let stats = DashboardStats {
                last_block: blockchain_info.blocks,
                mempool_count: mempool_info.size as usize,
                peer_count: peers.len(),
                hashrate: 0.0 //blockchain_info.networkhashps as f64 / 1e18,
            };

            HttpResponse::Ok().json(stats)
        }
        Err(e) => {
            eprintln!("Error locking RPC client: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to lock RPC client"
            }))
        }
    }
}