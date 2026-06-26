-- Reverse of identity.up.sql, dropping in reverse dependency order.
DROP TABLE IF EXISTS api_keys;
DROP TABLE IF EXISTS user_tokens;
DROP TABLE IF EXISTS recovery_codes;
DROP TABLE IF EXISTS totp_credentials;
DROP TABLE IF EXISTS passkeys;
DROP TABLE IF EXISTS oidc_identities;
DROP TABLE IF EXISTS oidc_providers;
DROP TABLE IF EXISTS password_credentials;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS crypto_keys;
