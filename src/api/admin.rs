use actix_web::{web, HttpResponse, Responder};
use bitcoincore_rpc::{Client, RpcApi};
use serde_json::json;
use bpd_capstone_project::AppState;
use crate::bitcoin::rpc::{import_descriptors, setup_mining_address};

#[derive(Debug, serde::Deserialize)]
pub struct MineParams {
    pub count: u32,
}

pub async fn import_descriptors_handler(state: web::Data<AppState>) -> impl Responder {
    // Build wallet client for "student"
    match state.get_bitcoin_client("student").lock() {
        Ok(student_client) => {
            match import_descriptors(&student_client) {
                Ok(_) => HttpResponse::Ok().json(json!({
                    "status": "success",
                    "message": "Descriptors imported successfully"
                })),
                Err(e) => HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Import failed: {}", e)
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Cannot connect to student wallet: {}", e)
        })),
    }
}

pub async fn mine_blocks_handler(state: web::Data<AppState>, query: web::Query<MineParams>) -> impl Responder {
    let count = query.count;
    if count == 0 {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Count must be positive"
        }));
    }

    match state.get_bitcoin_client("mining_wallet").lock() {
        Ok(mining_client) => {
            match setup_mining_address(&mining_client) {
                Ok(mining_address) => {
                    match mining_client.generate_to_address(count as u64, &mining_address) {
                        Ok(block_hashes) => HttpResponse::Ok().json(json!({
                            "status": "success",
                            "message": format!("Mined {} blocks", count),
                            "blocks": block_hashes,
                        })),
                        Err(e) => HttpResponse::InternalServerError().json(json!({
                            "status": "error",
                            "message": format!("Mining failed: {}", e)
                        })),
                    }
                }
                Err(e) => HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Failed to get mining address: {}", e)
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Cannot connect to mining wallet: {}", e)
        })),
    }
}