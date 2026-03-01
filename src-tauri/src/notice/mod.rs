//! 通知模块入口
//! 
//! 本模块提供消息通知中心的数据访问和 IPC 命令接口
//! 使用 PostgreSQL 数据库存储通知数据

// 公开命令模块 - 暴露给前端调用的 Tauri 命令
pub mod commands;
// 私有模块 - 内部实现
mod models;    // 数据模型定义
mod repository; // 数据仓储层
mod services;  // 业务逻辑层

// 公开初始化函数 - 用于启动时初始化通知数据库
pub use repository::init_notice_database;
