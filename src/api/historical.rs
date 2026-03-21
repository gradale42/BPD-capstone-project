use crate::api::read_json_file;
use actix_web::HttpResponse;

pub async fn get_historical() -> HttpResponse {
    println!("📡 Handling /api/stats/historical request");
    read_json_file("historical.json").await
}