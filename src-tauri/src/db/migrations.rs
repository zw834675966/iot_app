//! # 数据库迁移与版本控制模块
//!
//! 负责管理 SQLite 数据库的所有 DDL 操作与初始化 DML（包含种子数据）。
//! 所有数据迁移脚本通过专门的 `app_migrations` 记录表跟踪，确保任何升级或结构调整只执行一次。
//! 如果应用首次启动，此模块将依次载入 `schema_sql` 和后续的各类版本变动，提供类似于完整 ORM 的自动迁移效果。

use rusqlite::{Connection, OptionalExtension};

use crate::core::error::AppError;

pub(crate) const DATA_FIX_MIGRATION_ID: &str = "0003_legacy_offline_cleanup";
pub(crate) const USER_REGISTRATION_MIGRATION_ID: &str = "0004_user_registration_extension";
pub(crate) const PERMISSION_ROUTE_RENAME_MIGRATION_ID: &str =
    "0005_permission_page_to_user_registration";
pub(crate) const HIDE_BUTTON_PERMISSION_ROUTE_MIGRATION_ID: &str =
    "0006_hide_button_permission_route";

/// 载入最基础的数据表结构（如果尚不存在）。此文件必须使用 `IF NOT EXISTS`，防止破坏现有应用数据。
pub(crate) fn init_schema(connection: &Connection) -> Result<(), AppError> {
    connection
        .execute_batch(schema_sql())
        .map_err(|err| AppError::Database(err.to_string()))
}

/// 载入核心的种子业务数据（如 admin 账号），利用 `INSERT OR IGNORE` 确保只插入一次。
pub(crate) fn init_seed_data(connection: &Connection) -> Result<(), AppError> {
    connection
        .execute_batch(seed_sql())
        .map_err(|err| AppError::Database(err.to_string()))
}

