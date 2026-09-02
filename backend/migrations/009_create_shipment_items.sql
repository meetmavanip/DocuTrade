-- 009_create_shipment_items.sql

CREATE TABLE IF NOT EXISTS shipment_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_id UUID NOT NULL REFERENCES shipments(id) ON DELETE CASCADE,
    product_name VARCHAR(255) NOT NULL,
    product_code VARCHAR(100),
    description TEXT,
    quantity NUMERIC(18,4) NOT NULL,
    unit VARCHAR(50),
    unit_price NUMERIC(18,2),
    total_price NUMERIC(18,2),
    currency VARCHAR(10),
    hs_code VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
