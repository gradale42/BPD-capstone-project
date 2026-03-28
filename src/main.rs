// src/main.rs
use actix_files as fs;
use actix_web::{App, HttpServer};
use std::io;
use std::sync::Arc;
use bpd_capstone_project::bitcoin::rpc::{connect, mine_until_positive_balance, setup_mining_address, setup_mining_wallet};

mod api;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let bitcoin_rpc = match init_bitcoin() {
        Ok(rpc) => {
            println!("✅ Bitcoin RPC client initialized successfully");
            Some(Arc::new(rpc))
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
    println!("💡 Tip: Use real endpoints for live data, mock endpoints for testing\n");

    // Создаем замыкание, которое будет возвращать App
    HttpServer::new(move || {
        // Сначала создаем App с конфигурацией API
        let mut app = App::new();

        // Конфигурируем API (передаем RPC клиент)
        app = app.configure(|cfg| api::config(cfg, bitcoin_rpc.clone()));

        // Затем добавляем статические файлы
        app = app
            .service(fs::Files::new("/ui", "./ui").show_files_listing())
            .service(fs::Files::new("/", "./ui").index_file("index.html"));

        app
    })
        .bind("127.0.0.1:3000")?
        .run()
        .await
}

fn init_bitcoin() -> Result<bitcoincore_rpc::Client, Box<dyn std::error::Error>> {
    println!("=== Starting bitcoin client ===");
    let bitcoin_rpc = connect()?;

    setup_mining_wallet(&bitcoin_rpc)?;
    let mining_address = setup_mining_address(&bitcoin_rpc)?;
    mine_until_positive_balance(&bitcoin_rpc, &mining_address)?;

    Ok(bitcoin_rpc)
}