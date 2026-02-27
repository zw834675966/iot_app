# src-tauri — Rust 桌面后端开发者指南

> pure-admin-thin 的 Tauri 桌面端 Rust 后端，负责提供本地模拟 API 和桌面窗口管理。

## 目录结构

```
src-tauri/
├── Cargo.toml          # 包配置、依赖声明、Lint 规则
├── Cargo.lock          # 依赖锁文件（提交到版本库保证可复现构建）
├── build.rs            # Tauri 编译期构建脚本
├── tauri.conf.json     # Tauri 运行时配置（窗口、打包、开发服务器）
├── capabilities/       # Tauri v2 权限能力声明
├── gen/                # Tauri 自动生成的平台绑定代码（勿手动编辑）
├── icons/              # 应用图标资源
├── target/             # Cargo 编译产物（已 gitignore）
└── src/
    ├── main.rs         # 桌面应用入口点
    ├── lib.rs          # 库入口 — 模块注册 + Tauri 运行时启动
    ├── error.rs        # 统一错误类型与 API 响应包装
    ├── models.rs       # 数据传输对象（DTO）定义
    ├── services.rs     # 纯业务逻辑（与 Tauri 框架解耦）
    └── commands.rs     # Tauri IPC 命令处理器 + 单元测试
```

## 模块依赖关系

```
main.rs
  └─► lib.rs（run 函数）
        ├─► commands.rs ─── Tauri 命令薄层
        │     ├─► error.rs ─── AppError / ApiResponse / AppResult
        │     ├─► models.rs ── 请求体 / 响应体 DTO
        │     └─► services.rs ─ 业务逻辑
        ├─► error.rs
        ├─► models.rs
        └─► services.rs
              ├─► models.rs
              └─► (标准库 / serde_json)
```

**设计原则**：依赖方向单向向下，`services.rs` 不依赖 Tauri，可独立测试。

## 快速开始

### 前置要求

| 工具     | 最低版本         | 说明                                                                   |
| -------- | ---------------- | ---------------------------------------------------------------------- |
| Rust     | 1.85+            | 通过 `rustup` 安装，Edition 2024                                       |
| Node.js  | 20.19+ 或 22.13+ | 前端构建依赖                                                           |
| pnpm     | 9+               | 前端包管理器                                                           |
| 系统依赖 | —                | Windows: WebView2；macOS: Xcode CLT；Linux: `libwebkit2gtk-4.1-dev` 等 |

### 开发模式

```bash
# 在项目根目录执行（会同时启动前端 Vite 和 Tauri 桌面窗口）
pnpm tauri:dev
```

### 仅运行 Rust 测试

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### 代码检查

```bash
# Clippy（含 pedantic 规则，所有 Warning 视为 Error）
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings

# 格式检查
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
```

### 生产构建

```bash
# 仅构建前端资源（Tauri 所需）
pnpm build:tauri

# 构建完整桌面安装包
pnpm tauri build
```

## IPC 命令参考

前端通过 Tauri 的 `invoke()` 函数调用后端命令，所有命令定义在 `commands.rs` 中。

### `auth_login` — 用户登录

```typescript
// 前端调用
const result = await invoke("auth_login", {
  payload: { username: "admin", password: "admin123" }
});
```

**请求体**：

| 字段       | 类型     | 必填 | 说明   |
| ---------- | -------- | ---- | ------ |
| `username` | `string` | ✅   | 用户名 |
| `password` | `string` | ✅   | 密码   |

**成功响应**（`{ success: true, data: LoginData }`）：

| 字段           | 类型       | 说明                        |
| -------------- | ---------- | --------------------------- |
| `avatar`       | `string`   | 头像 URL                    |
| `username`     | `string`   | 用户名                      |
| `nickname`     | `string`   | 昵称                        |
| `roles`        | `string[]` | 角色列表                    |
| `permissions`  | `string[]` | 权限标识列表                |
| `accessToken`  | `string`   | 访问令牌                    |
| `refreshToken` | `string`   | 刷新令牌                    |
| `expires`      | `number`   | 过期时间（Unix 毫秒时间戳） |

**内置用户**：

| 用户名   | 角色     | 权限                                        |
| -------- | -------- | ------------------------------------------- |
| `admin`  | `admin`  | `*:*:*`（超级管理员）                       |
| 其他任意 | `common` | `permission:btn:add`, `permission:btn:edit` |

---

### `auth_refresh_token` — 刷新令牌

```typescript
const result = await invoke("auth_refresh_token", {
  payload: { refreshToken: "tauri.admin.refresh.1234567890" }
});
```

