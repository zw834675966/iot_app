# 鉴权模块 (Auth)

本模块包含整个 pure-admin-thin 的**用户认证、权限校验和动态路由获取**等相关业务逻辑。采用领域驱动设计（DDD）的思想，将接口层、业务层、模型层完全解耦，以保证高可测试性和可维护性。

## 目录结构

```text
src-tauri/src/auth/
├── mod.rs          # 模块声明
├── commands.rs     # IPC 接口层（前端调用的入口）
├── services.rs     # 纯业务逻辑层（无副作用，独立于 Tauri）
└── models.rs       # 数据模型层（DTO，序列化结构体）
```

## 职责划分

### 1. 接口层 (`commands.rs`)

**职责**：作为前端和后端的桥梁（Adapter Layer）。
- 使用 `#[tauri::command]` 宏暴露接口供前端通过 `invoke()` 调用。
- **仅做参数的基础校验**（如必填项非空检查）。
- 绝不包含实际的业务逻辑和复杂计算。
- 将合法的参数转发给 `services` 层处理，并将结果包装为 `AppResult<T>` 返回。

### 2. 业务逻辑层 (`services.rs`)

**职责**：处理核心的业务规则（Domain Layer）。
- **纯函数为主**：尽量不依赖外部状态（如无必要，不要在这里直接读写全局锁或直接处理 HTTP 请求），保证 `#[must_use]` 和高度可测试性。
- 负责：生成 Token（`mint_token_pair`）、校验用户信息（`resolve_user_profile`）、构造登录返回数据（`build_login_data`）、组装动态路由树（`build_async_routes`）。
- **完全解耦**：这里没有任何 Tauri 相关的宏（如 `#[tauri::command]`），这意味着你可以在不启动 Tauri 的情况下（例如在普通的 CLI 工具或 Web 服务中）直接复用这些业务逻辑。

### 3. 数据模型层 (`models.rs`)

**职责**：定义跨层和跨端的数据传输对象（DTO）。
- 包含所有需要序列化（向前端返回）和反序列化（从前端接收）的结构体。
- **序列化风格**：为了匹配前端 JavaScript/TypeScript 的命名习惯，使用 `#[serde(rename_all = "camelCase")]` 将 Rust 的 `snake_case` 自动转换为 `camelCase`（例如 `refresh_token` -> `refreshToken`）。

## 扩展指南

当你需要在此模块添加一个新的功能（例如：**登出接口** `auth_logout`）时，请遵循以下流程：

1. **定义模型 (models.rs)**
   - 如果登出接口需要前端传参，新增一个 `LogoutPayload` 结构体。
   - 如果需要返回特定的结构，新增一个 `LogoutData` 结构体。
2. **实现业务逻辑 (services.rs)**
   - 编写一个 `pub fn perform_logout(payload: ...) -> ...` 函数。
   - 在该函数内部处理缓存清理、Token 吊销等业务逻辑。
3. **暴露命令 (commands.rs)**
   - 编写 `#[tauri::command] pub fn auth_logout(...) -> AppResult<...>`。
   - 进行参数校验，然后调用 `services::perform_logout`。
   - 添加对应的单元测试。
4. **注册命令 (lib.rs)**
   - 在 `src/lib.rs` 的 `invoke_handler!` 宏中添加 `auth::commands::auth_logout`。

## 测试策略

- **单元测试**：目前所有的测试都写在 `commands.rs` 的 `mod tests` 中。
- 由于 `services` 里的函数是纯粹的业务逻辑且没有 Tauri 的依赖，你可以非常轻易地对它们进行 Mock 和断言（例如测试 token 过期时间计算是否准确，测试不同角色的路由过滤是否正确）。
