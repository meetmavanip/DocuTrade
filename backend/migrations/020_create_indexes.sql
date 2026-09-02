-- 020_create_indexes.sql

-- users
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_organization_id ON users(organization_id);

-- organizations
CREATE INDEX IF NOT EXISTS idx_orgs_wallet ON organizations(wallet_address);
CREATE INDEX IF NOT EXISTS idx_orgs_type ON organizations(organization_type);

-- wallets
CREATE INDEX IF NOT EXISTS idx_wallets_address ON wallets(address);
CREATE INDEX IF NOT EXISTS idx_wallets_org_id ON wallets(organization_id);
CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets(user_id);

-- shipments
CREATE INDEX IF NOT EXISTS idx_shipments_id ON shipments(shipment_id);
CREATE INDEX IF NOT EXISTS idx_shipments_exporter ON shipments(exporter_id);
CREATE INDEX IF NOT EXISTS idx_shipments_buyer ON shipments(buyer_id);
CREATE INDEX IF NOT EXISTS idx_shipments_logistics ON shipments(logistics_provider_id);
CREATE INDEX IF NOT EXISTS idx_shipments_status ON shipments(current_status);
CREATE INDEX IF NOT EXISTS idx_shipments_created ON shipments(created_at);

-- shipment_items
CREATE INDEX IF NOT EXISTS idx_shipment_items_shipment ON shipment_items(shipment_id);

-- documents
CREATE INDEX IF NOT EXISTS idx_documents_id ON documents(document_id);
CREATE INDEX IF NOT EXISTS idx_documents_shipment ON documents(shipment_id);
CREATE INDEX IF NOT EXISTS idx_documents_sha256 ON documents(sha256);
CREATE INDEX IF NOT EXISTS idx_documents_status ON documents(status);
CREATE INDEX IF NOT EXISTS idx_documents_approval ON documents(approval_status);

-- document_versions
CREATE INDEX IF NOT EXISTS idx_doc_versions_doc_id ON document_versions(document_id);
CREATE INDEX IF NOT EXISTS idx_doc_versions_sha256 ON document_versions(sha256);

-- document_approvals
CREATE INDEX IF NOT EXISTS idx_doc_approvals_doc_id ON document_approvals(document_id);

-- tracking_events
CREATE INDEX IF NOT EXISTS idx_tracking_shipment ON tracking_events(shipment_id);
CREATE INDEX IF NOT EXISTS idx_tracking_timestamp ON tracking_events(timestamp);

-- shipment_locations
CREATE INDEX IF NOT EXISTS idx_locations_shipment ON shipment_locations(shipment_id);
CREATE INDEX IF NOT EXISTS idx_locations_timestamp ON shipment_locations(timestamp);

-- blockchain_transactions
CREATE INDEX IF NOT EXISTS idx_tx_hash ON blockchain_transactions(transaction_hash);
CREATE INDEX IF NOT EXISTS idx_tx_shipment ON blockchain_transactions(shipment_id);
CREATE INDEX IF NOT EXISTS idx_tx_doc ON blockchain_transactions(document_id);

-- blockchain_events
CREATE INDEX IF NOT EXISTS idx_event_hash ON blockchain_events(transaction_hash);
CREATE INDEX IF NOT EXISTS idx_event_block ON blockchain_events(block_number);

-- audit_logs
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_org ON audit_logs(organization_id);
CREATE INDEX IF NOT EXISTS idx_audit_type ON audit_logs(entity_type);
CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_logs(entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_logs(created_at);

-- verification_requests
CREATE INDEX IF NOT EXISTS idx_verify_shipment ON verification_requests(shipment_id);
CREATE INDEX IF NOT EXISTS idx_verify_doc ON verification_requests(document_id);

-- qr_codes
CREATE INDEX IF NOT EXISTS idx_qr_code ON qr_codes(code);
