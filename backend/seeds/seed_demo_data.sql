-- backend/seeds/seed_demo_data.sql
-- Seed Data for DocuTrade Full Schema

-- 1. Organizations
INSERT INTO organizations (id, name, legal_name, organization_type, country, is_verified)
VALUES
    ('11111111-1111-1111-1111-111111111111', 'ABC Manufacturing Pvt. Ltd.', 'ABC Mfg Pvt. Ltd.', 'EXPORTER', 'India', true),
    ('22222222-2222-2222-2222-222222222222', 'XYZ Imports LLC', 'XYZ Imports LLC', 'BUYER', 'UAE', true),
    ('33333333-3333-3333-3333-333333333333', 'Global Shipping Corp', 'Global Shipping Corporation', 'LOGISTICS', 'India', true),
    ('44444444-4444-4444-4444-444444444444', 'QualityCert India', 'QualityCert Pvt. Ltd.', 'INSPECTION', 'India', false);

-- 2. Users (Passwords: password123 hashed via Argon2)
INSERT INTO users (id, organization_id, email, password_hash, first_name, last_name, is_active, email_verified)
VALUES
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '11111111-1111-1111-1111-111111111111', 'demo@docutrade.io', '$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$c29tZWhhc2g', 'Demo', 'User', true, true),
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '44444444-4444-4444-4444-444444444444', 'inspector@docutrade.io', '$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$c29tZWhhc2g', 'Inspector', 'Patel', true, true);

-- 3. Roles & User Roles
INSERT INTO roles (id, name, description)
VALUES 
    ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'EXPORTER', 'Can create shipments and upload documents'),
    ('dddddddd-dddd-dddd-dddd-dddddddddddd', 'INSPECTOR', 'Can review and approve documents');

INSERT INTO user_roles (user_id, role_id)
VALUES
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'cccccccc-cccc-cccc-cccc-cccccccccccc'),
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'dddddddd-dddd-dddd-dddd-dddddddddddd');

-- 4. Wallets
INSERT INTO wallets (user_id, organization_id, address, chain_id, is_primary, is_verified)
VALUES
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '11111111-1111-1111-1111-111111111111', '0x82F9a4c1b7E3d5f8A2e6C9b0D4f7a1B3c5E7d9A72C', 421614, true, true);

-- 5. Shipments
INSERT INTO shipments (id, shipment_id, exporter_id, buyer_id, logistics_provider_id, origin_country, origin_location, destination_country, destination_location, total_value, currency, incoterms, current_status)
VALUES
    ('55555555-5555-5555-5555-555555555555', 'EXP-IND-2026-00981', '11111111-1111-1111-1111-111111111111', '22222222-2222-2222-2222-222222222222', '33333333-3333-3333-3333-333333333333', 'India', 'Mundra Port', 'UAE', 'Jebel Ali Port', 45000.00, 'USD', 'FOB', 'IN_TRANSIT');

-- 6. Shipment Items
INSERT INTO shipment_items (shipment_id, product_name, quantity, unit, unit_price, total_price, currency, hs_code)
VALUES
    ('55555555-5555-5555-5555-555555555555', 'Industrial Valves Grade A', 1000, 'pcs', 45.00, 45000.00, 'USD', '848180');

-- 7. Documents
INSERT INTO documents (id, document_id, shipment_id, uploaded_by, document_type, filename, mime_type, file_size, current_version, sha256, status, approval_status)
VALUES
    ('77777777-7777-7777-7777-777777777777', 'DOC-001', '55555555-5555-5555-5555-555555555555', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'COMMERCIAL_INVOICE', 'invoice.pdf', 'application/pdf', 102400, 1, 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855', 'APPROVED', 'APPROVED');

-- 8. Document Versions
INSERT INTO document_versions (id, document_id, version, filename, mime_type, file_size, sha256, uploaded_by, status)
VALUES
    ('99999999-9999-9999-9999-999999999999', '77777777-7777-7777-7777-777777777777', 1, 'invoice.pdf', 'application/pdf', 102400, 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'APPROVED');

-- 9. Audit Logs
INSERT INTO audit_logs (user_id, organization_id, action, entity_type, entity_id, description)
VALUES
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '11111111-1111-1111-1111-111111111111', 'SHIPMENT_CREATED', 'SHIPMENT', '55555555-5555-5555-5555-555555555555', 'Shipment created successfully');