/// 应用一次性数据修复脚本：清理早期的冗余配置，该迁移编号为 `0003`。
pub(crate) fn apply_one_time_data_fix(connection: &Connection) -> Result<(), AppError> {
    ensure_migration_log_table(connection)?;
    if is_data_fix_applied(connection)? {
        return Ok(());
    }

    connection
        .execute_batch(data_fix_sql())
        .map_err(|err| AppError::Database(err.to_string()))?;

    connection
        .execute(
            r"
            INSERT INTO app_migrations (id, applied_at)
            VALUES (?1, CAST(strftime('%s', 'now') AS INTEGER))
            ",
            [DATA_FIX_MIGRATION_ID],
        )
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

/// 迁移编号 `0004`：在原有的数据基础上扩展用户注册表的结构或配置字段。
pub(crate) fn apply_user_registration_extension(connection: &Connection) -> Result<(), AppError> {
    ensure_migration_log_table(connection)?;
    if is_user_registration_extension_applied(connection)? {
        return Ok(());
    }

    connection
        .execute_batch(user_registration_extension_sql())
        .map_err(|err| AppError::Database(err.to_string()))?;

    connection
        .execute(
            r"
            INSERT INTO app_migrations (id, applied_at)
            VALUES (?1, CAST(strftime('%s', 'now') AS INTEGER))
            ",
            [USER_REGISTRATION_MIGRATION_ID],
        )
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

/// 迁移编号 `0005`：重命名权限控制相关路由（从 permission_page 重命名为 user_registration）。
pub(crate) fn apply_permission_route_rename(connection: &Connection) -> Result<(), AppError> {
    ensure_migration_log_table(connection)?;
    if is_permission_route_rename_applied(connection)? {
        return Ok(());
    }

    connection
        .execute_batch(permission_route_rename_sql())
        .map_err(|err| AppError::Database(err.to_string()))?;

    connection
        .execute(
            r"
            INSERT INTO app_migrations (id, applied_at)
            VALUES (?1, CAST(strftime('%s', 'now') AS INTEGER))
            ",
            [PERMISSION_ROUTE_RENAME_MIGRATION_ID],
        )
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

/// 迁移编号 `0006`：隐藏按钮级别的权限路由以优化界面展示逻辑。
pub(crate) fn apply_hide_button_permission_route(connection: &Connection) -> Result<(), AppError> {
    ensure_migration_log_table(connection)?;
    if is_hide_button_permission_route_applied(connection)? {
        return Ok(());
    }

    connection
        .execute_batch(hide_button_permission_route_sql())
        .map_err(|err| AppError::Database(err.to_string()))?;

    connection
        .execute(
            r"
            INSERT INTO app_migrations (id, applied_at)
            VALUES (?1, CAST(strftime('%s', 'now') AS INTEGER))
            ",
            [HIDE_BUTTON_PERMISSION_ROUTE_MIGRATION_ID],
        )
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

/// 辅助函数：确保迁移日志表 `app_migrations` 存在。
fn ensure_migration_log_table(connection: &Connection) -> Result<(), AppError> {
    connection
        .execute_batch(
            r"
            CREATE TABLE IF NOT EXISTS app_migrations (
              id TEXT PRIMARY KEY,
              applied_at INTEGER NOT NULL
            );
            ",
        )
        .map_err(|err| AppError::Database(err.to_string()))
}

/// 查询 `0003_legacy_offline_cleanup` 迁移是否已执行过。
fn is_data_fix_applied(connection: &Connection) -> Result<bool, AppError> {
    connection
        .query_row(
            "SELECT 1 FROM app_migrations WHERE id = ?1 LIMIT 1",
            [DATA_FIX_MIGRATION_ID],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map(|row| row.is_some())
        .map_err(|err| AppError::Database(err.to_string()))
}

/// 查询 `0004_user_registration_extension` 迁移是否已执行过。
fn is_user_registration_extension_applied(connection: &Connection) -> Result<bool, AppError> {
    connection
        .query_row(
            "SELECT 1 FROM app_migrations WHERE id = ?1 LIMIT 1",
            [USER_REGISTRATION_MIGRATION_ID],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map(|row| row.is_some())
        .map_err(|err| AppError::Database(err.to_string()))
}

/// 查询 `0005_permission_page_to_user_registration` 迁移是否已执行过。
fn is_permission_route_rename_applied(connection: &Connection) -> Result<bool, AppError> {
    connection
        .query_row(
            "SELECT 1 FROM app_migrations WHERE id = ?1 LIMIT 1",
            [PERMISSION_ROUTE_RENAME_MIGRATION_ID],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map(|row| row.is_some())
        .map_err(|err| AppError::Database(err.to_string()))
}

/// 查询 `0006_hide_button_permission_route` 迁移是否已执行过。
fn is_hide_button_permission_route_applied(connection: &Connection) -> Result<bool, AppError> {
    connection
        .query_row(
            "SELECT 1 FROM app_migrations WHERE id = ?1 LIMIT 1",
            [HIDE_BUTTON_PERMISSION_ROUTE_MIGRATION_ID],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map(|row| row.is_some())
        .map_err(|err| AppError::Database(err.to_string()))
}

/// 返回内置的 `0001_schema.sql` SQL 文件文本内容。
pub(crate) fn schema_sql() -> &'static str {
    include_str!("migrations/0001_schema.sql")
}

/// 返回内置的 `0002_seed.sql` SQL 文件文本内容。
pub(crate) fn seed_sql() -> &'static str {
    include_str!("migrations/0002_seed.sql")
}

/// 返回内置的 `0003_legacy_offline_cleanup.sql` SQL 文件文本内容。
pub(crate) fn data_fix_sql() -> &'static str {
    include_str!("migrations/0003_legacy_offline_cleanup.sql")
}

/// 返回内置的 `0004_user_registration_extension.sql` SQL 文件文本内容。
pub(crate) fn user_registration_extension_sql() -> &'static str {
    include_str!("migrations/0004_user_registration_extension.sql")
}

/// 返回内置的 `0005_permission_page_to_user_registration.sql` SQL 文件文本内容。
pub(crate) fn permission_route_rename_sql() -> &'static str {
    include_str!("migrations/0005_permission_page_to_user_registration.sql")
}

/// 返回内置的 `0006_hide_button_permission_route.sql` SQL 文件文本内容。
pub(crate) fn hide_button_permission_route_sql() -> &'static str {
    include_str!("migrations/0006_hide_button_permission_route.sql")
}
