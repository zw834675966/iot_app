//! SQLx 报表查询模块
//! 
//! 本模块使用原生 SQL（通过 SQLx）执行报表相关的查询
//! 适用于复杂的聚合查询和报表数据提取

// 引入 SQLx 查询相关类型
use sqlx::{Row, query, query_scalar};

// 引入应用错误类型
use crate::core::error::AppError;

// 引入数据库模块
use crate::db;

// 引入父模块的数据结构
use super::{ManagedUserRecord, split_csv_sorted};

/// 查询用户的有效角色
/// 
/// 有效角色是指用户当前可用的角色（考虑账号有效期）
/// 
/// # 参数
/// * `username` - 用户名
/// * `now_millis` - 当前时间戳（用于判断账号是否过期）
/// 
/// # 返回
/// * 有效角色列表
pub(super) fn find_effective_roles(
    username: &str,
    now_millis: i64,
) -> Result<Vec<String>, AppError> {
    db::block_on(async {
        let mut connection = db::connect_async().await?;

        // 使用 SQLx 执行复杂查询
        // 查询条件包括：账号激活状态 + 账号有效期判断
        let row = query_scalar::<_, String>(
            r"
            SELECT COALESCE(STRING_AGG(DISTINCT ur.role, ','), '') AS roles
            FROM users u
            LEFT JOIN user_roles ur ON ur.user_id = u.id
            WHERE u.username = $1
              AND u.is_active = 1
              AND (
                COALESCE(u.account_is_permanent, 1) = 1
                OR u.account_expire_at IS NULL
                OR u.account_expire_at > $2
              )
            GROUP BY u.id
            LIMIT 1
            ",
        )
        .bind(username)
        .bind(now_millis)
        .fetch_optional(&mut connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

        // 拆分并排序角色列表
        Ok(row.map_or_else(Vec::new, |roles_csv| split_csv_sorted(&roles_csv)))
    })
}

/// 检查用户是否为管理员
/// 
/// # 参数
/// * `username` - 用户名
/// * `now_millis` - 当前时间戳
/// 
/// # 返回
/// * 包含 admin 角色返回 true
pub(super) fn is_admin_user(username: &str, now_millis: i64) -> Result<bool, AppError> {
    let roles = find_effective_roles(username, now_millis)?;
    Ok(roles.iter().any(|role| role == "admin"))
}

/// 获取所有用户列表
/// 
/// 查询所有用户及其关联的角色信息
/// 
/// # 返回
/// * 所有可管理的用户记录列表
pub(super) fn list_users() -> Result<Vec<ManagedUserRecord>, AppError> {
    db::block_on(async {
        let mut connection = db::connect_async().await?;
        
        // 查询用户及其角色（使用 LEFT JOIN 保留没有角色的用户）
        let rows = query(
            r"
            SELECT
              u.id,
              u.username,
              u.nickname,
              u.phone,
              u.is_active,
              COALESCE(u.account_is_permanent, 1) AS account_is_permanent,
              u.account_valid_days,
              u.account_expire_at,
              u.created_at,
              u.updated_at,
              u.created_by,
              COALESCE(STRING_AGG(DISTINCT ur.role, ','), '') AS roles
            FROM users u
            LEFT JOIN user_roles ur ON ur.user_id = u.id
            GROUP BY
              u.id,
              u.username,
              u.nickname,
              u.phone,
              u.is_active,
              u.account_is_permanent,
              u.account_valid_days,
              u.account_expire_at,
              u.created_at,
              u.updated_at,
              u.created_by
            ORDER BY u.id ASC
            ",
        )
        .fetch_all(&mut connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

        // 将查询结果转换为 ManagedUserRecord 列表
        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            // 提取字段值
            let is_active: i32 = row
                .try_get(4)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let account_is_permanent: i32 = row
                .try_get(5)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let roles_csv: String = row
                .try_get(11)
                .map_err(|err| AppError::Database(err.to_string()))?;

            // 构建用户记录
            records.push(ManagedUserRecord {
                user_id: row
                    .try_get(0)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                username: row
                    .try_get(1)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                nickname: row
                    .try_get(2)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                phone: row
                    .try_get(3)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                roles: split_csv_sorted(&roles_csv),
                is_active: is_active == 1,
                account_is_permanent: account_is_permanent == 1,
                account_valid_days: row
                    .try_get(6)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                account_expire_at: row
                    .try_get(7)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                created_at: row
                    .try_get(8)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                updated_at: row
                    .try_get(9)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                created_by: row
                    .try_get(10)
                    .map_err(|err| AppError::Database(err.to_string()))?,
            });
        }

        Ok(records)
    })
}
