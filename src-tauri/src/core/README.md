# 核心模块 (Core)

本模块定义了 **能源管理系统 (Tauri 后端)** 跨越所有子业务领域的基础设施和共用结构。这里的代码与具体的业务逻辑无关（如用户、权限、设备），它们是支撑整个应用稳定运行的基石。

## 目录结构

```text
src-tauri/src/core/
├── mod.rs          # 模块声明
└── error.rs        # 全局统一错误与响应封装 (AppError, ApiResponse)
```

## 核心设计

### 1. 统一 API 响应格式 (`ApiResponse`)

在与前端（Vue 3）交互时，整个项目的标准响应格式非常严格，统一包装为：

```json
{
  "success": true,
  "data": { ... }
}
```

- `ApiResponse<T>` 是个泛型结构体，`T` 代表实际的业务数据（如 `LoginData`、`Vec<Value>`）。
- 当执行成功时，所有 Tauri Command 都必须返回 `ApiResponse::ok(data)`。
- 这个结构保证了前端在处理 `invoke` 返回结果时可以通过 `res.success` 统一判断请求状态。

### 2. 全局应用级错误枚举 (`AppError`)

Tauri 强制要求 Command 函数返回 `Result<T, E>` 时，**`E` 必须实现 `Serialize`**（能够将错误文本转换成 JSON 传回前端的 Promise 拒绝结果中）。

目前我们使用 [`thiserror`](https://docs.rs/thiserror) 派生宏来自动生成标准 `std::error::Error` 和 `Display` 的实现。为了将错误信息转换为前端能识别的字符串文本：

```rust
// error.rs 中我们为 AppError 手动实现了 Serialize
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 直接序列化 Display 的输出（如 "username is required" 或 "database error: xxx"）
        serializer.serialize_str(&self.to_string())
    }
}
```

### 3. Tauri 命令统一返回类型 (`AppResult<T>`)

为简化在 `commands.rs` 中的函数签名，我们定义了类型别名：

```rust
pub type AppResult<T> = Result<ApiResponse<T>, AppError>;
```

所有返回给前端的接口，返回值必须是这个别名，以便保持风格统一并自动适配 Tauri 的 IPC 机制。

## 扩展指南

当你未来需要添加新的全局基础设施时，你可以将代码放在 `src/core/` 目录下。

### 扩展新错误变体

随着业务的复杂化，你可以很方便地扩展错误类型：

1. 打开 `src/core/error.rs`
2. 在 `AppError` 枚举下追加新的变体：
   ```rust
   #[derive(Debug, Error, PartialEq, Eq)]
   pub enum AppError {
       #[error("{0}")]
       Validation(String),

       // 数据库操作失败变体（已内置）
       #[error("database error: {0}")]
       Database(String),

       // 新增的其他变体
       #[error("IO error: {0}")]
       Io(String),
   }
   ```
3. 因为手动实现了 `Serialize`，前端在调用失败时会自动收到 `"database error: table not found"` 或者相应的格式化文本。

### 未来可能的核心扩展（示例）

如果项目变大，你可以在 `src/core/` 中新增以下模块：
- `config.rs`: 应用全局配置解析逻辑（结合 Tauri 的 `app_data_dir` 提取用户配置）。
- `logger.rs`: 自定义的文件日志记录器模块（辅助排查生产问题）。
- `hardware.rs`: 串口或外部硬件通讯的全局管理状态封装。
