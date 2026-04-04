CREATE TABLE IF NOT EXISTS block_info (
    -- Primary Key: Block height is unique in the main chain
    height           BIGINT PRIMARY KEY,
    -- Block hash (hex string)
    hash             TEXT NOT NULL UNIQUE,
    -- Block timestamp (Unix epoch)
    time             BIGINT NOT NULL,
    
    -- Transaction count
    tx_count         INT NOT NULL,
    
    -- Technical metrics (Bytes and Weight Units)
    size             INT NOT NULL,
    weight           INT NOT NULL,
    
    -- Financial metrics in Satoshis (1 BTC = 10^8 Satoshis)
    -- Using BIGINT to prevent rounding errors during analysis
    subsidy_sat      BIGINT NOT NULL,
    total_fees_sat   BIGINT NOT NULL,
    avg_fee_sat      BIGINT NOT NULL,
    
    -- Rate and Network metrics
    avg_feerate      DOUBLE PRECISION NOT NULL,
    difficulty       DOUBLE PRECISION NOT NULL,
    
    -- Metadata for the indexer
    indexed_at       TIMESTAMPTZ DEFAULT NOW()
);

-- Index for hash-based lookups
CREATE INDEX IF NOT EXISTS idx_block_info_hash ON block_info(hash);
