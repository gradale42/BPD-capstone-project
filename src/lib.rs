#![allow(unused)]

use actix_files as actix_fs;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::dev::Server;
use bitcoincore_rpc::{Auth, Client};
use sqlx::PgPool;
use dashmap::DashMap;
use serde_json::json;
use crate::bitcoin::rpc::setup_wallet;
use crate::configuration::get_configuration;

pub mod api;

pub mod bitcoin;

pub mod configuration;

pub struct AppState {
    pub bitcoin_clients: DashMap<String, Arc<Mutex<Client>>>,
    pub db_pool: PgPool
}

impl AppState {

    pub fn get_default_bitcoin_client(&self) -> Arc<Mutex<Client>> {
        self.get_bitcoin_client("")
    }

    pub fn get_bitcoin_client(&self, wallet_name: &str) -> Arc<Mutex<Client>> {
        let entry = self.bitcoin_clients.entry(wallet_name.to_string()).or_insert_with(|| {

            println!("=== Starting bitcoin client for wallet {} ===", wallet_name);
            let configuration = get_configuration().expect("Failed to read configuration.");
            let rpc_url = configuration.bitcoin.rpc_url;
            let rpc_auth = Auth::UserPass(configuration.bitcoin.rpc_user, configuration.bitcoin.rpc_password);

            let wallet_url = if wallet_name.is_empty() {
                rpc_url.clone() // Default to main RPC endpoint if no wallet name is provided
            } else {
                format!("{}/wallet/{}", rpc_url, wallet_name)
            };

            let client = Client::new(&wallet_url, rpc_auth.clone())
                .expect("Failed to create Bitcoin RPC client");

            Arc::new(Mutex::new(client))
        });
        entry.value().clone()
    }

    pub async fn execute_rpc<F, R>(&self, wallet_name: &str, f: F) -> HttpResponse
    where
    // F must be Send to go into web::block
    // R must be Serialize to be returned as JSON
        F: FnOnce(&bitcoincore_rpc::Client) -> Result<R, bitcoincore_rpc::Error> + Send + 'static,
        R: serde::Serialize + Send + 'static,
    {
        let client_arc = self.get_bitcoin_client(wallet_name);

        let result = web::block(move || {
            let client_guard = client_arc.lock().map_err(|_| "Lock poisoning error".to_string())?;

            // CRITICAL: Convert the Result<R, bitcoincore_rpc::Error>
            // into Result<R, String> HERE, before returning from the closure.
            f(&*client_guard).map_err(|e| e.to_string())
        }).await;

        match result {
            // result is Result<Result<R, String>, BlockingError>
            //Ok(Ok(data)) => HttpResponse::Ok().json(data),
        Ok(Ok(data)) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": data
        })),
            Ok(Err(rpc_err_string)) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": rpc_err_string
        })),
            Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Thread pool error: {}", e)
        })),
        }
    }

}

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {

    let app_state = web::Data::new(AppState {
        bitcoin_clients: DashMap::new(),
        db_pool,
    });

    println!("=== Starting bitcoin client ===");
    match app_state.get_default_bitcoin_client().lock() {
        Ok(client) => {
            setup_wallet(&client, "mining_wallet");
            setup_wallet(&client, "student");
        },
        Err(e) => {
            eprintln!("⚠️  Failed to initialize bitcoin client: {}", e);
            eprintln!("⚠️  Starting with mock data only");
        }
    };

    let config = get_configuration().expect("Failed to read configuration.");
    println!("\n================================================");
    println!("🌐 WEB SERVER:    http://127.0.0.1:{}", config.application_port);
    println!("🐘 POSTGRES:      {}:{}", config.database.host, config.database.port);
    println!("📀 DATABASE:      {}", config.database.database_name);
    println!("₿  BITCOIN RPC:   {}", config.bitcoin.rpc_url);
    println!("👤 RPC USER:      {}", config.bitcoin.rpc_user);
    println!("================================================\n");

    println!("📊 Real RPC endpoints: /api/stats, /api/blocks, /api/mempool, /api/peers");
    println!("🎭 Mock endpoints: /api/mock/stats, /api/mock/blocks, /api/mock/mempool, /api/mock/peers");
    println!("🛠️ Admin endpoints: /api/admin/import-descriptors, /api/admin/mine-blocks\n");

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

