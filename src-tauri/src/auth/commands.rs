//! # Tauri IPC 命令模块
//!
//! 本模块定义所有前端可通过 `invoke()` 调用的命令处理器。
//! 每个命令函数是一个薄层适配器：负责参数校验，然后委托给 [`services`](crate::auth::services) 执行业务逻辑。
//!
//! ## 命令清单
//!
//! | 命令名 | 前端调用方式 | 说明 |
//! |--------|-------------|------|
//! | `auth_login` | `invoke("auth_login", { username, password })` | 用户登录 |
//! | `auth_refresh_token` | `invoke("auth_refresh_token", { refreshToken })` | 刷新令牌 |
//! | `auth_get_async_routes` | `invoke("auth_get_async_routes")` | 获取动态路由配置 |

use serde_json::Value;

use crate::auth::models::{LoginData, LoginPayload, RefreshTokenData, RefreshTokenPayload};
use crate::auth::services::{
    build_async_routes, build_login_data, mint_token_pair, resolve_user_profile,
};
use crate::core::error::{ApiResponse, AppError, AppResult};

/// 用户登录命令。
///
/// ## 参数校验
/// - `username` 不能为空
/// - `password` 不能为空
///
/// ## 返回
/// 成功时返回 [`LoginData`]，包含用户档案信息和令牌对。
///
/// # Errors
/// - 若 `username` 为空字符串或仅包含空白字符，返回 `AppError::Validation("username is required")`
/// - 若 `password` 为空字符串或仅包含空白字符，返回 `AppError::Validation("password is required")`
#[tauri::command]
pub fn auth_login(payload: LoginPayload) -> AppResult<LoginData> {
    let LoginPayload { username, password } = payload;

    if username.trim().is_empty() {
        return Err(AppError::Validation("username is required".to_string()));
    }
    if password.trim().is_empty() {
        return Err(AppError::Validation("password is required".to_string()));
    }

    let profile = resolve_user_profile(&username);
    Ok(ApiResponse::ok(build_login_data(profile)))
}

/// 刷新令牌命令。
///
/// 前端在访问令牌即将过期时调用此命令获取新的令牌对。
///
/// ## 参数校验
/// - `refresh_token` 不能为空
///
/// ## 返回
/// 成功时返回 [`RefreshTokenData`]，包含新的令牌对和过期时间。
///
/// # Errors
/// - 若 `refresh_token` 为空字符串或仅包含空白字符，返回 `AppError::Validation("refreshToken is required")`
#[tauri::command]
pub fn auth_refresh_token(payload: RefreshTokenPayload) -> AppResult<RefreshTokenData> {
    let refresh_token = payload.refresh_token;
    if refresh_token.trim().is_empty() {
        return Err(AppError::Validation("refreshToken is required".to_string()));
    }
    let refreshed = mint_token_pair("refresh");
    Ok(ApiResponse::ok(RefreshTokenData {
        access_token: refreshed.access_token,
        refresh_token: refreshed.refresh_token,
        expires: refreshed.expires,
    }))
}

/// 获取异步（动态）路由配置命令。
///
/// 前端在登录成功后调用此命令获取基于角色的动态路由表，
/// 与前端静态路由合并后构建完整的导航菜单。
///
/// ## 返回
/// 返回 JSON 数组格式的路由配置，结构与 `vue-router` 路由表兼容。
///
/// # Errors
/// 当前实现永远不会失败。使用 `Result` 仅为保持统一的 API 响应格式。
#[tauri::command]
#[allow(clippy::unnecessary_wraps)]
pub fn auth_get_async_routes() -> AppResult<Vec<Value>> {
    Ok(ApiResponse::ok(build_async_routes()))
}

// ---------------------------------------------------------------------------
// 单元测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::auth::models::{LoginPayload, RefreshTokenPayload};
    use crate::auth::services::{
        build_async_routes, mint_token_pair, now_millis, resolve_user_profile,
    };
    use crate::core::error::AppError;

    use super::*;

    /// 验证 admin 用户档案解析正确
    #[test]
    fn resolves_admin_user_profile() {
        let profile = resolve_user_profile("admin");
        assert_eq!(profile.username, "admin");
        assert_eq!(profile.roles, vec!["admin".to_string()]);
        assert_eq!(profile.permissions, vec!["*:*:*".to_string()]);
    }

    /// 验证普通用户档案解析正确
    #[test]
    fn resolves_common_user_profile() {
        let profile = resolve_user_profile("common");
        assert_eq!(profile.username, "common");
        assert_eq!(profile.roles, vec!["common".to_string()]);
        assert_eq!(
            profile.permissions,
            vec![
                "permission:btn:add".to_string(),
                "permission:btn:edit".to_string()
            ]
        );
    }

    /// 验证令牌过期时间在未来
    #[test]
    fn token_expiration_is_in_future() {
        let token = mint_token_pair("admin");
        assert!(token.expires > now_millis());
    }

    /// 验证动态路由包含权限管理根节点
    #[test]
    fn routes_include_permission_root() {
        let routes = build_async_routes();
        let root_path = routes[0]
            .get("path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        assert_eq!(root_path, "/permission");
    }

    /// 验证用户名为空时返回校验错误
    #[test]
    fn login_requires_username() {
        let payload = LoginPayload {
            username: String::new(),
            password: "admin123".to_string(),
        };
        let err = auth_login(payload).expect_err("expected validation error");
        assert_eq!(
            err,
            AppError::Validation("username is required".to_string())
        );
    }

    /// 验证刷新令牌为空时返回校验错误
    #[test]
    fn refresh_requires_token() {
        let payload = RefreshTokenPayload {
            refresh_token: String::new(),
        };
        let err = auth_refresh_token(payload).expect_err("expected validation error");
        assert_eq!(
            err,
            AppError::Validation("refreshToken is required".to_string())
        );
    }
}
