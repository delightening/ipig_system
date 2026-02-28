-- Migration 016: TOTP Two-Factor Authentication
-- Adds 2FA columns to users table for TOTP-based authentication.

ALTER TABLE users
    ADD COLUMN totp_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN totp_secret_encrypted TEXT,
    ADD COLUMN totp_backup_codes TEXT[];

COMMENT ON COLUMN users.totp_enabled IS 'Whether TOTP 2FA is enabled for this user';
COMMENT ON COLUMN users.totp_secret_encrypted IS 'Encrypted TOTP secret (base32 encoded, encrypted at rest)';
COMMENT ON COLUMN users.totp_backup_codes IS 'Hashed one-time backup/recovery codes';
