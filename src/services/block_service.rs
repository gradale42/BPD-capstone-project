use crate::domain::block::{BlockInfo, TimeseriesPoint};
use crate::db::block_repository::{BlockRepository, PostgresBlockRepository};
use crate::services::ExecutionCtx;
use sqlx::{Error, PgConnection, PgPool};
use std::sync::Arc;

pub struct BlockService {
    repo: Arc<dyn BlockRepository>,
}

impl BlockService {
    pub fn new(repo: Arc<dyn BlockRepository>) -> Self {
        BlockService { repo }
    }
}

impl BlockService {
    pub async fn find_by_height(
        &self, ctx: &mut ExecutionCtx, height: i64,
    ) -> Result<BlockInfo, Error> {
        let count = self.repo.find_by_height(&mut ctx.conn, ctx.network, height).await?;
        Ok(count)
    }

    pub async fn find_by_hash(
        &self, ctx: &mut ExecutionCtx, hash: &str,
    ) -> Result<BlockInfo, Error> {
        let count = self.repo.find_by_hash(&mut ctx.conn, ctx.network, hash).await?;
        Ok(count)
    }

    pub async fn save(&self, ctx: &mut ExecutionCtx, block: BlockInfo) -> Result<(), Error> {
        let network = ctx.network;
        ctx.execute_in_transaction(async |tx| {
            self.repo.save(tx, network, block).await?;
            Ok(())
        })
        .await
    }

    pub async fn save_blocks(
        &self, ctx: &mut ExecutionCtx, blocks: Vec<BlockInfo>,
    ) -> Result<(), Error> {
        let network = ctx.network;
        ctx.execute_in_transaction(async |tx| {
            for block in blocks {
                self.repo.save(tx, network, block).await?;
            }
            Ok(())
        })
        .await
    }

    pub async fn save_blocks_ignore_duplicates(
        &self, ctx: &mut ExecutionCtx, blocks: Vec<BlockInfo>,
    ) -> Result<(usize, usize), Error> {
        let network = ctx.network;
        ctx.execute_in_transaction(async |tx| {
            println!("Starting to save {} blocks", blocks.len());
            let mut saved = 0;
            let mut skipped = 0;
            for block in blocks {
                match self.repo.find_by_height(tx, network, block.height).await {
                    Ok(_) => skipped += 1,
                    Err(sqlx::Error::RowNotFound) => {
                        self.repo.save(tx, network, block).await?;
                        saved += 1;
                    }
                    Err(e) => return Err(e),
                }
            }
            println!("Saved {} blocks, skipped {} duplicates", saved, skipped);
            Ok((saved, skipped))
        })
        .await
    }

    pub async fn get_blocks_from_db(
        &self, ctx: &mut ExecutionCtx, limit: i64, offset: i64, order_by: &str,
    ) -> Result<Vec<BlockInfo>, Error> {
        println!("Loading {} blocks from database (offset: {})", limit, offset);
        let blocks =
            self.repo.list_blocks(&mut ctx.conn, ctx.network, limit, offset, order_by).await?;
        println!("Loaded {} blocks from database", blocks.len());
        Ok(blocks)
    }

    pub async fn count_blocks_in_db(&self, ctx: &mut ExecutionCtx) -> Result<i64, Error> {
        let count = self.repo.count_blocks(&mut ctx.conn, ctx.network).await?;
        Ok(count)
    }

    pub async fn get_last_block_height(
        &self, ctx: &mut ExecutionCtx,
    ) -> Result<Option<i64>, sqlx::Error> {
        let height = self.repo.get_last_block_height(&mut ctx.conn, ctx.network).await?;
        Ok(height)
    }

    pub async fn get_timeseries(
        &self, ctx: &mut ExecutionCtx, from: i64, to: i64,
    ) -> Result<Vec<TimeseriesPoint>, Error> {
        self.repo.get_timeseries(&mut ctx.conn, ctx.network, from, to).await
    }
}
