ALTER TABLE totp_credentials
  DROP CONSTRAINT totp_period_check,
  DROP CONSTRAINT totp_digits_check,
  DROP CONSTRAINT totp_algorithm_check;

DROP INDEX recovery_codes_code_hash_key;
