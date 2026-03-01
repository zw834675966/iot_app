//! ==========================================================================================
//! 管理员命令模块（适配器层）
//!
//! 模块职责：
//! 本模块负责接收前端发起的管理员特定 IPC 命令（Tauri Commands）。
//! 主要是针对用户生命周期的增删改查（CRUD）操作、密码重置及账号续期功能。
//! 该层作为适配器层，负责数据的解析、验证并转交业务逻辑（`admin_services`）处理。
//!
//! 功能清单：
//!
//! | 命令名 | 功能说明 |
//! |--------|----------|
//! | `auth_admin_register_user` | 管理员注册新用户 |
//! | `auth_admin_renew_user_account` | 管理员续期用户账号 |
//! | `auth_admin_list_users` | 管理员列出所有用户 |
//! | `auth_admin_update_user` | 管理员更新用户信息 |
//! | `auth_admin_delete_user` | 管理员删除用户 |
//! | `auth_admin_change_user_password` | 管理员重置用户密码 |
//! | `user_device_scope_get` | 获取用户设备范围（预留） |
//! | `user_device_scope_upsert` | 更新用户设备范围（预留） |
//!
//! 设计原则：
//! - 薄层适配：本模块仅做参数校验和结果封装，不包含业务逻辑
//! - 权限校验：验证操作者是否为管理员
//! - 安全防护：防止误删核心 admin 账号
//!
//! ==========================================================================================

// 引入管理员服务模块，用于处理具体的业务逻辑
use crate::auth::admin_services;

// 引入鉴权模块的所有模型定义，这些结构体用于前后端数据交互
use crate::auth::models::{
    AdminChangeUserPasswordData, AdminChangeUserPasswordPayload, AdminDeleteUserPayload,
    AdminListUsersPayload, AdminManagedUserData, AdminRegisterUserPayload, AdminRegisteredUserData,
    AdminRenewUserAccountData, AdminRenewUserAccountPayload, AdminUpdateUserPayload,
    UserDeviceScopeGetPayload, UserDeviceScopeReservedData, UserDeviceScopeSnapshot,
    UserDeviceScopeUpsertPayload,
};

// 引入时间工具函数，用于获取当前时间戳
use crate::auth::services::now_millis;

// 引入核心错误和响应类型，用于统一错误处理和响应格式
use crate::core::error::{ApiResponse, AppError, AppResult};
use crate::core::tracing::{TraceContext, execute_traced_command};

// ==========================================================================================
// 管理员命令实现
// ==========================================================================================

// 管理员注册新系统用户命令
//
// 功能说明：
// 管理员调用此命令在系统中创建新用户账号。
// 需要验证操作者的管理员权限，并确保必要的账户信息及租户有效期限完整。
//
// 参数说明：
// - operator_username: 操作的管理员用户名
// - username: 新用户的用户名
// - password: 新用户的密码
// - nickname: 新用户的昵称
// - phone: 手机号（可选）
// - roles: 角色列表
// - account_term_type: 账号期限类型（permanent/days）
// - account_valid_days: 有效天数（当 term_type 为 days 时必填）
//
// 返回值：
// 返回新创建用户的 ID、用户名、角色和账号状态
#[tauri::command]
pub fn auth_admin_register_user(
    payload: AdminRegisterUserPayload,
    trace: Option<TraceContext>,
) -> AppResult<AdminRegisteredUserData> {
    execute_traced_command("auth_admin_register_user", trace, || {
        let data = admin_services::register_user_by_admin(payload, now_millis())?;
        Ok(ApiResponse::ok(data))
    })
}

// 管理员延长用户账号有效期限命令
//
// 功能说明：
// 管理员可以按固定天数延长用户账号有效期，或将账号转为永久有效。
// 不允许对超级管理员（admin）进行此操作。
//
// 参数说明：
// - operator_username: 操作的管理员用户名
// - user_id: 目标用户 ID
// - renew_mode: 续期模式（permanent/days）
// - renew_days: 续期天数（当 renew_mode 为 days 时必填）
//
// 返回值：
// 返回更新后的账号状态信息
#[tauri::command]
pub fn auth_admin_renew_user_account(
    payload: AdminRenewUserAccountPayload,
    trace: Option<TraceContext>,
) -> AppResult<AdminRenewUserAccountData> {
    execute_traced_command("auth_admin_renew_user_account", trace, || {
        let data = admin_services::renew_user_account_by_admin(payload, now_millis())?;
        Ok(ApiResponse::ok(data))
    })
}

