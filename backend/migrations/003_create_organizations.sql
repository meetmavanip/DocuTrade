-- 003_create_organizations.sql

CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    legal_name VARCHAR(255),
    organization_type organization_type NOT NULL,
    registration_number VARCHAR(100),
    tax_id VARCHAR(100),
    country VARCHAR(100) NOT NULL,
    address TEXT,
    city VARCHAR(100),
    state VARCHAR(100),
    postal_code VARCHAR(30),
    email VARCHAR(255),
    phone VARCHAR(50),
    website TEXT,
    wallet_address VARCHAR(255),
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    verified_at TIMESTAMPTZ,
    verified_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_organizations_wallet_address UNIQUE (wallet_address)
);
