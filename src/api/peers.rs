use crate::api::read_json_file;
use actix_web::HttpResponse;

pub async fn get_peers() -> HttpResponse {
    println!("📡 Handling /api/peers request");
    read_json_file("peers.json").await
}