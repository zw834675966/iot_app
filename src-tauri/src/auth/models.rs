//! ==========================================================================================
//! # 数据模型（DTO）模块
//!
//! ## 模块概述
//! 本模块定义了前后端之间传输的所有数据结构（Data Transfer Object，DTO）。
//! 采用"贫血模型"设计：结构体仅包含数据字段，不包含业务逻辑。
//!
//! ## 模型分类
//!
//! | 分类 | 结构体 | 用途 | 方向 |
//! |------|--------|------|------|
//! | 内部模型 | `UserProfile` | 业务逻辑内部使用 | 数据库 → services |
//! | 内部模型 | `TokenPair` | 业务逻辑内部使用 | services → commands |
//! | 响应体 | `LoginData` | 登录成功返回 | commands → 前端 |
//! | 响应体 | `RefreshTokenData` | 令牌刷新返回 | commands → 前端 |
//! | 请求体 | `LoginPayload` | 登录请求接收 | 前端 → commands |
//! | 请求体 | `RefreshTokenPayload` | 令牌刷新请求接收 | 前端 → commands |
//!
//! ## 序列化约定
//!
//! ### 命名风格转换
//! Rust 使用 `snake_case`（蛇形命名），而 JavaScript/TypeScript 使用 `camelCase`（驼峰命名）。
//! 为确保前后端数据交互顺畅，所有面向前端的结构体均使用以下 serde 属性：
//!
//! ```rust
//! #[serde(rename_all = "camelCase")]
//! ```
//!
//! 这会自动将 Rust 字段名转换为前端期望的格式：
//!
//! | Rust 字段名 | JSON 字段名 (前端) |
//! |-------------|-------------------|
//! | `access_token` | `accessToken` |
//! | `refresh_token` | `refreshToken` |
//! | `user_id` | `userId` |
//! | `is_active` | `isActive` |
//!
//! ### 请求体默认值
//! 请求体结构使用 `#[serde(default)]` 和 `#[derive(Default)]`，
//! 允许前端省略可选字段，缺失字段将被初始化为默认值（空字符串）。
//! 命令层（commands.rs）负责进行必填字段的非空校验。
//!
//! ## 数据流示意
//!
//! ```text
//! 前端 (JSON)
//!    │
//!    │ invoke("auth_login", { username: "admin", password: "123456" })
//!    ▼
//! LoginPayload (反序列化)
//!    │
//!    │ 验证通过后提取字段
//!    ▼
//! services::resolve_user_profile() ──► 数据库查询
//!    │
//!    ▼
//! UserProfile (内部模型)
//!    │
//!    │ mint_token_pair() 生成令牌
//!    ▼
//! TokenPair + UserProfile ──► build_login_data()
//!    │
//!    ▼
//! LoginData (序列化)
//!    │
//!    │ { success: true, data: { ... } }
//!    ▼
//! 前端接收
//! ```
//!
//! ==========================================================================================

use serde::{Deserialize, Serialize};

// ==========================================================================================
// 内部模型（业务逻辑层使用）
// ==========================================================================================

/// 用户档案信息（内部模型）
///
/// 表示从数据库查询到的用户完整信息，包含身份信息和权限信息。
/// 此模型在业务逻辑层（services.rs）内部使用，不直接暴露给前端。
///
/// ## 数据来源
/// 由 [`services::resolve_user_profile`](crate::auth::services::resolve_user_profile) 从数据库构建：
/// - 用户基本信息：从 `users` 表查询
/// - 角色列表：从 `user_roles` 表关联查询
/// - 权限列表：从 `user_permissions` 表关联查询
///
/// ## 后续使用
/// 此模型会被合并到 [`LoginData`] 中返回给前端，前端根据 roles 和 permissions
/// 进行路由访问控制和按钮级权限控制。
///
/// ## 示例数据
/// ```json
/// {
///   "avatar": "",
///   "username": "admin",
///   "nickname": "小铭",
///   "roles": ["admin"],
///   "permissions": ["*:*:*", "permission:btn:add", "permission:btn:edit", "permission:btn:delete"]
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct UserProfile {
    /// 用户头像 URL
    /// - 空字符串表示使用本地默认头像
    /// - 外链头像会被清理脚本移除（见 0003_legacy_offline_cleanup.sql）
    pub avatar: String,

    /// 登录用户名
    /// 用于唯一标识用户身份，也是 JWT 令牌的主题（subject）
    pub username: String,

    /// 用户昵称（显示名）
    /// 在前端界面中展示给用户
    pub nickname: String,

    /// 角色列表
    /// 用于前端路由权限判断，如 `["admin", "common"]`
    /// - `admin` 角色可访问所有路由
    /// - `common` 角色仅可访问分配了对应角色的路由
    pub roles: Vec<String>,

    /// 细粒度权限标识列表
    /// 用于按钮级鉴权，如 `["permission:btn:add", "permission:btn:edit"]`
    /// 前端通过 `v-permission` 指令根据此列表控制按钮显示/隐藏
    pub permissions: Vec<String>,
}

