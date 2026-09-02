-- 002_create_enums.sql

CREATE TYPE organization_type AS ENUM (
    'EXPORTER',
    'BUYER',
    'LOGISTICS',
    'INSPECTION',
    'CUSTOMS',
    'PLATFORM'
);

CREATE TYPE shipment_status AS ENUM (
    'DRAFT',
    'DOCUMENTS_PENDING',
    'UNDER_REVIEW',
    'APPROVED',
    'READY_TO_SHIP',
    'IN_TRANSIT',
    'DELIVERED',
    'CLOSED'
);

CREATE TYPE document_type AS ENUM (
    'COMMERCIAL_INVOICE',
    'PACKING_LIST',
    'CERTIFICATE_OF_ORIGIN',
    'QUALITY_CERTIFICATE',
    'INSPECTION_CERTIFICATE',
    'INSURANCE_DOCUMENT',
    'SHIPPING_DOCUMENT'
);

CREATE TYPE document_status AS ENUM (
    'PENDING',
    'APPROVED',
    'REJECTED',
    'SUPERSEDED',
    'REVOKED'
);

CREATE TYPE approval_decision AS ENUM (
    'APPROVED',
    'REJECTED',
    'REVOKED'
);

CREATE TYPE blockchain_transaction_status AS ENUM (
    'PENDING',
    'CONFIRMED',
    'FAILED'
);

CREATE TYPE verification_result AS ENUM (
    'MATCH',
    'MISMATCH',
    'NOT_FOUND',
    'ERROR'
);

CREATE TYPE role_name AS ENUM (
    'ADMIN',
    'EXPORTER',
    'INSPECTOR',
    'LOGISTICS',
    'BUYER'
);
