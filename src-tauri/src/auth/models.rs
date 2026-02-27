//! # 数据模型（DTO）模块
//!
//! 定义在前后端之间传输的所有数据结构，包括：
//! - **内部模型**：[`UserProfile`]、[`TokenPair`] —— 业务逻辑内部使用
//! - **响应体**：[`LoginData`]、[`RefreshTokenData`] —— 序列化后返回给前端
//! - **请求体**：[`LoginPayload`]、[`RefreshTokenPayload`] —— 从前端 `invoke` 反序列化
//!
//! ## 序列化约定
//!
//! 所有面向前端的结构体均使用 `#[serde(rename_all = "camelCase")]`，
//! 确保 JSON 字段名与前端 JavaScript/TypeScript 的命名风格一致。

use serde::{Deserialize, Serialize};

/// 用户档案信息（内部模型）。
///
/// 由 [`services::resolve_user_profile`](crate::services::resolve_user_profile) 根据用户名构建，
/// 随后合并到 [`LoginData`] 中返回给前端。
#[derive(Debug, Clone, Serialize)]
pub struct UserProfile {
    /// 用户头像 URL
    pub avatar: String,
    /// 登录用户名
    pub username: String,
    /// 用户昵称（显示名）
    pub nickname: String,
    /// 角色列表（如 `["admin"]`），用于前端路由权限判断
    pub roles: Vec<String>,
    /// 细粒度权限标识列表（如 `["permission:btn:add"]`），用于按钮级鉴权
    pub permissions: Vec<String>,
}

/// 令牌对（内部模型）。
///
/// 包含访问令牌、刷新令牌及过期时间戳，
/// 由 [`services::mint_token_pair`](crate::services::mint_token_pair) 生成。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
    /// 访问令牌，格式为 `tauri.{subject}.{timestamp}`
    pub access_token: String,
    /// 刷新令牌，格式为 `tauri.{subject}.refresh.{timestamp}`
    pub refresh_token: String,
    /// 过期时间（Unix 毫秒时间戳）
    pub expires: u64,
}

/// 登录成功响应体。
///
/// 将用户档案与令牌对扁平合并为单一结构，前端登录成功后一次性获取全部所需数据。
/// JSON 字段名使用 camelCase 以匹配前端命名风格。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginData {
    /// 用户头像 URL
    pub avatar: String,
    /// 登录用户名
    pub username: String,
    /// 用户昵称
    pub nickname: String,
    /// 角色列表
    pub roles: Vec<String>,
    /// 权限标识列表
    pub permissions: Vec<String>,
    /// 访问令牌
    pub access_token: String,
    /// 刷新令牌
    pub refresh_token: String,
    /// 令牌过期时间（Unix 毫秒时间戳）
    pub expires: u64,
}

/// 令牌刷新响应体。
///
/// 仅包含新的令牌对信息，不重复返回用户档案数据。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenData {
    /// 新的访问令牌
    pub access_token: String,
    /// 新的刷新令牌
    pub refresh_token: String,
    /// 新令牌过期时间（Unix 毫秒时间戳）
    pub expires: u64,
}

/// 登录请求体。
///
/// 前端通过 `invoke("auth_login", { username, password })` 传入。
/// 使用 `#[serde(default)]` 允许缺省字段，由命令层做非空校验。
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct LoginPayload {
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
}

/// 令牌刷新请求体。
///
/// 前端通过 `invoke("auth_refresh_token", { refreshToken })` 传入。
/// 注意：前端字段名为 camelCase（`refreshToken`），通过 `rename_all` 自动映射。
#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct RefreshTokenPayload {
    /// 当前持有的刷新令牌
    pub refresh_token: String,
}
