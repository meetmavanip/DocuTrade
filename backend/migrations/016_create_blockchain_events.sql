-- 016_create_blockchain_events.sql

CREATE TABLE IF NOT EXISTS blockchain_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transaction_id UUID REFERENCES blockchain_transactions(id) ON DELETE CASCADE,
    event_name VARCHAR(150) NOT NULL,
    contract_address VARCHAR(255),
    transaction_hash VARCHAR(255) NOT NULL,
    block_number BIGINT,
    log_index BIGINT,
    event_data JSONB,
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_blockchain_events_tx_log UNIQUE (transaction_hash, log_index)
);
