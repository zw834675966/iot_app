//! ==========================================================================================
//! 管理员业务逻辑层（领域层）
//!
//! 模块职责：
//! 这里承载了管理员操作所有核心的业务规则校验和执行。
//! 它不感知 Tauri 框架本身，且完全通过纯函数的形式编写（易于测试）。
//! 依赖 `admin_repository` 进行底层数据的存取操作。
//!
//! 核心功能：
//! - 用户注册与管理
//! - 用户账号续期
//! - 用户信息更新
//! - 用户删除
//! - 密码重置
//! - 用户状态检查
//!
//! 设计原则：
//! - 纯函数：所有业务函数不包含副作用，结果只依赖于输入参数
//! - 无框架依赖：不引入任何 Tauri 特定代码，便于单元测试
//! - 错误显式：所有可能失败的操作返回 `Result<T, AppError>` 类型
//! - 安全防护：防止误删/误改核心管理员账号
//!
//! 角色定义：
//!
//! | 角色 | 说明 |
//! |------|------|
//! | admin | 超级管理员，拥有所有权限 |
//! | operator | 操作员，可进行日常操作 |
//! | tenant | 租户，拥有受限的管理权限 |
//! | maintainer | 维护人员，负责系统维护 |
//!
//! 账号期限类型：
//!
//! | 类型 | 说明 |
//! |------|------|
//! | permanent | 永久有效，无过期时间 |
//! | days | 指定天数后过期 |
//!
//! ==========================================================================================

// 引入标准库的 HashSet，用于去重
use std::collections::HashSet;

// 引入鉴权模块的所有模型定义
use crate::auth::models::{
    AdminChangeUserPasswordData, AdminChangeUserPasswordPayload, AdminDeleteUserPayload,
    AdminListUsersPayload, AdminManagedUserData, AdminRegisterUserPayload, AdminRegisteredUserData,
    AdminRenewUserAccountData, AdminRenewUserAccountPayload, AdminUpdateUserPayload,
};
// 引入核心错误处理模块
use crate::core::error::AppError;
// 引入管理员数据访问层
use crate::db::admin_repository;

// ==========================================================================================
// 常量定义
// ==========================================================================================

// 预留设备范围接口的消息
const RESERVED_DEVICE_SCOPE_MESSAGE: &str = "RESERVED_API_NOT_IMPLEMENTED";

// 受保护的管理员用户名
const PROTECTED_ADMIN_USERNAME: &str = "admin";

// 操作员角色标识
const ROLE_OPERATOR: &str = "operator";

// 租户角色标识
const ROLE_TENANT: &str = "tenant";

// 维护人员角色标识
const ROLE_MAINTAINER: &str = "maintainer";

// 永久期限类型标识
const TERM_PERMANENT: &str = "permanent";

// 按天期限类型标识
const TERM_DAYS: &str = "days";

// ==========================================================================================
// 用户注册
// ==========================================================================================

// 管理员注册新用户
//
// 功能说明：
// 根据管理员提供的参数在系统中新增一名用户。
// 首先进行角色规范化、然后计算有效期、随机盐值与密码散列计算，最后将结果落库。
//
// 参数说明：
// - payload: 包含新用户所有信息的请求体
// - now_millis: 当前时间戳（毫秒）
//
// 返回值：
// - 成功：返回新创建用户的 ID、用户名、角色和账号状态
// - 失败：返回 AppError 错误
//
// 执行流程：
// 1. 校验操作员是否为管理员
// 2. 校验新用户名的有效性
// 3. 校验密码的有效性
// 4. 校验昵称的有效性
// 5. 校验手机号格式（如果提供）
// 6. 规范化角色列表
// 7. 计算账号有效期
// 8. 调用数据访问层创建用户
pub fn register_user_by_admin(
    payload: AdminRegisterUserPayload,
    now_millis: u64,
) -> Result<AdminRegisteredUserData, AppError> {
    // 将时间戳转换为 i64 类型
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;

    // 获取并校验操作员用户名
    let operator_username = payload.operator_username.trim().to_string();
    if operator_username.is_empty() {
        return Err(AppError::Validation(
            "operatorUsername is required".to_string(),
        ));
    }
    // 验证操作员是否为管理员
    assert_operator_is_admin(&operator_username, now_millis)?;

    // 获取并校验新用户名
    let username = payload.username.trim().to_string();
    if username.is_empty() {
        return Err(AppError::Validation("username is required".to_string()));
    }
    // 获取并校验密码
    let password = payload.password.trim().to_string();
    if password.is_empty() {
        return Err(AppError::Validation("password is required".to_string()));
    }
    // 获取并校验昵称
    let nickname = payload.nickname.trim().to_string();
    if nickname.is_empty() {
        return Err(AppError::Validation("nickname is required".to_string()));
    }

    // 校验手机号格式（如果提供）
    validate_phone(payload.phone.as_deref())?;
    // 规范化角色列表
    let roles = normalize_roles(payload.roles)?;
    // 计算账号有效期
    let (account_is_permanent, account_valid_days, account_expire_at) = build_account_term(
        payload.account_term_type.as_str(),
        payload.account_valid_days,
        now_millis,
    )?;

    // 调用数据访问层创建用户
    let result = admin_repository::create_user(admin_repository::NewUserInput {
        username,
        password,
        nickname,
        phone: payload.phone,
        roles,
        account_is_permanent,
        account_valid_days,
        account_expire_at,
        created_by: operator_username,
        now_millis,
    })?;

    // 返回创建结果
    Ok(AdminRegisteredUserData {
        user_id: result.user_id,
        username: result.username,
        roles: result.roles,
        is_active: result.is_active,
        account_is_permanent: result.account_is_permanent,
        account_expire_at: result.account_expire_at,
    })
}

