#[async_trait]
pub trait Transactional {
    async fn in_transaction<F, T, E, Fut>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(Transaction<'static, Postgres>) -> Fut + Send,
        Fut: Future<Output = Result<(T, Transaction<'static, Postgres>), E>> + Send,
        E: From<sqlx::Error> + Send;
}

#[async_trait]
impl Transactional for PgPool {
    async fn in_transaction<F, T, E, Fut>(&self, f: F) -> Result<T, E> {
        let tx = self.begin().await?;
        let (res, tx) = f(tx).await?;
        tx.commit().await?;
        Ok(res)
    }
}