use actix_web::{web, HttpResponse, Responder};
use bitcoincore_rpc::{RpcApi};
use serde_json::json;
use crate::AppState;
use crate::bitcoin::rpc::{get_blocks_info, import_descriptors, setup_mining_address};
use crate::services::ExecutionCtx;

#[derive(Debug, serde::Deserialize)]
pub struct MineParams {
    pub count: u32,
}

pub async fn import_descriptors_handler(state: web::Data<AppState>) -> impl Responder {

    // Build wallet client for "student"
    let self1 = &state.node_manager;
    match self1.get_current_client("student").lock() {
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

    let self1 = &state.node_manager;
    match self1.get_current_client("miner_wallet").lock() {
        Ok(mining_client) => {
            match setup_mining_address(&mining_client) {
                Ok(mining_address) => {
                    println!("\n=== Start up mining {} blocks to address {} ===", count, mining_address);
                    let response = match mining_client.generate_to_address(count as u64, &mining_address) {
                        Ok(block_hashes) => {
                            println!("Mined blocks count: {}", block_hashes.len());
                               HttpResponse::Ok().json(json!({
                                "status": "success",
                                "message": format!("Mined {} blocks", count),
                                "blocks": block_hashes,
                            }))
                        },
                        Err(e) => {
                            println!("❌ Mining error: {}", e);
                            HttpResponse::InternalServerError().json(json!({
                                "status": "error",
                                "message": format!("Mining failed: {}", e)
                            }))
                        }
                    };
                    println!("\n=== Done ===");
                    response
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
pub async fn save_blocks(state: web::Data<AppState>) -> impl Responder {
    const DEFAULT_COUNT: u64 = 500;
    let blocks = match get_blocks_info(&state.clone(), Some(DEFAULT_COUNT)).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("RPC error: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Failed to fetch blocks: {}", e)
            }));
        }
    };

    let network = state.node_manager.get_current_network();
    let mut ctx = match ExecutionCtx::new(&state.db_pool, network).await {
        Ok(context) => context,
        Err(e) => {
            eprintln!("Failed to acquire DB connection: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Database error: {}", e)
            }));
        }
    };

    match state.block_service.save_blocks_ignore_duplicates(& mut ctx, blocks).await {
        Ok((saved, skipped)) => {
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": format!("Saved {} new blocks, skipped {} existing", saved, skipped)
            }))
        }
        Err(e) => {
            eprintln!("DB error: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Failed to save blocks: {}", e)
            }))
        }
    }
}