use crate::domain::scheduler::{SchedulerLog, SyncResult};
use async_trait::async_trait;
use sqlx::{Error, PgConnection};
use uuid::Uuid;

#[async_trait]
pub trait SchedulerLogRepository: Send + Sync {
    async fn create_log(&self, conn: &mut PgConnection, description: &str) -> Result<Uuid, Error>;
    async fn update_log_success(
        &self, conn: &mut PgConnection, id: Uuid, result: &SyncResult,
    ) -> Result<(), Error>;
    async fn update_log_failure(
        &self, conn: &mut PgConnection, id: Uuid, error: &str,
    ) -> Result<(), Error>;
    async fn list_logs(
        &self, conn: &mut PgConnection, limit: i64, offset: i64,
    ) -> Result<Vec<SchedulerLog>, Error>;
    async fn count_logs(&self, conn: &mut PgConnection) -> Result<i64, Error>;
    async fn get_log(&self, conn: &mut PgConnection, id: Uuid) -> Result<SchedulerLog, Error>;
}

pub struct PostgresSchedulerLogRepository;

#[async_trait]
impl SchedulerLogRepository for PostgresSchedulerLogRepository {
    async fn create_log(&self, conn: &mut PgConnection, description: &str) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO scheduler_log (id, description, status)
               VALUES ($1, $2, 'running')"#,
        )
        .bind(id)
        .bind(description)
        .execute(conn)
        .await?;
        Ok(id)
    }

    async fn update_log_success(
        &self, conn: &mut PgConnection, id: Uuid, result: &SyncResult,
    ) -> Result<(), Error> {
        let result_json = serde_json::to_value(result).unwrap();
        sqlx::query(
            r#"UPDATE scheduler_log
               SET end_time = NOW(), status = 'completed', result = $2
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(result_json)
        .execute(conn)
        .await?;
        Ok(())
    }

    async fn update_log_failure(
        &self, conn: &mut PgConnection, id: Uuid, error: &str,
    ) -> Result<(), Error> {
        let result = serde_json::json!({ "error": error });
        sqlx::query(
            r#"UPDATE scheduler_log
               SET end_time = NOW(), status = 'failed', result = $2
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(result)
        .execute(conn)
        .await?;
        Ok(())
    }

    async fn list_logs(
        &self, conn: &mut PgConnection, limit: i64, offset: i64,
    ) -> Result<Vec<SchedulerLog>, Error> {
        let rows = sqlx::query_as::<_, SchedulerLog>(
            r#"SELECT id, start_time, end_time, description, result, status
           FROM scheduler_log
           ORDER BY start_time DESC
           LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(conn)
        .await?;
        Ok(rows)
    }

    async fn count_logs(&self, conn: &mut PgConnection) -> Result<i64, Error> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM scheduler_log").fetch_one(conn).await?;
        Ok(row.0)
    }

    async fn get_log(&self, conn: &mut PgConnection, id: Uuid) -> Result<SchedulerLog, Error> {
        let row = sqlx::query_as::<_, SchedulerLog>(
            r#"SELECT id, start_time, end_time, description, result, status
           FROM scheduler_log WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(conn)
        .await?;
        Ok(row)
    }
}
