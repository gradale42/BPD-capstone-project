use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod admin;
pub mod blocks;
pub mod indexer;
pub mod live;
pub mod mempool;
pub mod network;
pub mod peers;
pub mod scheduler;
pub mod wallets;

use actix_web::{web, HttpResponse};
use serde_json::json;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Blocks
        blocks::get_blocks,
        blocks::get_block_time_series,
        blocks::get_block_by_hash,

        // Mempool
        mempool::get_mempool,
        mempool::get_mempool_timeseries,
        mempool::get_mempool_stats,
        mempool::get_mempool_transactions,
        mempool::get_transaction_details,

        // Scheduler
        scheduler::start_scheduler,
        scheduler::stop_scheduler,
        scheduler::get_scheduler_status,
        scheduler::get_scheduler_logs,
        scheduler::get_scheduler_log,

        // Admin
        admin::import_descriptors_handler,
        admin::mine_blocks_handler,
        admin::save_blocks,

        // Network
        network::switch_network,
        network::get_current_network,
        network::get_network_info,

        // Wallets
        wallets::list_wallets,
        wallets::get_wallet_details_handler,
        wallets::get_descriptors_handler,

        // Misc
        live::get_live_stats,
        indexer::get_indexer_stats,
        peers::get_peers,
    ),
    components(
        schemas(
            crate::domain::address::AddressInfo,
            crate::domain::address::DescriptorInfo,
            crate::domain::block::BlockInfo,
            crate::domain::block::TimeseriesPoint,
            crate::domain::mempool::MempoolSnapshot,
            crate::domain::mempool::MempoolTransaction,
            crate::domain::mempool::MempoolMetrics,
            crate::domain::mempool::MempoolMetricsPoint,
            crate::domain::peers::PeerInfo,
            crate::domain::scheduler::SchedulerLog,
            crate::domain::scheduler::SyncResult,
            crate::domain::wallet::WalletInfo,
        )
    ),
    tags(
        (name = "Blocks", description = "Blockchain data and history"),
        (name = "Mempool", description = "Memory pool and unconfirmed transactions"),
        (name = "Wallets", description = "Wallet management"),
        (name = "Network", description = "Node and network configuration"),
        (name = "Scheduler", description = "Background task management"),
        (name = "Admin", description = "Privileged node operations"),
        (name = "System", description = "Health and indexing status")
    )
)]
pub struct ApiDoc;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        SwaggerUi::new("/api/v1/swagger-ui/{_:.*}")
            .url("/api/v1/api-docs/openapi.json", ApiDoc::openapi()),
    );
    cfg.service(
        web::scope("/api/v1")
            .service(
                web::scope("/blocks")
                    .route("", web::get().to(blocks::get_blocks))
                    .route("/timeseries", web::get().to(blocks::get_block_time_series))
                    .route("/{block_hash}", web::get().to(blocks::get_block_by_hash))
            )
            .service(
                web::scope("/mempool")
                    .route("", web::get().to(mempool::get_mempool))
                    .route("/timeseries", web::get().to(mempool::get_mempool_timeseries))
                    .route("/stats", web::get().to(mempool::get_mempool_stats))
                    .route("/transactions", web::get().to(mempool::get_mempool_transactions))
                    .route("/transactions/{txid}", web::get().to(mempool::get_transaction_details))
            )
            // TODO: This should be moved to a separate service and not be nested under mempool
            // .service(
            //     web::scope("/transactions")
            //         .route("", web::get().to(tx::list))
            //         .route("/{txid}", web::get().to(tx::details))
            // )
            .service(
                web::scope("/scheduler")
                    .route("start", web::post().to(scheduler::start_scheduler))
                    .route("stop", web::post().to(scheduler::stop_scheduler))
                    .route("status", web::get().to(scheduler::get_scheduler_status))
                    .route("logs", web::get().to(scheduler::get_scheduler_logs))
                    .route("logs/{id}", web::get().to(scheduler::get_scheduler_log)),

            )
            .service(
                web::scope("/admin")
                    .route("/import-descriptors", web::post().to(admin::import_descriptors_handler))
                    .route("/mine-blocks", web::post().to(admin::mine_blocks_handler))
                    .route("/save-blocks", web::post().to(admin::save_blocks))
            )
            .service(
                web::scope("/network")
                    .route("/switch", web::post().to(network::switch_network))
                    .route("/current", web::get().to(network::get_current_network))
                    .route("/info", web::get().to(network::get_network_info))
            )
            .service(
                web::scope("/wallets")
                    .route("", web::get().to(wallets::list_wallets))
                    .route("/{wallet_name}", web::get().to(wallets::get_wallet_details_handler))
                    .route("/{wallet_name}/descriptors", web::get().to(wallets::get_descriptors_handler))
            )
            .route("/live", web::get().to(live::get_live_stats))
            .route("/indexer", web::get().to(indexer::get_indexer_stats))
            .route("/peers", web::get().to(peers::get_peers))
    )
    ;
}

pub fn wrap_response<R>(result: anyhow::Result<R>) -> HttpResponse
where
    R: serde::Serialize,
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
