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
use crate::services::ExecutionCtx;

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
                println!("Running scheduled blockchain synchronization...");
                Self::run_sync(&state_clone , blocks_count).await;
                Self::run_mempool_sync(&state_clone ).await;
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

        println!("Saving blocks...");

        let network = state.node_manager.get_current_network();
        let mut ctx = match ExecutionCtx::new(&state.db_pool, network).await {
            Ok(context) => context,
            Err(e) => {
                eprintln!("Failed to acquire DB connection: {}", e);
                return;
            }
        };

        let log_id =
            match state.scheduler_log_service.create_log(&mut ctx, "On-chain blocks synchronization").await {
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
                    state.scheduler_log_service.update_log_failure(&mut ctx, log_id, &e.to_string()).await;
                return;
            }
        };

        for block in blocks {
            match state.block_service.find_by_height(&mut ctx, block.height).await {
                Ok(_) => {
                    result.skipped += 1;
                    result.skipped_blocks.push(block.hash);
                }
                Err(sqlx::Error::RowNotFound) => {
                    if let Err(e) = state.block_service.save(&mut ctx, block.clone()).await {
                        eprintln!("Failed to save block {}: {}", block.height, e);
                        result.error = Some(e.to_string());
                        let _ = state
                            .scheduler_log_service
                            .update_log_failure(&mut ctx, log_id, &e.to_string())
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
                        .update_log_failure(&mut ctx, log_id, &e.to_string())
                        .await;
                    return;
                }
            }
        }

        if result.error.is_none() {
            if let Err(e) = state.scheduler_log_service.update_log_success(&mut ctx, log_id, &result).await {
                eprintln!("Failed to update log: {}", e);
            } else {
                println!("Sync completed: saved {}, skipped {}", result.saved, result.skipped);
            }
        }
    }

    async fn run_mempool_sync(state: &AppState) {

        println!("Saving mempool...");

        let network = state.node_manager.get_current_network();
        let mut ctx = match ExecutionCtx::new(&state.db_pool, network).await {
            Ok(context) => context,
            Err(e) => {
                eprintln!("Failed to acquire DB connection for mempool sync: {}", e);
                return;
            }
        };

        // Fetch current mempool stats via RPC
        let mempool_info = match state.node_manager.get_current_client().get_mempool_info() {
            Ok(info) => info,
            Err(e) => {
                eprintln!("Failed to get mempool info: {}", e);
                return;
            }
        };

        // Calculate fee rate statistics
        let mempool_tx_ids = match state.node_manager.get_rpc_client().get_raw_mempool() {
            Ok(tx_ids) => tx_ids,
            Err(e) => {
                eprintln!("Failed to get mempool transactions: {}", e);
                return;
            }
        };

        let mut total_feerate = 0.0;
        let mut min_feerate = f64::MAX;
        let mut max_feerate = f64::MIN;
        let mut count = 0;

        // Sample first 1000 transactions for performance
        let sample_size = std::cmp::min(1000, mempool_tx_ids.len());
        for txid in mempool_tx_ids.iter().take(sample_size) {
            if let Ok(tx) = state.node_manager.get_rpc_client().get_raw_transaction_info(txid, None) {
                let vsize = tx.vsize as f64;
                if vsize > 0.0 {
                    let fee_rate = (tx.fee as f64) / vsize;
                    total_feerate += fee_rate;
                    count += 1;
                    if fee_rate < min_feerate { min_feerate = fee_rate; }
                    if fee_rate > max_feerate { max_feerate = fee_rate; }
                }
            }
        }

        let avg_feerate = if count > 0 { Some(total_feerate / count as f64) } else { None };
        let min_feerate_opt = if min_feerate != f64::MAX { Some(min_feerate) } else { None };
        let max_feerate_opt = if max_feerate != f64::MIN { Some(max_feerate) } else { None };

        // Save metrics to database
        if let Err(e) = state.mempool_metrics_service.save_metrics(
            &mut ctx,
            mempool_info.size as i32,
            mempool_info.bytes as i64,
            mempool_info.total_fee as f64 / 100_000_000.0, // Convert sat to BTC
            min_feerate_opt,
            max_feerate_opt,
            avg_feerate,
        ).await {
            eprintln!("Failed to save mempool metrics: {}", e);
        } else {
            println!("Mempool metrics saved successfully");
        }
    }
}
