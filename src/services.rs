use std::future::Future;
use async_trait::async_trait;
use futures::future::BoxFuture;
use sqlx::Acquire;
use crate::configuration::Network;

pub mod bitcoin;
pub mod block_service;
pub mod scheduler;
pub mod scheduler_log_service;

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
}

#[async_trait]
pub trait Transactional {
    async fn in_transaction<'a, F, T, E>(&'a mut self, f: F) -> Result<T, E>
    where
        F: for<'c> FnOnce(&'c mut sqlx::PgConnection) -> BoxFuture<'c, Result<T, E>> + Send + 'a,
        E: From<sqlx::Error> + Send + 'a,
        T: Send + 'a;
}

#[async_trait]
impl Transactional for ExecutionCtx {
    async fn in_transaction<'a, F, T, E>(&'a mut self, f: F) -> Result<T, E>
    where
        F: for<'c> FnOnce(&'c mut sqlx::PgConnection) -> BoxFuture<'c, Result<T, E>> + Send + 'a,
        E: From<sqlx::Error> + Send + 'a,
        T: Send + 'a,
    {
        let mut tx = self.conn.begin().await?;

        let res = f(&mut *tx).await;

        match res {
            Ok(output) => {
                tx.commit().await?;
                Ok(output)
            }
            Err(e) => Err(e)
        }
    }
}
