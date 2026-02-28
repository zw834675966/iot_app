pub mod auth;
pub mod core;
pub mod db;
pub mod notice;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let db_dir = app.path().app_data_dir().map_err(|err| {
                std::io::Error::other(format!("resolve app data dir failed: {err}"))
            })?;

            let sqlite_path = db_dir.join("db").join("pure-admin-thin.sqlite3");
            db::set_database_path(sqlite_path)
                .map_err(|err| std::io::Error::other(format!("configure db path failed: {err}")))?;
            db::init_database()
                .map_err(|err| std::io::Error::other(format!("initialize db failed: {err}")))?;
            auth::admin_services::run_startup_expiration_compensation(auth::services::now_millis())
                .map_err(|err| {
                    std::io::Error::other(format!(
                        "run account expiration compensation failed: {err}"
                    ))
                })?;

            let notice_db_path = db_dir.join("db").join("pure-admin-thin-notice.redb");
            notice::set_notice_database_path(notice_db_path).map_err(|err| {
                std::io::Error::other(format!("configure notice db path failed: {err}"))
            })?;
            notice::init_notice_database().map_err(|err| {
                std::io::Error::other(format!("initialize notice db failed: {err}"))
            })?;

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth::commands::auth_login,
            auth::commands::auth_refresh_token,
            auth::commands::auth_get_async_routes,
            auth::admin_commands::auth_admin_register_user,
            auth::admin_commands::auth_admin_renew_user_account,
            auth::admin_commands::auth_admin_list_users,
            auth::admin_commands::auth_admin_update_user,
            auth::admin_commands::auth_admin_delete_user,
            auth::admin_commands::auth_admin_change_user_password,
            auth::admin_commands::user_device_scope_get,
            auth::admin_commands::user_device_scope_upsert,
            notice::commands::notice_get_unread_items,
            notice::commands::notice_get_read_items,
            notice::commands::notice_mark_read
        ])
        .run(tauri::generate_context!())
        .expect("Tauri runtime startup failed");
}
