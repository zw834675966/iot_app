# Tauri 后端模块说明（src-tauri/src）

## 概览
本目录包含 Tauri v2 后端 Rust 代码，负责应用启动、配置加载、日志初始化、数据库初始化，以及对前端暴露的 IPC 命令注册。

## 模块结构
- `auth/`：认证与账号管理逻辑（登录、刷新、管理员操作、设备权限等）。
- `core/`：运行时配置、日志与基础设施能力。
- `db/`：业务数据库初始化与连接配置。
- `notice/`：通知中心数据与读/未读状态管理。
- `lib.rs`：应用启动入口与命令注册。
- `main.rs`：Tauri 启动入口（调用 `lib::run`）。

## 启动流程摘要（lib.rs）
1. 读取运行时配置。
2. 初始化 tracing 日志系统。
3. 设置数据库 URL 并初始化业务数据库。
4. 执行认证到期补偿任务。
5. 初始化通知中心数据库。
6. 注册 IPC 命令并启动 Tauri 运行时。

## IPC 命令概览
- 认证相关：
  - `auth_login`
  - `auth_refresh_token`
  - `auth_get_async_routes`
- 管理员操作：
  - `auth_admin_register_user`
  - `auth_admin_renew_user_account`
  - `auth_admin_list_users`
  - `auth_admin_update_user`
  - `auth_admin_delete_user`
  - `auth_admin_change_user_password`
  - `user_device_scope_get`
  - `user_device_scope_upsert`
- 通知中心：
  - `notice_get_unread_items`
  - `notice_get_read_items`
  - `notice_mark_read`

## 关键约束（Tauri v2）
- 如需新增命令、能力或权限，必须遵循 `docs/tauri-framework-constraints.md` 的安全边界要求。
- 命令尽量保持轻量，重逻辑下沉到服务模块；耗时操作优先 `async`。

## 常用验证命令
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `pnpm typecheck`（前端 IPC 调用变更时）
- `pnpm lint`（前端 IPC 包装变更时）
- `pnpm tauri:dev`（Tauri 相关变更后手动冒烟）
