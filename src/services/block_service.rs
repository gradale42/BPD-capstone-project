use crate::db::Transactional;
use crate::domain::block::{BlockInfo, TimeseriesPoint};
use crate::repositories::block_repository::{BlockRepository, PostgresBlockRepository};
use sqlx::{Error, PgPool};
use std::sync::Arc;

pub struct BlockService {
    repo: Arc<dyn BlockRepository>,
    pool: PgPool,
}

impl BlockService {
    pub fn new(repo: Arc<dyn BlockRepository>, pool: PgPool) -> Self {
        BlockService { repo, pool }
    }
}

impl BlockService {
    pub async fn save_blocks(&self, blocks: Vec<BlockInfo>) -> Result<(), Error> {
        self.pool
            .in_transaction(|mut tx| async move {
                for block in blocks {
                    self.repo.save(&mut *tx, block).await?;
                }
                Ok(((), tx))
            })
            .await
    }

    pub async fn save_blocks_ignore_duplicates(&self, blocks: Vec<BlockInfo>) -> Result<(usize, usize), Error> {
        self.pool
            .in_transaction(|mut tx| async move {
                println!("Starting to save {} blocks", blocks.len());
                let mut saved = 0;
                let mut skipped = 0;
                for block in blocks {
                    match self.repo.find_by_height(&mut *tx, block.height).await {
                        Ok(_) => skipped += 1,
                        Err(sqlx::Error::RowNotFound) => {
                            self.repo.save(&mut *tx, block).await?;
                            saved += 1;
                        }
                        Err(e) => return Err(e),
                    }
                }
                println!("Saved {} blocks, skipped {} duplicates", saved, skipped);
                Ok(((saved, skipped), tx))
            })
            .await
    }

    pub async fn get_blocks_from_db(&self, limit: i64, offset: i64, order_by: &str) -> Result<Vec<BlockInfo>, Error> {
        let mut conn = self.pool.acquire().await?;
        println!("Loading {} blocks from database (offset: {})", limit, offset);
        let blocks = self.repo.list_blocks(&mut conn, limit, offset, order_by).await?;
        println!("Loaded {} blocks from database", blocks.len());
        Ok(blocks)
    }

    pub async fn count_blocks_in_db(&self) -> Result<i64, Error> {
        let mut conn = self.pool.acquire().await?;
        let count = self.repo.count_blocks(&mut conn).await?;
        Ok(count)
    }

    pub async fn get_last_block_height(&self) -> Result<Option<i64>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query!("SELECT height FROM block_info ORDER BY height DESC LIMIT 1")
            .fetch_optional(&mut conn)
            .await?;
        Ok(row.map(|r| r.height))
    }

    pub async fn get_timeseries(&self, from: i64, to: i64) -> Result<Vec<TimeseriesPoint>, Error> {
        let mut conn = self.pool.acquire().await?;
        self.repo.get_timeseries(&mut conn, from, to).await
    }
}
