pub mod blocks;
pub mod block;
pub mod mempool;
pub mod peers;
pub mod admin;
pub mod wallets;
pub mod scheduler;
pub mod indexer;
pub mod live;
pub mod network;

use std::fmt::Display;
use std::future::Future;
use actix_web::{web, HttpResponse};
use serde::Serialize;
use serde_json::json;
use crate::bitcoin::node_manager::BitcoinNodeManager;
use crate::services::ExecutionCtx;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/network/switch", web::post().to(network::switch_network))
            .route("/network/current", web::get().to(network::get_current_network))
            .route("/network/info", web::get().to(network::get_network_info))
            .route("/live", web::get().to(live::get_live_stats))
            .route("/indexer", web::get().to(indexer::get_indexer_stats))
            .route("/block/{block_hash}", web::get().to(block::get_block_by_hash))
            .route("/blocks", web::get().to(blocks::get_blocks))
            .route("/blocks/timeseries", web::get().to(blocks::get_block_time_series))
            .route("/mempool", web::get().to(mempool::get_mempool))
            .route("/mempool/transactions", web::get().to(mempool::get_mempool_transactions))
            .route("/mempool/transaction/{txid}", web::get().to(mempool::get_transaction_details))
            .route("/mempool/timeseries", web::get().to(mempool::get_mempool_timeseries))
            .route("/mempool/stats", web::get().to(mempool::get_mempool_stats))
            .route("/peers", web::get().to(peers::get_peers))
            .route("/admin/import-descriptors", web::post().to(admin::import_descriptors_handler))
            .route("/admin/mine-blocks", web::post().to(admin::mine_blocks_handler))
            .route("/admin/save-blocks", web::post().to(admin::save_blocks))
            .route("/wallets", web::get().to(wallets::list_wallets))
            .route("/wallets/{wallet_name}", web::get().to(wallets::get_wallet_details_handler))
            .route("/wallets/{wallet_name}/descriptors", web::get().to(wallets::get_descriptors_handler))
            .route("/scheduler/start", web::post().to(scheduler::start_scheduler))
            .route("/scheduler/stop", web::post().to(scheduler::stop_scheduler))
            .route("/scheduler/status", web::get().to(scheduler::get_scheduler_status))
            .route("/scheduler/logs", web::get().to(scheduler::get_scheduler_logs))
            .route("/scheduler/log/{id}", web::get().to(scheduler::get_scheduler_log)),
    );
}

pub fn wrap_response<R, E>(result: Result<R, E>) -> HttpResponse
where
    R: Serialize,
    E: Display,
{
    match result {
        Ok(data) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": data
        })),
        Err(err) => {
            eprintln!("API Error: {}", err);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": err.to_string()
            }))
        }
    }
}