// 管理员列出系统中所有被管理用户命令
//
// 功能说明：
// 获取系统中所有用户的列表，用于用户管理页面展示。
// 需要验证操作者的管理员权限。
//
// 参数说明：
// - operator_username: 操作的管理员用户名
//
// 返回值：
// 返回所有用户的详细信息列表
#[tauri::command]
pub fn auth_admin_list_users(
    payload: AdminListUsersPayload,
    trace: Option<TraceContext>,
) -> AppResult<Vec<AdminManagedUserData>> {
    execute_traced_command("auth_admin_list_users", trace, || {
        let data = admin_services::list_users_by_admin(payload, now_millis())?;
        Ok(ApiResponse::ok(data))
    })
}

// 管理员更新指定用户信息命令
//
// 功能说明：
// 管理员可以更新用户的昵称、角色、手机号及状态信息。
// 必须遵循严格的校验规则，防止更改超级管理员的敏感字段。
//
// 参数说明：
// - operator_username: 操作的管理员用户名
// - user_id: 目标用户 ID
// - username: 用户名
// - nickname: 昵称
// - phone: 手机号（可选）
// - roles: 角色列表
// - is_active: 是否激活
// - account_term_type: 账号期限类型
// - account_valid_days: 有效天数
//
// 返回值：
// 返回更新后的用户信息
#[tauri::command]
pub fn auth_admin_update_user(
    payload: AdminUpdateUserPayload,
    trace: Option<TraceContext>,
) -> AppResult<AdminManagedUserData> {
    execute_traced_command("auth_admin_update_user", trace, || {
        let data = admin_services::update_user_by_admin(payload, now_millis())?;
        Ok(ApiResponse::ok(data))
    })
}

// 管理员删除指定用户命令
//
// 功能说明：
// 软删除或硬删除指定用户。
// 内置安全保护机制，防止核心 admin 账号被意外删除。
//
// 参数说明：
// - operator_username: 操作的管理员用户名
// - user_id: 目标用户 ID
//
// 返回值：
// 返回是否删除成功
#[tauri::command]
pub fn auth_admin_delete_user(
    payload: AdminDeleteUserPayload,
    trace: Option<TraceContext>,
) -> AppResult<bool> {
    execute_traced_command("auth_admin_delete_user", trace, || {
        let data = admin_services::delete_user_by_admin(payload, now_millis())?;
        Ok(ApiResponse::ok(data))
    })
}

// 管理员强制重置或修改任意用户密码命令
//
// 功能说明：
// 管理员可以重置任何普通用户的登录密码。
// 此操作会生成新的密码盐值并重新计算密码哈希。
//
// 参数说明：
// - operator_username: 操作的管理员用户名
// - user_id: 目标用户 ID
// - password: 新密码
//
// 返回值：
// 返回被修改的用户 ID 和用户名
#[tauri::command]
pub fn auth_admin_change_user_password(
    payload: AdminChangeUserPasswordPayload,
    trace: Option<TraceContext>,
) -> AppResult<AdminChangeUserPasswordData> {
    execute_traced_command("auth_admin_change_user_password", trace, || {
        let data = admin_services::change_user_password_by_admin(payload, now_millis())?;
        Ok(ApiResponse::ok(data))
    })
}

// ==========================================================================================
// 预留接口
// ==========================================================================================

// 预留接口：获取指定用户的设备与区域管理边界配置
//
// 功能说明：
// 这是一个预留接口，目前尚未实现。
// 未来将用于管理用户对设备、区域和楼层的访问权限。
//
// 参数说明：
// - user_id: 用户 ID
//
// 返回值：
// 返回预留状态信息，implemented 字段为 false
#[tauri::command]
pub fn user_device_scope_get(
    _payload: UserDeviceScopeGetPayload,
    trace: Option<TraceContext>,
) -> AppResult<UserDeviceScopeReservedData> {
    execute_traced_command("user_device_scope_get", trace, || {
        Ok(ApiResponse::ok(UserDeviceScopeReservedData {
            implemented: false,
            message: admin_services::reserved_device_scope_message().to_string(),
            scope: UserDeviceScopeSnapshot {
                all_areas: false,
                all_floors: false,
                all_devices: false,
                areas: vec![],
                floors: vec![],
                devices: vec![],
            },
        }))
    })
}

// 预留接口：更新指定用户的设备管理边界
//
// 功能说明：
// 这是一个预留接口，目前尚未实现。
// 未来将用于更新用户对设备、区域和楼层的访问权限。
//
// 参数说明：
// - user_id: 用户 ID
// - all_areas: 是否可访问所有区域
// - all_floors: 是否可访问所有楼层
// - all_devices: 是否可访问所有设备
// - areas: 可访问的区域列表
// - floors: 可访问的楼层列表
// - devices: 可访问的设备列表
//
// 返回值：
// 返回错误，表示接口尚未实现
#[tauri::command]
pub fn user_device_scope_upsert(
    _payload: UserDeviceScopeUpsertPayload,
    trace: Option<TraceContext>,
) -> AppResult<bool> {
    execute_traced_command("user_device_scope_upsert", trace, || {
        Err(AppError::Validation(
            admin_services::reserved_device_scope_message().to_string(),
        ))
    })
}

