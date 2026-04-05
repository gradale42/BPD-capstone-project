use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct SchedulerLog {
    pub id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub description: String,
    pub result: Option<serde_json::Value>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncResult {
    pub saved: usize,
    pub saved_blocks: Vec<String>,  // hashes
    pub skipped: usize,
    pub skipped_blocks: Vec<String>, // hashes
    pub error: Option<String>,
}