// 管理员续期用户账号
//
// 功能说明：

// ==========================================================================================
// 用户账号续期
// ==========================================================================================

// 管理员续期用户账号

// 功能说明：
// 对指定用户的账户时效进行展期。

// 失败情形：
// 1. 尝试给超级管理员（admin）设定过期时间（它是永久的）
// 2. 传入的续期时长为负或不合法

// 参数说明：
// - payload: 包含续期信息的请求体
// - now_millis: 当前时间戳（毫秒）

// 返回值：
// - 成功：返回更新后的账号状态
// - 失败：返回 AppError 错误
pub fn renew_user_account_by_admin(
    payload: AdminRenewUserAccountPayload,
    now_millis: u64,
) -> Result<AdminRenewUserAccountData, AppError> {
    // 将时间戳转换为 i64 类型
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;

    // 获取并校验操作员用户名
    let operator_username = payload.operator_username.trim().to_string();
    if operator_username.is_empty() {
        return Err(AppError::Validation(
            "operatorUsername is required".to_string(),
        ));
    }
    // 验证操作员是否为管理员
    assert_operator_is_admin(&operator_username, now_millis)?;

    // 校验用户 ID
    if payload.user_id <= 0 {
        return Err(AppError::Validation("userId is required".to_string()));
    }
    // 验证目标用户是否可编辑
    assert_target_user_editable(payload.user_id)?;

    // 处理续期模式
    let renew_mode = payload.renew_mode.trim().to_ascii_lowercase();
    let (account_is_permanent, account_valid_days, account_expire_at) =
        if renew_mode == TERM_PERMANENT {
            // 永久模式
            (true, None, None)
        } else if renew_mode == TERM_DAYS {
            // 按天模式
            let renew_days = payload
                .renew_days
                .ok_or_else(|| AppError::Validation("renewDays is required".to_string()))?;
            if renew_days <= 0 {
                return Err(AppError::Validation(
                    "renewDays must be greater than 0".to_string(),
                ));
            }
            // 计算毫秒数
            let millis = renew_days
                .checked_mul(24 * 60 * 60 * 1000)
                .ok_or_else(|| AppError::Validation("renewDays is too large".to_string()))?;
            // 计算过期时间
            let expire_at = now_millis
                .checked_add(millis)
                .ok_or_else(|| AppError::Validation("renewDays is too large".to_string()))?;
            (false, Some(renew_days), Some(expire_at))
        } else {
            return Err(AppError::Validation(
                "renewMode must be 'permanent' or 'days'".to_string(),
            ));
        };

    // 调用数据访问层续期账号
    let result = admin_repository::renew_user_account(
        payload.user_id,
        account_is_permanent,
        account_valid_days,
        account_expire_at,
        now_millis,
    )?;

    // 返回续期结果
    Ok(AdminRenewUserAccountData {
        user_id: result.user_id,
        account_is_permanent: result.account_is_permanent,
        account_expire_at: result.account_expire_at,
        is_active: result.is_active,
    })
}

// ==========================================================================================
// 用户列表
// ==========================================================================================

// 管理员列出所有用户

// 功能说明：
// 返回所有用户的简略档案列表供前端渲染数据表格使用。

// 参数说明：
// - payload: 包含操作员信息的请求体
// - now_millis: 当前时间戳（毫秒）

