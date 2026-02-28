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
    ├── auth/           # 鉴权与管理员业务领域（领域驱动设计）
    │   ├── mod.rs
    │   ├── commands.rs       # 用户鉴权 IPC 接口层
    │   ├── admin_commands.rs # 管理员操作 IPC 接口层
    │   ├── services.rs       # 用户鉴权业务逻辑层 (JWT 签发校验等)
    │   ├── admin_services.rs # 管理员业务逻辑层 (用户增删改查等)
    │   └── models.rs         # 鉴权数据模型层 (DTO)
    ├── notice/         # 消息通知业务领域
    │   ├── mod.rs
    │   ├── commands.rs       # 消息通知 IPC 接口层
    │   ├── services.rs       # 消息通知业务逻辑层
    │   ├── repository.rs     # 消息通知数据访问层
    │   └── models.rs         # 消息通知数据模型层
    └── db/             # 数据库基础设施层 (SQLite)
        ├── mod.rs             # 数据库连接管理入口
        ├── path_store.rs      # 数据库路径存储器
        ├── bootstrap.rs       # 数据库建表与种子数据引导
        ├── migrations.rs      # 数据库版本迁移逻辑
        ├── auth_repository.rs # 用户鉴权数据仓储
        └── admin_repository.rs# 管理员数据仓储
```

## 模块架构与设计原则

本项目后端严格遵循**领域驱动设计 (DDD)** 和 **职责分离** 的原则：

```
main.rs
  └─► lib.rs（挂载 Tauri 生命周期与 IPC 路由）
        │
        ├─► auth/commands.rs & notice/commands.rs  (Adapter Layer: 仅负责参数校验与 IPC 包装)
        │     └─► auth/services.rs & notice/services.rs  (Domain Layer: 纯业务规则，无副作用，高可测试)
        │           ├─► auth/models.rs & notice/models.rs  (DTO 数据传输对象)
        │           └─► db/auth_repository.rs & notice/repository.rs (Infrastructure Layer: 数据库交互)
        │
        ├─► db/mod.rs (基础设施: 管理 SQLite 连接、路径管理、数据迁移)
        │
        └─► core/error.rs (公共核心层: 统一错误枚举 `AppError` 与标准 JSON 响应 `ApiResponse`)
```

**核心设计原则**：
1. **依赖方向单向向下**：`commands` 依赖 `services`，`services` 依赖 `db` 和 `models`，绝不允许反向依赖。
2. **框架解耦**：所有 `services.rs` 和 `repository.rs` 中**绝不包含**任何 Tauri 特定的宏（如 `#[tauri::command]`），使得业务逻辑和数据持久化层可以完全脱离 GUI 环境，便于在命令行甚至不同框架中重用与独立测试。
3. **数据分离**：随着领域复杂度上升，数据层被切分为不同的 repository（如 `auth_repository`, `admin_repository`, `notice/repository` 等），降低了单一文件的维护成本。

## 快速开始

### 前置要求

| 工具     | 最低版本         | 说明                                                                   |
| -------- | ---------------- | ---------------------------------------------------------------------- |
| Rust     | 1.85+            | 通过 `rustup` 安装，采用 Edition 2024 规范                           |
| Node.js  | 20.19+ 或 22.13+ | 前端构建依赖                                                           |
| pnpm     | 9+               | 前端包管理器                                                           |
| 系统依赖 | —                | Windows: WebView2；macOS: Xcode CLT；Linux: `libwebkit2gtk-4.1-dev` 等 |

### 开发模式

```bash
# 在项目根目录执行（会自动启动前端 Vite 服务器和 Tauri 桌面窗口）
pnpm tauri:dev
```

### 仅运行 Rust 测试

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### 代码检查

```bash
# Clippy（采用 pedantic 等严格规则，所有 Warning 将触发 Error 报错）
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings

# 格式检查
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
```

### 生产构建

```bash
# 仅构建前端资源（为 Tauri 准备的相对路径静态资产）
pnpm build:tauri

# 构建完整桌面安装包（执行构建并将静态资产打包进可执行文件）
pnpm tauri build
```

## IPC 命令参考

前端通过 Tauri 的 `invoke()` 函数异步调用后端命令。
后端命令被按领域划分为不同的模块（目前有 `auth` 模块和 `notice` 模块）。

### `auth` 领域

#### `auth_login` — 用户登录
验证用户名与密码，成功后返回用户信息及 JWT 令牌对。

```typescript
const result = await invoke("auth_login", {
  payload: { username: "admin", password: "admin123" }
});
```

