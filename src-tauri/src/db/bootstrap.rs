use sqlx::query;

// 引入应用错误类型
use crate::core::error::AppError;

// 引入父模块的连接函数和迁移模块
use super::{connect_async, migrations};

/// 初始化数据库（异步函数）
///
/// 这是数据库初始化的主入口函数，执行以下步骤:
/// 1. 建立数据库连接
/// 2. 获取咨询锁防止并发初始化
/// 3. 执行数据库迁移（表结构、种子数据、数据修复等）
/// 4. 释放咨询锁
///
/// # 返回
/// * 成功返回 `Ok(())`
/// * 失败返回 `AppError`
pub(crate) async fn init_database() -> Result<(), AppError> {
    // 第一步：建立异步数据库连接
    let mut connection = connect_async().await?;

    // 定义咨询锁键值 - 用于防止多个应用实例同时初始化数据库
    // 这是一个业务特定的唯一键
    let advisory_lock_key: i64 = 2026022801;

    // 第二步：获取 PostgreSQL 咨询锁
    // 咨询锁是数据库级别的互斥锁，确保并发安全
    query("SELECT pg_advisory_lock($1)")
        .bind(advisory_lock_key)
        .execute(&mut connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    // 第三步：执行所有数据库迁移操作
    let init_result = async {
        // 3.1 创建所有数据库表结构
        migrations::init_schema(&mut connection).await?;

        // 3.2 插入初始种子数据（默认用户、权限、路由等）
        migrations::init_seed_data(&mut connection).await?;

        // 3.3 执行一次性数据修复（清理旧版本的外链数据）
        migrations::apply_one_time_data_fix(&mut connection).await?;

        // 3.4 执行用户注册扩展迁移（添加手机号、账号有效期等字段）
        migrations::apply_user_registration_extension(&mut connection).await?;

        // 3.5 执行权限路由重命名迁移
        migrations::apply_permission_route_rename(&mut connection).await?;

        // 3.6 执行隐藏按钮权限路由迁移
        migrations::apply_hide_button_permission_route(&mut connection).await?;

        Ok::<(), AppError>(())
    }
    .await;

    // 第四步：释放咨询锁
    let _ = query("SELECT pg_advisory_unlock($1)")
        .bind(advisory_lock_key)
        .execute(&mut connection)
        .await;

    // 返回初始化结果
    init_result
}
