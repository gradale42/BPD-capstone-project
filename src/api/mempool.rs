use actix_web::{web, HttpResponse, Responder};
use bitcoincore_rpc::RpcApi;
use serde_json::json;
use std::fs;
use crate::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct MempoolParams {
    pub draw: i32,
    pub start: Option<i32>,
    pub length: Option<i32>,
}

#[derive(Debug, serde::Serialize)]
pub struct MempoolSnapshot {
    pub timestamp: String,
    pub tx_count: usize,
    pub vbytes: usize,
    pub total_fees: f64,
    pub min_relay_feerate: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct DataTableResponse<T> {
    pub draw: i32,
    pub records_total: usize,
    pub records_filtered: usize,
    pub data: Vec<T>,
}

pub async fn get_mock_mempool(web::Query(params): web::Query<serde_json::Value>) -> impl Responder {
    match fs::read_to_string("resources/mempool_history.json") {
        Ok(content) => HttpResponse::Ok().json(json!(content)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error reading file: {}", e)),
    }
}

pub async fn get_mempool(
    state: web::Data<AppState>,
    web::Query(params): web::Query<MempoolParams>,
) -> impl Responder {
    match state.get_default_bitcoin_client().lock() {
        Ok(client) => {
            let mempool_info = match client.get_mempool_info() {
                Ok(info) => info,
                Err(e) => {
                    eprintln!("Error getting mempool info: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to get mempool info: {}", e)
                    }));
                }
            };

            let snapshot = MempoolSnapshot {
                timestamp: chrono::Utc::now().to_rfc3339(),
                tx_count: mempool_info.size as usize,
                vbytes: mempool_info.bytes as usize,
                total_fees: mempool_info.total_fee.unwrap().to_sat() as f64,
                min_relay_feerate: mempool_info.min_relay_tx_fee.to_sat() as f64,
            };

            let response = DataTableResponse {
                draw: params.draw,
                records_total: 1,
                records_filtered: 1,
                data: vec![snapshot],
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