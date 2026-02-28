//! ==========================================================================================
//! 鉴权业务逻辑层（领域层）
//!
//! 模块职责：
//! 本模块是整个鉴权系统的核心业务层，负责处理与用户认证、令牌管理和权限相关的所有业务逻辑。
//! 该层完全独立于 Tauri 框架，不包含任何 `#[tauri::command]` 宏或其他框架特定的代码，
//! 因此可以直接在命令行环境或其他框架中复用和测试。
//!
//! 核心功能：
//! - JWT 令牌生成与验证
//! - 用户档案解析
//! - 动态路由组装
//! - 密码验证（委托给数据访问层）
//!
//! 设计原则：
//! - 纯函数：所有业务函数不包含副作用，结果只依赖于输入参数
//! - 无框架依赖：不引入任何 Tauri 特定代码，便于单元测试
//! - 错误显式：所有可能失败的操作返回 `Result<T, AppError>` 类型
//!
//! 关键概念：
//!
//! JWT 令牌类型：
//! | 令牌类型 | 有效期 | 用途 |
//! |---------|--------|------|
//! | access_token | 2 小时 | API 请求认证 |
//! | refresh_token | 7 天 | 令牌刷新 |
//!
//! JWT 载荷结构：
//! ```json
//! {
//!   "sub": "admin",
//!   "token_type": "access",
//!   "iat": 1704067200,
//!   "exp": 1704074400
//! }
//! ```
//!
//! 安全特性：
//! - HS256 算法签名
//! - 区分 access/refresh 令牌类型
//! - 密钥从环境变量读取
//! - 验证过期时间
//!
//! ==========================================================================================

// 引入 jsonwebtoken 库，用于 JWT 令牌的生成和验证
// Algorithm: JWT 签名算法
// DecodingKey: JWT 解码密钥
// EncodingKey: JWT 编码密钥
// Header: JWT 头部
// Validation: JWT 验证配置
// decode: JWT 解码函数
// encode: JWT 编码函数
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};

// 引入 serde 库的序列化和反序列化 trait
use serde::{Deserialize, Serialize};

// 引入 serde_json 库中的 Value 类型，用于处理动态路由的 JSON 数据
use serde_json::Value;

// 引入标准库的 OnceLock，用于延迟初始化只读数据
use std::sync::OnceLock;

// 引入时间相关的类型
// Duration: 时间段
// SystemTime: 系统时间
// UNIX_EPOCH: Unix 纪元（1970年1月1日）
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// 引入管理员服务模块，用于验证用户账号状态
use crate::auth::admin_services;
// 引入鉴权模块的数据模型
use crate::auth::models::{LoginData, TokenPair, UserProfile};
// 引入核心错误处理模块
use crate::core::error::AppError;
// 引入鉴权数据访问层
use crate::db::auth_repository;

// ==========================================================================================
// JWT 配置常量
// ==========================================================================================

// 访问令牌的有效期（秒）
// 2 * 60 * 60 = 7200 秒 = 2 小时
const ACCESS_TOKEN_LIFETIME_SECONDS: u64 = 2 * 60 * 60;

// 刷新令牌的有效期（秒）
// 7 * 24 * 60 * 60 = 604800 秒 = 7 天
const REFRESH_TOKEN_LIFETIME_SECONDS: u64 = 7 * 24 * 60 * 60;

// 访问令牌的类型标识
const ACCESS_TOKEN_TYPE: &str = "access";

// 刷新令牌的类型标识
const REFRESH_TOKEN_TYPE: &str = "refresh";

// 默认的 JWT 密钥
// 注意：生产环境应通过环境变量 PURE_ADMIN_JWT_SECRET 设置
const DEFAULT_JWT_SECRET: &str = "pure-admin-thin-dev-secret-change-me";

// ==========================================================================================
// JWT 声明结构体
// ==========================================================================================

// JWT 声明结构体
// 包含令牌的主题、类型、签发时间和过期时间
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtClaims {
    // sub: 主题，通常为用户名
    sub: String,
    // token_type: 令牌类型，"access" 或 "refresh"
    token_type: String,
    // iat: 签发时间（Unix 时间戳，秒）
    iat: u64,
    // exp: 过期时间（Unix 时间戳，秒）
    exp: u64,
}

// ==========================================================================================
// 时间工具函数
// ==========================================================================================

// 获取当前时间的毫秒数
//
// 返回值：
// 返回自 Unix 纪元以来的毫秒数
//
// 注意：
// 如果系统时间在 Unix 纪元之前，返回 0
// 如果毫秒数超出 u64 范围，返回 u64::MAX
#[must_use]
pub fn now_millis() -> u64 {
    // 获取当前系统时间并计算与 Unix 纪元的时间差
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0));
    // 将毫秒数转换为 u64 类型
    u64::try_from(now.as_millis()).unwrap_or(u64::MAX)
}

