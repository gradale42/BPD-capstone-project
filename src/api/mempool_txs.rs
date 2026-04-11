use actix_web::{web, HttpResponse, Responder};
use bitcoincore_rpc::RpcApi;
use serde_json::json;
use crate::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct MempoolTxsParams {
    pub draw: i32,
    pub start: Option<i32>,
    pub length: Option<i32>,
    pub order: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Clone, serde::Serialize)]  // Добавлен Clone
pub struct MempoolTransaction {
    pub txid: String,
    pub vsize: u64,
    pub weight: u64,
    pub time: u64,
    pub height: u64,
    pub fee: f64,
    pub fee_rate: f64,
    pub ancestor_count: u64,
    pub descendant_count: u64,
    pub bip125_replaceable: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct DataTableResponse<T> {
    pub draw: i32,
    pub records_total: usize,
    pub records_filtered: usize,
    pub data: Vec<T>,
}

pub async fn get_mempool_transactions(
    state: web::Data<AppState>,
    web::Query(params): web::Query<MempoolTxsParams>,
) -> impl Responder {
    match state.node_manager.get_default_bitcoin_client().lock() {
        Ok(client) => {
            // Get all mempool transaction IDs
            let mempool_txids = match client.get_raw_mempool() {
                Ok(txids) => txids,
                Err(e) => {
                    eprintln!("Error getting mempool transactions: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to get mempool transactions: {}", e)
                    }));
                }
            };

            let mut transactions = Vec::new();

            // Get details for each transaction
            for txid in mempool_txids.iter().take(1000) { // Limit to 1000 for performance
                match client.get_mempool_entry(txid) {
                    Ok(entry) => {
                        // Calculate fee rate (satoshis per vbyte)
                        let fee_rate = if entry.vsize > 0 {
                            entry.fees.base.to_sat() as f64 / entry.vsize as f64
                        } else {
                            0.0
                        };

                        transactions.push(MempoolTransaction {
                            txid: txid.to_string(),
                            vsize: entry.vsize,
                            weight: entry.weight.unwrap_or(0),
                            time: entry.time,
                            height: entry.height,
                            fee: entry.fees.base.to_btc(),
                            fee_rate: fee_rate,
                            ancestor_count: entry.ancestor_count,
                            descendant_count: entry.descendant_count,
                            bip125_replaceable: entry.bip125_replaceable,
                        });
                    }
                    Err(e) => {
                        eprintln!("Error getting mempool entry for {}: {}", txid, e);
                    }
                }
            }

            // Apply sorting if specified
            if let Some(order) = params.order {
                if let Some(first_order) = order.first() {
                    if first_order.len() >= 2 {
                        let column_idx = first_order[0].parse::<usize>().unwrap_or(5);
                        let direction = &first_order[1];

                        match column_idx {
                            0 => transactions.sort_by(|a, b| if direction == "asc" { a.txid.cmp(&b.txid) } else { b.txid.cmp(&a.txid) }),
                            1 => transactions.sort_by(|a, b| if direction == "asc" { a.vsize.cmp(&b.vsize) } else { b.vsize.cmp(&a.vsize) }),
                            2 => transactions.sort_by(|a, b| if direction == "asc" { a.weight.cmp(&b.weight) } else { b.weight.cmp(&a.weight) }),
                            3 => transactions.sort_by(|a, b| if direction == "asc" { a.time.cmp(&b.time) } else { b.time.cmp(&a.time) }),
                            4 => transactions.sort_by(|a, b| if direction == "asc" { a.height.cmp(&b.height) } else { b.height.cmp(&a.height) }),
                            5 => transactions.sort_by(|a, b| if direction == "asc" { a.fee_rate.partial_cmp(&b.fee_rate).unwrap() } else { b.fee_rate.partial_cmp(&a.fee_rate).unwrap() }),
                            _ => transactions.sort_by(|a, b| b.fee_rate.partial_cmp(&a.fee_rate).unwrap()),
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

            // Исправлено: используем slice и clone или просто создаем новый Vec
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

pub async fn get_mempool_stats(
    state: web::Data<AppState>,
) -> impl Responder {
    match state.node_manager.get_default_bitcoin_client().lock() {
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

            HttpResponse::Ok().json(json!({
                "tx_count": mempool_info.size,
                "vbytes": mempool_info.bytes,
                "total_fees": mempool_info.total_fee.unwrap_or_default().to_btc(),
                "min_relay_feerate": mempool_info.min_relay_tx_fee.to_sat(),
            }))
        }
        Err(e) => {
            eprintln!("Error locking RPC client: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to lock RPC client: {}", e)
            }))
        }
    }
}

pub async fn get_transaction_details(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let txid = path.into_inner();
    match state.node_manager.get_default_bitcoin_client().lock() {
        Ok(client) => {
            let txid_parsed = match txid.parse::<bitcoin::Txid>() {
                Ok(t) => t,
                Err(e) => {
                    return HttpResponse::BadRequest().json(json!({
                        "error": format!("Invalid txid: {}", e)
                    }));
                }
            };

            // Try to get from mempool first
            if let Ok(entry) = client.get_mempool_entry(&txid_parsed) {
                return HttpResponse::Ok().json(json!({
                    "txid": txid,
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
                }));
            }

            // If not in mempool, try to get from blockchain
            match client.get_transaction(&txid_parsed, None) {
                Ok(tx) => {
                    HttpResponse::Ok().json(json!({
                        "txid": txid,
                        "in_block": true,
                        "details": format!("{:?}", tx)
                    }))
                }
                Err(e) => {
                    HttpResponse::NotFound().json(json!({
                        "error": format!("Transaction not found: {}", e)
                    }))
                }
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to lock RPC client: {}", e)
            }))
        }
    }
}