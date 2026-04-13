use crate::api::blocks::DataTableResponse;
use crate::api::wrap_response;
use crate::bitcoin::rpc::{
    get_blocks_info, get_mempool_tx_count, get_network_hashrate, get_peer_count,
    import_descriptors, setup_mining_address,
};
use crate::services::ExecutionCtx;
use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
use bitcoincore_rpc::RpcApi;
use serde_json::json;
use std::error::Error;

#[derive(Debug, serde::Deserialize)]
pub struct MineParams {
    pub count: u32,
}

pub async fn import_descriptors_handler(state: web::Data<AppState>) -> impl Responder {
    let node_manager = &state.node_manager;
    let mut service_call = async move || -> Result<_, String> {
        let rpc_result = node_manager
            .execute_rpc("student_wallet", move |client| {
                import_descriptors(client);
                Ok(())
            })
            .await
            .map_err(|e| format!("RPC failed: {}", e));
        rpc_result
    };
    wrap_response(service_call().await)
}

pub async fn mine_blocks_handler(
    state: web::Data<AppState>, query: web::Query<MineParams>,
) -> impl Responder {
    let count = query.count;
    if count == 0 {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Count must be positive"
        }));
    }

    let node_manager = &state.node_manager;
    let mut service_call = async move || -> Result<_, String> {
        let rpc_result = node_manager
            .execute_rpc("", move |client| {
                let mining_address = setup_mining_address(client).unwrap();
                let block_hashes =
                    client.generate_to_address(count as u64, &mining_address).unwrap();
                let res = format!("Mined blocks count: {}", block_hashes.len());
                println!("\n=== {} ===", res);
                println!("\n=== Done ===");
                Ok(res)
            })
            .await
            .map_err(|e| format!("RPC failed: {}", e))?;

        Ok(rpc_result)
    };

    wrap_response(service_call().await)
}
pub async fn save_blocks(state: web::Data<AppState>, mut ctx: ExecutionCtx) -> impl Responder {
    const DEFAULT_COUNT: u64 = 500;

    let node_manager = &state.node_manager;
    let block_service = &state.block_service;
    let network = state.node_manager.get_current_network();

    let mut service_call = async move || -> Result<_, String> {
        let rpc_result = node_manager
            .execute_rpc("", move |client| {
                let blocks =
                    get_blocks_info(client, network, Some(DEFAULT_COUNT)).unwrap_or_default();
                Ok(blocks)
            })
            .await
            .map_err(|e| format!("RPC failed: {}", e))?;

        let db_result = block_service
            .save_blocks_ignore_duplicates(&mut ctx, rpc_result)
            .await
            .map_err(|e| format!("DB failed: {}", e))?;

        Ok(db_result)
    };

    wrap_response(service_call().await)
}
