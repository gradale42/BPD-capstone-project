use crate::api::wrap_response;
use crate::bitcoin::rpc::get_block_details;
use crate::AppState;
use actix_web::{web, Responder};
use anyhow::Context;

pub async fn get_block_by_hash(
    state: web::Data<AppState>, block_hash: web::Path<String>,
) -> impl Responder {
    let hash = block_hash.into_inner();

    let node_manager = &state.node_manager;
    let result: anyhow::Result<_> = async {
        let rpc_result = node_manager
            .execute_rpc("", move |client| {
                get_block_details(client, hash)
            })
            .await
            .context("Failed to extract block")?;
        Ok(rpc_result)
    }.await;

    wrap_response(result)
}

