-- 014_create_shipment_locations.sql

CREATE TABLE IF NOT EXISTS shipment_locations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_id UUID NOT NULL REFERENCES shipments(id) ON DELETE CASCADE,
    latitude NUMERIC(10,7) NOT NULL,
    longitude NUMERIC(10,7) NOT NULL,
    speed NUMERIC(10,2),
    heading NUMERIC(10,2),
    timestamp TIMESTAMPTZ NOT NULL,
    source VARCHAR(50),
    accuracy NUMERIC(10,2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
