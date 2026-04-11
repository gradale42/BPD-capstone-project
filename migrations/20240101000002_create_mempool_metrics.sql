CREATE TABLE IF NOT EXISTS mempool_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    network bitcoin_network NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tx_count INTEGER NOT NULL,
    vbytes BIGINT NOT NULL,
    total_fees_btc DOUBLE PRECISION NOT NULL,
    min_feerate DOUBLE PRECISION,
    max_feerate DOUBLE PRECISION,
    avg_feerate DOUBLE PRECISION,
    indexed_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT pk_mempool_metrics PRIMARY KEY (network, timestamp)
);

CREATE INDEX IF NOT EXISTS idx_mempool_metrics_timestamp ON mempool_metrics(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_mempool_metrics_network_time ON mempool_metricфs(network, timestamp);