//! ==========================================================================================
//! # Tauri IPC 命令模块（Adapter Layer - 适配器层）
//!
//! ## 模块概述
//! 本模块是整个鉴权系统的**入口层**，定义了所有可供前端通过 `invoke()` 调用的命令处理器。
//! 每个命令函数扮演着"适配器"的角色：负责接收前端参数、进行基础校验，
//! 然后将业务逻辑委托给 [`services`](crate::auth::services) 层执行。
//!
//! ## 设计原则
//! - **薄层适配**：本模块仅做参数校验和结果封装，不包含任何业务逻辑
//! - **单一职责**：每个命令对应一个明确的业务场景
//! - **错误先行**：在调用业务逻辑前先进行参数校验，提前返回错误
//!
//! ## 命令清单
//!
//! | 命令名 | 前端调用方式 | 说明 | 返回类型 |
//! |--------|-------------|------|----------|
//! | `auth_login` | `invoke("auth_login", { username, password })` | 用户登录验证 | `LoginData` |
//! | `auth_refresh_token` | `invoke("auth_refresh_token", { refreshToken })` | 刷新访问令牌 | `RefreshTokenData` |
//! | `auth_get_async_routes` | `invoke("auth_get_async_routes")` | 获取动态路由配置 | `Vec<Value>` |
//!
//! ## 数据流示意
//!
//! ```text
//! 前端 (TypeScript/Vue)
//!      │
//!      │ invoke("auth_login", { username, password })
//!      ▼
//! ┌─────────────────────────────────────────┐
//! │  commands.rs (本模块)                    │
//! │  - 参数反序列化 (serde)                  │
//! │  - 参数基础校验 (非空、格式)              │
//! │  - 调用 services 层                     │
//! │  - 封装响应结果                          │
//! └─────────────────────────────────────────┘
//!      │
//!      │ services 函数调用
//!      ▼
//! ┌─────────────────────────────────────────┐
//! │  services.rs (Domain Layer - 领域层)     │
//! │  - 业务逻辑处理                          │
//! │  - JWT 令牌生成/验证                     │
//! │  - 数据库查询                            │
//! └─────────────────────────────────────────┘
//!      │
//!      │ repository 函数调用
//! ▼
//! ┌─────────────────────────────────────────┐
//! │  db/auth_repository.rs (数据访问层)      │
//! │  - SQL 查询执行                          │
//! │  - 数据映射转换                          │
//! └─────────────────────────────────────────┘
//!      │
//!      ▼
//! SQLite 数据库
//! ```
//!
//! ## 前端集成示例
//!
//! ### 1. 用户登录
//! ```typescript
//! // 前端调用方式
//! const loginResult = await invoke<{ success: boolean; data: LoginData }>("auth_login", {
//!   payload: { username: "admin", password: "admin123" }
//! });
//!
//! if (loginResult.success) {
//!   const { accessToken, refreshToken, roles, permissions } = loginResult.data;
//!   // 保存令牌到本地存储
//!   localStorage.setItem("accessToken", accessToken);
//!   localStorage.setItem("refreshToken", refreshToken);
//! }
//! ```
//!
//! ### 2. 令牌刷新
//! ```typescript
//! // 前端调用方式
//! const refreshResult = await invoke<{ success: boolean; data: RefreshTokenData }>(
//!   "auth_refresh_token",
//!   { payload: { refreshToken: currentRefreshToken } }
//! );
//! ```
//!
//! ### 3. 获取动态路由
//! ```typescript
//! // 前端调用方式
//! const routesResult = await invoke<{ success: boolean; data: RouteRecordRaw[] }>(
//!   "auth_get_async_routes"
//! );
//!
//! if (routesResult.success) {
//!   // 将后台返回的路由与前端静态路由合并
//!   router.addRoutes(routesResult.data);
//! }
//! ```
//!
//! ## 错误处理机制
//!
//! 本模块中所有命令的返回值类型为 `AppResult<T>`，成功时返回 `ApiResponse::ok(data)`，
//! 失败时返回 `AppError` 枚举的变体。前端可通过 `try-catch` 捕获错误并获取错误信息。
//!
//! 错误类型汇总：
//! - `AppError::Validation(String)`：参数校验失败（如空用户名、错误密码）
//! - `AppError::Database(String)`：数据库操作失败（通常不会直接暴露给前端）
//!
//! ## 扩展开发指南
//!
//! 如需添加新的鉴权相关命令（如登出、获取用户信息等），请遵循以下步骤：
//! 1. 在 [`models.rs`](crate::auth::models) 中定义请求/响应结构体
//! 2. 在 [`services.rs`](crate::auth::services) 中实现业务逻辑函数
//! 3. 在本文件中添加 `#[tauri::command]` 修饰的命令函数
//! 4. 在 `lib.rs` 的 `invoke_handler!` 中注册新命令
//! 5. 编写对应的单元测试
//!
//! ==========================================================================================

