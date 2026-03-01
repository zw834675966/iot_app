# 核心模块 (Core)

本模块定义了 **能源管理系统 (Tauri 后端)** 跨越所有子业务领域的基础设施和共用结构。这里的代码与具体的业务逻辑无关（如用户、权限、设备），它们是支撑整个应用稳定运行的基石。

## 目录结构

```text
src-tauri/src/core/
├── mod.rs          # 模块声明
├── error.rs        # 全局统一错误与响应封装 (AppError, ApiResponse)
├── config.rs       # 运行时配置加载（config.toml + env）
└── tracing.rs      # tracing 初始化与请求链路 span 包装
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

### 已有扩展

- `config.rs`: 使用 `config` crate 统一读取 `src-tauri/config/default.toml`、`src-tauri/config/local.toml`（可选）以及环境变量覆盖。
- `tracing.rs`: 使用 `tracing + tracing-subscriber + tracing-appender` 提供分级日志与请求链路追踪：
  - 控制台输出：按级别输出（INFO/WARN/ERROR）。
  - 文件输出：按天滚动写入日志文件（daily rotation）。
  - 请求链路：在 Tauri command 入口创建 `tauri_request` span，包含 `request_id` 与 `command` 字段。
  - 前端关联：前端 `invoke` 会携带 `trace.requestId`，后端优先使用该值作为 `request_id`，用于端到端链路对齐。
  - 运行约束：不要再与 `tauri-plugin-log` 同时初始化 logger（否则会触发“logging system was already initialized”）。

### Logging 配置项

`src-tauri/config/default.toml` 支持：

```toml
[logging]
level = "info"
directory = "logs"
```

可用环境变量覆盖：
- `PURE_ADMIN_LOGGING_LEVEL` / `PURE_ADMIN_LOGGING__LEVEL`
- `PURE_ADMIN_LOGGING_DIR` / `PURE_ADMIN_LOGGING__DIRECTORY`

### 前端-后端链路字段约定

前端通过 Tauri `invoke` 传入：

```json
{
  "trace": {
    "requestId": "fe-<session>-<command>-<counter>"
  }
}
```

后端 command 入口参数中使用 `Option<TraceContext>` 接收；若该字段缺失，则后端自动生成内部递增 `request_id`。

### 未来可能的核心扩展（示例）

- `logger.rs`: 自定义的文件日志记录器模块（辅助排查生产问题）。
- `hardware.rs`: 串口或外部硬件通讯的全局管理状态封装。
