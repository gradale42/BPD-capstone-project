use crate::domain::block::{BlockInfo, BlocksParams, TimeRange};
use crate::bitcoin::rpc::get_blocks_info;
use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
use bitcoincore_rpc::bitcoin::Witness;
use bitcoincore_rpc::RpcApi;
use serde_json::{json, Value};
use std::fs;
use serde::Deserialize;
use crate::services::ExecutionCtx;

#[derive(Debug, serde::Serialize)]
pub struct DataTableResponse<T> {
    pub draw: i32,
    pub records_total: usize,
    pub records_filtered: usize,
    pub data: Vec<T>,
}

pub async fn get_blocks(
    state: web::Data<AppState>,
    params: web::Query<BlocksParams>,
) -> impl Responder {
    let mode = params.mode.as_deref().unwrap_or("live");
    let length = params.length.unwrap_or(25) as u64;
    let start = params.start.unwrap_or(0) as u64;


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

    match mode {
        "index" => {
            // database query
            let total_blocks = match state.block_service.count_blocks_in_db(&mut ctx).await {
                Ok(count) => count as usize,
                Err(e) => {
                    eprintln!("DB count error: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "error": format!("Database error: {}", e)
                    }));
                }
            };

            let order_by = if let Some(order) = &params.order {
                let col_idx = order[0].column;
                let dir = &order[0].dir;
                let col_name = match col_idx {
                    0 => "height",
                    1 => "hash",
                    2 => "time",
                    3 => "tx_count",
                    4 => "avg_fee_sat",
                    5 => "avg_feerate",
                    6 => "total_fees_sat",
                    7 => "difficulty",
                    _ => "height",
                };
                format!("{} {}", col_name, dir)
            } else {
                "height DESC".to_string()
            };

            let blocks = match state.block_service.get_blocks_from_db(&mut ctx, length as i64, start as i64, &order_by).await {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("DB list error: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "error": format!("Database error: {}", e)
                    }));
                }
            };

            let response = DataTableResponse {
                draw: params.draw,
                records_total: total_blocks,
                records_filtered: total_blocks,
                data: blocks,
            };
            HttpResponse::Ok().json(response)
        }
        _ => {
            // Live mode (RPC)
            match get_blocks_info(&state.clone(), Some(length)).await {
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
                    eprintln!("RPC error: {}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "error": format!("RPC error: {}", e)
                    }))
                }
            }
        }
    }
}

pub async fn get_block_timeseries(
    state: web::Data<AppState>,
    query: web::Query<TimeRange>,
) -> impl Responder {

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

    match state.block_service.get_timeseries(&mut ctx, query.from, query.to).await {
        Ok(points) => HttpResponse::Ok().json(points),
        Err(e) => {
            eprintln!("Timeseries error: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to fetch timeseries: {}", e)
            }))
        }
    }
}
