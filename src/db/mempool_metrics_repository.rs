use crate::configuration::Network;
use crate::domain::mempool::{MempoolMetrics, MempoolMetricsPoint};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Error, PgConnection};

#[async_trait]
pub trait MempoolMetricsRepository: Send + Sync {
    async fn save(
        &self, conn: &mut PgConnection, network: Network, metrics: &MempoolMetrics,
    ) -> Result<(), Error>;

    async fn get_timeseries(
        &self, conn: &mut PgConnection, network: Network, from: i64, to: i64,
    ) -> Result<Vec<MempoolMetricsPoint>, Error>;

    async fn get_latest(
        &self, conn: &mut PgConnection, network: Network,
    ) -> Result<Option<MempoolMetrics>, Error>;

    async fn get_last_timestamp(
        &self, conn: &mut PgConnection, network: Network,
    ) -> Result<Option<DateTime<Utc>>, Error>;
}

pub struct PostgresMempoolMetricsRepository;

#[async_trait]
impl MempoolMetricsRepository for PostgresMempoolMetricsRepository {
    async fn save(
        &self, conn: &mut PgConnection, network: Network, metrics: &MempoolMetrics,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"INSERT INTO mempool_metrics (
                id, network, timestamp, tx_count, vbytes, total_fees_btc,
                min_feerate, max_feerate, avg_feerate
            )
            VALUES ($1, $2::bitcoin_network, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (network, timestamp) DO UPDATE SET
                tx_count = EXCLUDED.tx_count,
                vbytes = EXCLUDED.vbytes,
                total_fees_btc = EXCLUDED.total_fees_btc,
                min_feerate = EXCLUDED.min_feerate,
                max_feerate = EXCLUDED.max_feerate,
                avg_feerate = EXCLUDED.avg_feerate"#,
        )
        .bind(&metrics.id)
        .bind(network.as_str())
        .bind(metrics.timestamp)
        .bind(metrics.tx_count)
        .bind(metrics.vbytes)
        .bind(metrics.total_fees_btc)
        .bind(metrics.min_feerate)
        .bind(metrics.max_feerate)
        .bind(metrics.avg_feerate)
        .execute(conn)
        .await?;

        Ok(())
    }

    async fn get_timeseries(
        &self, conn: &mut PgConnection, network: Network, from: i64, to: i64,
    ) -> Result<Vec<MempoolMetricsPoint>, Error> {
        let points = sqlx::query_as::<_, MempoolMetricsPoint>(
            r#"
            SELECT
                EXTRACT(EPOCH FROM timestamp)::BIGINT as time,
                tx_count,
                vbytes,
                total_fees_btc,
                avg_feerate
            FROM mempool_metrics
            WHERE network = $1::bitcoin_network 
                AND timestamp BETWEEN to_timestamp($2) AND to_timestamp($3)
            ORDER BY timestamp ASC
            "#,
        )
        .bind(network.as_str())
        .bind(from)
        .bind(to)
        .fetch_all(conn)
        .await?;

        Ok(points)
    }

    async fn get_latest(
        &self, conn: &mut PgConnection, network: Network,
    ) -> Result<Option<MempoolMetrics>, Error> {
        let metrics = sqlx::query_as::<_, MempoolMetrics>(
            r#"
            SELECT id, network, timestamp, tx_count, vbytes, total_fees_btc,
                   min_feerate, max_feerate, avg_feerate, indexed_at
            FROM mempool_metrics
            WHERE network = $1::bitcoin_network
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(network.as_str())
        .fetch_optional(conn)
        .await?;

        Ok(metrics)
    }

    async fn get_last_timestamp(
        &self, conn: &mut PgConnection, network: Network,
    ) -> Result<Option<DateTime<Utc>>, Error> {
        let timestamp: Option<DateTime<Utc>> = sqlx::query_scalar(
            r#"
            SELECT timestamp
            FROM mempool_metrics
            WHERE network = $1::bitcoin_network
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(network.as_str())
        .fetch_optional(conn)
        .await?;

        Ok(timestamp)
    }
}
