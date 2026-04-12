use actix_web::{get, web, HttpResponse, Responder};
use bitcoincore_rpc::RpcApi;
use serde_json::json;
use std::fs;
use crate::api::bitcoin_rpc;
use crate::AppState;

#[derive(Debug, serde::Serialize)]
pub struct DashboardStats {
    pub last_block: u64,
    pub mempool_count: usize,
    pub peer_count: usize,
    pub hashrate: f64,
}

pub async fn get_stats(state: web::Data<AppState>) -> impl Responder {
    bitcoin_rpc(&state.node_manager,"", |client| {
        let blockchain_info = client.get_blockchain_info()?;
        let mempool_info = client.get_mempool_info()?;
        let peers = client.get_peer_info()?;

        let stats = DashboardStats {
            last_block: blockchain_info.blocks,
            mempool_count: mempool_info.size,
            peer_count: peers.len(),
            hashrate: blockchain_info.difficulty as f64,
        };

        Ok(stats)
    }).await
}