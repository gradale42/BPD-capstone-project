use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, ToSchema)]
pub struct SchedulerLog {
    pub id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub description: String,
    pub result: Option<serde_json::Value>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SyncResult {
    pub saved: usize,
    pub saved_blocks: Vec<String>,  // hashes
    pub skipped: usize,
    pub skipped_blocks: Vec<String>, // hashes
    pub error: Option<String>,
}