//! # 错误与响应类型模块
//!
//! 定义全局统一的错误枚举 [`AppError`]、标准化 API 响应包装 [`ApiResponse`]
//! 以及命令返回值的类型别名 [`AppResult`]。
//!
//! ## 设计思路
//!
//! Tauri 的 `#[tauri::command]` 要求返回值同时满足 `Serialize`，
//! 因此 `AppError` 手动实现了 `Serialize`，将错误信息序列化为字符串后传递给前端。

use serde::{Serialize, Serializer};
use thiserror::Error;

/// 应用级错误枚举。
///
/// 通过 [`thiserror`] 自动实现 `std::error::Error` 和 `Display`。
/// 当前仅包含参数校验错误变体，后续可按需扩展（如 `Database`、`Network` 等）。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AppError {
    /// 请求参数校验失败，内含具体的错误描述文本。
    #[error("{0}")]
    Validation(String),
}

/// 为 `AppError` 手动实现 `Serialize`。
///
/// Tauri 命令的返回类型 `Result<T, E>` 要求 `E: Serialize`，
/// 此处将错误的 `Display` 输出直接序列化为 JSON 字符串，
/// 前端收到的格式为：`"username is required"`。
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// 标准化 API 响应包装。
///
/// 所有 Tauri 命令的成功返回值都封装在此结构中，
/// 保证前端收到的 JSON 格式统一为 `{ "success": true, "data": ... }`。
///
/// # 泛型参数
///
/// - `T` — 实际业务数据类型，需满足 `Serialize`
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    /// 请求是否成功
    pub success: bool,
    /// 业务数据载荷
    pub data: T,
}

impl<T> ApiResponse<T> {
    /// 构造一个成功响应，自动设置 `success = true`。
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data,
        }
    }
}

/// Tauri 命令的统一返回类型别名。
///
/// - `Ok` 分支：`ApiResponse<T>` —— 标准化成功响应
/// - `Err` 分支：`AppError` —— 序列化后的错误字符串
pub type AppResult<T> = Result<ApiResponse<T>, AppError>;
