use actix_web::{get, web, HttpResponse, Responder};
use bitcoincore_rpc::RpcApi;
use serde_json::json;
use std::fs;
use crate::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct BlocksParams {
    pub draw: i32,
    pub start: Option<i32>,
    pub length: Option<i32>,
}

#[derive(Debug, serde::Serialize)]
pub struct BlockInfo {
    pub height: i64,
    pub hash: String,
    pub time: i64,
    pub tx_count: i32,
    pub avg_fee: f64,
    pub avg_feerate: f64,
    pub total_fees: f64,
    pub difficulty: f64,

    // pub height: i64,
    // pub hash: String,
    // pub time: i64,        // Unix timestamp from RPC
    // pub tx_count: i32,
    // pub size: i32,        // Block size in bytes
    // pub weight: i32,      // Block weight (vSize * 4)
    // pub subsidy: i64,     // Block reward in Satoshis (e.g., 312500000)
    // pub total_fees: i64,  // Total fees in Satoshis
    // pub avg_fee: i64,     // Average fee in Satoshis
    // pub avg_feerate: f64, // Usually sat/vB, so float is fine here
    // pub difficulty: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct DataTableResponse<T> {
    pub draw: i32,
    pub records_total: usize,
    pub records_filtered: usize,
    pub data: Vec<T>,
}

pub async fn get_mock_blocks(web::Query(params): web::Query<serde_json::Value>) -> impl Responder {
    match fs::read_to_string("resources/blocks.json") {
        Ok(content) => HttpResponse::Ok().json(json!(content)),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error reading file: {}", e)),
    }
}

pub async fn get_blocks(
    state: web::Data<AppState>,
    web::Query(params): web::Query<BlocksParams>,
) -> impl Responder {
    match state.get_default_bitcoin_client().lock() {
        Ok(client) => {
   
            let blockchain_info = match client.get_blockchain_info() {
                Ok(info) => info,
                Err(e) => {
                    eprintln!("Error getting blockchain info: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to get blockchain info: {}", e)
                    }));
                }
            };

            let current_height = blockchain_info.blocks;
            let length = params.length.unwrap_or(25) as u64;
            let start = if current_height >= length {
                current_height - length + 1
            } else {
                0
            };

            let mut blocks = Vec::new();

            for height in start..=current_height {
                
                let block_hash = match client.get_block_hash(height) {
                    Ok(hash) => hash,
                    Err(e) => {
                        eprintln!("Error getting block hash at height {}: {}", height, e);
                        continue;
                    }
                };

                let block = match client.get_block(&block_hash) {
                    Ok(block) => block,
                    Err(e) => {
                        eprintln!("Error getting block {}: {}", block_hash, e);
                        continue;
                    }
                };

                let block_stats = match client.get_block_stats(height) {
                    Ok(stats) => stats,
                    Err(e) => {
                        eprintln!("Error getting block stats for height {}: {}", height, e);
                        continue; 
                    }
                };

                let avg_fee_sats = block_stats.avg_fee.to_sat() as f64;
                let avg_fee_rate_sats = block_stats.avg_fee_rate.to_sat() as f64;
                let total_fees_sats = block_stats.total_fee.to_sat() as f64; // total_fee это Amount, не Option

                blocks.push(BlockInfo {
                    height: height as i64,
                    hash: block_hash.to_string(),
                    time: block.header.time as i64,
                    tx_count: block.txdata.len() as i32,
                    avg_fee: avg_fee_sats,
                    avg_feerate: avg_fee_rate_sats,
                    total_fees: total_fees_sats,
                    difficulty: block.header.difficulty() as f64,
                });
            }

            blocks.sort_by(|a, b| b.height.cmp(&a.height));

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