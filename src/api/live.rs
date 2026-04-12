use actix_web::{web, HttpResponse, Responder};
use futures::TryFutureExt;
use serde_json::json;
use crate::api::bitcoin_rpc;
use crate::AppState;
use crate::bitcoin::rpc::{get_mempool_tx_count, get_peer_count, get_network_hashrate};

pub async fn get_live_stats(state: web::Data<AppState>) -> impl Responder {
    bitcoin_rpc(&state.node_manager, "", |client| {
        let mempool_count = get_mempool_tx_count(client).unwrap_or(0);
        let peer_count = get_peer_count(client).unwrap_or(0);
        let hashrate = get_network_hashrate(client).unwrap_or(0.0);

        dbg!(mempool_count);
        dbg!(peer_count);
        dbg!(hashrate);


       Ok(json!({
            "mempool_count": mempool_count,
            "peer_count": peer_count,
            "hashrate": hashrate as f64,
        }))
    }).await
}