#![allow(unused)]

pub mod domain;
pub mod configuration;
pub mod api;
pub mod services;
pub mod repositories;
pub mod db;

use actix_files as actix_fs;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::dev::Server;
use bitcoincore_rpc::{Auth, Client};
use sqlx::PgPool;
use dashmap::DashMap;
use serde_json::json;
use configuration::{get_configuration, Network, BitcoinNodeConfig};
use services::bitcoin::rpc::setup_wallet;
use services::bitcoin::node_manager::BitcoinNodeManager;
use crate::services::block_service::BlockService;
use crate::services::scheduler_log_service::SchedulerLogService;
use crate::repositories::{
    BlockRepository, PostgresBlockRepository,
    SchedulerLogRepository, PostgresSchedulerLogRepository,
};
use crate::services::scheduler::SchedulerService;

pub struct AppState {
    pub node_manager: Arc<BitcoinNodeManager>,
    pub db_pool: PgPool,
    pub block_service: Arc<BlockService>,
    pub scheduler_log_service: Arc<SchedulerLogService>,
    pub scheduler_service : Arc<SchedulerService>,
}

impl AppState {
    // Deprecated methods for backward compatibility
    #[deprecated(note = "Use node_manager.get_default_current_client() instead")]
    pub fn get_default_bitcoin_client(&self) -> Arc<Mutex<Client>> {
        self.node_manager.get_default_current_client()
    }

    #[deprecated(note = "Use node_manager.get_current_client() instead")]
    pub fn get_bitcoin_client(&self, wallet_name: &str) -> Arc<Mutex<Client>> {
        self.node_manager.get_current_client(wallet_name)
    }

    #[deprecated(note = "Use node_manager.execute_rpc() instead")]
    pub async fn execute_rpc<F, R>(&self, wallet_name: &str, f: F) -> HttpResponse
    where
        F: FnOnce(&bitcoincore_rpc::Client) -> Result<R, bitcoincore_rpc::Error> + Send + 'static,
        R: serde::Serialize + Send + 'static,
    {
        match self.node_manager.execute_rpc(wallet_name, f).await {
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
}

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let config = get_configuration().expect("Failed to read configuration.");

    // Initialize node configurations for each network
    let mut network_configs = HashMap::new();

    // Regtest configuration
    network_configs.insert(Network::Regtest, BitcoinNodeConfig {
        rpc_url: "http://localhost:18443".to_string(),
        rpc_user: config.bitcoin.default_config.rpc_user.clone(),
        rpc_password: config.bitcoin.default_config.rpc_password.clone(),
    });

    // Signet configuration
    network_configs.insert(Network::Signet, BitcoinNodeConfig {
        rpc_url: "http://localhost:38332".to_string(),
        rpc_user: config.bitcoin.default_config.rpc_user.clone(),
        rpc_password: config.bitcoin.default_config.rpc_password.clone(),
    });

    // Testnet configuration
    network_configs.insert(Network::Testnet, BitcoinNodeConfig {
        rpc_url: "http://localhost:18332".to_string(),
        rpc_user: config.bitcoin.default_config.rpc_user.clone(),
        rpc_password: config.bitcoin.default_config.rpc_password.clone(),
    });

    // Mainnet configuration (adjust URL as needed)
    network_configs.insert(Network::Mainnet, BitcoinNodeConfig {
        rpc_url: config.bitcoin.get_node_config(Network::Mainnet).rpc_url,
        rpc_user: config.bitcoin.default_config.rpc_user.clone(),
        rpc_password: config.bitcoin.default_config.rpc_password.clone(),
    });

    let node_manager = Arc::new(BitcoinNodeManager::new(network_configs, Network::Regtest));

    // repo layer
    let block_repo = Arc::new(PostgresBlockRepository);
    let scheduler_log_repo = Arc::new(PostgresSchedulerLogRepository);

    // service layer
    let block_service = Arc::new(BlockService::new(block_repo.clone(), db_pool.clone()));
    let scheduler_log_service = Arc::new(SchedulerLogService::new(
        db_pool.clone(),
        block_repo.clone(),
        scheduler_log_repo.clone(),
    ));
    let scheduler_service = Arc::new(SchedulerService::new());

    let app_state = web::Data::new(AppState {
        node_manager: node_manager.clone(),
        db_pool,
        block_service,
        scheduler_log_service,
        scheduler_service
    });

    println!("\n================================================");
    println!("🌐 WEB SERVER:    http://127.0.0.1:{}", config.application_port);
    println!("🐘 POSTGRES:      {}:{}", config.database.host, config.database.port);
    println!("📀 DATABASE:      {}", config.database.database_name);
    println!("================================================");
    
    let networks_info: Vec<String> = config.bitcoin.networks
        .iter()
        .map(|(name, cfg)| format!("{}:{}", name, cfg.rpc_url))
        .collect();

    println!("\n================================================");
    println!("🌐 Bitcoin Networks Configured:");
    for info in &networks_info {
        println!("   • {}", info);
    }
    println!("   • All networks: {}", networks_info.join(", "));
    println!("================================================");

    println!("📊 Real RPC endpoints: /api/stats, /api/blocks, /api/mempool, /api/peers");
    println!("🎭 Mock endpoints: /api/mock/stats, /api/mock/blocks, /api/mock/mempool, /api/mock/peers");
    println!("🛠️ Admin endpoints: /api/admin/import-descriptors, /api/admin/mine-blocks\n");
    println!("🌐 Network switch endpoint: POST /api/network/switch\n");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .configure(api::config)
            .service(actix_fs::Files::new("/ui", "./ui").show_files_listing())
            .service(actix_fs::Files::new("/", "./ui").index_file("index.html"))
    })
        .listen(listener)?
        .run();

    Ok(server)
}