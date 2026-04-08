use crate::db::Transactional;
use crate::domain::block::BlockInfo;
use crate::domain::scheduler_log::{SchedulerLog, SyncResult};
use crate::repositories::block_repository::BlockRepository;
use crate::repositories::scheduler_log_repository::SchedulerLogRepository;
use crate::services::bitcoin::rpc::get_blocks_info;
use crate::AppState;
use sqlx::{Error, PgConnection, PgPool};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;
use uuid::Uuid;

#[derive()]
pub struct SchedulerService {
    is_running: Arc<Mutex<bool>>,
    task_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl SchedulerService {
    pub fn new() -> Self {
        Self { is_running: Arc::new(Mutex::new(false)), task_handle: Arc::new(Mutex::new(None)) }
    }

    pub async fn start(self: Arc<Self>, state: Arc<AppState>, interval_minutes: u64, blocks_count: u64) {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            println!("Scheduler is already running");
            return;
        }
        *is_running = true;
        drop(is_running);

        let is_running_clone = self.is_running.clone();
        let state_clone  = state.clone();

        let handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(interval_minutes * 60));

            while *is_running_clone.lock().await {
                interval.tick().await;
                println!("Running scheduled blocks synchronization...");
                Self::run_sync(&state_clone , blocks_count).await;
            }
        });

        let mut task_handle = self.task_handle.lock().await;
        *task_handle = Some(handle);
    }

    pub async fn stop(&self, state: &AppState) {
        let mut is_running = self.is_running.lock().await;
        if !*is_running {
            println!("Scheduler is not running");
            return;
        }
        *is_running = false;
        drop(is_running);

        let mut task_handle = self.task_handle.lock().await;
        if let Some(handle) = task_handle.take() {
            handle.abort();
        }
        println!("Scheduler stopped");
    }

    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }

    async fn run_sync(state: &AppState, blocks_count: u64) {
        let mut conn = match state.db_pool.acquire().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Failed to acquire DB connection: {}", e);
                return;
            }
        };

        let log_id =
            match state.scheduler_log_service.create_log("On-chain blocks synchronization").await {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("Failed to create log: {}", e);
                    return;
                }
            };

        let mut result = SyncResult {
            saved: 0,
            saved_blocks: Vec::new(),
            skipped: 0,
            skipped_blocks: Vec::new(),
            error: None,
        };

        println!("Fetching {} blocks from RPC...", blocks_count);

        let blocks = match get_blocks_info(state, Some(blocks_count)).await {
            Ok(blocks) => blocks,
            Err(e) => {
                result.error = Some(e.to_string());
                let _ =
                    state.scheduler_log_service.update_log_failure(log_id, &e.to_string()).await;
                return;
            }
        };

        for block in blocks {
            match state.block_service.find_by_height(block.height).await {
                Ok(_) => {
                    result.skipped += 1;
                    result.skipped_blocks.push(block.hash);
                }
                Err(sqlx::Error::RowNotFound) => {
                    if let Err(e) = state.block_service.save(block.clone()).await {
                        eprintln!("Failed to save block {}: {}", block.height, e);
                        result.error = Some(e.to_string());
                        let _ = state
                            .scheduler_log_service
                            .update_log_failure(log_id, &e.to_string())
                            .await;
                        return;
                    }
                    result.saved += 1;
                    result.saved_blocks.push(block.hash);
                }
                Err(e) => {
                    result.error = Some(e.to_string());
                    let _ = state
                        .scheduler_log_service
                        .update_log_failure(log_id, &e.to_string())
                        .await;
                    return;
                }
            }
        }

        if result.error.is_none() {
            if let Err(e) = state.scheduler_log_service.update_log_success(log_id, &result).await {
                eprintln!("Failed to update log: {}", e);
            } else {
                println!("Sync completed: saved {}, skipped {}", result.saved, result.skipped);
            }
        }
    }
}
