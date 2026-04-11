use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::configuration::Network;

#[derive(Debug, Deserialize)]
pub struct BlocksParams {
    pub draw: i32,
    pub start: Option<i32>,
    pub length: Option<i32>,
    pub order: Option<Vec<Order>>,
    pub mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct BlockInfo {
    pub network: Network,
    pub height: i64,
    pub hash: String,
    pub time: i64,        // Unix timestamp from RPC
    pub tx_count: i32,
    pub size: i32,        // Block size in bytes
    pub weight: i32,      // Block weight (vSize * 4)
    pub subsidy_sat: i64,     // Block reward in Satoshis (e.g., 312500000)
    pub total_fees_sat: i64,  // Total fees in Satoshis
    pub avg_fee_sat: i64,     // Average fee in Satoshis
    pub avg_feerate: f64, // Usually sat/vB, so float is fine here
    pub difficulty: f64,
    pub indexed_at: Option<DateTime<Utc>>,    // Unix timestamp when indexed
}

#[derive(Debug, Deserialize)]
pub struct Order {
    pub column: usize,
    pub dir: String,
}

#[derive(Debug, Deserialize)]
pub struct TimeRange {
    pub from: i64,  // Unix timestamp (seconds)
    pub to: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct TimeseriesPoint {
    pub time: i64,           // Unix timestamp (seconds)
    pub tx_count: i32,
    pub avg_fee_sat: i64,
    pub avg_feerate: f64,
}
