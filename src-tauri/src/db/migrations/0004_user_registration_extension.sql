-- 为 users (用户表) 动态添加业务扩展列，用于完善用户注册和账号生命周期管理
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone TEXT;                        -- 增加手机号码字段
ALTER TABLE users ADD COLUMN IF NOT EXISTS account_is_permanent INTEGER NOT NULL DEFAULT 1; -- 账号是否永久有效标志 (1: 永久, 0: 有效期限制)
ALTER TABLE users ADD COLUMN IF NOT EXISTS account_valid_days BIGINT;         -- 账号有效天数（供逻辑层计算参考）
ALTER TABLE users ADD COLUMN IF NOT EXISTS account_expire_at BIGINT;          -- 账号过期时间戳 (毫秒)，过期后账号将被系统限制登录
ALTER TABLE users ADD COLUMN IF NOT EXISTS created_at BIGINT;                 -- 账号创建时间戳 (毫秒)
ALTER TABLE users ADD COLUMN IF NOT EXISTS updated_at BIGINT;                 -- 账号最后更新时间戳 (毫秒)
ALTER TABLE users ADD COLUMN IF NOT EXISTS created_by TEXT;                   -- 记录创建此账号的操作人员（或系统标识）

-- 为所有既存的用户数据填充新增加列的默认值，确保历史数据的完整性
UPDATE users
SET
  account_is_permanent = COALESCE(account_is_permanent, 1),                   -- 默认设为永久账号
  created_at = COALESCE(created_at, EXTRACT(EPOCH FROM NOW())::BIGINT * 1000),-- 默认创建时间设为当前执行迁移的时刻
  updated_at = COALESCE(updated_at, EXTRACT(EPOCH FROM NOW())::BIGINT * 1000),-- 默认更新时间设为当前执行迁移的时刻
  created_by = COALESCE(created_by, 'system-seed');                           -- 标记早期数据的来源为系统种子数据

-- 为了提升基于手机号或过期时间进行检索和定时任务清理的查询性能，创建相应的数据库索引
CREATE INDEX IF NOT EXISTS idx_users_phone ON users(phone);                   -- 基于手机号创建索引
CREATE INDEX IF NOT EXISTS idx_users_account_expire_at ON users(account_expire_at); -- 基于账号过期时间创建索引