**请求体**：

| 字段           | 类型     | 必填 | 说明               |
| -------------- | -------- | ---- | ------------------ |
| `refreshToken` | `string` | ✅   | 当前持有的刷新令牌 |

**成功响应**（`{ success: true, data: RefreshTokenData }`）：

| 字段           | 类型     | 说明       |
| -------------- | -------- | ---------- |
| `accessToken`  | `string` | 新访问令牌 |
| `refreshToken` | `string` | 新刷新令牌 |
| `expires`      | `number` | 新过期时间 |

---

### `auth_get_async_routes` — 获取动态路由

```typescript
const result = await invoke("auth_get_async_routes");
```

**成功响应**（`{ success: true, data: Route[] }`）：

返回与 `vue-router` 兼容的路由配置 JSON 数组，路由树结构如下：

```
/permission                      — 权限管理
├── /permission/page/index       — 页面权限
└── /permission/button           — 按钮权限
    ├── /permission/button/router — 路由返回按钮权限
    └── /permission/button/login  — 登录接口返回按钮权限
```

## 错误处理

所有命令的错误响应格式统一：

```json
// 校验失败时，前端 invoke() 的 catch 会收到错误字符串
"username is required"
```

错误类型定义在 `error.rs` 中，当前仅有 `Validation` 变体。扩展新错误类型时：

1. 在 `AppError` 枚举中新增变体
2. `thiserror` 的 `#[error("...")]` 属性自动生成 `Display` 实现
3. `Serialize` 实现会将 `Display` 输出序列化为字符串

## 配置文件说明

### `tauri.conf.json`

| 字段                 | 值                      | 说明                        |
| -------------------- | ----------------------- | --------------------------- |
| `build.devUrl`       | `http://localhost:8848` | 开发模式前端地址            |
| `build.frontendDist` | `../dist`               | 生产构建前端资源目录        |
| `app.windows[0]`     | 1280×800                | 默认窗口尺寸，最小 1024×720 |
| `bundle.targets`     | `"all"`                 | 构建所有平台安装包格式      |

### `Cargo.toml` Lint 配置

- `unsafe_code = "warn"` — 项目原则上禁止 `unsafe`，出现时发出警告
- `clippy::all = "warn"` — Clippy 全部默认检查
- `clippy::pedantic = "warn"` — 更严格的代码风格检查

## 扩展指南

### 添加新的 IPC 命令

1. **定义请求/响应体**：在 `models.rs` 中添加新的结构体
2. **实现业务逻辑**：在 `services.rs` 中编写与 Tauri 无关的纯函数
3. **注册命令**：在 `commands.rs` 中添加 `#[tauri::command]` 函数
4. **挂载命令**：在 `lib.rs` 的 `invoke_handler` 宏中注册新命令名
5. **编写测试**：在 `commands.rs` 的 `mod tests` 中添加单元测试

### 添加新的错误类型

```rust
// error.rs
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),

    // 新增：数据库错误
    #[error("数据库操作失败: {0}")]
    Database(String),
}
```

### 代码质量要求

- **零 `unwrap()`**：业务代码中禁止使用 `.unwrap()` / `.expect()`，使用 `?` 操作符链式传递错误
- **零 `unsafe`**：除非 FFI 绑定绝对必要，否则禁止使用 `unsafe` 块
- **Clippy 零警告**：CI 中 `cargo clippy -- -D warnings` 必须通过
- **所有测试通过**：`cargo test` 必须全部绿色

## 技术栈

| 依赖                                                  | 版本 | 用途                |
| ----------------------------------------------------- | ---- | ------------------- |
| [Tauri](https://tauri.app/)                           | 2.10 | 桌面应用框架        |
| [serde](https://serde.rs/)                            | 1.0  | 序列化/反序列化框架 |
| [serde_json](https://docs.rs/serde_json/)             | 1.0  | JSON 处理           |
| [thiserror](https://docs.rs/thiserror/)               | 2.0  | 错误类型派生宏      |
| [tauri-plugin-log](https://docs.rs/tauri-plugin-log/) | 2    | 开发模式日志插件    |
| [log](https://docs.rs/log/)                           | 0.4  | Rust 日志门面       |

## AI Coding Workflow (Project Rule)
- Mandatory workflow: `../skills/project-aicode-workflow/SKILL.md`
- Rust scoped rules: `./AGENTS.md`
- Tauri framework constraints: `../docs/tauri-framework-constraints.md`
- Deployment strategy: `../docs/deployment-strategy.md`
- Progress log: `../docs/development-progress.md`
