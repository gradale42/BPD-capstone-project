use crate::api::wrap_response;
use crate::services::ExecutionCtx;
use crate::AppState;
use actix_web::{web, Responder};
use anyhow::Context;


#[utoipa::path(
    get,
    path = "/api/v1/indexer",
    responses((status = 200, body = u64, description = "Current block height indexed")),
    tag = "Mempool"
)]
pub async fn get_indexer_stats(
    state: web::Data<AppState>,
    mut ctx: ExecutionCtx,
) -> impl Responder {

    let block_service = &state.block_service;
    let result: anyhow::Result<_> = async {
        let last_block_index = block_service.get_last_block_height(&mut ctx)
            .await
            .context("Failed to read blocks from db")?;
        Ok(last_block_index)
    }.await;
    
    wrap_response(result)
}