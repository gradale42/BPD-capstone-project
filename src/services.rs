use std::future::Future;
use sqlx::Acquire;
use crate::configuration::Network;

pub mod block_service;
pub mod scheduler_log_service;
pub mod mempool_metrics_service;

pub struct ExecutionCtx {
    pub conn: sqlx::pool::PoolConnection<sqlx::Postgres>,
    pub network: Network,
}

impl ExecutionCtx {
    pub async fn new(pool: &sqlx::PgPool, network: Network) -> Result<Self, sqlx::Error> {
        let conn = pool.acquire().await?;
        Ok(Self {
            conn,
            network: network,
        })
    }
    
    pub async fn execute_in_transaction<F, T, E>(&mut self, f: F) -> Result<T, E>
    where
        F: for<'c> AsyncFnOnce(&'c mut sqlx::PgConnection) -> Result<T, E> + Send,
        E: From<sqlx::Error> + Send,
        T: Send,
    {
        let mut tx = self.conn.begin().await?;
        let res = f(&mut *tx).await;

        match res {
            Ok(output) => {
                tx.commit().await?;
                
                Ok(output)
            }
            Err(e) => Err(e),
        }
    }
}
