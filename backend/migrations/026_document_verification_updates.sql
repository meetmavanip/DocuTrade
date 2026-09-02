-- 026_document_verification_updates.sql
-- Extend document_status enum with blockchain verification statuses
-- and create document_verifications table for blockchain verification history.

-- Add new enum values for blockchain verification workflow
ALTER TYPE document_status ADD VALUE IF NOT EXISTS 'VERIFIED';
ALTER TYPE document_status ADD VALUE IF NOT EXISTS 'BLOCKCHAIN_PENDING';
ALTER TYPE document_status ADD VALUE IF NOT EXISTS 'BLOCKCHAIN_FAILED';
ALTER TYPE document_status ADD VALUE IF NOT EXISTS 'BLOCKCHAIN_REJECTED';

-- Need to commit/begin since ALTER TYPE ADD VALUE can't run in a transaction in some PG versions
COMMIT;
BEGIN;

-- Create document_verifications table for blockchain verification history
CREATE TABLE IF NOT EXISTS document_verifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    document_hash CHAR(64) NOT NULL,
    verifier_user_id UUID NOT NULL REFERENCES users(id),
    wallet_address VARCHAR(255) NOT NULL,
    network VARCHAR(100) NOT NULL DEFAULT 'Arbitrum Sepolia',
    chain_id BIGINT NOT NULL DEFAULT 421614,
    contract_address VARCHAR(255) NOT NULL,
    transaction_hash VARCHAR(255) NOT NULL UNIQUE,
    block_number BIGINT,
    status VARCHAR(50) NOT NULL DEFAULT 'CONFIRMED',
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_doc_verifications_document ON document_verifications(document_id);
CREATE INDEX IF NOT EXISTS idx_doc_verifications_tx_hash ON document_verifications(transaction_hash);
CREATE INDEX IF NOT EXISTS idx_doc_verifications_wallet ON document_verifications(wallet_address);

-- Add rejection_reason column to documents table for inline storage
ALTER TABLE documents ADD COLUMN IF NOT EXISTS rejection_reason TEXT;
