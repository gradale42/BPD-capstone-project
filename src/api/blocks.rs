use crate::api::read_json_file;
use actix_web::HttpResponse;

pub async fn get_blocks() -> HttpResponse {
    println!("📡 Handling /api/blocks request");
    read_json_file("blocks.json").await
}