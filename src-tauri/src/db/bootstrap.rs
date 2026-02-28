//! # 数据库启动与引导模块
//!
//! 本模块负责在应用启动时自动建立数据库连接，并执行包括建表、填充初始数据、以及各类历史迁移脚本。
//! 确保在应用完全启动前，SQLite 数据库处于最新且可用的状态。

use std::fs;

use crate::core::error::AppError;

use super::{connect, migrations, path_store};

/// 数据库全局初始化入口
///
/// 该函数会在 Tauri 启动的 `setup` 阶段被调用。主要流程如下：
/// 1. 获取 SQLite 数据库文件存放路径
/// 2. 若上级目录不存在，则自动创建该目录
/// 3. 打开到该 SQLite 数据库的连接
/// 4. 依次执行表结构初始化、种子数据填充、以及多个渐进式的版本迁移脚本
pub(crate) fn init_database() -> Result<(), AppError> {
    let db_path = path_store::database_path();
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|err| AppError::Database(err.to_string()))?;
    }

    let connection = connect()?;

    // 初始化空数据库表结构
    migrations::init_schema(&connection)?;
    // 初始化系统必须的默认数据（如超级管理员、默认路由等）
    migrations::init_seed_data(&connection)?;

    // 逐版本应用增量迁移脚本，通过 app_migrations 表记录规避重复执行
    migrations::apply_one_time_data_fix(&connection)?;
    migrations::apply_user_registration_extension(&connection)?;
    migrations::apply_permission_route_rename(&connection)?;
    migrations::apply_hide_button_permission_route(&connection)?;

    Ok(())
}
