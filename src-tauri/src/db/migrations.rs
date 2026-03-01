//! 数据库迁移管理模块
//! 
//! 本模块负责管理数据库的版本迁移，包括：
//! - 创建数据库表结构
//! - 插入初始种子数据
//! - 执行数据修复和扩展迁移
//! - 记录迁移执行状态（确保幂等性）

// 引入 SQLx 相关类型
use sqlx::{PgConnection, query, query_scalar, raw_sql};

// 引入应用错误类型
use crate::core::error::AppError;

/// 数据修复迁移的唯一标识符
/// 对应 migrations/0003_legacy_offline_cleanup.sql
pub(crate) const DATA_FIX_MIGRATION_ID: &str = "0003_legacy_offline_cleanup";

/// 用户注册扩展迁移的唯一标识符
/// 对应 migrations/0004_user_registration_extension.sql
pub(crate) const USER_REGISTRATION_MIGRATION_ID: &str = "0004_user_registration_extension";

/// 权限路由重命名迁移的唯一标识符
/// 对应 migrations/0005_permission_page_to_user_registration.sql
pub(crate) const PERMISSION_ROUTE_RENAME_MIGRATION_ID: &str =
    "0005_permission_page_to_user_registration";

/// 隐藏按钮权限路由迁移的唯一标识符
/// 对应 migrations/0006_hide_button_permission_route.sql
pub(crate) const HIDE_BUTTON_PERMISSION_ROUTE_MIGRATION_ID: &str =
    "0006_hide_button_permission_route";

