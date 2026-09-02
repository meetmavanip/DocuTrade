-- Add SELLER to enums if not exists
ALTER TYPE role_name ADD VALUE IF NOT EXISTS 'SELLER';
ALTER TYPE organization_type ADD VALUE IF NOT EXISTS 'SELLER';

-- Note: In PostgreSQL, ALTER TYPE cannot run inside a transaction block in older versions, 
-- but SQLx wraps migrations in transactions. To work around this for enum updates:
COMMIT;
-- We start a new transaction so sqlx doesn't fail on COMMIT
BEGIN;

-- Migrate existing EXPORTERs to SELLER
UPDATE roles SET name = 'SELLER' WHERE name = 'EXPORTER';
UPDATE organizations SET organization_type = 'SELLER' WHERE organization_type = 'EXPORTER';

-- trade_codes table
CREATE TABLE IF NOT EXISTS trade_codes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_id UUID NOT NULL REFERENCES shipments(id) ON DELETE CASCADE,
    code_hash TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trade_codes_shipment ON trade_codes(shipment_id);
CREATE INDEX IF NOT EXISTS idx_trade_codes_hash ON trade_codes(code_hash);

-- trade_access table
CREATE TABLE IF NOT EXISTS trade_access (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_id UUID NOT NULL REFERENCES shipments(id) ON DELETE CASCADE,
    buyer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(shipment_id, buyer_id)
);

-- audit_logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(255) NOT NULL,
    resource_type VARCHAR(255),
    resource_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
