# pure-admin-thin 项目介绍（本地单程序 / Tauri 优先）

## 1. 文档目标

本文用于说明当前仓库在“本地单程序运行”场景下的架构与使用方式，面向：

- 新加入项目、需要快速建立全局认知的开发者
- 负责本地交付与二次开发的工程师
- 需要扩展 Tauri 桌面能力的开发者

## 2. 项目定位与当前状态

`pure-admin-thin` 基于 `vue-pure-admin` 精简版，技术栈为 Vue 3 + Vite + TypeScript + Element Plus + Pinia + Vue Router，并已接入 `Tauri v2`。

当前仓库已按本地部署场景做了工程收敛：

- 保留登录、权限、动态路由、多标签与布局系统
- 保留 Web 与 Tauri 两套运行入口，但以 Tauri 本地运行为主
- 移除 Mock、staging、CDN/压缩构建插件、Docker 与 Git hooks 工具链

## 3. 技术栈总览

### 3.1 前端

- 框架：`Vue 3.5`
- 构建工具：`Vite 7`
- 语言：`TypeScript 5`
- UI：`Element Plus`
- 状态管理：`Pinia`
- 路由：`Vue Router 4`
- 样式：`SCSS + TailwindCSS`
- 通信：`@tauri-apps/api`（本地命令）+ Axios（通用 HTTP 工具仍保留）
- 图标：`Iconify + unplugin-icons + iconfont`

### 3.2 桌面端

- 框架：`Tauri v2`
- Rust 版本约束：`rust-version = "1.85"`
- 关键 crate：`tauri`、`tauri-build`、`tauri-plugin-log`

## 4. 目录结构与职责

```text
pure-admin-thin/
├─ build/                 # Vite 构建配置（精简后）
├─ public/                # 静态资源（含 platform-config.json）
├─ src/                   # 前端业务源码
│  ├─ api/                # API 抽象层（含 Tauri invoke 适配）
│  ├─ router/             # 静态路由、守卫、动态路由转换
│  ├─ store/              # Pinia 状态模块
│  ├─ utils/              # auth/http/message 等工具
│  └─ views/              # 页面视图
├─ src-tauri/             # Tauri Rust 端工程
│  ├─ src/lib.rs          # Tauri 命令与应用装配
│  └─ tauri.conf.json     # Tauri 应用配置
├─ .env*                  # 环境变量（当前仅 dev/production）
├─ package.json           # 前端与 Tauri 脚本入口
└─ vite.config.ts         # Vite 主配置
```

## 5. 启动与构建

### 5.1 环境要求

- `node`: `^20.19.0 || >=22.13.0`
- `pnpm`: `>=9`
- `Rust toolchain`

### 5.2 常用命令

| 场景           | 命令               | 说明                          |
| -------------- | ------------------ | ----------------------------- |
| 安装依赖       | `pnpm install`     | 安装前端依赖                  |
| Web 开发       | `pnpm dev`         | 启动 Vite 开发服务            |
| Web 构建       | `pnpm build`       | 生成前端产物                  |
| Tauri 前端构建 | `pnpm build:tauri` | `--base ./` 适配桌面资源加载  |
| 桌面开发       | `pnpm tauri:dev`   | 启动桌面开发模式              |
| 桌面构建       | `pnpm tauri:build` | 生成桌面安装包/可执行产物     |
| 类型检查       | `pnpm typecheck`   | `tsc + vue-tsc`               |
| 代码检查       | `pnpm lint`        | eslint + prettier + stylelint |

## 6. 关键架构说明

### 6.1 应用入口

`src/main.ts` 负责：

1. 创建 Vue 应用
2. 注册全局指令与组件
3. 加载 `platform-config.json`
4. 挂载 Pinia 与 Router
5. 安装插件并挂载

### 6.2 路由系统（静态 + 动态）

- 静态路由来自 `src/router/modules/**/*.ts`
- 动态路由由 `getAsyncRoutes()` 拉取，再经过 `addAsyncRoutes` 转换
- 动态路由完成后补 `/:pathMatch(.*)*`，避免刷新误判 404

### 6.3 认证与权限

- 登录、刷新 token、动态路由使用 Tauri 命令：
  - `auth_login`
  - `auth_refresh_token`
  - `auth_get_async_routes`
- 前端通过 `@tauri-apps/api/core` 的 `invoke` 调用
- 权限分为：
  - 页面级：`meta.roles`
  - 按钮级：`meta.auths` / `permissions`

### 6.4 状态管理

核心 Pinia 模块：

- `user`：用户资料、角色、权限、登录状态
- `permission`：菜单与路由缓存态
- `multiTags`：多标签状态
- `settings` / `app` / `epTheme`：布局和主题

### 6.5 配置体系

环境变量（当前有效项）：

- `VITE_PORT`
- `VITE_PUBLIC_PATH`
- `VITE_ROUTER_HISTORY`
- `VITE_HIDE_HOME`

运行时配置：`public/platform-config.json`

## 7. Tauri 侧说明

关键文件：

- `src-tauri/src/main.rs`：入口
- `src-tauri/src/lib.rs`：命令与 Builder 装配
- `src-tauri/Cargo.toml`：依赖与 lint 约束

当前 Rust 侧已提供本地认证与动态路由命令，满足“本地单程序运行”需求。

## 8. 质量与验证建议

建议本地改动后至少执行：

1. `pnpm typecheck`
2. `pnpm lint`
3. `cargo test --manifest-path src-tauri/Cargo.toml`
4. `pnpm tauri:dev`（验证完整登录与路由流程）

## 9. 快速上手

1. `pnpm install`
2. `pnpm tauri:dev`
3. 使用默认账号（如 `admin`）走一遍登录与权限页面
4. 阅读以下核心文件：
   - `src/router/index.ts`
   - `src/router/utils.ts`
   - `src/api/user.ts`
   - `src/api/routes.ts`
   - `src-tauri/src/lib.rs`
