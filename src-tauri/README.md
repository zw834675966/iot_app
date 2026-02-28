# src-tauri — Rust 桌面后端开发者指南

> pure-admin-thin 的 Tauri 桌面端 Rust 后端，负责提供本地 SQLite 数据库支持、进程间通信（IPC）接口和桌面窗口管理。

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
    ├── core/           # 核心基础设施层（统一错误与响应封装）
    │   ├── mod.rs
    │   └── error.rs
    ├── auth/           # 鉴权业务领域（领域驱动设计）
    │   ├── mod.rs
    │   ├── commands.rs # Tauri IPC 接口层
    │   ├── services.rs # 纯业务逻辑层 (JWT 签发校验等)
    │   └── models.rs   # 数据模型层 (DTO)
    └── db/             # 数据库访问层 (SQLite)
        ├── mod.rs      # 数据库连接管理与初始化
        ├── migrations/ # SQL 迁移与种子数据脚本
        └── auth_repository.rs # 鉴权域数据查询仓储
```

## 模块架构与设计原则

本项目后端遵循**领域驱动设计 (DDD)** 和 **职责分离** 的原则：

```
main.rs
  └─► lib.rs（挂载 Tauri 生命周期与 IPC 路由）
        │
        ├─► auth/commands.rs  (Adapter Layer: 仅负责参数校验与 IPC 包装)
        │     └─► auth/services.rs  (Domain Layer: 纯业务规则，无副作用，高可测试)
        │           ├─► auth/models.rs  (DTO 定义)
        │           └─► db/auth_repository.rs (Infrastructure Layer: 数据库交互)
        │
        ├─► db/mod.rs (基础设施: 管理 SQLite 连接池/文件路径/建表)
        │
        └─► core/error.rs (公共层: 统一错误枚举与 JSON 响应格式)
```

**核心设计原则**：
1. **依赖方向单向向下**：`commands` 依赖 `services`，`services` 依赖 `db` 和 `models`，绝不允许反向依赖。
2. **框架解耦**：`auth/services.rs` 和 `db/*` 中完全不包含任何 Tauri 特定的宏（如 `#[tauri::command]`），使得业务逻辑和数据层可以脱离 GUI 环境独立测试。

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

前端通过 Tauri 的 `invoke()` 函数调用后端命令，所有鉴权相关命令定义在 `src/auth/commands.rs` 中。

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

---

### `auth_refresh_token` — 刷新令牌

```typescript
const result = await invoke("auth_refresh_token", {
  payload: { refreshToken: "eyJ0eXAiOi..." }
});
```

**请求体**：

| 字段           | 类型     | 必填 | 说明               |
| -------------- | -------- | ---- | ------------------ |
| `refreshToken` | `string` | ✅   | 当前持有的刷新令牌 |

---

### `auth_get_async_routes` — 获取动态路由

```typescript
const result = await invoke("auth_get_async_routes");
```

**成功响应**（`{ success: true, data: Route[] }`）：

返回与 `vue-router` 兼容的路由配置 JSON 数组，数据动态来源于底层的 `SQLite` 数据库。

## 错误处理

所有命令的错误响应格式统一：

```json
// 校验或数据库执行失败时，前端 invoke() 的 catch 会收到错误字符串
"username is required"
```

错误类型定义在 `core/error.rs` 中。当前包含：
- `Validation(String)`: 参数校验错误
- `Database(String)`: `SQLite` 数据库操作错误

## 扩展指南

当你需要开发新模块（例如 `system` 系统设置）时：

1. **创建目录**：`mkdir src/system`
2. **定义模型**：在 `system/models.rs` 定义 DTO
3. **实现仓储**：在 `db/system_repository.rs` 编写对应的 SQL 增删改查
4. **业务逻辑**：在 `system/services.rs` 调用仓储并处理业务
5. **暴露命令**：在 `system/commands.rs` 使用 `#[tauri::command]` 包装服务
6. **注册模块**：在 `src/lib.rs` 中引入 `pub mod system;` 并在 `invoke_handler!` 中注册命令。

### 代码质量要求

- **零 `unwrap()`**：业务代码中禁止使用 `.unwrap()` / `.expect()`，使用 `?` 操作符链式传递错误。
- **零 `unsafe`**：除非 FFI 绑定绝对必要，否则禁止使用 `unsafe` 块。
- **Clippy 零警告**：CI 中 `cargo clippy -- -D warnings` 必须通过，所有的 API 需包含适当的文档注释和 `# Errors` 声明。
- **所有测试通过**：`cargo test` 必须全部绿色，业务逻辑需编写单元测试。

## 技术栈

| 依赖                                                  | 版本 | 用途                |
| ----------------------------------------------------- | ---- | ------------------- |
| [Tauri](https://tauri.app/)                           | 2.10 | 桌面应用框架        |
| [serde](https://serde.rs/)                            | 1.0  | 序列化/反序列化框架 |
| [rusqlite](https://docs.rs/rusqlite/)                 | 0.37 | SQLite 数据库驱动   |
| [thiserror](https://docs.rs/thiserror/)               | 2.0  | 错误类型派生宏      |
| [tauri-plugin-log](https://docs.rs/tauri-plugin-log/) | 2    | 开发模式日志插件    |
| [jsonwebtoken](https://docs.rs/jsonwebtoken/)         | 10.3 | JWT 令牌管理        |

## AI Coding Workflow (Project Rule)
- Mandatory workflow: `../skills/project-aicode-workflow/SKILL.md`
- Rust scoped rules: `./AGENTS.md`
- Tauri framework constraints: `../docs/tauri-framework-constraints.md`
- Deployment strategy: `../docs/deployment-strategy.md`
- Progress log: `../docs/development-progress.md`
