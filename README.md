# 能源管理系统 (Energy Management System)

本项目是一个基于 **Vue 3 + TypeScript + Vite** 与 **Tauri v2 + Rust + PostgreSQL** 构建的桌面端能源管理控制台。

项目由 `vue-pure-admin` 精简演进而来，重点面向离线优先、内网部署和本地 IPC 通信场景。

## 核心技术栈

- 前端：Vue 3 (Composition API), Vite, TypeScript
- UI：Element Plus, Tailwind CSS, 本地图标资源
- 状态与路由：Pinia, Vue Router
- 桌面宿主：Tauri v2（Rust）
- 数据与鉴权：PostgreSQL 17.x + TimescaleDB，JWT (`jsonwebtoken`)
- 数据访问层：`sqlx`（PostgreSQL 驱动）

## 主要特性

- 本地桌面运行：支持 Windows / macOS / Linux 打包部署
- IPC 架构：前后端通过 Tauri `invoke` 通信，移除传统 HTTP API 依赖
- PostgreSQL 统一存储：用户、权限、路由、通知数据均由 PostgreSQL 管理
- TimescaleDB 扩展：数据库启用时序扩展能力，便于后续时序数据演进
- 离线优先：适配内网/隔离网络场景

## 开发与构建

### 环境要求

- Node.js `^20.19.0` 或 `>=22.13.0`
- pnpm `>=9`
- Rust toolchain（建议 `1.85+`）
- PostgreSQL `17.x`
- TimescaleDB（建议 `2.19+`，当前本机验证为 `2.25.0`）

### 常用命令

```bash
pnpm install
pnpm dev
pnpm tauri:dev
pnpm lint
pnpm typecheck
cargo test --manifest-path src-tauri/Cargo.toml
pnpm build
pnpm tauri build
```

## 数据库配置 (PostgreSQL + TimescaleDB)

应用不再使用本地 sqlite/redb 文件；运行时改为 PostgreSQL 连接。

### 连接字符串来源

`src-tauri` 运行时读取顺序：

1. `src-tauri/config/default.toml`
2. `src-tauri/config/local.toml`（可选，本地覆盖层）
3. 环境变量覆盖（最高优先级）

测试环境可使用：

- `PURE_ADMIN_TEST_DATABASE_URL`

常用覆盖变量：

- `PURE_ADMIN_DATABASE_URL` / `PURE_ADMIN_DATABASE__URL`
- `PURE_ADMIN_TEST_DATABASE_URL` / `PURE_ADMIN_DATABASE__TEST_URL`
- `PURE_ADMIN_JWT_SECRET` / `PURE_ADMIN_AUTH__JWT_SECRET`
- `PURE_ADMIN_SERVER_PORT` / `PURE_ADMIN_SERVER__PORT`

### 推荐本地初始化

```powershell
$env:PGPASSWORD='你的postgres密码'
psql -h 127.0.0.1 -U postgres -d postgres -c "CREATE DATABASE pure_admin_thin"
psql -h 127.0.0.1 -U postgres -d postgres -c "CREATE DATABASE pure_admin_thin_test"
psql -h 127.0.0.1 -U postgres -d pure_admin_thin -c "CREATE EXTENSION IF NOT EXISTS timescaledb"
psql -h 127.0.0.1 -U postgres -d pure_admin_thin_test -c "CREATE EXTENSION IF NOT EXISTS timescaledb"
```

### 快速自检

```powershell
psql --version
psql -h 127.0.0.1 -U postgres -d postgres -tAc "SHOW server_version"
psql -h 127.0.0.1 -U postgres -d pure_admin_thin -tAc "SELECT extversion FROM pg_extension WHERE extname='timescaledb'"
```

## 相关文档

- 前端说明：`src/README.md`
- Rust/Tauri 说明：`src-tauri/README.md`
- PostgreSQL 运行说明：`docs/postgresql-timescaledb-runtime.md`
- Tauri 约束：`docs/tauri-framework-constraints.md`
- 开发进度：`docs/development-progress.md`

## 致谢

前端基础结构与部分 UI 体系来自 [vue-pure-admin](https://github.com/pure-admin/vue-pure-admin)（MIT License）。

## AI Coding Rules For This Repo

- Mandatory skill workflow: `skills/project-aicode-workflow/SKILL.md`
- Global agent rules: `AGENTS.md`
- Frontend scope rules: `src/AGENTS.md`
- AI skills setup/routing: `docs/ai-skills-usage.md`
- Rust scope rules: `src-tauri/AGENTS.md`
- Tauri framework constraints: `docs/tauri-framework-constraints.md`
- Deployment strategy: `docs/deployment-strategy.md`
- Progress tracking: `docs/development-progress.md`
