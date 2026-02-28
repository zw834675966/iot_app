use std::sync::Once;

use rusqlite::Connection;

use super::migrations::{
    DATA_FIX_MIGRATION_ID, HIDE_BUTTON_PERMISSION_ROUTE_MIGRATION_ID,
    PERMISSION_ROUTE_RENAME_MIGRATION_ID, USER_REGISTRATION_MIGRATION_ID,
    apply_hide_button_permission_route, apply_one_time_data_fix, apply_permission_route_rename,
    apply_user_registration_extension, data_fix_sql, hide_button_permission_route_sql, init_schema,
    init_seed_data, permission_route_rename_sql, schema_sql, seed_sql,
    user_registration_extension_sql,
};
use super::{connect, init_database, set_database_path};

fn ensure_db_ready() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let test_db = std::env::temp_dir().join("pure-admin-thin-auth-tests.sqlite3");
        let _ = std::fs::remove_file(&test_db);
        set_database_path(test_db).expect("configure database path");
        init_database().expect("init db");
    });
}

#[test]
fn initializes_device_registry_seed() {
    ensure_db_ready();
    let conn = connect().expect("open db");
    let count: i64 = conn
        .query_row("SELECT COUNT(1) FROM device_registry", [], |row| row.get(0))
        .expect("query count");
    assert!(count >= 1);
}

#[test]
fn loads_sql_scripts_from_files() {
    let schema = schema_sql();
    let seed = seed_sql();
    let data_fix = data_fix_sql();
    let user_registration_extension = user_registration_extension_sql();
    let permission_route_rename = permission_route_rename_sql();
    let hide_button_permission_route = hide_button_permission_route_sql();
    assert!(schema.contains("CREATE TABLE IF NOT EXISTS users"));
    assert!(seed.contains("INSERT OR IGNORE INTO routes"));
    assert!(data_fix.contains("UPDATE users"));
    assert!(user_registration_extension.contains("ALTER TABLE users ADD COLUMN phone"));
    assert!(permission_route_rename.contains("UPDATE routes"));
    assert!(hide_button_permission_route.contains("DELETE FROM routes"));
}

#[test]
fn applies_legacy_data_fix_only_once() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    init_schema(&conn).expect("init schema");
    init_seed_data(&conn).expect("init seed");

    conn.execute(
        "UPDATE users SET avatar = 'https://example.com/avatar.png' WHERE username = 'admin'",
        [],
    )
    .expect("seed legacy avatar");
    conn.execute(
        "UPDATE routes SET meta_icon = 'ep:lollipop' WHERE id = 1",
        [],
    )
    .expect("seed legacy icon");

    apply_one_time_data_fix(&conn).expect("apply one-time fix");

    let avatar: String = conn
        .query_row(
            "SELECT avatar FROM users WHERE username = 'admin'",
            [],
            |row| row.get(0),
        )
        .expect("query avatar");
    let meta_icon: Option<String> = conn
        .query_row("SELECT meta_icon FROM routes WHERE id = 1", [], |row| {
            row.get(0)
        })
        .expect("query icon");
    let applied_count: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM app_migrations WHERE id = ?1",
            [DATA_FIX_MIGRATION_ID],
            |row| row.get(0),
        )
        .expect("query migration count");

    assert_eq!(avatar, "");
    assert!(meta_icon.is_none());
    assert_eq!(applied_count, 1);

    conn.execute(
        "UPDATE users SET avatar = 'https://example.com/second-run.png' WHERE username = 'admin'",
        [],
    )
    .expect("inject value after migration mark");

    apply_one_time_data_fix(&conn).expect("skip second run");

    let avatar_after_second_run: String = conn
        .query_row(
            "SELECT avatar FROM users WHERE username = 'admin'",
            [],
            |row| row.get(0),
        )
        .expect("query avatar after second run");
    assert_eq!(
        avatar_after_second_run,
        "https://example.com/second-run.png"
    );
}

#[test]
fn applies_user_registration_extension_only_once() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    init_schema(&conn).expect("init schema");
    init_seed_data(&conn).expect("init seed");

    apply_user_registration_extension(&conn).expect("apply user registration extension");

    let phone_column_count: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM pragma_table_info('users') WHERE name = 'phone'",
            [],
            |row| row.get(0),
        )
        .expect("query users table info");
    let migration_count: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM app_migrations WHERE id = ?1",
            [USER_REGISTRATION_MIGRATION_ID],
            |row| row.get(0),
        )
        .expect("query migration count");

    assert_eq!(phone_column_count, 1);
    assert_eq!(migration_count, 1);

    apply_user_registration_extension(&conn).expect("skip second run");

    let migration_count_after_second_run: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM app_migrations WHERE id = ?1",
            [USER_REGISTRATION_MIGRATION_ID],
            |row| row.get(0),
        )
        .expect("query migration count after second run");
    assert_eq!(migration_count_after_second_run, 1);
}

#[test]
fn applies_permission_route_rename_only_once() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    init_schema(&conn).expect("init schema");
    init_seed_data(&conn).expect("init seed");

    conn.execute("UPDATE routes SET meta_title = '页面权限' WHERE id = 2", [])
        .expect("set legacy route title");

    apply_permission_route_rename(&conn).expect("apply route title rename");

    let title_after_first_run: String = conn
        .query_row("SELECT meta_title FROM routes WHERE id = 2", [], |row| {
            row.get(0)
        })
        .expect("query route title");
    let migration_count: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM app_migrations WHERE id = ?1",
            [PERMISSION_ROUTE_RENAME_MIGRATION_ID],
            |row| row.get(0),
        )
        .expect("query migration count");
    assert_eq!(title_after_first_run, "用户注册管理");
    assert_eq!(migration_count, 1);

    conn.execute("UPDATE routes SET meta_title = '页面权限' WHERE id = 2", [])
        .expect("reset route title after migration mark");

    apply_permission_route_rename(&conn).expect("skip second run");

    let title_after_second_run: String = conn
        .query_row("SELECT meta_title FROM routes WHERE id = 2", [], |row| {
            row.get(0)
        })
        .expect("query route title after second run");
    assert_eq!(title_after_second_run, "页面权限");
}

#[test]
fn hides_button_permission_routes() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    init_schema(&conn).expect("init schema");
    init_seed_data(&conn).expect("init seed");
    apply_one_time_data_fix(&conn).expect("apply data fix");
    apply_user_registration_extension(&conn).expect("apply user registration extension");
    apply_permission_route_rename(&conn).expect("apply permission route rename");
    apply_hide_button_permission_route(&conn).expect("apply hide button route migration");

    let route_count: i64 = conn
        .query_row(
            r"
            SELECT COUNT(1)
            FROM routes
            WHERE path = '/permission/button'
               OR path LIKE '/permission/button/%'
            ",
            [],
            |row| row.get(0),
        )
        .expect("query button route count");

    let migration_count: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM app_migrations WHERE id = ?1",
            [HIDE_BUTTON_PERMISSION_ROUTE_MIGRATION_ID],
            |row| row.get(0),
        )
        .expect("query hide button route migration count");

    assert_eq!(route_count, 0);
    assert_eq!(migration_count, 1);
}
