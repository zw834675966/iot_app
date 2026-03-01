//! 数据库连接路径存储模块
//!
//! 本模块负责管理 PostgreSQL 数据库连接 URL 的存储和获取
//! 使用单例模式确保全局只有一个数据库 URL 配置

// 引入标准库的单例锁
use std::sync::OnceLock;

// 引入应用错误类型
use crate::core::error::AppError;

/// 数据库连接 URL 的静态存储（单例模式）
///
/// OnceLock 确保全局只有一个 URL 实例
/// 线程安全且延迟初始化
static DB_URL: OnceLock<String> = OnceLock::new();

/// 设置数据库连接 URL
///
/// 仅在首次调用时生效，后续调用会被忽略
/// 这确保了数据库 URL 在应用生命周期内保持一致
///
/// # 参数
/// * `url` - PostgreSQL 连接字符串
///
/// # 返回
/// * 始终返回 `Ok(())`（允许重复调用）
#[allow(clippy::unnecessary_wraps)]
pub fn set_database_url(url: String) -> Result<(), AppError> {
    // 检查是否已设置过 URL
    if DB_URL.get().is_some() {
        return Ok(());
    }

    // 尝试设置 URL（OnceLock::set 只在首次调用时成功）
    if DB_URL.set(url).is_err() {
        return Ok(());
    }

    Ok(())
}

/// 获取数据库连接 URL
///
/// 获取优先级:
/// 1. 内存中已设置的 URL
/// 2. 运行时配置文件中的数据库 URL
///
/// # 返回
/// * 数据库连接字符串
pub(crate) fn database_url() -> String {
    // 优先返回内存中设置的 URL
    if let Some(url) = DB_URL.get() {
        return url.clone();
    }

    // 回退到配置文件中的数据库 URL
    crate::core::config::runtime_config().database.url.clone()
}