#### `auth_refresh_token` — 刷新令牌
使用有效的 `refreshToken` 换取新的 `accessToken`。

```typescript
const result = await invoke("auth_refresh_token", {
  payload: { refreshToken: "eyJ0eXAiOi..." }
});
```

#### `auth_get_async_routes` — 获取动态路由
拉取当前登录用户被授权访问的动态路由结构树。

```typescript
const result = await invoke("auth_get_async_routes");
```

#### 管理员特定命令
在 `auth/admin_commands.rs` 中实现了丰富的管理员 IPC 命令，例如：
- `auth_admin_register_user`: 注册新用户
- `auth_admin_renew_user_account`: 续期用户账号
- `auth_admin_list_users`: 获取用户列表
- `auth_admin_update_user`: 更新用户信息
- `auth_admin_delete_user`: 删除用户
- `auth_admin_change_user_password`: 重置/修改用户密码
- 等等（更多请参见源码 `admin_commands.rs`）

### `notice` 领域

包含系统通知与消息中心的查询及交互功能：
- `notice_get_unread_items`: 获取未读消息列表
- `notice_get_read_items`: 获取已读消息列表
- `notice_mark_read`: 将特定消息标记为已读

## 错误处理范式

所有命令失败时，后端的错误格式统一向前端抛出纯字符串，以便前端进行 Toast 提示或国际化匹配。

```json
// 在参数校验失败或数据库异常时，前端 invoke() 的 catch 闭包将直接捕获该错误字符串
"username is required"
```

错误类型在 `core/error.rs` 的 `AppError` 枚举中统一收口：
- `AppError::Validation(String)`: 参数校验错误
- `AppError::Database(String)`: SQLite 底层数据库抛出的错误

## 扩展与规范指南

### 添加新模块的推荐步骤

以新建 `system`（系统设置）模块为例：
1. **创建目录结构**：`mkdir src/system`
2. **定义模型**：在 `system/models.rs` 中声明请求/响应的 DTO（需派生 `Serialize`/`Deserialize`）
3. **实现仓储**：在 `system/repository.rs` 编写对应的 SQL 增删改查逻辑（只返回 `Result<T, AppError>`）
4. **业务逻辑**：在 `system/services.rs` 处理具体业务规则、调用数据层
5. **暴露命令**：在 `system/commands.rs` 使用 `#[tauri::command]` 宏定义与前端的 IPC 契约
6. **注册模块**：
   - 在 `src/system/mod.rs` 导出上述子模块。
   - 在 `src/lib.rs` 中引入 `pub mod system;` 并在 `invoke_handler!` 宏里注册所有命令。

### 代码质量与安全要求

- **零 `unwrap()` 原则**：业务代码中绝不允许使用隐式 panic（`.unwrap()` 或 `.expect()`）。遇到错误必须显式映射为 `AppError` 并使用 `?` 向上冒泡。
- **零 `unsafe` 原则**：除极其特殊的底层 FFI 绑定（目前项目中无此需求），禁止在代码中使用 `unsafe` 块。
- **Clippy 零警告**：每次提交前必须确保 `cargo clippy -- -D warnings` 没有抛出任何警告。所有对外公共 API 均需包含合理的模块级与函数级文档注释。
- **并发与可变性**：全局共享状态应通过 `tauri::State` 传递给 Command；使用 `OnceLock` 替代老旧的 `lazy_static!`。

## 技术栈与依赖库

| 依赖                                                  | 版本 | 用途                |
| ----------------------------------------------------- | ---- | ------------------- |
| [Tauri](https://tauri.app/)                           | 2.10 | 桌面跨平台应用框架  |
| [serde](https://serde.rs/)                            | 1.0  | 数据序列化反序列化  |
| [sqlx](https://docs.rs/sqlx/)                         | 0.8  | SQLite 异步驱动与查询层 |
| [thiserror](https://docs.rs/thiserror/)               | 2.0  | 统一领域错误派生宏  |
| [tauri-plugin-log](https://docs.rs/tauri-plugin-log/) | 2    | 文件及终端日志输出  |
| [jsonwebtoken](https://docs.rs/jsonwebtoken/)         | 10.3 | 鉴权 JWT 签名与解析 |

## AI Coding Workflow (Project Rule)
- Mandatory workflow: `../skills/project-aicode-workflow/SKILL.md`
- Rust scoped rules: `./AGENTS.md`
- Tauri framework constraints: `../docs/tauri-framework-constraints.md`
- Deployment strategy: `../docs/deployment-strategy.md`
- Progress log: `../docs/development-progress.md`
