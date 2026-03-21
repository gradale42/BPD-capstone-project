use crate::api::read_json_file;
use actix_web::HttpResponse;

pub async fn get_mempool() -> HttpResponse {
    println!("📡 Handling /api/mempool request");
    read_json_file("mempool.json").await
}