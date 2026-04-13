use crate::api::wrap_response;
use crate::bitcoin::rpc::{
    get_mempool_tx_count, get_network_hashrate, get_peer_count,
};
use crate::AppState;
use actix_web::{web, Responder};
use serde_json::json;

pub async fn get_live_stats(state: web::Data<AppState>) -> impl Responder {
    let node_manager = &state.node_manager;

    let service_call = async move || -> Result<_, String> {
        let rpc_result = node_manager
            .execute_rpc("", move |client| {
                let mempool_count = get_mempool_tx_count(client).unwrap_or(0);
                let peer_count = get_peer_count(client).unwrap_or(0);
                let hashrate = get_network_hashrate(client).unwrap_or(0.0);

                Ok(json!({
                    "mempool_count": mempool_count,
                    "peer_count": peer_count,
                    "hashrate": hashrate as f64,
                }))
            })
            .await
            .map_err(|e| format!("RPC failed: {}", e))?;

        Ok(rpc_result)
    };

    wrap_response(service_call().await)
}
