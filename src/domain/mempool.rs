use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, serde::Serialize)]
pub struct MempoolSnapshot {
    pub timestamp: String,
    pub tx_count: usize,
    pub vbytes: usize,
    pub total_fees: f64,
    pub min_relay_feerate: f64,
}


#[derive(Debug, Clone, serde::Serialize)]  
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


#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

// Point for timeseries charts
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MempoolMetricsPoint {
    pub time: i64,  // Unix timestamp
    pub tx_count: i32,
    pub vbytes: i64,
    pub total_fees_btc: f64,
    pub avg_feerate: Option<f64>,
}

// Query parameters for time range
#[derive(Debug, Deserialize)]
pub struct MempoolTimeRange {
    pub from: i64,
    pub to: i64,
}