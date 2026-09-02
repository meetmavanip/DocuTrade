-- 008_create_shipments.sql

CREATE TABLE IF NOT EXISTS shipments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_id VARCHAR(100) NOT NULL UNIQUE,
    exporter_id UUID NOT NULL REFERENCES organizations(id),
    buyer_id UUID NOT NULL REFERENCES organizations(id),
    logistics_provider_id UUID REFERENCES organizations(id),
    origin_country VARCHAR(100) NOT NULL,
    origin_location TEXT,
    destination_country VARCHAR(100) NOT NULL,
    destination_location TEXT,
    product_category VARCHAR(255),
    quantity NUMERIC(18,4),
    total_value NUMERIC(18,2),
    currency VARCHAR(10),
    incoterms VARCHAR(20),
    departure_date TIMESTAMPTZ,
    expected_arrival TIMESTAMPTZ,
    current_status shipment_status NOT NULL DEFAULT 'DRAFT',
    metadata_hash VARCHAR(255),
    blockchain_transaction VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