// 获取当前时间的秒数
//
// 返回值：
// 返回自 Unix 纪元以来的秒数
//
// 注意：
// 内部调用 now_millis() 并除以 1000
#[must_use]
fn now_secs() -> u64 {
    // 将毫秒转换为秒
    now_millis() / 1000
}

// ==========================================================================================
// JWT 密钥管理
// ==========================================================================================

// 获取 JWT 密钥
//
// 获取优先级：
// 1. 环境变量 PURE_ADMIN_JWT_SECRET
// 2. 默认密钥（仅用于开发）
//
// 返回值：
// 返回 JWT 密钥的字符串引用
//
// 线程安全：
// 使用 OnceLock 确保密钥只初始化一次，线程安全
#[must_use]
fn jwt_secret() -> &'static str {
    // 创建静态 OnceLock 用于延迟初始化
    static JWT_SECRET: OnceLock<String> = OnceLock::new();
    JWT_SECRET
        // 尝试获取已初始化的密钥，如果未初始化则执行闭包
        .get_or_init(|| {
            // 尝试从环境变量读取密钥
            std::env::var("PURE_ADMIN_JWT_SECRET")
                // 如果环境变量不存在，返回 None
                .ok()
                // 过滤掉空字符串
                .filter(|secret| !secret.trim().is_empty())
                // 如果环境变量不存在或为空，使用默认密钥
                .unwrap_or_else(|| DEFAULT_JWT_SECRET.to_string())
        })
        // 转换为字符串引用返回
        .as_str()
}

// ==========================================================================================
// JWT 配置构建函数
// ==========================================================================================

// 构建默认的 JWT 头部
//
// 返回值：
// 返回使用 HS256 算法的 JWT 头部
#[must_use]
fn default_header() -> Header {
    // 创建新的 JWT 头部，使用 HS256 算法
    Header::new(Algorithm::HS256)
}

// 构建默认的 JWT 验证配置
//
// 返回值：
// 返回使用 HS256 算法的验证配置
//
// 说明：
// 该配置用于验证传入的 JWT 令牌
#[must_use]
fn default_validation() -> Validation {
    // 创建新的验证配置，使用 HS256 算法
    Validation::new(Algorithm::HS256)
}

// 构建 JWT 声明
//
// 参数：
// - subject: 令牌主题，通常为用户名
// - token_type: 令牌类型，"access" 或 "refresh"
// - issued_at: 签发时间（Unix 时间戳，秒）
// - ttl: 有效期时长（秒）
//
// 返回值：
// 返回包含给定参数的 JwtClaims 结构体
//
// 注意：
// 使用 saturating_add 防止整数溢出
#[must_use]
fn build_claims(subject: &str, token_type: &'static str, issued_at: u64, ttl: u64) -> JwtClaims {
    JwtClaims {
        // 将主题转换为字符串
        sub: subject.to_string(),
        // 将令牌类型转换为字符串
        token_type: token_type.to_string(),
        // 设置签发时间
        iat: issued_at,
        // 计算过期时间，使用 saturating_add 防止溢出
        exp: issued_at.saturating_add(ttl),
    }
}

// ==========================================================================================
// JWT 编码函数
// ==========================================================================================

