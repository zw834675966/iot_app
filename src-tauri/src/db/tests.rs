use std::sync::Once;

use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use sqlx::{Connection as _, PgConnection, query, query_scalar};

use super::migrations::{
    DATA_FIX_MIGRATION_ID, HIDE_BUTTON_PERMISSION_ROUTE_MIGRATION_ID,
    PERMISSION_ROUTE_RENAME_MIGRATION_ID, USER_REGISTRATION_MIGRATION_ID,
    apply_hide_button_permission_route, apply_one_time_data_fix, apply_permission_route_rename,
    apply_user_registration_extension, data_fix_sql, hide_button_permission_route_sql, init_schema,
    init_seed_data, permission_route_rename_sql, schema_sql, seed_sql,
    user_registration_extension_sql,
};
use super::{connect, init_database, set_database_url};

struct IsolatedDb {
    conn: PgConnection,
    schema: String,
}

impl IsolatedDb {
    fn new() -> Self {
        let mut conn = super::block_on(PgConnection::connect(&super::test_database_url()))
            .expect("open postgres test db");

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let schema = format!("test_schema_{nanos}");

        let create_schema_sql = format!("CREATE SCHEMA {schema}");
        super::block_on(query(&create_schema_sql).execute(&mut conn)).expect("create test schema");

        let set_search_path_sql = format!("SET search_path TO {schema}");
        super::block_on(query(&set_search_path_sql).execute(&mut conn)).expect("set search_path");

        Self { conn, schema }
    }

    fn conn(&mut self) -> &mut PgConnection {
        &mut self.conn
    }
}

impl Drop for IsolatedDb {
    fn drop(&mut self) {
        let _ = super::block_on(query("SET search_path TO public").execute(&mut self.conn));

        let drop_schema_sql = format!("DROP SCHEMA IF EXISTS {} CASCADE", self.schema);
        let _ = super::block_on(query(&drop_schema_sql).execute(&mut self.conn));
    }
}

fn ensure_db_ready() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        set_database_url(super::test_database_url()).expect("configure database url");
        init_database().expect("init db");
    });
}

#[test]
fn initializes_device_registry_seed() {
    ensure_db_ready();
    let mut conn = connect().expect("open db");
    let count: i64 =
        super::block_on(query_scalar("SELECT COUNT(1) FROM device_registry").fetch_one(&mut conn))
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
    assert!(seed.contains("ON CONFLICT (id) DO NOTHING"));
    assert!(data_fix.contains("UPDATE users"));
    assert!(
        user_registration_extension.contains("ALTER TABLE users ADD COLUMN IF NOT EXISTS phone")
    );
    assert!(permission_route_rename.contains("UPDATE routes"));
    assert!(hide_button_permission_route.contains("DELETE FROM routes"));
}

#[test]
fn applies_legacy_data_fix_only_once() {
    let mut isolated = IsolatedDb::new();
    let conn = isolated.conn();

    super::block_on(init_schema(&mut *conn)).expect("init schema");
    super::block_on(init_seed_data(&mut *conn)).expect("init seed");

    super::block_on(
        query(
            "UPDATE users SET avatar = 'https://example.com/avatar.png' WHERE username = 'admin'",
        )
        .execute(&mut *conn),
    )
    .expect("seed legacy avatar");
    super::block_on(
        query("UPDATE routes SET meta_icon = 'ep:lollipop' WHERE id = 1").execute(&mut *conn),
    )
    .expect("seed legacy icon");

    super::block_on(apply_one_time_data_fix(&mut *conn)).expect("apply one-time fix");

    let avatar: String = super::block_on(
        query_scalar("SELECT avatar FROM users WHERE username = 'admin'").fetch_one(&mut *conn),
    )
    .expect("query avatar");
    let meta_icon: Option<String> = super::block_on(
        query_scalar("SELECT meta_icon FROM routes WHERE id = 1").fetch_one(&mut *conn),
    )
    .expect("query icon");
    let applied_count: i64 = super::block_on(
        query_scalar("SELECT COUNT(1) FROM app_migrations WHERE id = $1")
            .bind(DATA_FIX_MIGRATION_ID)
            .fetch_one(&mut *conn),
    )
    .expect("query migration count");

    assert_eq!(avatar, "");
    assert!(meta_icon.is_none());
    assert_eq!(applied_count, 1);

    super::block_on(
        query("UPDATE users SET avatar = 'https://example.com/second-run.png' WHERE username = 'admin'")
            .execute(&mut *conn),
    )
    .expect("inject value after migration mark");

    super::block_on(apply_one_time_data_fix(&mut *conn)).expect("skip second run");

    let avatar_after_second_run: String = super::block_on(
        query_scalar("SELECT avatar FROM users WHERE username = 'admin'").fetch_one(&mut *conn),
    )
    .expect("query avatar after second run");
    assert_eq!(
        avatar_after_second_run,
        "https://example.com/second-run.png"
    );
}

