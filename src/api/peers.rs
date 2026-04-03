use actix_web::{web, HttpResponse, Responder};
use bitcoincore_rpc::RpcApi;
use serde_json::json;
use std::fs;
use crate::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct PeersParams {
    pub draw: i32,
    pub start: Option<i32>,
    pub length: Option<i32>,
}

#[derive(Debug, serde::Serialize)]
pub struct PeerInfo {
    pub peer_id: u64,
    pub inbound: bool,
    pub subver: String,
    pub version: u64,
    pub bytes_sent_total: u64,
    pub bytes_recv_total: u64,
    pub bytes_sent_delta: u64,
    pub bytes_recv_delta: u64,
    pub ping: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct DataTableResponse<T> {
    pub draw: i32,
    pub records_total: usize,
    pub records_filtered: usize,
    pub data: Vec<T>,
}

pub async fn get_mock_peers(web::Query(_params): web::Query<serde_json::Value>) -> impl Responder {
    match fs::read_to_string("resources/peers.json") {
        Ok(content) => HttpResponse::Ok().json(json!(content)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error reading file: {}", e)),
    }
}

pub async fn get_peers(
    state: web::Data<AppState>,
    web::Query(params): web::Query<PeersParams>,
) -> impl Responder {
    match state.get_default_bitcoin_client().lock() {
        Ok(client) => {
            
            let peers_raw = match client.get_peer_info() {
                Ok(peers) => peers,
                Err(e) => {
                    eprintln!("Error getting peer info: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to get peer info: {}", e)
                    }));
                }
            };

            let mut peers = Vec::new();
            for peer in peers_raw {
                peers.push(PeerInfo {
                    peer_id: peer.id,
                    inbound: peer.inbound,
                    subver: peer.subver.clone(),
                    version: peer.version,
                    bytes_sent_total: peer.bytessent,
                    bytes_recv_total: peer.bytesrecv,
                    bytes_sent_delta: 0,
                    bytes_recv_delta: 0,
                    ping: peer.pingtime.unwrap_or(0.0) * 1000.0, // Convert to ms
                });
            }

            let response = DataTableResponse {
                draw: params.draw,
                records_total: peers.len(),
                records_filtered: peers.len(),
                data: peers,
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error locking RPC client: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to lock RPC client: {}", e)
            }))
        }
    }
}