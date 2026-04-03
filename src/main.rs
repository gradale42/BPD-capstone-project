use actix_files as fs;
use actix_web::{App, HttpServer};
use bpd_capstone_project::bitcoin::rpc::{connect, setup_wallet};
use std::io;
use std::net::TcpListener;
use std::sync::Arc;
use bitcoincore_rpc::{Auth, Client};
use sqlx::{PgPool, Pool, Postgres};
use bpd_capstone_project::configuration::get_configuration;

mod api;
mod bitcoin;

#[actix_web::main]
async fn main() -> io::Result<()> {

    let rpc_client_info = match init_bitcoin() {
        Ok((client, url, auth)) => {
            println!("✅ Bitcoin RPC client initialized successfully");
            Some((Arc::new(client), url, auth))
        }
        Err(e) => {
            eprintln!("⚠️  Failed to initialize bitcoin client: {}", e);
            eprintln!("⚠️  Starting with mock data only");
            None
        }
    };

    let connection_pool = init_database();
    let listener = init_api_listener();

    println!("\n🚀 Server running on http://127.0.0.1:3000");
    println!("\n🚀 Postgres running on localhost:5432/bitcoin_dashboard");
    println!("📊 Real RPC endpoints: /api/stats, /api/blocks, /api/mempool, /api/peers");
    println!("🎭 Mock endpoints: /api/mock/stats, /api/mock/blocks, /api/mock/mempool, /api/mock/peers");
    println!("🛠️ Admin endpoints: /api/admin/import-descriptors, /api/admin/mine-blocks\n");

    HttpServer::new(move || {
        let mut app = App::new();
        app = app.configure(|cfg| api::config(cfg, rpc_client_info.clone()));
        app = app
            .service(fs::Files::new("/ui", "./ui").show_files_listing())
            .service(fs::Files::new("/", "./ui").index_file("index.html"));
        app
    })
        .bind("127.0.0.1:3000")?
        .run()
        .await
}

fn init_bitcoin() -> Result<(Client, String, Auth), Box<dyn std::error::Error>> {
    println!("=== Starting bitcoin client ===");
    let configuration = get_configuration().expect("Failed to read configuration.");
    let rpc_url = configuration.bitcoin.rpc_url;
    let rpc_auth = Auth::UserPass(configuration.bitcoin.rpc_user, configuration.bitcoin.rpc_password);

    let client = Client::new(&rpc_url, rpc_auth.clone())?;

    // Ensure wallets exist (but do NOT mine or import descriptors)
    setup_wallet(&client, "mining_wallet")?;
    setup_wallet(&client, "student")?;

    Ok((client, rpc_url, rpc_auth))
}

fn init_database() -> Pool<Postgres> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let db_url = configuration.database.connection_string();
    PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to Postgres.")
}

fn init_api_listener() -> TcpListener{
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    listener
}