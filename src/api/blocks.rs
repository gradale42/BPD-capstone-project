use crate::api::wrap_response;
use crate::bitcoin::rpc::get_blocks_info;
use crate::domain::block::{BlocksParams, TimeRange};
use crate::services::ExecutionCtx;
use crate::AppState;
use actix_web::{web, Responder};

#[derive(Debug, serde::Serialize)]
pub struct DataTableResponse<T> {
    pub draw: i32,
    pub records_total: usize,
    pub records_filtered: usize,
    pub data: Vec<T>,
}

pub async fn get_blocks(
    state: web::Data<AppState>, mut ctx: ExecutionCtx, query: web::Query<BlocksParams>,
) -> impl Responder {
    let mode = query.mode.as_deref().unwrap_or("live");
    let length = query.length.unwrap_or(25) as u64;
    let start = query.start.unwrap_or(0) as u64;

    match mode {
        "index" => {
            // database mode
            let order_by = if let Some(order) = &query.order {
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

            let block_service = &state.block_service;
            let service_call: Result<_, String> = async {
                let db_result = block_service
                    .get_blocks_from_db(&mut ctx, length as i64, start as i64, &order_by)
                    .await
                    .map_err(|e| format!("DB failed: {}", e))?;
                Ok(db_result)
            }.await;

            wrap_response(service_call)
        }
        _ => {
            // Live mode (RPC)
            let node_manager = &state.node_manager;
            let network = state.node_manager.get_current_network();

            let service_call: Result<_, String> = async {
                let rpc_result = node_manager
                    .execute_rpc("", move |client| {
                        let blocks = get_blocks_info(client, network, Some(length))?;
                        let response = DataTableResponse {
                            draw: query.draw,
                            records_total: blocks.len(),
                            records_filtered: blocks.len(),
                            data: blocks,
                        };
                        Ok(response)
                    })
                    .await
                    .map_err(|e| format!("RPC failed: {}", e))?;
                Ok(rpc_result)
            }.await;

            wrap_response(service_call)
        }
    }
}

pub async fn get_block_time_series(
    state: web::Data<AppState>,
    mut ctx: ExecutionCtx,
    query: web::Query<TimeRange>,
) -> impl Responder {
    let block_service = &state.block_service;
    let service_call: Result<_, String> = async {
        let db_result = block_service
            .get_timeseries(&mut ctx, query.from, query.to)
            .await
            .map_err(|e| format!("DB failed: {}", e))?;
          Ok(db_result)
    }.await;
    wrap_response(service_call)
}
