//! PostgreSQL 数据库访问入口模块
//!
//! 本模块提供 PostgreSQL 数据库的连接管理、初始化和迁移功能
//! 为上层服务层提供统一的数据访问接口

// 公开管理员仓储模块 - 包含用户管理等管理员功能
pub mod admin_repository;
// 公开鉴权仓储模块 - 包含用户登录、路由查询等功能
pub mod auth_repository;
// 公开 SeaORM 实体模型模块 - 包含 users、user_roles 等实体定义
pub mod entities;

// 私有模块 - 内部实现细节
mod bootstrap; // 数据库初始化引导 - 执行表创建和种子数据
mod migrations; // 数据库迁移管理 - 处理数据库版本升级
mod path_store; // 数据库连接路径存储 - 管理数据库 URL 配置

// 引入异步运行时和单例锁
use std::future::Future;
use std::sync::OnceLock;

// 引入 SeaORM 和 SQLx 相关类型
use sea_orm::{Database, DatabaseConnection}; // SeaORM 数据库连接和连接池
use sqlx::postgres::PgConnectOptions; // PostgreSQL 连接选项
use sqlx::{Connection as _, PgConnection}; // SQLx PostgreSQL 连接
use tokio::runtime::{Builder, Runtime}; // Tokio 异步运行时构建器

// 引入应用错误类型
use crate::core::error::AppError;

/// 设置数据库连接 URL（通常在 Tauri 初始化时调用一次）
///
/// # 参数
/// * `url` - PostgreSQL 连接字符串，格式: postgres://user:password@host:port/database
///
/// # 返回
/// * 成功返回 `Ok(())`
/// * 失败返回 `AppError`
pub fn set_database_url(url: String) -> Result<(), AppError> {
    path_store::set_database_url(url)
}

/// 获取当前数据库连接 URL
///
/// 优先级:
/// 1. 内存中设置的值（通过 set_database_url 设置）
/// 2. 运行时配置（`config/*.toml` + 环境变量覆盖）
///
/// # 返回
/// * 数据库连接字符串
pub fn database_url() -> String {
    path_store::database_url()
}

/// 获取测试数据库连接 URL
///
/// 优先级:
/// 1. 环境变量 PURE_ADMIN_TEST_DATABASE_URL（测试专用）
/// 2. 运行时配置 database.test_url
/// 3. 运行时配置 database.url
///
/// # 返回
/// * 测试数据库连接字符串
#[cfg(test)]
pub fn test_database_url() -> String {
    std::env::var("PURE_ADMIN_TEST_DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            crate::core::config::runtime_config()
                .database
                .test_url
                .clone()
        })
        .unwrap_or_else(|| crate::core::config::runtime_config().database.url.clone())
}

/// 获取异步运行时实例（单例模式）
///
/// 使用 OnceLock 确保全局只有一个 tokio 运行时
/// 多线程运行时用于执行异步数据库操作
///
/// # 返回
/// * 引用指向全局 Runtime 实例
fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("create sqlx runtime")
    })
}

/// 在同步上下文中执行异步 Future
///
/// 这是将 async 函数转换为同步调用的包装器
/// 允许在同步代码中调用异步数据库操作
///
/// # 参数
/// * `future` - 任意异步 Future
///
/// # 返回
/// * Future 的结果
pub(crate) fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    runtime().block_on(future)
}

/// 打开 PostgreSQL 连接（同步包装器）
///
/// 同步版本的异步连接函数
///
/// # 返回
/// * 成功返回 `PgConnection`
/// * 失败返回 `AppError`
pub fn connect() -> Result<PgConnection, AppError> {
    block_on(connect_async())
}

/// 打开 SeaORM 连接（同步包装器）
///
/// 用于基于实体的 CRUD 操作
/// SeaORM 提供了更高级的 ORM 功能
///
/// # 返回
/// * 成功返回 `DatabaseConnection`
/// * 失败返回 `AppError`
pub fn connect_orm() -> Result<DatabaseConnection, AppError> {
    block_on(connect_orm_async())
}

/// 打开异步 PostgreSQL 连接
///
/// 使用 sqlx 直接执行原生 SQL
///
/// # 返回
/// * 成功返回 `PgConnection`
/// * 失败返回 `AppError`
pub async fn connect_async() -> Result<PgConnection, AppError> {
    let url = path_store::database_url();
    let options = url
        .parse::<PgConnectOptions>()
        .map_err(|err| AppError::Database(format!("invalid postgres url: {err}")))?;

    PgConnection::connect_with(&options)
        .await
        .map_err(|err| AppError::Database(err.to_string()))
}

/// 打开异步 SeaORM 连接（用于实体 CRUD）
///
/// SeaORM 提供了类型安全的 ORM 查询
///
/// # 返回
/// * 成功返回 `DatabaseConnection`
/// * 失败返回 `AppError`
pub async fn connect_orm_async() -> Result<DatabaseConnection, AppError> {
    let url = path_store::database_url();
    Database::connect(url)
        .await
        .map_err(|err| AppError::Database(err.to_string()))
}

/// 初始化数据库表结构和种子数据（同步包装器）
///
/// 依次执行:
/// 1. 创建数据库表结构
/// 2. 插入初始种子数据
/// 3. 执行数据修复迁移
/// 4. 执行用户注册扩展迁移
/// 5. 执行权限路由重命名迁移
/// 6. 执行隐藏按钮权限路由迁移
///
/// # 返回
/// * 成功返回 `Ok(())`
/// * 失败返回 `AppError`
pub fn init_database() -> Result<(), AppError> {
    block_on(init_database_async())
}

/// 初始化数据库表结构和种子数据（异步版本）
///
/// # 返回
/// * 成功返回 `Ok(())`
/// * 失败返回 `AppError`
pub async fn init_database_async() -> Result<(), AppError> {
    bootstrap::init_database().await
}

/// 数据库模块测试模块
#[cfg(test)]
mod tests;
