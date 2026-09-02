-- 011_create_document_versions.sql

CREATE TABLE IF NOT EXISTS document_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    filename VARCHAR(500) NOT NULL,
    mime_type VARCHAR(100),
    file_size BIGINT,
    sha256 CHAR(64) NOT NULL,
    storage_reference TEXT,
    ipfs_cid TEXT,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status document_status NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_document_versions UNIQUE (document_id, version)
);
