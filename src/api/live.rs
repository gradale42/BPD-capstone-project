use crate::api::read_json_file;
use actix_web::HttpResponse;

pub async fn get_live() -> HttpResponse {
    println!("📡 Handling /api/live request");
    read_json_file("live.json").await
}