use serde_json::Value;

use crate::auth::admin_services;
use crate::auth::models::{LoginData, LoginPayload, RefreshTokenData, RefreshTokenPayload};
use crate::auth::services::{
    build_async_routes, build_login_data, mint_token_pair, resolve_user_profile,
    verify_refresh_token,
};
use crate::core::error::{ApiResponse, AppError, AppResult};

// ==========================================================================================
// 用户登录命令 (auth_login)
// ==========================================================================================

/// 用户登录命令处理器
///
/// ## 功能说明
/// 接收前端传入的用户名和密码，验证用户身份，并返回用户档案信息和 JWT 令牌对。
///
/// ## 执行流程
/// 1. **参数接收**：从 `LoginPayload` 中解构用户名和密码
/// 2. **参数校验**：
///    - 检查用户名是否为空（包含空白字符）
///    - 检查密码是否为空（包含空白字符）
/// 3. **业务处理**：调用 `resolve_user_profile` 查询数据库验证凭据
/// 4. **令牌生成**：调用 `build_login_data` 生成 JWT 令牌对并组装返回数据
/// 5. **响应封装**：将结果封装为 `ApiResponse::ok()` 返回
///
/// ## 参数校验规则
/// - `username`：不能为空字符串或仅包含空白字符，否则返回 `"username is required"`
/// - `password`：不能为空字符串或仅包含空白字符，否则返回 `"password is required"`
///
/// ## 返回数据 [`LoginData`]
/// 成功时返回包含以下字段的结构体：
/// - `avatar`: 用户头像 URL
/// - `username`: 用户名
/// - `nickname`: 用户昵称
/// - `roles`: 角色列表（如 `["admin"]`）
/// - `permissions`: 权限标识列表（如 `["*:*:*"]`）
/// - `access_token`: JWT 访问令牌（有效期 2 小时）
/// - `refresh_token`: JWT 刷新令牌（有效期 7 天）
/// - `expires`: 令牌过期时间（Unix 毫秒时间戳）
///
/// ## 错误处理
/// - **参数校验失败**：
///   - 用户名为空 → `AppError::Validation("username is required")`
///   - 密码为空 → `AppError::Validation("password is required")`
/// - **业务逻辑失败**：
///   - 用户名或密码错误 → `AppError::Validation("invalid username or password")`
///   - 数据库错误 → `AppError::Database(...)`
///
/// ## 前端调用示例
/// ```typescript
/// try {
///   const result = await invoke("auth_login", {
///     payload: { username: "admin", password: "admin123" }
///   });
///   if (result.success) {
///     console.log("登录成功", result.data);
///   }
/// } catch (error) {
///   console.error("登录失败", error);
/// }
/// ```
///
/// ## 测试覆盖
/// - `resolves_admin_user_profile`：验证管理员用户档案解析
/// - `resolves_common_user_profile`：验证普通用户档案解析
/// - `login_requires_username`：验证空用户名返回校验错误
/// - `login_rejects_unknown_user`：验证未知用户登录被拒绝
#[tauri::command]
pub fn auth_login(payload: LoginPayload) -> AppResult<LoginData> {
    let LoginPayload { username, password } = payload;

    if username.trim().is_empty() {
        return Err(AppError::Validation("username is required".to_string()));
    }
    if password.trim().is_empty() {
        return Err(AppError::Validation("password is required".to_string()));
    }

    let profile = resolve_user_profile(&username, &password)?;
    Ok(ApiResponse::ok(build_login_data(profile)))
}

// ==========================================================================================
// 令牌刷新命令 (auth_refresh_token)
// ==========================================================================================

