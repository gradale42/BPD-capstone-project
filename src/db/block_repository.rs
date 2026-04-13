use crate::configuration::Network;
use crate::domain::block::{BlockInfo, TimeseriesPoint};
use async_trait::async_trait;
use sqlx::{Error, PgConnection};

#[async_trait]
pub trait BlockRepository: Send + Sync {
    async fn find_by_height(
        &self, conn: &mut PgConnection, network: Network, height: i64,
    ) -> Result<BlockInfo, Error>;

    async fn find_by_hash(
        &self, conn: &mut PgConnection, network: Network, hash: &str,
    ) -> Result<BlockInfo, Error>;

    async fn save(
        &self, conn: &mut PgConnection, network: Network, user: BlockInfo,
    ) -> Result<(), Error>;

    async fn list_blocks(
        &self, conn: &mut PgConnection, network: Network, limit: i64, offset: i64, order_by: &str,
    ) -> Result<Vec<BlockInfo>, Error>;

    async fn count_blocks(&self, conn: &mut PgConnection, network: Network) -> Result<i64, Error>;

    async fn get_last_block_height(
        &self, conn: &mut PgConnection, network: Network,
    ) -> Result<Option<i64>, sqlx::Error>;

    async fn get_timeseries(
        &self, conn: &mut PgConnection, network: Network, from: i64, to: i64,
    ) -> Result<Vec<TimeseriesPoint>, Error>;
}

pub struct PostgresBlockRepository;

#[async_trait]
impl BlockRepository for PostgresBlockRepository {
    async fn find_by_height(
        &self, conn: &mut PgConnection, network: Network, height: i64,
    ) -> Result<BlockInfo, Error> {
        sqlx::query_as::<_, BlockInfo>(
            r#"SELECT
               network,
               height,
               hash,
               EXTRACT(EPOCH FROM time)::BIGINT as time, -- Convert to i64
               tx_count,
               size,
               weight,
               subsidy_sat,
               total_fees_sat,
               avg_fee_sat,
               avg_feerate,
               difficulty,
               indexed_at
            FROM block_info WHERE height = $1 AND network = $2::bitcoin_network"#,
        )
        .bind(height)
        .bind(network.as_str())
        .fetch_one(conn)
        .await
    }

    async fn find_by_hash(
        &self, conn: &mut PgConnection, network: Network, hash: &str,
    ) -> Result<BlockInfo, Error> {
        sqlx::query_as::<_, BlockInfo>(
            r#"SELECT
               network,
               height,
               hash,
               EXTRACT(EPOCH FROM time)::BIGINT as time, -- Convert to i64
               tx_count,
               size,
               weight,
               subsidy_sat,
               total_fees_sat,
               avg_fee_sat,
               avg_feerate,
               difficulty,
               indexed_at
            FROM block_info WHERE hash = $1 AND network = $2::bitcoin_network"#,
        )
        .bind(hash)
        .bind(network.as_str())
        .fetch_one(conn)
        .await
    }

    async fn save(
        &self, conn: &mut PgConnection, network: Network, block: BlockInfo,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"INSERT INTO block_info (
            network, height, hash, time, tx_count, size, weight,
            subsidy_sat, total_fees_sat, avg_fee_sat, avg_feerate, difficulty
        )
        VALUES ($1::bitcoin_network, $2, $3, to_timestamp($4), $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT (network, height) DO NOTHING"#,
        )
        .bind(network.as_str())
        .bind(block.height)
        .bind(block.hash)
        .bind(block.time)
        .bind(block.tx_count)
        .bind(block.size)
        .bind(block.weight)
        .bind(block.subsidy_sat)
        .bind(block.total_fees_sat)
        .bind(block.avg_fee_sat)
        .bind(block.avg_feerate)
        .bind(block.difficulty)
        .execute(conn)
        .await?;

        Ok(())
    }

    async fn list_blocks(
        &self, conn: &mut PgConnection, network: Network, limit: i64, offset: i64, order_by: &str,
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
               network,
               height,
               hash,
               EXTRACT(EPOCH FROM time)::BIGINT as time, -- Convert to i64
               tx_count,
               size,
               weight,
               subsidy_sat,
               total_fees_sat,
               avg_fee_sat,
               avg_feerate,
               difficulty,
               indexed_at
            FROM block_info
            WHERE network = $1::bitcoin_network
            ORDER BY {}
            LIMIT $2 OFFSET $3"#,
            order_clause
        );

        let rows = sqlx::query_as::<_, BlockInfo>(&query)
            .bind(network.as_str())
            .bind(limit)
            .bind(offset)
            .fetch_all(conn)
            .await?;

        Ok(rows)
    }

    async fn count_blocks(&self, conn: &mut PgConnection, network: Network) -> Result<i64, Error> {
        let count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM block_info WHERE network = $1::bitcoin_network",
        )
        .bind(network.as_str())
        .fetch_one(conn)
        .await?;
        Ok(count)
    }

    async fn get_last_block_height(
        &self, conn: &mut PgConnection, network: Network,
    ) -> Result<Option<i64>, sqlx::Error> {
        let height: Option<i64> = sqlx::query_scalar(
            "SELECT height FROM block_info WHERE network = $1::bitcoin_network ORDER BY height DESC LIMIT 1",
        )
        .bind(network.as_str())
        .fetch_optional(conn)
        .await?;

        Ok(height)
    }

    async fn get_timeseries(
        &self, conn: &mut PgConnection, network: Network, from: i64, to: i64,
    ) -> Result<Vec<TimeseriesPoint>, Error> {
        let points = sqlx::query_as::<_, TimeseriesPoint>(
            r#"
            SELECT
                EXTRACT(EPOCH FROM time)::BIGINT as time, -- Convert to i64
                tx_count,
                avg_fee_sat,
                avg_feerate
            FROM block_info
            WHERE network = $1::bitcoin_network AND time BETWEEN to_timestamp($2) AND to_timestamp($3)
            ORDER BY time ASC
            "#,
        )
        .bind(network.as_str())
        .bind(from)
        .bind(to)
        .fetch_all(conn)
        .await?;
        Ok(points)
    }
}
