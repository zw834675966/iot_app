//! ==========================================================================================
//! 鉴权模块 (Auth)
//!
//! 模块职责：
//! 本模块是整个能源管理系统的鉴权核心，负责处理用户认证、权限校验和动态路由获取。
//! 采用领域驱动设计（DDD）思想，将接口层、业务层、模型层完全解耦，确保高可测试性和可维护性。
//!
//! 模块结构：
//!
//! ```text
//! auth/
//! ├── mod.rs              # 模块声明和导出
//! ├── commands.rs         # IPC 接口层（Adapter Layer）- 前端调用入口
//! ├── services.rs         # 业务逻辑层（Domain Layer）- 核心业务规则
//! ├── models.rs           # 数据模型层（DTO）- 数据传输对象
//! ├── admin_commands.rs   # 管理员 IPC 接口层
//! ├── admin_services.rs   # 管理员业务逻辑层
//! └── README.md           # 模块文档
//! ```
//!
//! 设计目标：
//!
//! 1. 离线安全：无需外部网络依赖，所有数据存储在本地数据库
//! 2. 无状态认证：使用 JWT 令牌实现无状态的身份验证
//! 3. 解耦设计：业务逻辑完全独立于 Tauri 框架，可复用
//! 4. 类型安全：使用 Rust 强类型系统确保数据安全
//!
//! 各层职责：
//!
//! | 文件 | 层级 | 职责 | 特点 |
//! |------|------|------|------|
//! | `commands.rs` | Adapter Layer | 参数校验、结果封装、IPC 路由 | 薄层适配，仅做转发 |
//! | `admin_commands.rs` | Adapter Layer | 管理员命令处理 | 薄层适配 |
//! | `services.rs` | Domain Layer | 业务规则、令牌管理、数据库查询 | 纯函数，无框架依赖 |
//! | `admin_services.rs` | Domain Layer | 管理员业务规则 | 纯函数 |
//! | `models.rs` | DTO Layer | 数据结构定义、序列化配置 | 仅包含数据字段 |
//!
//! 核心功能：
//!
//! - 用户登录 (`auth_login`)
//! - 令牌刷新 (`auth_refresh_token`)
//! - 获取动态路由 (`auth_get_async_routes`)
//! - 管理员注册用户 (`auth_admin_register_user`)
//! - 管理员续期用户账号 (`auth_admin_renew_user_account`)
//! - 管理员列出用户 (`auth_admin_list_users`)
//! - 管理员更新用户 (`auth_admin_update_user`)
//! - 管理员删除用户 (`auth_admin_delete_user`)
//! - 管理员修改密码 (`auth_admin_change_user_password`)
//!
//! ==========================================================================================

// 声明并导出管理员命令模块
pub mod admin_commands;
// 声明并导出管理员服务模块
pub mod admin_services;
// 声明并导出命令模块
pub mod commands;
// 声明并导出模型模块
pub mod models;
// 声明并导出服务模块
pub mod services;
