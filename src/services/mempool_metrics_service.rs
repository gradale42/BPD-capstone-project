use crate::configuration::Network;
use crate::domain::mempool_metrics::{MempoolMetrics, MempoolMetricsPoint};
use crate::db::mempool_metrics_repository::MempoolMetricsRepository;
use crate::services::ExecutionCtx;
use sqlx::Error;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

pub struct MempoolMetricsService {
    repo: Arc<dyn MempoolMetricsRepository + Send + Sync>,
}

impl MempoolMetricsService {
    pub fn new(repo: Arc<dyn MempoolMetricsRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    pub async fn save_metrics(
        &self,
        ctx: &mut ExecutionCtx,
        tx_count: i32,
        vbytes: i64,
        total_fees_btc: f64,
        min_feerate: Option<f64>,
        max_feerate: Option<f64>,
        avg_feerate: Option<f64>,
    ) -> Result<(), Error> {
        let metrics = MempoolMetrics {
            id: Uuid::new_v4(),
            network: ctx.network.as_str().to_string(),
            timestamp: Utc::now(),
            tx_count,
            vbytes,
            total_fees_btc,
            min_feerate,
            max_feerate,
            avg_feerate,
            indexed_at: Utc::now(),
        };
        let network = ctx.network;
        ctx.execute_in_transaction(async |tx| {
            self.repo.save(tx, network, &metrics).await?;
            Ok(())
        }).await
    }

    pub async fn get_timeseries(
        &self,
        ctx: &mut ExecutionCtx,
        from: i64,
        to: i64,
    ) -> Result<Vec<MempoolMetricsPoint>, Error> {
        self.repo.get_timeseries(&mut ctx.conn, ctx.network, from, to).await
    }

    pub async fn get_latest(
        &self,
        ctx: &mut ExecutionCtx,
    ) -> Result<Option<MempoolMetrics>, Error> {
        self.repo.get_latest(&mut ctx.conn, ctx.network).await
    }
}