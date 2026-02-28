-- ==========================================================================================
-- User registration extension migration
-- File: 0004_user_registration_extension.sql
-- Purpose: add optional phone, account validity lifecycle and audit fields for admin register
-- ==========================================================================================

PRAGMA foreign_keys = ON;

ALTER TABLE users ADD COLUMN phone TEXT;
ALTER TABLE users ADD COLUMN account_is_permanent INTEGER NOT NULL DEFAULT 1;
ALTER TABLE users ADD COLUMN account_valid_days INTEGER;
ALTER TABLE users ADD COLUMN account_expire_at INTEGER;
ALTER TABLE users ADD COLUMN created_at INTEGER;
ALTER TABLE users ADD COLUMN updated_at INTEGER;
ALTER TABLE users ADD COLUMN created_by TEXT;

UPDATE users
SET
  account_is_permanent = COALESCE(account_is_permanent, 1),
  created_at = COALESCE(created_at, CAST(strftime('%s', 'now') AS INTEGER) * 1000),
  updated_at = COALESCE(updated_at, CAST(strftime('%s', 'now') AS INTEGER) * 1000),
  created_by = COALESCE(created_by, 'system-seed');

CREATE INDEX IF NOT EXISTS idx_users_phone ON users(phone);
CREATE INDEX IF NOT EXISTS idx_users_account_expire_at ON users(account_expire_at);