/// 刷新访问令牌命令处理器
///
/// ## 功能说明
/// 当用户的访问令牌（access_token）即将过期时，前端使用刷新令牌（refresh_token）
/// 调用此接口获取新的令牌对，实现"无感刷新"功能。
///
/// ## 执行流程
/// 1. **参数接收**：从 `RefreshTokenPayload` 中解构刷新令牌
/// 2. **参数校验**：检查刷新令牌是否为空
/// 3. **令牌验证**：调用 `verify_refresh_token` 验证刷新令牌的有效性
///    - 校验 JWT 签名是否有效
///    - 校验令牌是否过期
///    - 校验令牌类型是否为 "refresh"（防止用 access_token 刷新）
/// 4. **生成新令牌**：从验证通过的用户主题（subject）生成新的令牌对
/// 5. **响应封装**：将新的令牌信息封装返回
///
/// ## 参数校验规则
/// - `refresh_token`：不能为空字符串或仅包含空白字符，否则返回 `"refreshToken is required"`
///
/// ## 返回数据 [`RefreshTokenData`]
/// 成功时返回包含以下字段的结构体：
/// - `access_token`: 新的 JWT 访问令牌（有效期 2 小时）
/// - `refresh_token`: 新的 JWT 刷新令牌（有效期 7 天）
/// - `expires`: 新令牌的过期时间（Unix 毫秒时间戳）
///
/// ## 安全设计
/// - **令牌类型校验**：仅接受 `token_type` 为 "refresh" 的令牌，防止 access_token 被滥用
/// - **主题验证**：从有效的刷新令牌中提取用户主题（username），确保签发对象正确
/// - **密钥一致**：使用与应用启动时相同的密钥进行验签，确保令牌兼容性
///
/// ## 错误处理
/// - **参数校验失败**：
///   - 刷新令牌为空 → `AppError::Validation("refreshToken is required")`
/// - **令牌验证失败**：
///   - 令牌格式非法 → `AppError::Validation("invalid refreshToken")`
///   - 令牌签名无效 → `AppError::Validation("invalid refreshToken")`
///   - 令牌已过期 → `AppError::Validation("invalid refreshToken")`
///   - 令牌类型错误（用 access_token 尝试刷新）→ `AppError::Validation("invalid refreshToken")`
///
/// ## 前端调用示例
/// ```typescript
/// // 在 access_token 即将过期时调用
/// const refreshToken = localStorage.getItem("refreshToken");
/// try {
///   const result = await invoke("auth_refresh_token", {
///     payload: { refreshToken }
///   });
///   if (result.success) {
///     // 更新本地存储的令牌
///     localStorage.setItem("accessToken", result.data.accessToken);
///     localStorage.setItem("refreshToken", result.data.refreshToken);
///   }
/// } catch (error) {
///   // 刷新失败，可能需要重新登录
///   console.error("令牌刷新失败", error);
/// }
/// ```
///
/// ## 测试覆盖
/// - `refresh_requires_token`：验证空刷新令牌返回校验错误
/// - `refresh_rejects_malformed_token`：验证非法格式令牌被拒绝
/// - `refresh_rejects_access_token`：验证 access_token 不能用于刷新
#[tauri::command]
pub fn auth_refresh_token(payload: RefreshTokenPayload) -> AppResult<RefreshTokenData> {
    let refresh_token = payload.refresh_token;
    if refresh_token.trim().is_empty() {
        return Err(AppError::Validation("refreshToken is required".to_string()));
    }
    let subject = verify_refresh_token(&refresh_token)?;
    admin_services::ensure_user_available_with_message(
        &subject,
        "invalid refreshToken",
        crate::auth::services::now_millis(),
    )?;
    let refreshed = mint_token_pair(&subject);
    Ok(ApiResponse::ok(RefreshTokenData {
        access_token: refreshed.access_token,
        refresh_token: refreshed.refresh_token,
        expires: refreshed.expires,
    }))
}

// ==========================================================================================
// 获取动态路由命令 (auth_get_async_routes)
// ==========================================================================================

/// 获取异步（动态）路由配置命令处理器
///
/// ## 功能说明
/// 前端在用户登录成功后调用此命令，从数据库获取基于角色的动态路由配置。
/// 返回的路由数据将与前端静态路由合并，共同构建完整的导航菜单和权限控制体系。
///
/// ## 执行流程
/// 1. **调用业务层**：直接调用 `build_async_routes` 函数
/// 2. **数据库查询**：通过 `auth_repository::find_async_routes` 查询路由数据
/// 3. **数据组装**：服务层将查询结果组装为 vue-router 兼容的 JSON 格式
/// 4. **响应封装**：将路由数组封装为 `ApiResponse::ok()` 返回
///
/// ## 返回数据格式
/// 返回 JSON 数组，每个元素代表一个路由配置节点，结构与 vue-router 兼容：
/// ```json
/// [
///   {
///     "path": "/permission",
///     "meta": { "title": "权限管理", "icon": "ri/information-line", "rank": 10 },
///     "children": [
///       {
///         "path": "/permission/page/index",
///         "name": "PermissionPage",
///         "meta": { "title": "用户注册管理" }
///       }
///     ]
///   }
/// ]
/// ```
///
/// ## 路由数据来源
/// - 路由基本信息：数据库 `routes` 表
/// - 角色访问控制：数据库 `route_roles` 表
/// - 操作权限控制：数据库 `route_auths` 表
/// - 菜单图标：数据库 `routes.meta_icon` 字段（离线环境使用本地 Iconify 图标）
///
/// ## 路由层级设计
/// 通过 `parent_id` 字段实现树形结构：
/// - `parent_id = NULL`：顶级路由（根菜单）
/// - `parent_id = X`：ID 为 X 的路由的子路由
///
/// ## 权限控制机制
/// - **菜单级别**：通过 `route_roles` 控制哪些角色可以看到该菜单
/// - **按钮级别**：通过 `route_auths` 控制页面内按钮的显示/隐藏
/// - 前端使用 `v-permission` 指令根据用户权限动态控制元素显示
///
/// ## 前端使用示例
/// ```typescript
/// // 登录成功后获取动态路由
/// const result = await invoke("auth_get_async_routes");
/// if (result.success) {
///   const asyncRoutes = result.data;
///   // 1. 过滤出需要动态添加的路由
///   const filteredRoutes = filterRoutesByRoles(asyncRoutes, userRoles);
///   // 2. 添加到 vue-router 实例
///   router.addRoutes(filteredRoutes);
///   // 3. 更新菜单
///   menuStore.setMenus(buildMenuTree(asyncRoutes));
/// }
/// ```
///
/// ## 测试覆盖
/// - `routes_include_permission_root`：验证动态路由包含权限管理根节点
/// - `token_expiration_is_in_future`：验证令牌过期时间在未来
///
/// ## 注意事项
/// - 当前实现不会返回错误（数据库查询失败时由 services 层处理）
/// - 返回的路由数据已经过权限过滤，仅包含用户有权访问的路由
/// - 路由组件路径需要在前端存在，否则会导致路由跳转失败
#[tauri::command]
pub fn auth_get_async_routes() -> AppResult<Vec<Value>> {
    Ok(ApiResponse::ok(build_async_routes()?))
}

