use std::future::Future;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};

#[async_trait]
pub trait Transactional {
    async fn in_transaction<'a, F, T, E, Fut>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(sqlx::Transaction<'static, sqlx::Postgres>) -> Fut + Send,
        Fut: Future<Output = Result<(T, sqlx::Transaction<'static, sqlx::Postgres>), E>> + Send,
        E: From<sqlx::Error> + Send; // Добавили Send и From
}

#[async_trait]
impl Transactional for sqlx::PgPool {
    async fn in_transaction<'a, F, T, E, Fut>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(sqlx::Transaction<'static, sqlx::Postgres>) -> Fut + Send,
        Fut: Future<Output = Result<(T, sqlx::Transaction<'static, sqlx::Postgres>), E>> + Send,
        E: From<sqlx::Error> + Send,
    {
        let tx = self.begin().await?;
        let (res, tx) = f(tx).await?;
        tx.commit().await?;
        Ok(res)
    }
}
