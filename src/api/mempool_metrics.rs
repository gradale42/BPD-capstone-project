// src/api/mempool_metrics.rs
use crate::domain::mempool_metrics::MempoolTimeRange;
use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use crate::services::ExecutionCtx;

pub async fn get_mempool_timeseries(
    state: web::Data<AppState>,
    query: web::Query<MempoolTimeRange>,
) -> impl Responder {
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

    match state.mempool_metrics_service.get_timeseries(&mut ctx, query.from, query.to).await {
        Ok(points) => {
            println!("Mempool timeseries: found {} points", points.len());
            HttpResponse::Ok().json(points)
        },
        Err(e) => {
            eprintln!("Mempool timeseries error: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to fetch mempool timeseries: {}", e)
            }))
        }
    }
}