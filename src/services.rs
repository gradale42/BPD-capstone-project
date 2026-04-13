use crate::configuration::Network;
use crate::AppState;
use actix_web::{dev::Payload, error::ErrorInternalServerError, FromRequest, HttpRequest};
use sqlx::pool::PoolConnection;
use sqlx::{Acquire, Postgres};
use std::future::Future;
use std::pin::Pin;

pub mod block_service;
pub mod scheduler_log_service;
pub mod mempool_metrics_service;

pub struct ExecutionCtx {
    pub conn: PoolConnection<Postgres>,
    pub network: Network,
}

impl ExecutionCtx {
    pub async fn new(conn: PoolConnection<Postgres>, network: Network) -> Self {
        Self { conn, network }
    }

    pub async fn from_state(state: &AppState) -> Result<Self, sqlx::Error> {
        let network = state.node_manager.get_current_network();
        let conn = state.db_pool.acquire().await?;

        Ok(Self { conn, network })
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

impl FromRequest for ExecutionCtx {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let state = req.app_data::<actix_web::web::Data<AppState>>()
            .expect("AppState is not configured in Actix-web!")
            .clone();

        Box::pin(async move {
            ExecutionCtx::from_state(&state)
                .await
                .map_err(|e| {
                    eprintln!("Failed to acquire DB connection: {}", e);
                    ErrorInternalServerError(format!("Database error: {}", e))
                })
        })
    }
}
