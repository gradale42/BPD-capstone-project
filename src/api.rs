pub mod stats;
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

use actix_web::{web, HttpResponse};
use serde_json::json;
use crate::services::bitcoin::node_manager::BitcoinNodeManager;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/network/switch", web::post().to(network::switch_network))
            .route("/network/current", web::get().to(network::get_current_network))
            .route("/network/info", web::get().to(network::get_network_info))
            .route("/live", web::get().to(live::get_live_stats))
            .route("/indexer", web::get().to(indexer::get_indexer_stats))
            .route("/stats", web::get().to(stats::get_stats))
            .route("/block/{block_hash}", web::get().to(block::get_block_by_hash))
            .route("/blocks", web::get().to(blocks::get_blocks))
            .route("/blocks/timeseries", web::get().to(blocks::get_block_timeseries))
            .route("/mempool", web::get().to(mempool::get_mempool))
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

pub async fn execute_rpc<F, R>(node_manager: &BitcoinNodeManager, wallet_name: &str, f: F) -> HttpResponse
where
    F: FnOnce(&bitcoincore_rpc::Client) -> Result<R, bitcoincore_rpc::Error> + Send + 'static,
    R: serde::Serialize + Send + 'static,
{
    match node_manager.execute_rpc(wallet_name, f).await {
        Ok(data) => HttpResponse::Ok().json(json!({
                "status": "success",
                "data": data
            })),
        Err(err) => HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": err
            })),
    }
}