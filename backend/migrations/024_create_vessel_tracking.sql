-- 024_create_vessel_tracking.sql

-- Add shipment fields
ALTER TABLE shipments
ADD COLUMN container_number VARCHAR(100),
ADD COLUMN booking_number VARCHAR(100),
ADD COLUMN bill_of_lading_number VARCHAR(100),
ADD COLUMN vessel_id UUID,
ADD COLUMN voyage_id VARCHAR(255);

CREATE INDEX IF NOT EXISTS idx_shipments_container_number ON shipments(container_number);

-- Create vessels table
CREATE TABLE IF NOT EXISTS vessels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    vessel_name VARCHAR(255) NOT NULL,
    imo VARCHAR(50),
    mmsi VARCHAR(50),
    vessel_type VARCHAR(100),
    capacity NUMERIC(18,2),
    deadweight_tonnage NUMERIC(18,2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vessels_imo ON vessels(imo);
CREATE INDEX IF NOT EXISTS idx_vessels_mmsi ON vessels(mmsi);
CREATE INDEX IF NOT EXISTS idx_vessels_name ON vessels(vessel_name);

-- Create vessel_voyages table
CREATE TABLE IF NOT EXISTS vessel_voyages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    vessel_id UUID NOT NULL REFERENCES vessels(id),
    voyage_id VARCHAR(255) NOT NULL,
    origin VARCHAR(255),
    destination VARCHAR(255),
    departure_time TIMESTAMPTZ,
    eta TIMESTAMPTZ,
    status VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vessel_voyages_voyage_id ON vessel_voyages(voyage_id);

-- Create port_calls table
CREATE TABLE IF NOT EXISTS port_calls (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    vessel_id UUID NOT NULL REFERENCES vessels(id),
    voyage_id VARCHAR(255),
    port_call_id VARCHAR(255),
    port_name VARCHAR(255),
    location TEXT,
    eta TIMESTAMPTZ,
    start_time TIMESTAMPTZ,
    end_time TIMESTAMPTZ,
    status VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create shipment_vessel_links table
CREATE TABLE IF NOT EXISTS shipment_vessel_links (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_id UUID NOT NULL REFERENCES shipments(id),
    vessel_id UUID NOT NULL REFERENCES vessels(id),
    voyage_id VARCHAR(255),
    confidence_score NUMERIC(5,2),
    source VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_shipment_vessel_links_shipment_id ON shipment_vessel_links(shipment_id);
