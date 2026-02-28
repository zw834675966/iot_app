#![allow(
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::needless_pass_by_value
)]
pub mod auth;
pub mod core;
pub mod db;
pub mod notice;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let database_url = std::env::var("PURE_ADMIN_DATABASE_URL")
                .ok()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(db::database_url);
            db::set_database_url(database_url)
                .map_err(|err| std::io::Error::other(format!("configure db url failed: {err}")))?;
            db::init_database()
                .map_err(|err| std::io::Error::other(format!("initialize db failed: {err}")))?;
            auth::admin_services::run_startup_expiration_compensation(auth::services::now_millis())
                .map_err(|err| {
                    std::io::Error::other(format!(
                        "run account expiration compensation failed: {err}"
                    ))
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
