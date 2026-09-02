-- 023_seed_roles.sql

INSERT INTO roles (name, description) VALUES
    ('ADMIN', 'System Administrator'),
    ('EXPORTER', 'Exporter Organization'),
    ('INSPECTOR', 'Inspection Agency'),
    ('LOGISTICS', 'Logistics Provider'),
    ('BUYER', 'Buyer Organization')
ON CONFLICT (name) DO NOTHING;
