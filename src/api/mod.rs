use actix_web::web;
use std::sync::{Mutex, Arc};
use bitcoincore_rpc::Client;

pub mod stats;
pub mod blocks;
pub mod mempool;
pub mod peers;

pub struct AppState {
    pub rpc_client: Mutex<Arc<Client>>,
}

pub fn config_mock(cfg: &mut web::ServiceConfig) {
    println!("⚙️  Configuring MOCK API routes...");

    cfg.service(
        web::scope("/api/mock")
            .route("/stats", web::get().to(stats::get_mock_stats))
            .route("/blocks", web::get().to(blocks::get_mock_blocks))
            .route("/mempool", web::get().to(mempool::get_mock_mempool))
            .route("/peers", web::get().to(peers::get_mock_peers))
    );

    println!("✅ MOCK API routes configured");
}

pub fn config_real(cfg: &mut web::ServiceConfig, rpc_client: Arc<Client>) {
    println!("⚙️  Configuring REAL API routes...");

    let app_state = web::Data::new(AppState {
        rpc_client: Mutex::new(rpc_client),
    });

    cfg.app_data(app_state);

    cfg.service(
        web::scope("/api")
            .route("/stats", web::get().to(stats::get_stats))
            .route("/blocks", web::get().to(blocks::get_blocks))
            .route("/mempool", web::get().to(mempool::get_mempool))
            .route("/peers", web::get().to(peers::get_peers))
    );

    println!("✅ REAL API routes configured");
}

pub fn config(cfg: &mut web::ServiceConfig, rpc_client: Option<Arc<Client>>) {
    config_mock(cfg);

    if let Some(rpc) = rpc_client {
        config_real(cfg, rpc);
    } else {
        println!("⚠️  No RPC client available, real endpoints disabled");
    }
}