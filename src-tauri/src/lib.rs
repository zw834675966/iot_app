#![allow( // 允许以下 clippy 警告以匹配当前工程规范
    clippy::must_use_candidate, // 忽略必须标记 must_use 的建议
    clippy::missing_errors_doc, // 忽略缺少错误文档的警告
    clippy::missing_panics_doc, // 忽略缺少 panic 文档的警告
    clippy::doc_markdown, // 忽略文档 markdown 规范警告
    clippy::needless_pass_by_value // 忽略可传引用却按值传递的警告
)] // clippy allow 列表结束
pub mod auth; // 暴露认证相关模块
pub mod core; // 暴露核心基础设施模块
pub mod db; // 暴露业务数据库模块
pub mod notice; // 暴露通知中心模块

#[cfg_attr(mobile, tauri::mobile_entry_point)] // 移动端使用 Tauri 的入口属性
pub fn run() { // 应用启动入口函数
    let runtime_config = core::config::runtime_config(); // 读取运行时配置
    core::tracing::init_tracing(runtime_config) // 初始化 tracing 日志
        .unwrap_or_else(|err| panic!("initialize tracing failed: {err}")); // 初始化失败则终止启动

    tauri::Builder::default() // 创建默认 Tauri 构建器
        .setup(|_app| { // 配置应用启动前的初始化逻辑
            let database_url = runtime_config.database.url.clone(); // 复制数据库连接字符串
            tracing::info!("startup: configuring database url"); // 记录数据库配置日志
            db::set_database_url(database_url) // 设置数据库连接地址
                .map_err(|err| std::io::Error::other(format!("configure db url failed: {err}")))?; // 失败时转换错误
            tracing::info!("startup: initializing business database"); // 记录初始化业务库日志
            db::init_database() // 初始化业务数据库
                .map_err(|err| std::io::Error::other(format!("initialize db failed: {err}")))?; // 失败时转换错误
            tracing::info!("startup: running auth expiration compensation"); // 记录补偿任务日志
            auth::admin_services::run_startup_expiration_compensation(auth::services::now_millis()) // 运行过期补偿
                .map_err(|err| { // 将补偿错误包装为 IO 错误
                    std::io::Error::other(format!( // 构造错误消息
                        "run account expiration compensation failed: {err}" // 说明补偿失败原因
                    )) // 格式化错误消息结束
                })?; // 失败时直接返回错误

            tracing::info!("startup: initializing notice database"); // 记录通知库初始化日志
            notice::init_notice_database().map_err(|err| { // 初始化通知数据库并映射错误
                std::io::Error::other(format!("initialize notice db failed: {err}")) // 构造通知库错误
            })?; // 失败时直接返回错误
            Ok(()) // setup 结束并返回成功
        }) // setup 闭包结束
        .invoke_handler(tauri::generate_handler![ // 注册前端可调用的 Tauri 命令
            auth::commands::auth_login, // 登录命令
            auth::commands::auth_refresh_token, // 刷新 token 命令
            auth::commands::auth_get_async_routes, // 获取异步路由命令
            auth::admin_commands::auth_admin_register_user, // 管理员注册用户
            auth::admin_commands::auth_admin_renew_user_account, // 管理员续期账号
            auth::admin_commands::auth_admin_list_users, // 管理员列出用户
            auth::admin_commands::auth_admin_update_user, // 管理员更新用户
            auth::admin_commands::auth_admin_delete_user, // 管理员删除用户
            auth::admin_commands::auth_admin_change_user_password, // 管理员修改密码
            auth::admin_commands::user_device_scope_get, // 获取用户设备权限
            auth::admin_commands::user_device_scope_upsert, // 更新用户设备权限
            notice::commands::notice_get_unread_items, // 获取未读通知
            notice::commands::notice_get_read_items, // 获取已读通知
            notice::commands::notice_mark_read // 标记通知已读
        ]) // 命令注册结束
        .run(tauri::generate_context!()) // 运行 Tauri 应用
        .expect("Tauri runtime startup failed"); // 启动失败直接 panic
} // 入口函数结束
