use std::fs;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
use bitcoincore_rpc::{Auth, Client};
use sqlx::PgPool;
use dashmap::DashMap;
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

}

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {

    let app_state = web::Data::new(AppState {
        bitcoin_clients: DashMap::new(),
        db_pool,
    });

    println!("\n🚀 Server running on http://127.0.0.1:3000");
    println!("\n🚀 Postgres running on localhost:5432/bitcoin_dashboard");
    println!("📊 Real RPC endpoints: /api/stats, /api/blocks, /api/mempool, /api/peers");
    println!("🎭 Mock endpoints: /api/mock/stats, /api/mock/blocks, /api/mock/mempool, /api/mock/peers");
    println!("🛠️ Admin endpoints: /api/admin/import-descriptors, /api/admin/mine-blocks\n");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .configure(api::config)
            .service(fs::Files::new("/ui", "./ui").show_files_listing())
            .index_file("index.html")
    })
    .listen(listener)?
    .run();

    Ok(server)
}

