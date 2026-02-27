//! # pure-admin-thin Tauri 后端库
//!
//! 本 crate 是 pure-admin-thin 桌面应用的 Rust 后端入口，负责：
//! - 启动 Tauri 运行时并注册前端可调用的命令处理器
//! - 在开发模式下挂载日志插件以便调试
//!
//! ## 模块结构
//!
//! | 模块 | 职责 |
//! |------|------|
//! | [`auth`] | 鉴权领域逻辑（登录、权限、路由配置） |
//! | [`core`] | 核心基础设施（统一错误类型与响应封装等） |

pub mod auth;
pub mod core;

/// 启动 Tauri 运行时并注册所有命令处理器。
///
/// 此函数是整个桌面应用的核心入口，由 [`main`](../main/fn.main.html) 调用。
/// 在移动端编译时会通过 `#[tauri::mobile_entry_point]` 标记为移动入口。
///
/// ## 初始化流程
///
/// 1. 创建默认的 `tauri::Builder`
/// 2. 在 `setup` 闭包中，仅当 **debug 模式** 时挂载日志插件（`tauri-plugin-log`）
/// 3. 通过 `invoke_handler` 注册所有前端可调用的 IPC 命令
/// 4. 调用 `run()` 启动事件循环
///
/// # Panics
///
/// 当 Tauri 运行时初始化失败时会触发 panic（例如缺少 `tauri.conf.json`）。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // 仅在开发模式下启用日志插件，避免 Release 包体积膨胀
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        // 注册前端可调用的 IPC 命令
        .invoke_handler(tauri::generate_handler![
            auth::commands::auth_login,            // 用户登录
            auth::commands::auth_refresh_token,    // 刷新令牌
            auth::commands::auth_get_async_routes  // 获取动态路由
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 运行时启动失败");
}
