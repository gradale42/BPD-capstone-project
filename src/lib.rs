#![allow(unused)]

pub mod domain;
pub mod configuration;
pub mod api;
pub mod services;
pub mod repositories;

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
use crate::configuration::ALL_BITCOIN_NETWORKS;
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

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let config = get_configuration().expect("Failed to read configuration.");

    let mut network_configs = HashMap::new();
    for network in ALL_BITCOIN_NETWORKS {
        let node_config = config.bitcoin.get_node_config(network);
        network_configs.insert(network, BitcoinNodeConfig {
            rpc_url: node_config.rpc_url,
            rpc_user: node_config.rpc_user,
            rpc_password: node_config.rpc_password,
        });
    }
    let node_manager = Arc::new(BitcoinNodeManager::new(network_configs, Network::Regtest));

    // repo layer
    let block_repo = Arc::new(PostgresBlockRepository);
    let scheduler_log_repo = Arc::new(PostgresSchedulerLogRepository);

    // service layer
    let block_service = Arc::new(BlockService::new(block_repo.clone()));
    let scheduler_log_service = Arc::new(SchedulerLogService::new(
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

    println!("📊 RPC endpoints: /api/stats, /api/blocks, /api/mempool, /api/peers");
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