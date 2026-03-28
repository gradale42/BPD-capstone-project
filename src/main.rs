use actix_files as fs;
use actix_web::{App, HttpServer};
use bpd_capstone_project::bitcoin::rpc::{connect, connect_to_wallet, import_descriptors, mine_until_positive_balance, setup_mining_address, setup_wallet};
use std::io;
use std::sync::Arc;

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

    HttpServer::new(move || {
        let mut app = App::new();
        app = app.configure(|cfg| api::config(cfg, bitcoin_rpc.clone()));
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
    let base_rpc = connect()?;

    setup_wallet(&base_rpc, "mining_wallet")?;
    let mining_rpc = connect_to_wallet("mining_wallet")?; 

    let mining_address = setup_mining_address(&mining_rpc)?;
    mine_until_positive_balance(&mining_rpc, &mining_address)?;

    setup_wallet(&base_rpc, "student")?;
    let student_rpc = connect_to_wallet("student")?;

    import_descriptors(&student_rpc)?;

    Ok(base_rpc)
}
