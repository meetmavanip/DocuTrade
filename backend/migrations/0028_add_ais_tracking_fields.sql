-- Add migration script here
-- Add AIS tracking fields to shipments
ALTER TABLE shipments
ADD COLUMN vessel_name VARCHAR(255),
ADD COLUMN mmsi VARCHAR(50),
ADD COLUMN imo_number VARCHAR(50),
ADD COLUMN carrier VARCHAR(255),
ADD COLUMN current_latitude NUMERIC(10, 6),
ADD COLUMN current_longitude NUMERIC(10, 6),
ADD COLUMN current_speed NUMERIC(8, 2),
ADD COLUMN current_course NUMERIC(8, 2),
ADD COLUMN current_vessel_status VARCHAR(255),
ADD COLUMN last_tracking_update TIMESTAMPTZ;
