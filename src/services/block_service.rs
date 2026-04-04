use std::sync::Arc;
use sqlx::{Error, PgPool};
use crate::domain::block::BlockInfo;
use crate::repositories::block_repository::BlockRepository;
use crate::db::Transactional;

pub struct BlockService {
    repo: Arc<dyn BlockRepository>,
    pool: PgPool,
}

impl BlockService {
    pub async fn save_blocks(&self, blocks: Vec<BlockInfo>) -> Result<(), Error> {
        self.pool.in_transaction(|mut tx| async move {
            for block in blocks {
                self.repo.save(&mut *tx, block).await?;
            }
       Ok(((), tx))
        }).await
    }
}