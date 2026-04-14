use crate::api::wrap_response;
use crate::services::ExecutionCtx;
use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use crate::domain::scheduler::SchedulerLog;

#[derive(Debug, Deserialize)]
pub struct StartSchedulerParams {
    pub interval_minutes: Option<u64>,
    pub blocks_count: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct SchedulerStatus {
    pub running: bool,
}

#[utoipa::path(
    post,
    path = "/api/v1/scheduler/start",
    responses((status = 200, description = "Scheduler started")),
    tag = "Scheduler"
)]
pub async fn start_scheduler(
    state: web::Data<AppState>, params: web::Query<StartSchedulerParams>,
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

#[utoipa::path(
    post,
    path = "/api/v1/scheduler/stop",
    responses((status = 200, description = "Scheduler stopped")),
    tag = "Scheduler"
)]
pub async fn stop_scheduler(state: web::Data<AppState>) -> impl Responder {
    state.scheduler_service.stop(&state).await;

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Scheduler stopped"
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/scheduler/status",
    responses((status = 200, description = "Current scheduler status")),
    tag = "Scheduler"
)]
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

#[utoipa::path(
    get,
    path = "/api/v1/scheduler/logs",
    responses((status = 200, body = Vec<SchedulerLog>)),
    tag = "Scheduler"
)]
pub async fn get_scheduler_logs(
    state: web::Data<AppState>, mut ctx: ExecutionCtx, params: web::Query<LogsParams>,
) -> impl Responder {
    let length = params.length.unwrap_or(25) as i64;
    let start = params.start.unwrap_or(0) as i64;

    let scheduler_log_service = &state.scheduler_log_service;
    let result: anyhow::Result<_> = async {
        let total = scheduler_log_service
            .count_logs(&mut ctx)
            .await
            .context("Failed to read scheduler logs from db")?;

        let logs = scheduler_log_service
            .list_logs(&mut ctx, length, start)
            .await
            .context("Failed to read scheduler logs from db")?;

        let response = DataTableResponse {
            draw: params.draw,
            records_total: total as usize,
            records_filtered: total as usize,
            data: logs,
        };

        Ok(response)
    }.await;

    wrap_response(result)
}

#[utoipa::path(
    get,
    path = "/api/v1/scheduler/log/{id}",
    params(("id" = Uuid, Path, description = "Log ID")),
    responses((status = 200, body = SchedulerLog), (status = 404)),
    tag = "Scheduler"
)]
pub async fn get_scheduler_log(
    state: web::Data<AppState>, mut ctx: ExecutionCtx, path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    let scheduler_log_service = &state.scheduler_log_service;
    let result: anyhow::Result<_> = async {
        let log = scheduler_log_service
            .get_log(&mut ctx, id)
            .await
            .context("Failed to read scheduler logs from db")?;
        Ok(log)
    }.await;

    wrap_response(result)
}
