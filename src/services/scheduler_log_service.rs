use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;
use sqlx::{Error, PgConnection, PgPool};
use uuid::Uuid;
use crate::domain::block::BlockInfo;
use crate::domain::scheduler_log::{SchedulerLog, SyncResult};
use crate::repositories::block_repository::BlockRepository;
use crate::repositories::scheduler_log_repository::SchedulerLogRepository;
use crate::services::bitcoin::rpc::get_blocks_info;
use crate::AppState;
use crate::db::Transactional;

pub struct SchedulerLogService {
    db_pool: PgPool,
    block_repo: Arc<dyn BlockRepository + Send + Sync>,
    log_repo: Arc<dyn SchedulerLogRepository + Send + Sync>,
}

impl SchedulerLogService {
    pub fn new(
        db_pool: PgPool,
        block_repo: Arc<dyn BlockRepository + Send + Sync>,
        log_repo: Arc<dyn SchedulerLogRepository + Send + Sync>,
    ) -> Self {
        Self {
            db_pool,
            block_repo,
            log_repo,
        }
    }

    pub async fn create_log(&self, description: &str) -> Result<Uuid, Error> {
        self.db_pool
            .in_transaction(|mut tx| async move {
                let uuid = self.log_repo.create_log(&mut tx, "On-chain blocks synchronization").await?;
                Ok((uuid, tx))
            })
            .await
    }

    pub async fn update_log_success(&self, id: Uuid, result: &SyncResult) -> Result<(), Error> {
        self.db_pool
            .in_transaction(|mut tx| async move {
                self.log_repo.update_log_success(&mut tx, id, result).await?;
                Ok(((),tx))
            })
            .await
    }

    pub async fn update_log_failure(&self, id: Uuid, error: &str) -> Result<(), Error> {
        self.db_pool
            .in_transaction(|mut tx| async move {
                self.log_repo.update_log_failure(&mut tx, id, error).await?;
                Ok(((),tx))
            })
            .await
    }

    pub async fn list_logs(&self, limit: i64, offset: i64) -> Result<Vec<SchedulerLog>, Error> {
        let mut conn = self.db_pool.acquire().await?;
        let logs = self.log_repo.list_logs(&mut conn, limit, offset).await?;
        Ok(logs)
    }

    pub async fn count_logs(&self) -> Result<i64, Error> {
        let mut conn = self.db_pool.acquire().await?;
        let count = self.log_repo.count_logs(&mut conn).await?;
        Ok(count)
    }

    pub async fn get_log(&self, id: Uuid) -> Result<SchedulerLog, Error> {
        let mut conn = self.db_pool.acquire().await?;
        let log = self.log_repo.get_log(&mut conn, id).await?;
        Ok(log)
    }

}