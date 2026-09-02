-- 015_create_blockchain_transactions.sql

CREATE TABLE IF NOT EXISTS blockchain_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_id UUID REFERENCES shipments(id) ON DELETE SET NULL,
    document_id UUID REFERENCES documents(id) ON DELETE SET NULL,
    transaction_hash VARCHAR(255) NOT NULL UNIQUE,
    chain_id BIGINT NOT NULL,
    network VARCHAR(100) NOT NULL,
    contract_address VARCHAR(255),
    block_number BIGINT,
    status blockchain_transaction_status NOT NULL DEFAULT 'PENDING',
    transaction_type VARCHAR(100) NOT NULL,
    submitted_at TIMESTAMPTZ,
    confirmed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