// ---------------------------------------------------------------------------
// 单元测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use crate::auth::models::{LoginPayload, RefreshTokenPayload};
    use crate::auth::services::{
        build_async_routes, mint_token_pair, now_millis, resolve_user_profile,
    };
    use crate::core::error::AppError;
    use crate::db;

    use super::*;

    fn ensure_test_db_ready() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let test_db = std::env::temp_dir().join("pure-admin-thin-auth-tests.sqlite3");
            let _ = std::fs::remove_file(&test_db);
            db::set_database_path(test_db).expect("configure database path");
            db::init_database().expect("init database");
        });
    }

    /// 验证 admin 用户档案解析正确
    #[test]
    fn resolves_admin_user_profile() {
        ensure_test_db_ready();
        let profile = resolve_user_profile("admin", "admin123").expect("query user");
        assert_eq!(profile.username, "admin");
        assert_eq!(profile.roles, vec!["admin".to_string()]);
        assert_eq!(profile.permissions, vec!["*:*:*".to_string()]);
    }

    /// 验证普通用户档案解析正确
    #[test]
    fn resolves_common_user_profile() {
        ensure_test_db_ready();
        let profile = resolve_user_profile("common", "admin123").expect("query user");
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
        ensure_test_db_ready();
        let routes = build_async_routes().expect("query routes");
        let root_path = routes[0]
            .get("path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        assert_eq!(root_path, "/permission");
    }

    /// 验证用户名为空时返回校验错误
    #[test]
    fn login_requires_username() {
        ensure_test_db_ready();
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

    /// 验证未知用户登录会被拒绝
    #[test]
    fn login_rejects_unknown_user() {
        ensure_test_db_ready();
        let payload = LoginPayload {
            username: "ghost".to_string(),
            password: "admin123".to_string(),
        };
        let err = auth_login(payload).expect_err("expected auth error");
        assert_eq!(
            err,
            AppError::Validation("invalid username or password".to_string())
        );
    }

    /// 验证刷新令牌为空时返回校验错误
    #[test]
    fn refresh_requires_token() {
        ensure_test_db_ready();
        let payload = RefreshTokenPayload {
            refresh_token: String::new(),
        };
        let err = auth_refresh_token(payload).expect_err("expected validation error");
        assert_eq!(
            err,
            AppError::Validation("refreshToken is required".to_string())
        );
    }

    /// 验证刷新接口会拒绝格式非法的 refresh token
    #[test]
    fn refresh_rejects_malformed_token() {
        ensure_test_db_ready();
        let payload = RefreshTokenPayload {
            refresh_token: "not-a-valid-jwt".to_string(),
        };
        let err = auth_refresh_token(payload).expect_err("expected validation error");
        assert_eq!(
            err,
            AppError::Validation("invalid refreshToken".to_string())
        );
    }

    /// 验证刷新接口不会接受 access token 作为 refresh token
    #[test]
    fn refresh_rejects_access_token() {
        ensure_test_db_ready();
        let token_pair = mint_token_pair("admin");
        let payload = RefreshTokenPayload {
            refresh_token: token_pair.access_token,
        };
        let err = auth_refresh_token(payload).expect_err("expected validation error");
        assert_eq!(
            err,
            AppError::Validation("invalid refreshToken".to_string())
        );
    }
}
