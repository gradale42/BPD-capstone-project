use chrono::{DateTime, Utc};

#[derive(Debug, serde::Deserialize)]
pub struct BlocksParams {
    pub draw: i32,
    pub start: Option<i32>,
    pub length: Option<i32>,
}

#[derive(Debug, serde::Serialize)]
pub struct BlockInfo {
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