// ==========================================================================================
// 单元测试
// ==========================================================================================

// 管理员命令模块的单元测试
#[cfg(test)]
mod tests {
    // 引入标准库的 Once 类型，用于确保测试数据库只初始化一次
    use std::sync::Once;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    // 引入数据库模块
    use crate::db;

    // 引入父模块的所有项
    use super::*;

    // 确保测试数据库已准备就绪的辅助函数
    fn unique_username(prefix: &str) -> String {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        format!("{prefix}_{counter}_{nanos}")
    }

    fn ensure_test_db_ready() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            // 设置测试数据库 URL 并初始化数据库
            db::set_database_url(db::test_database_url()).expect("configure database url");
            db::init_database().expect("init database");
        });
    }

    // 测试：验证管理员可以注册具有多个角色的用户
    #[test]
    fn admin_can_register_user_with_multiple_roles() {
        // 准备测试数据库
        ensure_test_db_ready();
        // 创建注册请求负载
        let payload = AdminRegisterUserPayload {
            operator_username: "admin".to_string(),
            username: unique_username("tenant_multi_role"),
            password: "admin123".to_string(),
            nickname: "多角色用户".to_string(),
            phone: Some("13800138000".to_string()),
            roles: vec!["tenant".to_string(), "operator".to_string()],
            account_term_type: "days".to_string(),
            account_valid_days: Some(30),
        };

        // 执行注册
        let result = auth_admin_register_user(payload, None).expect("register user");
        // 断言用户名正确
        assert!(result.data.username.starts_with("tenant_multi_role_"));
        // 断言包含 tenant 角色
        assert!(result.data.roles.contains(&"tenant".to_string()));
        // 断言包含 operator 角色
        assert!(result.data.roles.contains(&"operator".to_string()));
        // 断言不是永久账号
        assert!(!result.data.account_is_permanent);
        // 断言有过期时间
        assert!(result.data.account_expire_at.is_some());
    }

    // 测试：验证非管理员不能注册用户
    #[test]
    fn non_admin_cannot_register_user() {
        // 准备测试数据库
        ensure_test_db_ready();
        // 创建普通用户的注册请求
        let payload = AdminRegisterUserPayload {
            operator_username: "common".to_string(),
            username: "tenant_forbidden".to_string(),
            password: "admin123".to_string(),
            nickname: "禁止注册".to_string(),
            phone: None,
            roles: vec!["tenant".to_string()],
            account_term_type: "permanent".to_string(),
            account_valid_days: None,
        };

        // 执行注册并期望返回错误
        let err = auth_admin_register_user(payload, None).expect_err("expect forbidden");
        // 断言错误消息
        assert_eq!(
            err,
            AppError::Validation("forbidden: admin only".to_string())
        );
    }

    // 测试：验证管理员可以续期用户账号
    #[test]
    fn admin_can_renew_user_account() {
        // 准备测试数据库
        ensure_test_db_ready();
        // 首先注册一个用户
        let register_payload = AdminRegisterUserPayload {
            operator_username: "admin".to_string(),
            username: unique_username("tenant_for_renew"),
            password: "admin123".to_string(),
            nickname: "续期用户".to_string(),
            phone: None,
            roles: vec!["tenant".to_string()],
            account_term_type: "days".to_string(),
            account_valid_days: Some(7),
        };
        let register_result =
            auth_admin_register_user(register_payload, None).expect("register user for renew");

        // 续期该用户
        let renew_payload = AdminRenewUserAccountPayload {
            operator_username: "admin".to_string(),
            user_id: register_result.data.user_id,
            renew_mode: "days".to_string(),
            renew_days: Some(90),
        };
        let renewed = auth_admin_renew_user_account(renew_payload, None).expect("renew account");
        // 断言用户 ID 匹配
        assert_eq!(renewed.data.user_id, register_result.data.user_id);
        // 断言不是永久账号
        assert!(!renewed.data.account_is_permanent);
        // 断言有过期时间
        assert!(renewed.data.account_expire_at.is_some());
        // 断言账号已激活
        assert!(renewed.data.is_active);
    }

    // 测试：验证管理员不能续期受保护的 admin 用户
    #[test]
    fn admin_cannot_renew_protected_admin_user() {
        // 准备测试数据库
        ensure_test_db_ready();
        // 尝试续期 admin 用户
        let payload = AdminRenewUserAccountPayload {
            operator_username: "admin".to_string(),
            user_id: 1,
            renew_mode: "days".to_string(),
            renew_days: Some(7),
        };
        let err =
            auth_admin_renew_user_account(payload, None).expect_err("expect protected user check");
        // 断言错误消息
        assert_eq!(
            err,
            AppError::Validation("admin user only supports password change".to_string())
        );
    }

    // 测试：验证管理员可以列出所有用户
    #[test]
    fn admin_can_list_users() {
        // 准备测试数据库
        ensure_test_db_ready();
        // 创建列出用户请求
        let payload = AdminListUsersPayload {
            operator_username: "admin".to_string(),
        };
        let result = auth_admin_list_users(payload, None).expect("list users");
        // 断言用户列表不为空
        assert!(!result.data.is_empty());
        // 断言包含 admin 用户
        assert!(result.data.iter().any(|item| item.username == "admin"));
    }

    // 测试：验证管理员不能更新受保护的 admin 用户资料
    #[test]
    fn admin_cannot_update_protected_admin_profile() {
        // 准备测试数据库
        ensure_test_db_ready();
        // 尝试更新 admin 用户
        let payload = AdminUpdateUserPayload {
            operator_username: "admin".to_string(),
            user_id: 1,
            username: "admin".to_string(),
            nickname: "admin".to_string(),
            phone: None,
            roles: vec!["operator".to_string()],
            is_active: true,
            account_term_type: "permanent".to_string(),
            account_valid_days: None,
        };
        let err = auth_admin_update_user(payload, None).expect_err("expect protected user check");
        // 断言错误消息
        assert_eq!(
            err,
            AppError::Validation("admin user only supports password change".to_string())
        );
    }

    // 测试：验证管理员可以更新和删除非 admin 用户
    #[test]
    fn admin_can_update_and_delete_non_admin_user() {
        // 准备测试数据库
        ensure_test_db_ready();
        // 注册新用户
        let register_payload = AdminRegisterUserPayload {
            operator_username: "admin".to_string(),
            username: unique_username("tenant_for_crud"),
            password: "admin123".to_string(),
            nickname: "crud target".to_string(),
            phone: Some("13800138001".to_string()),
            roles: vec!["tenant".to_string()],
            account_term_type: "days".to_string(),
            account_valid_days: Some(30),
        };
        let registered = auth_admin_register_user(register_payload, None).expect("register user");

        // 更新用户
        let update_payload = AdminUpdateUserPayload {
            operator_username: "admin".to_string(),
            user_id: registered.data.user_id,
            username: unique_username("tenant_for_crud_renamed"),
            nickname: "crud target renamed".to_string(),
            phone: Some("13800138002".to_string()),
            roles: vec!["maintainer".to_string(), "operator".to_string()],
            is_active: true,
            account_term_type: "permanent".to_string(),
            account_valid_days: None,
        };
        let updated = auth_admin_update_user(update_payload, None).expect("update user");
        // 断言用户 ID 匹配
        assert_eq!(updated.data.user_id, registered.data.user_id);
        // 断言用户名已更新
        assert!(
            updated
                .data
                .username
                .starts_with("tenant_for_crud_renamed_")
        );
        // 断言包含新角色
        assert!(updated.data.roles.contains(&"maintainer".to_string()));
        assert!(updated.data.roles.contains(&"operator".to_string()));
        // 断言是永久账号
        assert!(updated.data.account_is_permanent);

        // 删除用户
        let delete_payload = AdminDeleteUserPayload {
            operator_username: "admin".to_string(),
            user_id: updated.data.user_id,
        };
        let deleted = auth_admin_delete_user(delete_payload, None).expect("delete user");
        // 断言删除成功
        assert!(deleted.data);
    }

    // 测试：验证管理员可以修改受保护的 admin 用户密码
    #[test]
    fn admin_can_change_password_for_protected_admin_user() {
        // 准备测试数据库
        ensure_test_db_ready();
        // 修改 admin 密码
        let payload = AdminChangeUserPasswordPayload {
            operator_username: "admin".to_string(),
            user_id: 1,
            password: "admin123".to_string(),
        };
        let result = auth_admin_change_user_password(payload, None).expect("change password");
        // 断言用户名正确
        assert_eq!(result.data.username, "admin");
    }

    // 测试：验证预留的 upsert 接口返回未实现错误
    #[test]
    fn reserved_upsert_returns_not_implemented() {
        // 准备测试数据库
        ensure_test_db_ready();
        // 尝试调用预留接口
        let payload = UserDeviceScopeUpsertPayload {
            user_id: 1,
            all_areas: false,
            all_floors: false,
            all_devices: false,
            areas: vec![],
            floors: vec![],
            devices: vec![],
        };
        let err = user_device_scope_upsert(payload, None).expect_err("expect reserved message");
        // 断言错误消息
        assert_eq!(
            err,
            AppError::Validation("RESERVED_API_NOT_IMPLEMENTED".to_string())
        );
    }
}
