use crate::api::wrap_response;
use crate::services::ExecutionCtx;
use crate::AppState;
use actix_web::{web, Responder};

pub async fn get_indexer_stats(
    state: web::Data<AppState>,
    mut ctx: ExecutionCtx,
) -> impl Responder {

    let block_service = &state.block_service;
    let service_call: Result<_, String> = async {
        let last_block_index = block_service.get_last_block_height(&mut ctx)
            .await
            .map_err(|e| format!("DB failed: {}", e))?;
        Ok(last_block_index)
    }.await;
    
    wrap_response(service_call)
}