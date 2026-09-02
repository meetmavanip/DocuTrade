-- 010_create_documents.sql

CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id VARCHAR(100) NOT NULL UNIQUE,
    shipment_id UUID NOT NULL REFERENCES shipments(id) ON DELETE CASCADE,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    document_type document_type NOT NULL,
    filename VARCHAR(500) NOT NULL,
    mime_type VARCHAR(100),
    file_size BIGINT,
    current_version INTEGER NOT NULL DEFAULT 1,
    sha256 CHAR(64) NOT NULL,
    storage_reference TEXT,
    ipfs_cid TEXT,
    status document_status NOT NULL DEFAULT 'PENDING',
    approval_status document_status NOT NULL DEFAULT 'PENDING',
    blockchain_transaction VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
