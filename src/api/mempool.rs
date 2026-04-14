use crate::api::wrap_response;
use crate::bitcoin::rpc;
use crate::domain::mempool::{MempoolSnapshot, MempoolTimeRange};
use crate::services::ExecutionCtx;
use crate::AppState;
use actix_web::{web, Responder};
use anyhow::Context;
use bitcoincore_rpc::{Error, RpcApi};
use serde_json::json;

#[derive(Debug, serde::Deserialize)]
pub struct MempoolParams {
    pub draw: i32,
    pub start: Option<i32>,
    pub length: Option<i32>,
}

#[derive(Debug, serde::Serialize)]
pub struct DataTableResponse<T> {
    pub draw: i32,
    pub records_total: usize,
    pub records_filtered: usize,
    pub data: Vec<T>,
}

#[derive(Debug, serde::Deserialize)]
pub struct MempoolTxsParams {
    pub draw: i32,
    pub start: Option<i32>,
    pub length: Option<i32>,
    pub order: Option<Vec<Vec<String>>>,
}

pub async fn get_mempool(
    state: web::Data<AppState>,
    web::Query(params): web::Query<MempoolParams>,
) -> impl Responder {
    let node_manager = &state.node_manager;

    let result: anyhow::Result<_> = async {
        let rpc_result = node_manager
            .execute_rpc("", move |client| {
                let mempool_info = client.get_mempool_info()?;

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

                Ok(response)
            })
            .await
            .context("Failed to read mempool info")?;

        Ok(rpc_result)
    }.await;

    wrap_response(result)
}

pub async fn get_mempool_timeseries(
    state: web::Data<AppState>, mut ctx: ExecutionCtx, query: web::Query<MempoolTimeRange>,
) -> impl Responder {
    let mempool_metrics_service = &state.mempool_metrics_service;
    let result: anyhow::Result<_> = async {
        let db_result = mempool_metrics_service
            .get_timeseries(&mut ctx, query.from, query.to)
            .await
            .context("Failed to read mempool metrics from db")?;
          Ok(db_result)
    }.await;
    wrap_response(result)
}

pub async fn get_mempool_transactions(
    state: web::Data<AppState>, web::Query(params): web::Query<MempoolTxsParams>,
) -> impl Responder {
    let node_manager = &state.node_manager;

    let result: anyhow::Result<_> = async {
        let rpc_result = node_manager
            .execute_rpc("", move |client| {
                let mut transactions = rpc::get_mempool_transactions(client)?;

                // Apply sorting if specified
                if let Some(order) = params.order {
                    if let Some(first_order) = order.first() {
                        if first_order.len() >= 2 {
                            let column_idx = first_order[0].parse::<usize>().unwrap_or(5);
                            let direction = &first_order[1];

                            match column_idx {
                                0 => transactions.sort_by(|a, b| {
                                    if direction == "asc" {
                                        a.txid.cmp(&b.txid)
                                    } else {
                                        b.txid.cmp(&a.txid)
                                    }
                                }),
                                1 => transactions.sort_by(|a, b| {
                                    if direction == "asc" {
                                        a.vsize.cmp(&b.vsize)
                                    } else {
                                        b.vsize.cmp(&a.vsize)
                                    }
                                }),
                                2 => transactions.sort_by(|a, b| {
                                    if direction == "asc" {
                                        a.weight.cmp(&b.weight)
                                    } else {
                                        b.weight.cmp(&a.weight)
                                    }
                                }),
                                3 => transactions.sort_by(|a, b| {
                                    if direction == "asc" {
                                        a.time.cmp(&b.time)
                                    } else {
                                        b.time.cmp(&a.time)
                                    }
                                }),
                                4 => transactions.sort_by(|a, b| {
                                    if direction == "asc" {
                                        a.height.cmp(&b.height)
                                    } else {
                                        b.height.cmp(&a.height)
                                    }
                                }),
                                5 => transactions.sort_by(|a, b| {
                                    if direction == "asc" {
                                        a.fee_rate.partial_cmp(&b.fee_rate).unwrap()
                                    } else {
                                        b.fee_rate.partial_cmp(&a.fee_rate).unwrap()
                                    }
                                }),
                                _ => transactions
                                    .sort_by(|a, b| b.fee_rate.partial_cmp(&a.fee_rate).unwrap()),
                            }
                        }
                    }
                } else {
                    // Default sort by fee rate (highest first)
                    transactions.sort_by(|a, b| b.fee_rate.partial_cmp(&a.fee_rate).unwrap());
                }

                // Apply pagination
                let start = params.start.unwrap_or(0) as usize;
                let length = params.length.unwrap_or(25) as usize;
                let end = std::cmp::min(start + length, transactions.len());

                let paginated_data = if start < transactions.len() {
                    transactions[start..end].to_vec()
                } else {
                    Vec::new()
                };

                let response = DataTableResponse {
                    draw: params.draw,
                    records_total: transactions.len(),
                    records_filtered: transactions.len(),
                    data: paginated_data,
                };

                Ok(response)
            })
            .await
            .context("Failed to read mempool info")?;

        Ok(rpc_result)
    }.await;

    wrap_response(result)
}

pub async fn get_mempool_stats(state: web::Data<AppState>) -> impl Responder {
    let node_manager = &state.node_manager;

    let result: anyhow::Result<_> = async {
        let rpc_result = node_manager
            .execute_rpc("", move |client| {
                let mempool_info = client.get_mempool_info()?;
                Ok(json!({
                    "tx_count": mempool_info.size,
                    "vbytes": mempool_info.bytes,
                    "total_fees": mempool_info.total_fee.unwrap_or_default().to_btc(),
                    "min_relay_feerate": mempool_info.min_relay_tx_fee.to_sat(),
                }))
            })
            .await
            .context("Failed to read mempool info")?;

        Ok(rpc_result)
    }.await;

    wrap_response(result)
}

pub async fn get_transaction_details(
    state: web::Data<AppState>, path: web::Path<String>,
) -> impl Responder {
    let txid = path.into_inner();
    let node_manager = &state.node_manager;

    let result: anyhow::Result<_> = async {
        let rpc_result = node_manager
            .execute_rpc("", move |client| {
                let txid_parsed = txid.parse::<bitcoin::Txid>().map_err(|e| {
                    Error::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Invalid TXID format: {}", e)
                    ))
                })?;
                let entry = client.get_mempool_entry(&txid_parsed)?;
                Ok(json!({
                    "time": entry.time,
                    "height": entry.height,
                    "fee_rate": entry.fees.base.to_sat() as f64 / entry.vsize as f64,
                    "vsize": entry.vsize,
                    "weight": entry.weight,
                    "ancestor_count": entry.ancestor_count,
                    "descendant_count": entry.descendant_count,
                    "bip125_replaceable": entry.bip125_replaceable,
                    "fees": {
                        "base": entry.fees.base.to_btc(),
                        "modified": entry.fees.modified.to_btc(),
                    }
                }))
            })
            .await
            .context("Failed to read mempool info")?;

        Ok(rpc_result)
    }.await;

    wrap_response(result)
}