// 将 JWT 声明编码为令牌字符串
//
// 参数：
// - claims: JWT 声明结构体引用
//
// 返回值：
// 返回编码后的 JWT 令牌字符串
//
// 恐慌：
// 如果编码失败（理论上不会发生），会触发 panic
// 实际使用中，编码已知有效的声明不会失败
#[must_use]
fn encode_claims(claims: &JwtClaims) -> String {
    // 使用默认头部、给定声明和密钥编码
    encode(
        &default_header(),
        claims,
        // 从密钥字节创建编码密钥
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    // 理论上编码已知有效的声明不会失败
    .expect("encoding known JWT claims should not fail")
}

// ==========================================================================================
// 令牌生成与验证函数
// ==========================================================================================

// 生成 JWT 令牌对
//
// 功能：
// 为指定用户生成访问令牌和刷新令牌
//
// 参数：
// - subject: 用户主题（用户名）
//
// 返回值：
// 返回包含访问令牌、刷新令牌和过期时间的 TokenPair
//
// 令牌有效期：
// - access_token: 2 小时
// - refresh_token: 7 天
#[must_use]
pub fn mint_token_pair(subject: &str) -> TokenPair {
    // 获取当前时间（秒）
    let issued_at = now_secs();

    // 构建访问令牌声明
    let access_claims = build_claims(
        subject,
        ACCESS_TOKEN_TYPE,
        issued_at,
        ACCESS_TOKEN_LIFETIME_SECONDS,
    );

    // 构建刷新令牌声明
    let refresh_claims = build_claims(
        subject,
        REFRESH_TOKEN_TYPE,
        issued_at,
        REFRESH_TOKEN_LIFETIME_SECONDS,
    );

    // 编码声明为令牌字符串
    TokenPair {
        // 编码访问令牌声明
        access_token: encode_claims(&access_claims),
        // 编码刷新令牌声明
        refresh_token: encode_claims(&refresh_claims),
        // 计算过期时间的毫秒时间戳
        // 使用 saturating_mul 防止溢出，然后乘以 1000 转换为毫秒
        expires: access_claims.exp.saturating_mul(1000),
    }
}

// 验证刷新令牌
//
// 功能：
// 验证刷新令牌的有效性，包括签名、过期时间和令牌类型
//
// 参数：
// - refresh_token: 待验证的刷新令牌字符串
//
// 返回值：
// - 成功：返回令牌中的用户主题（用户名）
// - 失败：返回 AppError 验证错误
//
// 验证步骤：
// 1. 解码 JWT 令牌并验证签名
// 2. 检查令牌是否过期
// 3. 验证令牌类型是否为 "refresh"
// 4. 验证主题是否有效（非空）
//
// 错误情况：
// - 令牌格式非法
// - 令牌签名无效
// - 令牌已过期
// - 令牌类型不是 "refresh"
// - 主题为空
pub fn verify_refresh_token(refresh_token: &str) -> Result<String, AppError> {
    // 使用密钥解码令牌并验证
    let decoded = decode::<JwtClaims>(
        refresh_token,
        // 从密钥字节创建解码密钥
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &default_validation(),
    )
    // 如果解码或验证失败，返回通用错误消息
    .map_err(|_| AppError::Validation("invalid refreshToken".to_string()))?;

    // 获取解码后的声明
    let claims = decoded.claims;

    // 验证令牌类型和主题有效性
    // 如果类型不是 refresh 或主题为空，返回错误
    if claims.token_type != REFRESH_TOKEN_TYPE || claims.sub.trim().is_empty() {
        return Err(AppError::Validation("invalid refreshToken".to_string()));
    }

    // 返回有效的用户主题
    Ok(claims.sub)
}

// ==========================================================================================
// 用户档案解析
// ==========================================================================================

// 解析用户档案
//
// 功能：
// 根据用户名和密码查询并验证用户，返回完整的用户档案信息
//
// 参数：
// - username: 用户名
// - password: 密码（明文）
//
// 返回值：
// - 成功：返回包含用户完整信息的 UserProfile
// - 失败：返回 AppError 错误
//
// 执行流程：
// 1. 调用数据访问层查询用户
// 2. 验证密码是否匹配
// 3. 检查用户账号状态（是否激活、是否过期）
//
// 错误情况：
// - 用户不存在
// - 密码错误
// - 用户账号未激活
// - 用户账号已过期
pub fn resolve_user_profile(username: &str, password: &str) -> Result<UserProfile, AppError> {
    // 从数据库查询用户档案
    // 如果用户不存在或密码错误，返回错误
    let profile = auth_repository::find_user_profile(username, password)?
        .ok_or_else(|| AppError::Validation("invalid username or password".to_string()))?;

    // 检查用户账号是否可用（未过期且未禁用）
    admin_services::ensure_user_available_with_message(
        &profile.username,
        "invalid username or password",
        now_millis(),
    )?;

    // 返回用户档案
    Ok(profile)
}

// ==========================================================================================
// 登录数据构建
// ==========================================================================================

// 构建登录返回数据
//
// 功能：
// 将用户档案转换为登录成功返回的数据格式，包含用户信息和令牌对
//
// 参数：
// - profile: 用户档案信息
//
// 返回值：
// 返回包含用户信息和 JWT 令牌对的 LoginData
//
// 执行流程：
// 1. 为用户生成令牌对
// 2. 合并用户档案和令牌信息
//
// 注意：
// 此函数会生成新的令牌，因此每次调用都会产生新的令牌
#[must_use]
pub fn build_login_data(profile: UserProfile) -> LoginData {
    // 为用户生成令牌对
    let token = mint_token_pair(&profile.username);

    // 构建并返回登录数据
    LoginData {
        // 用户头像
        avatar: profile.avatar,
        // 用户名
        username: profile.username,
        // 用户昵称
        nickname: profile.nickname,
        // 角色列表
        roles: profile.roles,
        // 权限列表
        permissions: profile.permissions,
        // 访问令牌
        access_token: token.access_token,
        // 刷新令牌
        refresh_token: token.refresh_token,
        // 过期时间（毫秒时间戳）
        expires: token.expires,
    }
}

// ==========================================================================================
// 动态路由构建
// ==========================================================================================

// 构建异步动态路由
//
// 功能：
// 从数据库查询基于角色的动态路由配置，返回 vue-router 兼容的路由数据
//
// 返回值：
// - 成功：返回路由数组
// - 失败：返回 AppError 错误
//
// 路由数据来源：
// - 路由基本信息：数据库 routes 表
// - 角色权限：数据库 route_roles 表
// - 按钮权限：数据库 route_auths 表
//
// 注意：
// - 只返回当前用户有权访问的路由
// - 数据格式与 vue-router 兼容
pub fn build_async_routes() -> Result<Vec<Value>, AppError> {
    // 调用数据访问层查询动态路由
    auth_repository::find_async_routes()
}
