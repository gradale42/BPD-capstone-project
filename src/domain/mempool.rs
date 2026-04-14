use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct MempoolSnapshot {
    pub timestamp: String,
    pub tx_count: usize,
    pub vbytes: usize,
    pub total_fees: f64,
    pub min_relay_feerate: f64,
}


#[derive(Debug, Clone, serde::Serialize, ToSchema)]
pub struct MempoolTransaction {
    pub txid: String,
    pub vsize: u64,
    pub weight: u64,
    pub time: u64,
    pub height: u64,
    pub fee: f64,
    pub fee_rate: f64,
    pub ancestor_count: u64,
    pub descendant_count: u64,
    pub bip125_replaceable: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct MempoolMetrics {
    pub id: Uuid,
    pub network: String,
    pub timestamp: DateTime<Utc>,
    pub tx_count: i32,
    pub vbytes: i64,
    pub total_fees_btc: f64,
    pub min_feerate: Option<f64>,
    pub max_feerate: Option<f64>,
    pub avg_feerate: Option<f64>,
    pub indexed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct MempoolMetricsPoint {
    pub time: i64,
    pub tx_count: i32,
    pub vbytes: i64,
    pub total_fees_btc: f64,
    pub avg_feerate: Option<f64>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct MempoolTimeRange {
    pub from: i64,
    pub to: i64,
}