// ==========================================================================================
// 令牌模型（内部模型）
// ==========================================================================================

/// 令牌对（内部模型）
///
/// 包含 JWT 访问令牌和刷新令牌，以及令牌的过期时间。
/// 此模型在业务逻辑层（services.rs）内部使用，不直接暴露给前端。
///
/// ## 生成方式
/// 由 [`services::mint_token_pair`](crate::auth::services::mint_token_pair) 函数生成。
///
/// ## 令牌格式
/// JWT 令牌由三部分组成：header.payload.signature
/// - Header: {"alg": "HS256", "typ": "JWT"}
/// - Payload: 包含 sub, token_type, iat, exp
/// - Signature: 使用 HS256 算法和密钥签名
///
/// ## 有效期
/// - `access_token`: 2 小时
/// - `refresh_token`: 7 天
/// - `expires`: 访问令牌的过期时间（Unix 毫秒时间戳）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
    /// 访问令牌
    /// 用于日常 API 请求的身份认证
    /// 格式示例：`eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhZG1pbiIsInRva2VuX3R5cGUiOiJhY2Nlc3MiLCJpYXQiOjE3MDQwNjc1NjAsImV4cCI6MTcwNDA2OTE2MH0.signature`
    pub access_token: String,

    /// 刷新令牌
    /// 用于在 access_token 过期时获取新的令牌对
    /// 格式示例：`eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhZG1pbiIsInRva2VuX3R5cGUiOiJyZWZyZXNoIiwiaWF0IjoxNzA0MDY3NTYwLCJleHAiOjE3MDQxNTM5NjB9.signature`
    pub refresh_token: String,

    /// 令牌过期时间
    /// Unix 毫秒时间戳，前端可用于判断令牌是否即将过期
    /// 建议在过期前 5-10 分钟开始刷新令牌
    pub expires: u64,
}

// ==========================================================================================
// 响应体模型（向前端返回）
// ==========================================================================================

/// 登录成功响应体
///
/// 用户登录验证成功后，返回给前端的完整数据。
/// 将用户档案与令牌对扁平合并为单一结构，前端登录成功后一次性获取全部所需数据。
///
/// ## 响应格式
/// ```json
/// {
///   "success": true,
///   "data": {
///     "avatar": "",
///     "username": "admin",
///     "nickname": "小铭",
///     "roles": ["admin"],
///     "permissions": ["*:*:*"],
///     "accessToken": "eyJ...",
///     "refreshToken": "eyJ...",
///     "expires": 1704069600000
///   }
/// }
/// ```
///
/// ## 前端使用方式
/// 1. 解析 `data` 字段获取用户信息
/// 2. 保存 `accessToken` 到本地存储（用于后续请求认证）
/// 3. 保存 `refreshToken` 到本地存储（用于令牌刷新）
/// 4. 根据 `roles` 动态添加路由
/// 5. 根据 `permissions` 控制按钮显示
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginData {
    /// 用户头像 URL
    /// 空字符串时前端使用本地默认头像
    pub avatar: String,

    /// 登录用户名
    pub username: String,

    /// 用户昵称
    /// 在导航栏等位置展示
    pub nickname: String,

    /// 角色列表
    /// 用于路由权限控制
    pub roles: Vec<String>,

    /// 权限标识列表
    /// 用于按钮级权限控制
    pub permissions: Vec<String>,

    /// 访问令牌
    /// 用于 API 请求认证，请求头格式：`Authorization: Bearer <accessToken>`
    pub access_token: String,

    /// 刷新令牌
    /// 用于令牌刷新接口
    pub refresh_token: String,

    /// 令牌过期时间
    /// Unix 毫秒时间戳
    pub expires: u64,
}

