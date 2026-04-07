use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct StartSchedulerParams {
    pub interval_minutes: Option<u64>,
    pub blocks_count: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct SchedulerStatus {
    pub running: bool,
}

pub async fn start_scheduler(
    state: web::Data<AppState>,
    params: web::Query<StartSchedulerParams>,
) -> impl Responder {
    let interval = params.interval_minutes.unwrap_or(1);
    let blocks_count = params.blocks_count.unwrap_or(100);

    let scheduler_service = state.scheduler_service.clone();
    let app_state_arc = state.into_inner();

    scheduler_service.start(app_state_arc, interval, blocks_count).await;

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": format!("Scheduler started with interval {} minutes", interval)
    }))
}

pub async fn stop_scheduler(state: web::Data<AppState>) -> impl Responder {
    state.scheduler_service.stop(&state).await;

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Scheduler stopped"
    }))
}

pub async fn get_scheduler_status(state: web::Data<AppState>) -> impl Responder {
    let running = state.scheduler_service.is_running().await;

    HttpResponse::Ok().json(SchedulerStatus { running })
}

#[derive(Debug, Deserialize)]
pub struct LogsParams {
    pub draw: i32,
    pub start: Option<usize>,
    pub length: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct DataTableResponse<T> {
    pub draw: i32,
    pub records_total: usize,
    pub records_filtered: usize,
    pub data: Vec<T>,
}

pub async fn get_scheduler_logs(
    state: web::Data<AppState>,
    params: web::Query<LogsParams>,
) -> impl Responder {
    let length = params.length.unwrap_or(25) as i64;
    let start = params.start.unwrap_or(0) as i64;

    let total = match state.scheduler_log_service.count_logs().await {
        Ok(count) => count as usize,
        Err(e) => {
            eprintln!("Failed to count logs: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Database error: {}", e)
            }));
        }
    };

    let logs = match state.scheduler_log_service.list_logs(length, start).await {
        Ok(logs) => logs,
        Err(e) => {
            eprintln!("Failed to list logs: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Database error: {}", e)
            }));
        }
    };

    let response = DataTableResponse {
        draw: params.draw,
        records_total: total,
        records_filtered: total,
        data: logs,
    };

    HttpResponse::Ok().json(response)
}

pub async fn get_scheduler_log(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    match state.scheduler_log_service.get_log(id).await {
        Ok(log) => HttpResponse::Ok().json(log),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(json!({
            "error": "Log not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Database error: {}", e)
        })),
    }
}