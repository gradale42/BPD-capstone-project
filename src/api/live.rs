use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use crate::AppState;
use crate::bitcoin::rpc::{get_mempool_tx_count, get_peer_count, get_network_hashrate};

pub async fn get_live_stats(state: web::Data<AppState>) -> impl Responder {
    let mempool_count = get_mempool_tx_count(&state.clone()).await.unwrap_or(0);
    let peer_count = get_peer_count(&state.clone()).await.unwrap_or(0);
    let hashrate = get_network_hashrate(&state.clone()).await.unwrap_or(0.0);

    HttpResponse::Ok().json(json!({
        "mempool_count": mempool_count,
        "peer_count": peer_count,
        "hashrate": hashrate,
    }))
}