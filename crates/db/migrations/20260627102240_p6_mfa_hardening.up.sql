-- P6 MFA hardening. The identity migration authored the passkeys, totp_credentials
-- and recovery_codes tables; these constraints finish them for the passkey + TOTP build.

-- Recovery codes are high-entropy, single-use tokens looked up by hash. A unique
-- index forbids a duplicate hash and gives the lookup an index, matching how
-- user_tokens.token_hash and api_keys.token_hash are stored.
CREATE UNIQUE INDEX recovery_codes_code_hash_key ON recovery_codes (code_hash);

-- TOTP parameters must stay within RFC 6238 bounds and the totp-rs Algorithm set.
ALTER TABLE totp_credentials
  ADD CONSTRAINT totp_algorithm_check CHECK (algorithm IN ('SHA1', 'SHA256', 'SHA512')),
  ADD CONSTRAINT totp_digits_check    CHECK (digits BETWEEN 6 AND 8),
  ADD CONSTRAINT totp_period_check    CHECK (period_seconds > 0);
