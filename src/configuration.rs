use std::sync::{Arc, Mutex};
use actix_web::web;
use bitcoincore_rpc::{Auth, Client};
use crate::api::{admin, blocks, mempool, peers, stats, wallets};
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub bitcoin: BitcoinSettings,
    pub application_port: u16
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new("configuration.yaml", config::FileFormat::Yaml))
        .build()?;
    settings.try_deserialize::<Settings>()
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

#[derive(serde::Deserialize)]
pub struct BitcoinSettings {
    pub rpc_url: String,
    pub rpc_user: String,
    pub rpc_password: String,
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