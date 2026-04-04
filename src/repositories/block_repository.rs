use async_trait::async_trait;
use sqlx::{Error, PgConnection, PgPool};
use uuid::{uuid, Uuid};
use crate::domain::block::BlockInfo;

#[async_trait]
pub trait BlockRepository: Send + Sync {
    async fn find_by_height(&self, conn: &mut PgConnection, height: i64) -> Result<BlockInfo, Error>;
    async fn find_by_hash(&self, conn: &mut PgConnection, hash: &str) -> Result<BlockInfo, Error>;
    async fn save(&self, conn: &mut PgConnection, user: BlockInfo) -> Result<(), Error>;
}

pub struct PostgresBlockRepository;

#[async_trait]
impl BlockRepository for PostgresBlockRepository {
    async fn find_by_height(&self, conn: &mut PgConnection, height: i64) -> Result<BlockInfo, Error> {
        sqlx::query_as!(BlockInfo, r#"SELECT * FROM block_info WHERE height = $1"#, height)
            .fetch_one(conn)
            .await
    }

    async fn find_by_hash(&self, conn: &mut PgConnection, hash: &str) -> Result<BlockInfo, Error> {
        sqlx::query_as!(BlockInfo, r#"SELECT * FROM block_info WHERE hash = $1"#, hash)
            .fetch_one(conn)
            .await
    }
    async fn save(&self, conn: &mut PgConnection, block: BlockInfo) -> Result<(), Error> {
        sqlx::query!(
            r#"INSERT INTO block_info (
                height,
                hash,
                time,
                tx_count,
                size,
                weight,
                subsidy_sat,
                total_fees_sat,
                avg_fee_sat,
                avg_feerate,
                difficulty
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#,
            block.height,
            block.hash,
            block.time,
            block.tx_count,
            block.size,
            block.weight,
            block.subsidy_sat,
            block.total_fees_sat,
            block.avg_fee_sat,
            block.avg_feerate,
            block.difficulty
        )
        .execute(conn)
        .await?;

        Ok(())
    }
}