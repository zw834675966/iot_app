//! 管理员数据仓储模块
//! 
//! 本模块提供管理员功能相关的数据访问接口：
//! - 用户创建、更新、删除
//! - 用户账号续期
//! - 用户列表查询
//! - 管理员权限验证
//! 
//! 采用仓储模式封装数据访问逻辑

// 引入标准库集合类型
use std::collections::HashSet;

// 引入应用错误类型
use crate::core::error::AppError;

/// SeaORM 用户管理模块（使用 SeaORM ORM 框架）
#[path = "admin_repository/seaorm_users.rs"]
mod seaorm_users;

/// SQLx 报表查询模块（使用原生 SQL 查询）
#[path = "admin_repository/sqlx_reports.rs"]
mod sqlx_reports;

/// 新用户输入数据结构
/// 
/// 用于创建新用户时的输入参数
pub struct NewUserInput {
    pub username: String,           // 用户名（唯一标识）
    pub password: String,           // 密码（明文，会被哈希存储）
    pub nickname: String,           // 昵称
    pub phone: Option<String>,      // 手机号（可选）
    pub roles: Vec<String>,        // 角色列表
    pub account_is_permanent: bool, // 是否永久账号
    pub account_valid_days: Option<i64>, // 有效天数（非永久账号）
    pub account_expire_at: Option<i64>,  // 过期时间戳（毫秒）
    pub created_by: String,        // 创建者用户名
    pub now_millis: i64,          // 当前时间戳（毫秒）
}

/// 已注册用户记录数据结构
/// 
/// 用户创建成功后返回的用户信息
#[derive(Debug, Clone)]
pub struct RegisteredUserRecord {
    pub user_id: i64,              // 用户 ID
    pub username: String,           // 用户名
    pub roles: Vec<String>,        // 角色列表
    pub is_active: bool,           // 是否激活
    pub account_is_permanent: bool, // 是否永久账号
    pub account_expire_at: Option<i64>, // 过期时间戳
}

/// 用户登录状态数据结构
/// 
/// 用于验证用户登录资格
#[derive(Debug, Clone)]
pub struct UserLoginState {
    pub is_active: bool,           // 是否激活
    pub account_is_permanent: bool, // 是否永久账号
    pub account_expire_at: Option<i64>, // 过期时间戳
}

/// 可管理的用户记录数据结构
/// 
/// 包含完整的用户信息，用于列表展示和详情查看
#[derive(Debug, Clone)]
pub struct ManagedUserRecord {
    pub user_id: i64,              // 用户 ID
    pub username: String,           // 用户名
    pub nickname: String,           // 昵称
    pub phone: Option<String>,      // 手机号
    pub roles: Vec<String>,        // 角色列表
    pub is_active: bool,           // 是否激活
    pub account_is_permanent: bool, // 是否永久账号
    pub account_valid_days: Option<i64>, // 有效天数
    pub account_expire_at: Option<i64>, // 过期时间戳
    pub created_at: Option<i64>,   // 创建时间戳
    pub updated_at: Option<i64>,   // 更新时间戳
    pub created_by: Option<String>, // 创建者
}

/// 用户更新输入数据结构
/// 
/// 用于更新用户信息时的输入参数
pub struct UpdateUserInput {
    pub user_id: i64,              // 用户 ID
    pub username: String,           // 用户名
    pub nickname: String,           // 昵称
    pub phone: Option<String>,      // 手机号
    pub roles: Vec<String>,        // 角色列表
    pub is_active: bool,           // 是否激活
    pub account_is_permanent: bool, // 是否永久账号
    pub account_valid_days: Option<i64>, // 有效天数
    pub account_expire_at: Option<i64>, // 过期时间戳
    pub now_millis: i64,          // 当前时间戳
}

/// 创建新用户
/// 
/// # 参数
/// * `input` - 新用户输入数据
/// 
/// # 返回
/// * 成功返回已注册用户记录
/// * 失败返回 AppError
pub fn create_user(input: NewUserInput) -> Result<RegisteredUserRecord, AppError> {
    seaorm_users::create_user(input)
}

/// 续期用户账号
/// 
/// # 参数
/// * `user_id` - 用户 ID
/// * `account_is_permanent` - 是否永久账号
/// * `account_valid_days` - 有效天数
/// * `account_expire_at` - 过期时间戳
/// * `now_millis` - 当前时间戳
/// 
/// # 返回
/// * 成功返回已注册用户记录
/// * 失败返回 AppError
pub fn renew_user_account(
    user_id: i64,
    account_is_permanent: bool,
    account_valid_days: Option<i64>,
    account_expire_at: Option<i64>,
    now_millis: i64,
) -> Result<RegisteredUserRecord, AppError> {
    seaorm_users::renew_user_account(
        user_id,
        account_is_permanent,
        account_valid_days,
        account_expire_at,
        now_millis,
    )
}

