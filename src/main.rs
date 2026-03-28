use actix_files as fs;
use actix_web::{App, HttpServer};
use bpd_capstone_project::bitcoin::rpc::{connect, setup_wallet};
use std::io;
use std::sync::Arc;
use bitcoincore_rpc::{Auth, Client};

mod api;
mod bitcoin;

const RPC_URL: &str = "http://localhost:18443";
const RPC_USER: &str = "alice";
const RPC_PASS: &str = "password";

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Initialize Bitcoin node connection and wallets
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

    println!("\n🚀 Server running on http://127.0.0.1:3000");
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
    let rpc_url = RPC_URL.to_string();
    let rpc_auth = Auth::UserPass(RPC_USER.to_string(), RPC_PASS.to_string());

    let client = Client::new(&rpc_url, rpc_auth.clone())?;

    // Ensure wallets exist (but do NOT mine or import descriptors)
    setup_wallet(&client, "mining_wallet")?;
    setup_wallet(&client, "student")?;

    Ok((client, rpc_url, rpc_auth))
}