// 返回值：
// - 成功：返回所有用户的列表
// - 失败：返回 AppError 错误
pub fn list_users_by_admin(
    payload: AdminListUsersPayload,
    now_millis: u64,
) -> Result<Vec<AdminManagedUserData>, AppError> {
    // 将时间戳转换为 i64 类型
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;
    // 获取操作员用户名
    let operator_username = payload.operator_username.trim().to_string();
    if operator_username.is_empty() {
        return Err(AppError::Validation(
            "operatorUsername is required".to_string(),
        ));
    }
    // 验证操作员是否为管理员
    assert_operator_is_admin(&operator_username, now_millis)?;
    // 获取用户列表
    let records = admin_repository::list_users()?;
    // 转换为响应格式并返回
    Ok(records.into_iter().map(map_managed_user_record).collect())
}

// ==========================================================================================
// 用户更新
// ==========================================================================================

// 管理员更新用户信息

// 功能说明：
// 提供管理员级别的用户信息变更。
// 将会对需要分配给该用户的新增角色执行差集运算并更新到路由关联表。

// 参数说明：
// - payload: 包含更新信息的请求体
// - now_millis: 当前时间戳（毫秒）

// 返回值：
// - 成功：返回更新后的用户信息
// - 失败：返回 AppError 错误
pub fn update_user_by_admin(
    payload: AdminUpdateUserPayload,
    now_millis: u64,
) -> Result<AdminManagedUserData, AppError> {
    // 将时间戳转换为 i64 类型
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;
    // 获取并校验操作员用户名
    let operator_username = payload.operator_username.trim().to_string();
    if operator_username.is_empty() {
        return Err(AppError::Validation(
            "operatorUsername is required".to_string(),
        ));
    }
    // 验证操作员是否为管理员
    assert_operator_is_admin(&operator_username, now_millis)?;

    // 校验用户 ID
    if payload.user_id <= 0 {
        return Err(AppError::Validation("userId is required".to_string()));
    }
    // 验证目标用户是否可编辑
    assert_target_user_editable(payload.user_id)?;

    // 获取并校验用户名
    let username = payload.username.trim().to_string();
    if username.is_empty() {
        return Err(AppError::Validation("username is required".to_string()));
    }
    // 获取并校验昵称
    let nickname = payload.nickname.trim().to_string();
    if nickname.is_empty() {
        return Err(AppError::Validation("nickname is required".to_string()));
    }

    // 校验手机号格式
    validate_phone(payload.phone.as_deref())?;
    // 规范化角色列表
    let roles = normalize_roles(payload.roles)?;
    // 计算账号有效期
    let (account_is_permanent, account_valid_days, account_expire_at) = build_account_term(
        payload.account_term_type.as_str(),
        payload.account_valid_days,
        now_millis,
    )?;

    // 调用数据访问层更新用户
    let record = admin_repository::update_user(admin_repository::UpdateUserInput {
        user_id: payload.user_id,
        username,
        nickname,
        phone: payload.phone,
        roles,
        is_active: payload.is_active,
        account_is_permanent,
        account_valid_days,
        account_expire_at,
        now_millis,
    })?;
    // 返回更新结果
    Ok(map_managed_user_record(record))
}

// ==========================================================================================
// 用户删除
// ==========================================================================================

// 管理员删除用户

// 功能说明：
// 物理删除某个用户，同时会自动级联清理角色表及路由表的绑定关系。

// 参数说明：
// - payload: 包含删除信息的请求体
// - now_millis: 当前时间戳（毫秒）

// 返回值：
// - 成功：返回 true
// - 失败：返回 AppError 错误
pub fn delete_user_by_admin(
    payload: AdminDeleteUserPayload,
    now_millis: u64,
) -> Result<bool, AppError> {
    // 将时间戳转换为 i64 类型
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;
    // 获取并校验操作员用户名
    let operator_username = payload.operator_username.trim().to_string();
    if operator_username.is_empty() {
        return Err(AppError::Validation(
            "operatorUsername is required".to_string(),
        ));
    }
    // 验证操作员是否为管理员
    assert_operator_is_admin(&operator_username, now_millis)?;
    // 校验用户 ID
    if payload.user_id <= 0 {
        return Err(AppError::Validation("userId is required".to_string()));
    }
    // 验证目标用户是否可删除
    assert_target_user_editable(payload.user_id)?;
    // 执行删除
    let deleted = admin_repository::delete_user(payload.user_id)?;
    if !deleted {
        return Err(AppError::Validation("user not found".to_string()));
    }
    // 返回删除成功
    Ok(true)
}