#[test]
fn applies_user_registration_extension_only_once() {
    let mut isolated = IsolatedDb::new();
    let conn = isolated.conn();

    super::block_on(init_schema(&mut *conn)).expect("init schema");
    super::block_on(init_seed_data(&mut *conn)).expect("init seed");

    super::block_on(apply_user_registration_extension(&mut *conn))
        .expect("apply user registration extension");

    let phone_column_count: i64 = super::block_on(
        query_scalar(
            r"
            SELECT COUNT(1)
            FROM information_schema.columns
            WHERE table_schema = current_schema()
              AND table_name = 'users'
              AND column_name = 'phone'
            ",
        )
        .fetch_one(&mut *conn),
    )
    .expect("query users table info");
    let migration_count: i64 = super::block_on(
        query_scalar("SELECT COUNT(1) FROM app_migrations WHERE id = $1")
            .bind(USER_REGISTRATION_MIGRATION_ID)
            .fetch_one(&mut *conn),
    )
    .expect("query migration count");

    assert_eq!(phone_column_count, 1);
    assert_eq!(migration_count, 1);

    super::block_on(apply_user_registration_extension(&mut *conn)).expect("skip second run");

    let migration_count_after_second_run: i64 = super::block_on(
        query_scalar("SELECT COUNT(1) FROM app_migrations WHERE id = $1")
            .bind(USER_REGISTRATION_MIGRATION_ID)
            .fetch_one(&mut *conn),
    )
    .expect("query migration count after second run");
    assert_eq!(migration_count_after_second_run, 1);
}

#[test]
fn applies_permission_route_rename_only_once() {
    let mut isolated = IsolatedDb::new();
    let conn = isolated.conn();

    super::block_on(init_schema(&mut *conn)).expect("init schema");
    super::block_on(init_seed_data(&mut *conn)).expect("init seed");

    super::block_on(
        query("UPDATE routes SET meta_title = 'legacy-title' WHERE id = 2").execute(&mut *conn),
    )
    .expect("set legacy route title");

    super::block_on(apply_permission_route_rename(&mut *conn)).expect("apply route title rename");

    let title_after_first_run: String = super::block_on(
        query_scalar("SELECT meta_title FROM routes WHERE id = 2").fetch_one(&mut *conn),
    )
    .expect("query route title");
    let migration_count: i64 = super::block_on(
        query_scalar("SELECT COUNT(1) FROM app_migrations WHERE id = $1")
            .bind(PERMISSION_ROUTE_RENAME_MIGRATION_ID)
            .fetch_one(&mut *conn),
    )
    .expect("query migration count");

    assert_ne!(title_after_first_run, "legacy-title");
    assert_eq!(migration_count, 1);

    super::block_on(
        query("UPDATE routes SET meta_title = 'legacy-title' WHERE id = 2").execute(&mut *conn),
    )
    .expect("reset route title after migration mark");

    super::block_on(apply_permission_route_rename(&mut *conn)).expect("skip second run");

    let title_after_second_run: String = super::block_on(
        query_scalar("SELECT meta_title FROM routes WHERE id = 2").fetch_one(&mut *conn),
    )
    .expect("query route title after second run");
    assert_eq!(title_after_second_run, "legacy-title");
}

#[test]
fn hides_button_permission_routes() {
    let mut isolated = IsolatedDb::new();
    let conn = isolated.conn();

    super::block_on(init_schema(&mut *conn)).expect("init schema");
    super::block_on(init_seed_data(&mut *conn)).expect("init seed");
    super::block_on(apply_one_time_data_fix(&mut *conn)).expect("apply data fix");
    super::block_on(apply_user_registration_extension(&mut *conn))
        .expect("apply user registration extension");
    super::block_on(apply_permission_route_rename(&mut *conn))
        .expect("apply permission route rename");
    super::block_on(apply_hide_button_permission_route(&mut *conn))
        .expect("apply hide button route migration");

    let route_count: i64 = super::block_on(
        query_scalar(
            r"
            SELECT COUNT(1)
            FROM routes
            WHERE path = '/permission/button'
               OR path LIKE '/permission/button/%'
            ",
        )
        .fetch_one(&mut *conn),
    )
    .expect("query button route count");

    let migration_count: i64 = super::block_on(
        query_scalar("SELECT COUNT(1) FROM app_migrations WHERE id = $1")
            .bind(HIDE_BUTTON_PERMISSION_ROUTE_MIGRATION_ID)
            .fetch_one(&mut *conn),
    )
    .expect("query hide button route migration count");

    assert_eq!(route_count, 0);
    assert_eq!(migration_count, 1);
}

#[test]
fn opens_seaorm_connection_for_postgres() {
    ensure_db_ready();

    let connection = super::block_on(super::connect_orm_async()).expect("open seaorm connection");
    let statement = Statement::from_string(
        DatabaseBackend::Postgres,
        "SELECT current_database()".to_string(),
    );
    let row = super::block_on(connection.query_one(statement))
        .expect("run seaorm query")
        .expect("database row should exist");

    let current_database: String = row.try_get_by_index(0).expect("decode database name");
    assert!(!current_database.trim().is_empty());
}
