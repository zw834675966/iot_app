ALTER TABLE users ADD COLUMN IF NOT EXISTS phone TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS account_is_permanent INTEGER NOT NULL DEFAULT 1;
ALTER TABLE users ADD COLUMN IF NOT EXISTS account_valid_days BIGINT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS account_expire_at BIGINT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS created_at BIGINT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS updated_at BIGINT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS created_by TEXT;

UPDATE users
SET
  account_is_permanent = COALESCE(account_is_permanent, 1),
  created_at = COALESCE(created_at, EXTRACT(EPOCH FROM NOW())::BIGINT * 1000),
  updated_at = COALESCE(updated_at, EXTRACT(EPOCH FROM NOW())::BIGINT * 1000),
  created_by = COALESCE(created_by, 'system-seed');

CREATE INDEX IF NOT EXISTS idx_users_phone ON users(phone);
CREATE INDEX IF NOT EXISTS idx_users_account_expire_at ON users(account_expire_at);
