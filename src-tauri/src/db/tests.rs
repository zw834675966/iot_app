use std::sync::Once;

use rusqlite::Connection;

use super::migrations::{
    apply_one_time_data_fix, apply_user_registration_extension, data_fix_sql, init_schema,
    init_seed_data, schema_sql, seed_sql, user_registration_extension_sql, DATA_FIX_MIGRATION_ID,
    USER_REGISTRATION_MIGRATION_ID,
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
    assert!(schema.contains("CREATE TABLE IF NOT EXISTS users"));
    assert!(seed.contains("INSERT OR IGNORE INTO routes"));
    assert!(data_fix.contains("UPDATE users"));
    assert!(user_registration_extension.contains("ALTER TABLE users ADD COLUMN phone"));
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
    conn.execute("UPDATE routes SET meta_icon = 'ep:lollipop' WHERE id = 1", [])
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
        .query_row("SELECT meta_icon FROM routes WHERE id = 1", [], |row| row.get(0))
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
    assert_eq!(avatar_after_second_run, "https://example.com/second-run.png");
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
