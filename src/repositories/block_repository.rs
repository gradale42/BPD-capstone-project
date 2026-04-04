use crate::domain::block::BlockInfo;
use async_trait::async_trait;
use sqlx::{Error, PgConnection, PgPool};
use uuid::{uuid, Uuid};

#[async_trait]
pub trait BlockRepository: Send + Sync {
    async fn find_by_height(
        &self, conn: &mut PgConnection, height: i64,
    ) -> Result<BlockInfo, Error>;
    async fn find_by_hash(&self, conn: &mut PgConnection, hash: &str) -> Result<BlockInfo, Error>;
    async fn save(&self, conn: &mut PgConnection, user: BlockInfo) -> Result<(), Error>;
    async fn list_blocks(
        &self, conn: &mut PgConnection, limit: i64, offset: i64, order_by: &str,
    ) -> Result<Vec<BlockInfo>, Error>;
    async fn count_blocks(&self, conn: &mut PgConnection) -> Result<i64, Error>;
}

pub struct PostgresBlockRepository;

#[async_trait]
impl BlockRepository for PostgresBlockRepository {
    async fn find_by_height(
        &self, conn: &mut PgConnection, height: i64,
    ) -> Result<BlockInfo, Error> {
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

    async fn list_blocks(
        &self, conn: &mut PgConnection, limit: i64, offset: i64, order_by: &str,
    ) -> Result<Vec<BlockInfo>, Error> {
        // white list for SQL injection protection
        let order_clause = match order_by {
            "height ASC" => "height ASC",
            "height DESC" => "height DESC",
            "time ASC" => "time ASC",
            "time DESC" => "time DESC",
            "tx_count ASC" => "tx_count ASC",
            "tx_count DESC" => "tx_count DESC",
            _ => "height DESC",
        };

        let query = format!(
            r#"SELECT
               height, hash, time, tx_count, size, weight,
               subsidy_sat, total_fees_sat, avg_fee_sat, avg_feerate, difficulty, indexed_at
            FROM block_info
            ORDER BY {}
            LIMIT $1 OFFSET $2"#,
            order_clause
        );

        let rows =
            sqlx::query_as::<_, BlockInfo>(&query).bind(limit).bind(offset).fetch_all(conn).await?;

        Ok(rows)
    }

    async fn count_blocks(&self, conn: &mut PgConnection) -> Result<i64, Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM block_info").fetch_one(conn).await?;
        Ok(row.0)
    }
}