/// 初始化数据库表结构
/// 
/// 执行 migrations/0001_schema.sql 中的所有 CREATE TABLE 语句
/// 创建系统所需的所有数据表
/// 
/// # 参数
/// * `connection` - 数据库连接
/// 
/// # 返回
/// * 成功返回 `Ok(())`
/// * 失败返回 `AppError`
pub(crate) async fn init_schema(connection: &mut PgConnection) -> Result<(), AppError> {
    // 使用 raw_sql 执行包含表结构的 SQL 脚本
    raw_sql(schema_sql())
        .execute(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(())
}

/// 初始化种子数据
/// 
/// 执行 migrations/0002_seed.sql 中的 INSERT 语句
/// 插入默认用户、权限定义、路由配置等初始数据
/// 
/// # 参数
/// * `connection` - 数据库连接
/// 
/// # 返回
/// * 成功返回 `Ok(())`
/// * 失败返回 `AppError`
pub(crate) async fn init_seed_data(connection: &mut PgConnection) -> Result<(), AppError> {
    // 使用 raw_sql 执行包含种子数据的 SQL 脚本
    // 使用 ON CONFLICT DO NOTHING 确保幂等性
    raw_sql(seed_sql())
        .execute(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(())
}

/// 应用一次性数据修复迁移
/// 
/// 清理旧版本遗留的外链数据（头像 URL、图标等）
/// 确保数据符合离线安全设计原则
/// 
/// # 参数
/// * `connection` - 数据库连接
/// 
/// # 返回
/// * 成功返回 `Ok(())`
/// * 失败返回 `AppError`
pub(crate) async fn apply_one_time_data_fix(connection: &mut PgConnection) -> Result<(), AppError> {
    // 确保迁移日志表存在
    ensure_migration_log_table(connection).await?;
    
    // 检查该迁移是否已执行过（幂等性保证）
    if is_data_fix_applied(connection).await? {
        return Ok(());
    }

    // 执行数据修复 SQL
    raw_sql(data_fix_sql())
        .execute(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    // 记录迁移执行状态
    query(
        r"
        INSERT INTO app_migrations (id, applied_at)
        VALUES ($1, EXTRACT(EPOCH FROM NOW())::BIGINT)
        ",
    )
    .bind(DATA_FIX_MIGRATION_ID)
    .execute(&mut *connection)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

/// 应用用户注册扩展迁移
/// 
/// 为用户表添加扩展字段（手机号、账号有效期等）
/// 支持管理员注册和账号生命周期管理功能
/// 
/// # 参数
/// * `connection` - 数据库连接
/// 
/// # 返回
/// * 成功返回 `Ok(())`
/// * 失败返回 `AppError`
pub(crate) async fn apply_user_registration_extension(
    connection: &mut PgConnection,
) -> Result<(), AppError> {
    // 确保迁移日志表存在
    ensure_migration_log_table(connection).await?;
    
    // 检查该迁移是否已执行过
    if is_user_registration_extension_applied(connection).await? {
        return Ok(());
    }

    // 执行用户扩展字段 SQL
    raw_sql(user_registration_extension_sql())
        .execute(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    // 记录迁移执行状态
    query(
        r"
        INSERT INTO app_migrations (id, applied_at)
        VALUES ($1, EXTRACT(EPOCH FROM NOW())::BIGINT)
        ",
    )
    .bind(USER_REGISTRATION_MIGRATION_ID)
    .execute(&mut *connection)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

/// 应用权限路由重命名迁移
/// 
/// 更新路由的 meta_title 字段以匹配新功能
/// 
/// # 参数
/// * `connection` - 数据库连接
/// 
/// # 返回
/// * 成功返回 `Ok(())`
/// * 失败返回 `AppError`
pub(crate) async fn apply_permission_route_rename(
    connection: &mut PgConnection,
) -> Result<(), AppError> {
    // 确保迁移日志表存在
    ensure_migration_log_table(connection).await?;
    
    // 检查该迁移是否已执行过
    if is_permission_route_rename_applied(connection).await? {
        return Ok(());
    }

    // 执行权限路由重命名 SQL
    raw_sql(permission_route_rename_sql())
        .execute(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    // 记录迁移执行状态
    query(
        r"
        INSERT INTO app_migrations (id, applied_at)
        VALUES ($1, EXTRACT(EPOCH FROM NOW())::BIGINT)
        ",
    )
    .bind(PERMISSION_ROUTE_RENAME_MIGRATION_ID)
    .execute(&mut *connection)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

/// 应用隐藏按钮权限路由迁移
/// 
/// 删除不需要的按钮权限路由
/// 
/// # 参数
/// * `connection` - 数据库连接
/// 
/// # 返回
/// * 成功返回 `Ok(())`
/// * 失败返回 `AppError`
pub(crate) async fn apply_hide_button_permission_route(
    connection: &mut PgConnection,
) -> Result<(), AppError> {
    // 确保迁移日志表存在
    ensure_migration_log_table(connection).await?;
    
    // 检查该迁移是否已执行过
    if is_hide_button_permission_route_applied(connection).await? {
        return Ok(());
    }

    // 执行隐藏按钮权限路由 SQL
    raw_sql(hide_button_permission_route_sql())
        .execute(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    // 记录迁移执行状态
    query(
        r"
        INSERT INTO app_migrations (id, applied_at)
        VALUES ($1, EXTRACT(EPOCH FROM NOW())::BIGINT)
        ",
    )
    .bind(HIDE_BUTTON_PERMISSION_ROUTE_MIGRATION_ID)
    .execute(&mut *connection)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

/// 确保迁移日志表存在
/// 
/// 创建 app_migrations 表用于记录已执行的迁移
/// 
/// # 参数
/// * `connection` - 数据库连接
/// 
/// # 返回
/// * 成功返回 `Ok(())`
/// * 失败返回 `AppError`
async fn ensure_migration_log_table(connection: &mut PgConnection) -> Result<(), AppError> {
    query(
        r"
        CREATE TABLE IF NOT EXISTS app_migrations (
          id TEXT PRIMARY KEY,
          applied_at BIGINT NOT NULL
        );
        ",
    )
    .execute(&mut *connection)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

/// 检查数据修复迁移是否已应用
/// 
/// # 参数
/// * `connection` - 数据库连接
/// 
/// # 返回
/// * 已应用返回 true，否则返回 false
async fn is_data_fix_applied(connection: &mut PgConnection) -> Result<bool, AppError> {
    let row = query_scalar::<_, i32>("SELECT 1 FROM app_migrations WHERE id = $1 LIMIT 1")
        .bind(DATA_FIX_MIGRATION_ID)
        .fetch_optional(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(row.is_some())
}

/// 检查用户注册扩展迁移是否已应用
/// 
/// # 参数
/// * `connection` - 数据库连接
/// 
/// # 返回
/// * 已应用返回 true，否则返回 false
async fn is_user_registration_extension_applied(
    connection: &mut PgConnection,
) -> Result<bool, AppError> {
    let row = query_scalar::<_, i32>("SELECT 1 FROM app_migrations WHERE id = $1 LIMIT 1")
        .bind(USER_REGISTRATION_MIGRATION_ID)
        .fetch_optional(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(row.is_some())
}

/// 检查权限路由重命名迁移是否已应用
/// 
/// # 参数
/// * `connection` - 数据库连接
/// 
/// # 返回
/// * 已应用返回 true，否则返回 false
async fn is_permission_route_rename_applied(
    connection: &mut PgConnection,
) -> Result<bool, AppError> {
    let row = query_scalar::<_, i32>("SELECT 1 FROM app_migrations WHERE id = $1 LIMIT 1")
        .bind(PERMISSION_ROUTE_RENAME_MIGRATION_ID)
        .fetch_optional(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(row.is_some())
}

/// 检查隐藏按钮权限路由迁移是否已应用
/// 
/// # 参数
/// * `connection` - 数据库连接
/// 
/// # 返回
/// * 已应用返回 true，否则返回 false
async fn is_hide_button_permission_route_applied(
    connection: &mut PgConnection,
) -> Result<bool, AppError> {
    let row = query_scalar::<_, i32>("SELECT 1 FROM app_migrations WHERE id = $1 LIMIT 1")
        .bind(HIDE_BUTTON_PERMISSION_ROUTE_MIGRATION_ID)
        .fetch_optional(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(row.is_some())
}

/// 获取表结构 SQL 脚本
/// 
/// # 返回
/// * 0001_schema.sql 文件内容的静态引用
pub(crate) fn schema_sql() -> &'static str {
    include_str!("migrations/0001_schema.sql")
}

/// 获取种子数据 SQL 脚本
/// 
/// # 返回
/// * 0002_seed.sql 文件内容的静态引用
pub(crate) fn seed_sql() -> &'static str {
    include_str!("migrations/0002_seed.sql")
}

/// 获取数据修复 SQL 脚本
/// 
/// # 返回
/// * 0003_legacy_offline_cleanup.sql 文件内容的静态引用
pub(crate) fn data_fix_sql() -> &'static str {
    include_str!("migrations/0003_legacy_offline_cleanup.sql")
}

/// 获取用户注册扩展 SQL 脚本
/// 
/// # 返回
/// * 0004_user_registration_extension.sql 文件内容的静态引用
pub(crate) fn user_registration_extension_sql() -> &'static str {
    include_str!("migrations/0004_user_registration_extension.sql")
}

/// 获取权限路由重命名 SQL 脚本
/// 
/// # 返回
/// * 0005_permission_page_to_user_registration.sql 文件内容的静态引用
pub(crate) fn permission_route_rename_sql() -> &'static str {
    include_str!("migrations/0005_permission_page_to_user_registration.sql")
}

/// 获取隐藏按钮权限路由 SQL 脚本
/// 
/// # 返回
/// * 0006_hide_button_permission_route.sql 文件内容的静态引用
pub(crate) fn hide_button_permission_route_sql() -> &'static str {
    include_str!("migrations/0006_hide_button_permission_route.sql")
}
