-- Network type to strictly match Rust enum strings
CREATE TYPE bitcoin_network AS ENUM ('regtest', 'signet', 'testnet', 'mainnet');

CREATE TABLE IF NOT EXISTS block_info (
    -- Network field with constraint to strictly match Rust enum strings
    network          bitcoin_network NOT NULL,
    -- Block height is unique in the chain
    height           BIGINT NOT NULL,
    -- Block hash (hex string)
    hash             TEXT NOT NULL UNIQUE,
    -- Block timestamp
    time             TIMESTAMP NOT NULL,
    
    -- Transaction count
    tx_count         INT NOT NULL,
    
    -- Technical metrics (Bytes and Weight Units)
    size             INT NOT NULL,
    weight           INT NOT NULL,
    
    -- Financial metrics in Satoshis
    subsidy_sat      BIGINT NOT NULL,
    total_fees_sat   BIGINT NOT NULL,
    avg_fee_sat      BIGINT NOT NULL,
    
    -- Rate and Network metrics
    avg_feerate      DOUBLE PRECISION NOT NULL,
    difficulty       DOUBLE PRECISION NOT NULL,
    
    -- Metadata for the indexer
    indexed_at       TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT pk_block_info PRIMARY KEY (network, height)
);

CREATE INDEX IF NOT EXISTS idx_block_info_hash ON block_info(hash);
CREATE INDEX IF NOT EXISTS idx_block_info_time ON block_info(time);