/// 检查用户是否为管理员
/// 
/// # 参数
/// * `username` - 用户名
/// * `now_millis` - 当前时间戳
/// 
/// # 返回
/// * 是管理员返回 true，否则返回 false
pub fn is_admin_user(username: &str, now_millis: i64) -> Result<bool, AppError> {
    sqlx_reports::is_admin_user(username, now_millis)
}

/// 查询用户的有效角色
/// 
/// # 参数
/// * `username` - 用户名
/// * `now_millis` - 当前时间戳
/// 
/// # 返回
/// * 有效角色列表
pub fn find_effective_roles(username: &str, now_millis: i64) -> Result<Vec<String>, AppError> {
    sqlx_reports::find_effective_roles(username, now_millis)
}

/// 查询用户登录状态
/// 
/// # 参数
/// * `username` - 用户名
/// 
/// # 返回
/// * 登录状态（包含是否激活、账号有效期等信息）
pub fn find_user_login_state(username: &str) -> Result<Option<UserLoginState>, AppError> {
    seaorm_users::find_user_login_state(username)
}

/// 停用指定用户
/// 
/// # 参数
/// * `username` - 用户名
/// * `now_millis` - 当前时间戳
pub fn deactivate_user_by_username(username: &str, now_millis: i64) -> Result<(), AppError> {
    seaorm_users::deactivate_user_by_username(username, now_millis)
}

/// 停用所有已过期的用户
/// 
/// # 参数
/// * `now_millis` - 当前时间戳
/// 
/// # 返回
/// * 成功停用的用户数量
pub fn deactivate_expired_users(now_millis: i64) -> Result<usize, AppError> {
    seaorm_users::deactivate_expired_users(now_millis)
}

/// 获取所有用户列表
/// 
/// # 返回
/// * 所有可管理的用户记录列表
pub fn list_users() -> Result<Vec<ManagedUserRecord>, AppError> {
    sqlx_reports::list_users()
}

/// 更新用户信息
/// 
/// # 参数
/// * `input` - 用户更新输入数据
/// 
/// # 返回
/// * 更新后的用户记录
pub fn update_user(input: UpdateUserInput) -> Result<ManagedUserRecord, AppError> {
    seaorm_users::update_user(input)
}

/// 删除用户
/// 
/// # 参数
/// * `user_id` - 用户 ID
/// 
/// # 返回
/// * 删除成功返回 true
pub fn delete_user(user_id: i64) -> Result<bool, AppError> {
    seaorm_users::delete_user(user_id)
}

/// 更新用户密码
/// 
/// # 参数
/// * `user_id` - 用户 ID
/// * `password` - 新密码
/// * `now_millis` - 当前时间戳
/// 
/// # 返回
/// * 更新后的用户记录
pub fn update_user_password(
    user_id: i64,
    password: &str,
    now_millis: i64,
) -> Result<ManagedUserRecord, AppError> {
    seaorm_users::update_user_password(user_id, password, now_millis)
}

/// 根据用户 ID 查询用户名
/// 
/// # 参数
/// * `user_id` - 用户 ID
/// 
/// # 返回
/// * 用户名（如果存在）
pub fn find_username_by_user_id(user_id: i64) -> Result<Option<String>, AppError> {
    seaorm_users::find_username_by_user_id(user_id)
}

/// 规范化角色列表（去重、排序）
/// 
/// # 参数
/// * `raw_roles` - 原始角色列表
/// 
/// # 返回
/// * 去重排序后的角色列表
pub(super) fn normalize_unique_roles(raw_roles: Vec<String>) -> Vec<String> {
    let unique_roles: HashSet<String> = raw_roles
        .into_iter()
        .map(|role| role.trim().to_string())
        .filter(|role| !role.is_empty())
        .collect();
    let mut roles: Vec<String> = unique_roles.into_iter().collect();
    roles.sort();
    roles
}

/// 拆分并排序逗号分隔的字符串
/// 
/// # 参数
/// * `raw` - 原始逗号分隔字符串
/// 
/// # 返回
/// * 去重排序后的字符串向量
pub(super) fn split_csv_sorted(raw: &str) -> Vec<String> {
    let mut values: Vec<String> = raw
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect();
    values.sort();
    values.dedup();
    values
}

/// 清理可选手机号
/// 
/// 如果手机号为空字符串，返回 None
/// 
/// # 参数
/// * `phone` - 原始手机号（可选）
/// 
/// # 返回
/// * 清理后的手机号（可能为 None）
pub(super) fn trim_optional_phone(phone: Option<String>) -> Option<String> {
    phone.and_then(|raw| {
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

/// 将用户操作错误映射为应用错误
/// 
/// # 参数
/// * `message` - 错误消息
/// 
/// # 返回
/// * 映射后的 AppError
pub(super) fn map_user_mutation_error(message: String) -> AppError {
    let lowered = message.to_lowercase();
    // 检查是否为用户名重复错误
    if lowered.contains("users.username")
        || lowered.contains("users_username_key")
        || lowered.contains("duplicate key value violates unique constraint")
    {
        return AppError::Validation("username already exists".to_string());
    }

    AppError::Database(message)
}
