pub mod stats;
pub mod blocks;
pub mod mempool;
pub mod peers;
pub mod admin;
pub mod wallets;

use actix_web::web;

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
    cfg: &mut web::ServiceConfig
) {
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

pub fn config(cfg: &mut web::ServiceConfig) {
    config_real(cfg);
    /*
    config_mock(cfg);

    if let Some((rpc, url, auth)) = rpc_client {
        config_real(cfg, appState);
    } else {
        println!("⚠️  No RPC client available, real endpoints disabled");
    }
    */
}