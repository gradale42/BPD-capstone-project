use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use crate::AppState;
use crate::services::bitcoin::rpc::get_last_block_height as get_last_block_from_rpc;

pub async fn get_indexer_stats(state: web::Data<AppState>) -> impl Responder {
    let last_block_live = match get_last_block_from_rpc(&state).await {
        Ok(height) => height,
        Err(e) => {
            eprintln!("RPC error: {}", e);
            -1
        }
    };

    let last_block_index = match state.block_service.get_last_block_height().await {
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