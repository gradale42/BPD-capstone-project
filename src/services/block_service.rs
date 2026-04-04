use crate::db::Transactional;
use crate::domain::block::BlockInfo;
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
}
