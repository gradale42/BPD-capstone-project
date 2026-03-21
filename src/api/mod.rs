pub mod blocks;
pub mod mempool;
pub mod peers;
pub mod historical;
pub mod live;

use actix_web::{web, HttpResponse};
use std::fs;
use std::path::Path;
pub use blocks::get_blocks;
pub use mempool::get_mempool;
pub use peers::get_peers;
pub use live::get_live;
pub use historical::get_historical;

/// Configure API routes
pub fn config(cfg: &mut web::ServiceConfig) {

    println!("⚙️  Configuring API routes...");  // <-- ДОБ

    cfg.service(
        web::scope("/api")
            .route("/blocks", web::get().to(get_blocks))
            .route("/mempool", web::get().to(get_mempool))
            .route("/peers", web::get().to(get_peers))
            .route("/live", web::get().to(get_live))
            .route("/stats/historical", web::get().to(get_historical))
    );

    println!("✅ API routes configured");  // <-- И ЭТО
}

/// Helper function to read JSON file
pub async fn read_json_file(filename: &str) -> HttpResponse {
    let path = Path::new("./resources").join(filename);
    println!("📖 Reading file: {:?}", path);  // <-- И ЭТО

    match fs::read_to_string(path) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => {
                    println!("✅ Successfully read {}", filename);
                    HttpResponse::Ok().json(json)
                },
                Err(e) => {
                    println!("❌ Invalid JSON in {}: {}", filename, e);
                    HttpResponse::InternalServerError().body("Invalid JSON format")
                }
            }
        }
        Err(e) => {
            println!("❌ File not found {}: {}", filename, e);
            HttpResponse::NotFound().body("File not found")
        }
    }
}