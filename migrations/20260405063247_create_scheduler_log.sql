CREATE TABLE IF NOT EXISTS scheduler_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    start_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    end_time TIMESTAMPTZ,
    description TEXT NOT NULL,
    result JSONB,
    status TEXT DEFAULT 'running' CHECK (status IN ('running', 'completed', 'failed'))
);

CREATE INDEX idx_scheduler_log_start_time ON scheduler_log(start_time DESC);
