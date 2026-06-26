-- Reverse of ops_audit.up.sql, in reverse dependency order.
DROP TRIGGER IF EXISTS audit_log_no_truncate ON audit_log;
DROP TRIGGER IF EXISTS audit_log_no_change ON audit_log;
DROP FUNCTION IF EXISTS audit_log_immutable();
-- Dropping the partitioned parent drops its partitions (audit_log_default) too.
DROP TABLE IF EXISTS audit_log;
DROP TABLE IF EXISTS import_jobs;
DROP TABLE IF EXISTS data_subject_requests;
DROP TABLE IF EXISTS oidc_role_mappings;
DROP TABLE IF EXISTS instance_settings;
DROP TABLE IF EXISTS mail_templates;
DROP TABLE IF EXISTS email_outbox;