// ==========================================================================================
// 密码修改
// ==========================================================================================

// 管理员修改用户密码

// 功能说明：
// 重新生成安全的随机盐（Salt）并重设所选用户的密码。
// 该操作一旦成功，会导致被修改用户原有的登录令牌全部失效。

// 参数说明：
// - payload: 包含密码修改信息的请求体
// - now_millis: 当前时间戳（毫秒）

// 返回值：
// - 成功：返回被修改用户的 ID 和用户名
// - 失败：返回 AppError 错误
pub fn change_user_password_by_admin(
    payload: AdminChangeUserPasswordPayload,
    now_millis: u64,
) -> Result<AdminChangeUserPasswordData, AppError> {
    // 将时间戳转换为 i64 类型
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;
    // 获取并校验操作员用户名
    let operator_username = payload.operator_username.trim().to_string();
    if operator_username.is_empty() {
        return Err(AppError::Validation(
            "operatorUsername is required".to_string(),
        ));
    }
    // 验证操作员是否为管理员
    assert_operator_is_admin(&operator_username, now_millis)?;
    // 校验用户 ID
    if payload.user_id <= 0 {
        return Err(AppError::Validation("userId is required".to_string()));
    }
    // 获取并校验新密码
    let password = payload.password.trim().to_string();
    if password.is_empty() {
        return Err(AppError::Validation("password is required".to_string()));
    }

    // 调用数据访问层修改密码
    let record = admin_repository::update_user_password(payload.user_id, &password, now_millis)?;
    // 返回修改结果
    Ok(AdminChangeUserPasswordData {
        user_id: record.user_id,
        username: record.username,
    })
}

// ==========================================================================================
// 用户状态检查
// ==========================================================================================

// 确保用户账号可用

// 功能说明：
// 检查用户账号是否处于可用状态（未过期且未禁用）

// 参数说明：
// - username: 用户名
// - error_message: 错误时返回的消息
// - now_millis: 当前时间戳（毫秒）

// 返回值：
// - 成功：返回 ()
// - 失败：返回 AppError 错误
pub fn ensure_user_available_with_message(
    username: &str,
    error_message: &str,
    now_millis: u64,
) -> Result<(), AppError> {
    // 将时间戳转换为 i64 类型
    let now_millis =
        i64::try_from(now_millis).map_err(|_| AppError::Validation(error_message.to_string()))?;
    // 查询用户登录状态
    let status = admin_repository::find_user_login_state(username)?;
    // 如果用户不存在，返回错误
    let Some(status) = status else {
        return Err(AppError::Validation(error_message.to_string()));
    };

    // 检查用户是否激活
    if !status.is_active {
        return Err(AppError::Validation(error_message.to_string()));
    }

    // 检查账号是否过期
    let expired = !status.account_is_permanent
        && status
            .account_expire_at
            .is_some_and(|expire_at| expire_at <= now_millis);
    if expired {
        // 如果账号过期，自动停用用户
        admin_repository::deactivate_user_by_username(username, now_millis)?;
        return Err(AppError::Validation(error_message.to_string()));
    }

    // 用户可用
    Ok(())
}

// 运行启动时过期账号补偿

// 功能说明：
// 在应用启动时自动停用所有已过期的用户账号

// 参数说明：
// - now_millis: 当前时间戳（毫秒）

// 返回值：
// - 成功：返回停用的用户数量
// - 失败：返回 AppError 错误
pub fn run_startup_expiration_compensation(now_millis: u64) -> Result<usize, AppError> {
    // 将时间戳转换为 i64 类型
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;
    // 执行过期账号停用
    admin_repository::deactivate_expired_users(now_millis)
}

// 获取预留设备范围接口消息
pub fn reserved_device_scope_message() -> &'static str {
    RESERVED_DEVICE_SCOPE_MESSAGE
}

// ==========================================================================================
// 内部辅助函数
// ==========================================================================================

// 验证操作员是否为管理员

// 参数说明：
// - operator_username: 操作员用户名
// - now_millis: 当前时间戳（毫秒）

// 返回值：
// - 成功：返回 ()
// - 失败：返回 AppError 错误
fn assert_operator_is_admin(operator_username: &str, now_millis: i64) -> Result<(), AppError> {
    // 查询操作员是否为管理员
    if !admin_repository::is_admin_user(operator_username, now_millis)? {
        return Err(AppError::Validation("forbidden: admin only".to_string()));
    }
    Ok(())
}

