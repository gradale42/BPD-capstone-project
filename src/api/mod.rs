use actix_web::web;
use std::sync::{Mutex, Arc};
use bitcoincore_rpc::{Auth, Client};

pub mod stats;
pub mod blocks;
pub mod mempool;
pub mod peers;
pub mod admin;
mod wallets;

pub struct AppState {
    pub rpc_client: Mutex<Arc<Client>>,
    pub rpc_url: String,
    pub rpc_auth: Auth,
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

pub fn config_real(
    cfg: &mut web::ServiceConfig,
    rpc_client: Arc<Client>,
    rpc_url: String,
    rpc_auth: Auth,
) {
    let app_state = web::Data::new(AppState {
        rpc_client: Mutex::new(rpc_client),
        rpc_url,
        rpc_auth,
    });

    cfg.app_data(app_state);

    cfg.service(
        web::scope("/api")
            .route("/stats", web::get().to(stats::get_stats))
            .route("/blocks", web::get().to(blocks::get_blocks))
            .route("/mempool", web::get().to(mempool::get_mempool))
            .route("/peers", web::get().to(peers::get_peers))
            .route("/admin/import-descriptors", web::post().to(admin::import_descriptors_handler))
            .route("/admin/mine-blocks", web::post().to(admin::mine_blocks_handler))
            .route("/wallets", web::get().to(wallets::list_wallets))
            .route("/wallets/{wallet_name}", web::get().to(wallets::get_wallet_details_handler))
            .route("/wallets/{wallet_name}/descriptors", web::get().to(wallets::get_descriptors_handler)),
    );
}

pub fn config(cfg: &mut web::ServiceConfig, rpc_client: Option<(Arc<Client>, String, Auth)>) {
    config_mock(cfg);

    if let Some((rpc, url, auth)) = rpc_client {
        config_real(cfg, rpc, url, auth);
    } else {
        println!("⚠️  No RPC client available, real endpoints disabled");
    }
}