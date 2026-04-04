use crate::domain::block::{BlockInfo, BlocksParams};
use crate::services::bitcoin::rpc::get_blocks_info;
use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
use bitcoincore_rpc::bitcoin::Witness;
use bitcoincore_rpc::RpcApi;
use serde_json::{json, Value};
use std::fs;

#[derive(Debug, serde::Serialize)]
pub struct DataTableResponse<T> {
    pub draw: i32,
    pub records_total: usize,
    pub records_filtered: usize,
    pub data: Vec<T>,
}

pub async fn get_mock_blocks(web::Query(_params): web::Query<serde_json::Value>) -> impl Responder {
    match fs::read_to_string("resources/blocks.json") {
        Ok(content) => HttpResponse::Ok().json(json!(content)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error reading file: {}", e)),
    }
}

pub async fn get_blocks(
    state: web::Data<AppState>, web::Query(params): web::Query<BlocksParams>,
) -> impl Responder {
    let length = params.length.unwrap_or(25) as u64;
    match get_blocks_info(state, Some(length)).await {
        Ok(blocks) => {
            let response = DataTableResponse {
                draw: params.draw,
                records_total: blocks.len(),
                records_filtered: blocks.len(),
                data: blocks,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error locking RPC client: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to lock RPC client: {}", e)
            }))
        }
    }
}