/// 令牌刷新响应体
///
/// 仅包含新的令牌对信息，不重复返回用户档案数据。
/// 因为用户已经登录，无需重复返回用户信息。
///
/// ## 响应格式
/// ```json
/// {
///   "success": true,
///   "data": {
///     "accessToken": "eyJ...",
///     "refreshToken": "eyJ...",
///     "expires": 1704073200000
///   }
/// }
/// ```
///
/// ## 前端使用方式
/// 1. 用新的令牌更新本地存储
/// 2. 继续使用新的 accessToken 进行 API 请求
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenData {
    /// 新的访问令牌
    pub access_token: String,

    /// 新的刷新令牌
    /// 每次刷新都会生成新的 refreshToken，旧的自动失效
    pub refresh_token: String,

    /// 新令牌过期时间
    /// Unix 毫秒时间戳
    pub expires: u64,
}

// ==========================================================================================
// 请求体模型（从前端接收）
// ==========================================================================================

/// 登录请求体
///
/// 前端通过 `invoke("auth_login", { payload: { username, password } })` 传入。
///
/// ## 序列化说明
/// - 使用 `#[serde(default)]` 允许缺省字段
/// - 缺失字段会被初始化为空字符串
/// - 命令层负责进行非空校验
///
/// ## 前端调用示例
/// ```typescript
/// const result = await invoke("auth_login", {
///   payload: { username: "admin", password: "admin123" }
/// });
/// ```
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct LoginPayload {
    /// 用户名
    /// 必填字段，命令层会校验不能为空
    pub username: String,

    /// 密码
    /// 必填字段，命令层会校验不能为空
    /// 注意：传输的是明文密码，后端会与数据库存储的哈希值比对
    pub password: String,
}

/// 令牌刷新请求体
///
/// 前端通过 `invoke("auth_refresh_token", { payload: { refreshToken } })` 传入。
///
/// ## 序列化说明
/// - 使用 `#[serde(rename_all = "camelCase")]` 自动将前端的 `refreshToken`
///   转换为 Rust 的 `refresh_token`
///
/// ## 前端调用示例
/// ```typescript
/// const refreshToken = localStorage.getItem("refreshToken");
/// const result = await invoke("auth_refresh_token", {
///   payload: { refreshToken }
/// });
/// ```
#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct RefreshTokenPayload {
    /// 当前持有的刷新令牌
    /// 必填字段，命令层会校验不能为空
    /// 必须是有效的 refresh 类型的 JWT 令牌
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct AdminRegisterUserPayload {
    pub operator_username: String,
    pub username: String,
    pub password: String,
    pub nickname: String,
    pub phone: Option<String>,
    pub roles: Vec<String>,
    pub account_term_type: String,
    pub account_valid_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminRegisteredUserData {
    pub user_id: i64,
    pub username: String,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub account_is_permanent: bool,
    pub account_expire_at: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct AdminRenewUserAccountPayload {
    pub operator_username: String,
    pub user_id: i64,
    pub renew_mode: String,
    pub renew_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminRenewUserAccountData {
    pub user_id: i64,
    pub account_is_permanent: bool,
    pub account_expire_at: Option<i64>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct AdminListUsersPayload {
    pub operator_username: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminManagedUserData {
    pub user_id: i64,
    pub username: String,
    pub nickname: String,
    pub phone: Option<String>,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub account_is_permanent: bool,
    pub account_valid_days: Option<i64>,
    pub account_expire_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub created_by: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct AdminUpdateUserPayload {
    pub operator_username: String,
    pub user_id: i64,
    pub username: String,
    pub nickname: String,
    pub phone: Option<String>,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub account_term_type: String,
    pub account_valid_days: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct AdminDeleteUserPayload {
    pub operator_username: String,
    pub user_id: i64,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct AdminChangeUserPasswordPayload {
    pub operator_username: String,
    pub user_id: i64,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminChangeUserPasswordData {
    pub user_id: i64,
    pub username: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserDeviceScopeGetPayload {
    pub user_id: i64,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserDeviceScopeUpsertPayload {
    pub user_id: i64,
    pub all_areas: bool,
    pub all_floors: bool,
    pub all_devices: bool,
    pub areas: Vec<String>,
    pub floors: Vec<String>,
    pub devices: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDeviceScopeSnapshot {
    pub all_areas: bool,
    pub all_floors: bool,
    pub all_devices: bool,
    pub areas: Vec<String>,
    pub floors: Vec<String>,
    pub devices: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDeviceScopeReservedData {
    pub implemented: bool,
    pub message: String,
    pub scope: UserDeviceScopeSnapshot,
}
