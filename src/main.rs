use crate::api::{get_blocks, get_historical, get_live, get_mempool, get_peers};
use actix_files as fs;
use actix_web::{web, App, HttpServer};
use std::io;

mod api;

#[actix_web::main]
async fn main() -> io::Result<()> {
    println!("Server running on http://127.0.0.1:3000");
    println!("Serving static JSON from ./resources folder");

    // ВРЕМЕННО: проверим, что модуль загружен
    println!("🔍 Testing api module...");
    let _ = api::config; // Просто проверяем, что символ существует

    HttpServer::new(|| {
        App::new()
            .configure(api::config)
            .service(fs::Files::new("/ui", "./ui").show_files_listing())
            .service(fs::Files::new("/", "./ui").index_file("index.html"))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}

/*
#[actix_web::main]
async fn main() -> io::Result<()> {
    println!("🚀 Server running on http://127.0.0.1:3000");
    println!("📁 Serving static JSON from ./resources folder");
    println!("📡 Available endpoints:");
    println!("   - GET /api/blocks");
    println!("   - GET /api/mempool");
    println!("   - GET /api/peers");
    println!("   - GET /api/live");
    println!("   - GET /api/stats/historical");
    println!("   - GET /ui/ (static files)");

    // ВРЕМЕННО: проверим, что модуль загружен
    println!("🔍 Testing api module...");
    let _ = api::config;  // Просто проверяем, что символ существует



    HttpServer::new(|| {
        App::new()

            .configure(api::config)
            // API endpoints
            .route("/api/blocks", web::get().to(get_blocks))
            .route("/api/mempool", web::get().to(get_mempool))
            .route("/api/peers", web::get().to(get_peers))
            .route("/api/live", web::get().to(get_live))
            .route("/api/stats/historical", web::get().to(get_historical))

            // Static files
            .service(fs::Files::new("/ui", "./ui").show_files_listing())
            .service(fs::Files::new("/", "./ui").index_file("index.html"))
    })
        .bind("127.0.0.1:3000")?
        .run()
        .await
}


 */
