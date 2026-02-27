//! # 业务逻辑模块
//!
//! 包含与 Tauri 框架 **完全解耦** 的纯业务函数，可独立进行单元测试。
//!
//! ## 函数一览
//!
//! | 函数 | 用途 |
//! |------|------|
//! | [`now_millis`] | 获取当前 Unix 毫秒时间戳 |
//! | [`mint_token_pair`] | 生成访问令牌与刷新令牌对 |
//! | [`resolve_user_profile`] | 根据用户名查找用户档案（当前为硬编码模拟） |
//! | [`build_login_data`] | 将用户档案与令牌合并为登录响应体 |
//! | [`build_async_routes`] | 构建前端动态路由配置 |

use serde_json::{json, Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::auth::models::{LoginData, TokenPair, UserProfile};

/// 获取当前时间的 Unix 毫秒时间戳。
///
/// 若系统时钟异常（早于 `UNIX_EPOCH`），返回 `0`；
/// 若毫秒值溢出 `u64`，返回 `u64::MAX`（实际不可能发生）。
#[must_use]
pub fn now_millis() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0));
    u64::try_from(now.as_millis()).unwrap_or(u64::MAX)
}

/// 根据主体标识生成令牌对。
///
/// 令牌格式为简单拼接（非加密签名），仅用于本地桌面应用的模拟鉴权。
/// 过期时间设置为当前时间 + 2 小时，使用 `saturating_add` 防止溢出。
///
/// # 参数
///
/// - `subject` — 令牌主体标识（通常为用户名或 `"refresh"`）
#[must_use]
pub fn mint_token_pair(subject: &str) -> TokenPair {
    let now = now_millis();
    // 2 小时 = 2 * 60 * 60 * 1000 毫秒
    let expires = now.saturating_add(2 * 60 * 60 * 1000);
    TokenPair {
        access_token: format!("tauri.{subject}.{now}"),
        refresh_token: format!("tauri.{subject}.refresh.{now}"),
        expires,
    }
}

/// 根据用户名解析用户档案。
///
/// 当前为硬编码模拟实现，后续可替换为数据库查询或远程 API 调用。
///
/// ## 内置用户
///
/// | 用户名 | 角色 | 权限 |
/// |--------|------|------|
/// | `admin` | `admin` | `*:*:*`（超级管理员，拥有全部权限） |
/// | 其他 | `common` | `permission:btn:add`, `permission:btn:edit` |
#[must_use]
pub fn resolve_user_profile(username: &str) -> UserProfile {
    if username == "admin" {
        UserProfile {
            avatar: "https://avatars.githubusercontent.com/u/44761321".to_string(),
            username: "admin".to_string(),
            nickname: "小铭".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec!["*:*:*".to_string()],
        }
    } else {
        UserProfile {
            avatar: "https://avatars.githubusercontent.com/u/52823142".to_string(),
            username: "common".to_string(),
            nickname: "小林".to_string(),
            roles: vec!["common".to_string()],
            permissions: vec![
                "permission:btn:add".to_string(),
                "permission:btn:edit".to_string(),
            ],
        }
    }
}

/// 将用户档案与新生成的令牌对合并，构建登录成功响应数据。
///
/// 此函数消费（move）传入的 `profile`，避免不必要的克隆开销。
#[must_use]
pub fn build_login_data(profile: UserProfile) -> LoginData {
    let token = mint_token_pair(&profile.username);
    LoginData {
        avatar: profile.avatar,
        username: profile.username,
        nickname: profile.nickname,
        roles: profile.roles,
        permissions: profile.permissions,
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        expires: token.expires,
    }
}

/// 构建前端异步（动态）路由配置。
///
/// 返回的 JSON 数组与前端 `vue-router` 的路由表结构一一对应。
/// 前端根据用户的 `roles` 和 `auths` 字段进行路由过滤与按钮权限控制。
///
/// ## 路由树结构
///
/// ```text
/// /permission                      — 权限管理（根节点）
/// ├── /permission/page/index       — 页面权限演示
/// └── /permission/button           — 按钮权限演示
///     ├── /permission/button/router — 路由返回按钮权限
///     └── /permission/button/login  — 登录接口返回按钮权限
/// ```
#[must_use]
pub fn build_async_routes() -> Vec<Value> {
    vec![json!({
      "path": "/permission",
      "meta": {
        "title": "权限管理",
        "icon": "ep:lollipop",
        "rank": 10
      },
      "children": [
        {
          "path": "/permission/page/index",
          "name": "PermissionPage",
          "meta": {
            "title": "页面权限",
            "roles": ["admin", "common"]
          }
        },
        {
          "path": "/permission/button",
          "meta": {
            "title": "按钮权限",
            "roles": ["admin", "common"]
          },
          "children": [
            {
              "path": "/permission/button/router",
              "component": "permission/button/index",
              "name": "PermissionButtonRouter",
              "meta": {
                "title": "路由返回按钮权限",
                "auths": ["permission:btn:add", "permission:btn:edit", "permission:btn:delete"]
              }
            },
            {
              "path": "/permission/button/login",
              "component": "permission/button/perms",
              "name": "PermissionButtonLogin",
              "meta": {
                "title": "登录接口返回按钮权限"
              }
            }
          ]
        }
      ]
    })]
}
