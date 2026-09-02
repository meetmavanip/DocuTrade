-- 013_create_tracking_events.sql

CREATE TABLE IF NOT EXISTS tracking_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_id UUID NOT NULL REFERENCES shipments(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    status VARCHAR(100),
    description TEXT,
    latitude NUMERIC(10,7),
    longitude NUMERIC(10,7),
    speed NUMERIC(10,2),
    heading NUMERIC(10,2),
    timestamp TIMESTAMPTZ NOT NULL,
    source VARCHAR(50),
    accuracy NUMERIC(10,2),
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
