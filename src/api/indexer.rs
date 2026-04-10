use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use crate::AppState;
use crate::services::bitcoin::rpc::get_last_block_height as get_last_block_from_rpc;
use crate::services::ExecutionCtx;

pub async fn get_indexer_stats(state: web::Data<AppState>) -> impl Responder {
    let last_block_live = match get_last_block_from_rpc(&state.clone()).await {
        Ok(height) => height,
        Err(e) => {
            eprintln!("RPC error: {}", e);
            -1
        }
    };


    let network = state.node_manager.get_current_network();
    let mut ctx = match ExecutionCtx::new(&state.db_pool, network).await {
        Ok(context) => context,
        Err(e) => {
            eprintln!("Failed to acquire DB connection: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Database error: {}", e)
            }));
        }
    };

    let last_block_index = match state.block_service.get_last_block_height(&mut ctx).await {
        Ok(Some(height)) => height,
        Ok(None) => -1,
        Err(e) => {
            eprintln!("DB error: {}", e);
            -1
        }
    };

    HttpResponse::Ok().json(json!({
        "last_block_live": last_block_live,
        "last_block_index": last_block_index,
    }))
}