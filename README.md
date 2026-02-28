# 能源管理系统 (Energy Management System)

本项目是一个基于 **Vue 3 + TypeScript + Vite** 和 **Tauri v2 + Rust + SQLite** 构建的现代化桌面端能源管理控制台。

本项目由 `vue-pure-admin` 精简版演进而来，并针对“本地单程序运行”与“离线优先”的工业/物联网应用场景进行了深度重构。项目中原有的前端 HTTP 网络请求已被彻底剥离，取而代之的是更加安全、高效的 Tauri IPC 进程间通信。

## 核心技术栈

- **前端框架**：Vue 3 (Composition API), Vite, TypeScript
- **UI 与样式**：Element-Plus, Tailwind CSS, 纯本地离线图标
- **状态与路由**：Pinia, Vue Router (支持前端动态路由注入)
- **桌面端宿主**：Tauri v2 (Rust 编写核心逻辑)
- **本地存储与鉴权**：SQLite (`rusqlite`), 离线 JWT 签发与验证 (jsonwebtoken)

## 主要特性

- **桌面级本地运行**：无需外部 Web 服务器，直接打包为跨平台桌面应用程序（Windows / macOS / Linux）。
- **IPC 安全架构**：全面移除传统 Axios/Fetch HTTP 请求机制。前端与后端的交互全部通过 `invoke` 本地命令完成，有效抵御针对 Web 端的网络攻击与数据窃听。
- **本地嵌入式数据库**：内置 SQLite 数据库支持，用于存储用户凭证、设备注册表与系统配置。所有的表结构设计及基础数据初始化均由 Rust 端迁移脚本自动化管控。
- **离线运行环境**：精简了原本高度依赖互联网的资源，所有头像、图标及核心静态依赖均已被本地化处理，以适应严格隔离的内网部署环境。

## 开发与构建指南

### 环境要求

- [Node.js](https://nodejs.org/) (推荐 `^20.19.0` 或 `>=22.13.0`)
- [pnpm](https://pnpm.io/) (`>= 9.0.0`)
- [Rust Toolchain](https://rustup.rs/) (edition 2021/2024)

### 常用命令

```bash
# 1. 安装前端依赖
pnpm install

# 2. 启动前端 Vite 开发服务 (纯前端预览，无后端 IPC 支持)
pnpm dev

# 3. 启动 Tauri 桌面端本地开发环境 (完整功能)
pnpm tauri:dev

# 4. 运行前端代码格式化与类型检查
pnpm lint
pnpm typecheck

# 5. 运行后端 Rust 单元测试
cargo test --manifest-path src-tauri/Cargo.toml

# 6. 构建并打包跨平台桌面安装包
pnpm build
pnpm tauri build
```

## 本地数据库位置 (Tauri SQLite)

本项目在客户端启动时会自动初始化并完成 SQLite 数据库迁移。

- 运行时数据库文件名：`pure-admin-thin.sqlite3`
- 启动后实际写入目录（按 Tauri `app_data_dir` + `db/` 子目录分配）：`<app_data_dir>/db/pure-admin-thin.sqlite3`
- Windows 示例路径：`C:\Users\<用户名>\AppData\Roaming\com.pureadmin.thin\db\pure-admin-thin.sqlite3`
- 开发环境默认兜底路径（未显式设置 DB_PATH 时）：`<项目当前工作目录>/db/pure-admin-thin.sqlite3`

### 快速自检（Windows / PowerShell）

可通过以下命令检查数据库文件是否在 Tauri 运行目录成功创建：

```powershell
Test-Path "$env:APPDATA\com.pureadmin.thin\db\pure-admin-thin.sqlite3"
```
如果返回 `True`，即表示运行正常。

## 本地数据库位置 (通知中心 redb)

消息通知（通知/消息/待办）数据由 Tauri 侧 `redb` 存储，不在 `pure-admin-thin.sqlite3` 中。

- 运行时数据库文件名：`pure-admin-thin-notice.redb`
- 启动后实际写入目录（按 Tauri `app_data_dir` + `db/` 子目录分配）：`<app_data_dir>/db/pure-admin-thin-notice.redb`
- Windows 示例路径：`C:\Users\<用户名>\AppData\Roaming\com.pureadmin.thin\db\pure-admin-thin-notice.redb`

### 快速自检（Windows / PowerShell）

```powershell
Test-Path "$env:APPDATA\com.pureadmin.thin\db\pure-admin-thin-notice.redb"
```
如果返回 `True`，即表示通知中心数据库文件已创建。

## 详细架构文档

项目各模块的设计和改造详情，请参阅以下专门的架构说明：
- [前端工程指南](./src/README.md)
- [后端工程指南](./src-tauri/README.md)
- [Tauri 框架约束规范](./docs/tauri-framework-constraints.md)

## 致谢与版权声明

本项目前端基础架构与 UI 界面体系基于 [vue-pure-admin](https://github.com/pure-admin/vue-pure-admin) 提炼修改。
- 原开源协议：MIT License
- 原版权所有：Copyright (c) 2020-present, pure-admin

## AI Coding Rules For This Repo
- Mandatory skill workflow: `skills/project-aicode-workflow/SKILL.md`
- Global agent rules: `AGENTS.md`
- Frontend scope rules: `src/AGENTS.md`
- AI skills setup/routing: `docs/ai-skills-usage.md`
- Rust scope rules: `src-tauri/AGENTS.md`
- Tauri framework constraints: `docs/tauri-framework-constraints.md`
- Deployment strategy: `docs/deployment-strategy.md`
- Progress tracking: `docs/development-progress.md`
