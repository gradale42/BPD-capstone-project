use crate::db::scheduler_log_repository::SchedulerLogRepository;
use crate::domain::scheduler::{SchedulerLog, SyncResult};
use crate::services::ExecutionCtx;
use sqlx::Error;
use std::sync::Arc;
use uuid::Uuid;

pub struct SchedulerLogService {
    repo: Arc<dyn SchedulerLogRepository + Send + Sync>,
}

impl SchedulerLogService {
    pub fn new(repo: Arc<dyn SchedulerLogRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    pub async fn create_log(
        &self, ctx: &mut ExecutionCtx, _description: &str,
    ) -> Result<Uuid, Error> {
        ctx.execute_in_transaction(async |tx| {
            let uuid = self.repo.create_log(tx, "On-chain blocks synchronization").await?;
            Ok(uuid)
        })
        .await
    }

    pub async fn update_log_success(
        &self, ctx: &mut ExecutionCtx, id: Uuid, result: &SyncResult,
    ) -> Result<(), Error> {
        ctx.execute_in_transaction(async |tx| {
            self.repo.update_log_success(tx, id, result).await?;
            Ok(())
        })
        .await
    }

    pub async fn update_log_failure(
        &self, ctx: &mut ExecutionCtx, id: Uuid, error: &str,
    ) -> Result<(), Error> {
        ctx.execute_in_transaction(async |tx| {
            self.repo.update_log_failure(tx, id, error).await?;
            Ok(())
        })
        .await
    }

    pub async fn list_logs(
        &self, ctx: &mut ExecutionCtx, limit: i64, offset: i64,
    ) -> Result<Vec<SchedulerLog>, Error> {
        let logs = self.repo.list_logs(&mut ctx.conn, limit, offset).await?;
        Ok(logs)
    }

    pub async fn count_logs(&self, ctx: &mut ExecutionCtx) -> Result<i64, Error> {
        let count = self.repo.count_logs(&mut ctx.conn).await?;
        Ok(count)
    }

    pub async fn get_log(&self, ctx: &mut ExecutionCtx, id: Uuid) -> Result<SchedulerLog, Error> {
        let log = self.repo.get_log(&mut ctx.conn, id).await?;
        Ok(log)
    }
}