// 规范化角色列表

// 功能说明：
// 将角色列表去重、排序并验证有效性

// 参数说明：
// - raw_roles: 原始角色列表

// 返回值：
// - 成功：返回规范化后的角色列表
// - 失败：返回 AppError 错误
fn normalize_roles(raw_roles: Vec<String>) -> Result<Vec<String>, AppError> {
    // 使用 HashSet 去重
    let mut normalized = HashSet::new();
    for role in raw_roles {
        // 去除空格并转为小写
        let role = role.trim().to_ascii_lowercase();
        if role.is_empty() {
            continue;
        }
        // 验证角色是否允许
        if !is_allowed_role(&role) {
            return Err(AppError::Validation(format!("invalid role: {role}")));
        }
        normalized.insert(role);
    }

    // 检查角色列表是否为空
    if normalized.is_empty() {
        return Err(AppError::Validation("roles is required".to_string()));
    }

    // 转换为向量并排序
    let mut roles: Vec<String> = normalized.into_iter().collect();
    roles.sort();
    Ok(roles)
}

// 检查角色是否允许
fn is_allowed_role(role: &str) -> bool {
    matches!(role, ROLE_OPERATOR | ROLE_TENANT | ROLE_MAINTAINER)
}

// 构建账号期限信息

// 参数说明：
// - account_term_type: 期限类型
// - account_valid_days: 有效天数
// - now_millis: 当前时间戳（毫秒）

// 返回值：
// - 成功：返回 (是否永久, 有效天数, 过期时间戳)
// - 失败：返回 AppError 错误
fn build_account_term(
    account_term_type: &str,
    account_valid_days: Option<i64>,
    now_millis: i64,
) -> Result<(bool, Option<i64>, Option<i64>), AppError> {
    let term = account_term_type.trim().to_ascii_lowercase();
    if term == TERM_PERMANENT {
        return Ok((true, None, None));
    }

    if term != TERM_DAYS {
        return Err(AppError::Validation(
            "accountTermType must be 'permanent' or 'days'".to_string(),
        ));
    }

    // 校验天数
    let days = account_valid_days
        .ok_or_else(|| AppError::Validation("accountValidDays is required".to_string()))?;
    if days <= 0 {
        return Err(AppError::Validation(
            "accountValidDays must be greater than 0".to_string(),
        ));
    }

    // 计算毫秒和过期时间
    let millis = days
        .checked_mul(24 * 60 * 60 * 1000)
        .ok_or_else(|| AppError::Validation("accountValidDays is too large".to_string()))?;
    let expire_at = now_millis
        .checked_add(millis)
        .ok_or_else(|| AppError::Validation("accountValidDays is too large".to_string()))?;
    Ok((false, Some(days), Some(expire_at)))
}

// 校验手机号格式
fn validate_phone(phone: Option<&str>) -> Result<(), AppError> {
    let Some(phone) = phone else {
        return Ok(());
    };
    let trimmed = phone.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    // 校验长度
    if trimmed.len() < 6 || trimmed.len() > 20 {
        return Err(AppError::Validation("invalid phone format".to_string()));
    }
    // 校验字符
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_digit() || c == '+' || c == '-' || c == ' ')
    {
        return Err(AppError::Validation("invalid phone format".to_string()));
    }
    Ok(())
}

// 验证目标用户是否可编辑
fn assert_target_user_editable(user_id: i64) -> Result<(), AppError> {
    // 根据用户 ID 查询用户名
    let username = admin_repository::find_username_by_user_id(user_id)?
        .ok_or_else(|| AppError::Validation("user not found".to_string()))?;
    // 检查是否为受保护的管理员用户
    if username.eq_ignore_ascii_case(PROTECTED_ADMIN_USERNAME) {
        return Err(AppError::Validation(
            "admin user only supports password change".to_string(),
        ));
    }
    Ok(())
}

// 将数据访问层的记录转换为 API 响应格式
fn map_managed_user_record(record: admin_repository::ManagedUserRecord) -> AdminManagedUserData {
    AdminManagedUserData {
        user_id: record.user_id,
        username: record.username,
        nickname: record.nickname,
        phone: record.phone,
        roles: record.roles,
        is_active: record.is_active,
        account_is_permanent: record.account_is_permanent,
        account_valid_days: record.account_valid_days,
        account_expire_at: record.account_expire_at,
        created_at: record.created_at,
        updated_at: record.updated_at,
        created_by: record.created_by,
    }
}
