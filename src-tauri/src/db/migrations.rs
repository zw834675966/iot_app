use rusqlite::{Connection, OptionalExtension};

use crate::core::error::AppError;

pub(crate) const DATA_FIX_MIGRATION_ID: &str = "0003_legacy_offline_cleanup";
pub(crate) const USER_REGISTRATION_MIGRATION_ID: &str = "0004_user_registration_extension";

pub(crate) fn init_schema(connection: &Connection) -> Result<(), AppError> {
    connection
        .execute_batch(schema_sql())
        .map_err(|err| AppError::Database(err.to_string()))
}

pub(crate) fn init_seed_data(connection: &Connection) -> Result<(), AppError> {
    connection
        .execute_batch(seed_sql())
        .map_err(|err| AppError::Database(err.to_string()))
}

pub(crate) fn apply_one_time_data_fix(connection: &Connection) -> Result<(), AppError> {
    ensure_migration_log_table(connection)?;
    if is_data_fix_applied(connection)? {
        return Ok(());
    }

    connection
        .execute_batch(data_fix_sql())
        .map_err(|err| AppError::Database(err.to_string()))?;

    connection
        .execute(
            r"
            INSERT INTO app_migrations (id, applied_at)
            VALUES (?1, CAST(strftime('%s', 'now') AS INTEGER))
            ",
            [DATA_FIX_MIGRATION_ID],
        )
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

pub(crate) fn apply_user_registration_extension(connection: &Connection) -> Result<(), AppError> {
    ensure_migration_log_table(connection)?;
    if is_user_registration_extension_applied(connection)? {
        return Ok(());
    }

    connection
        .execute_batch(user_registration_extension_sql())
        .map_err(|err| AppError::Database(err.to_string()))?;

    connection
        .execute(
            r"
            INSERT INTO app_migrations (id, applied_at)
            VALUES (?1, CAST(strftime('%s', 'now') AS INTEGER))
            ",
            [USER_REGISTRATION_MIGRATION_ID],
        )
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

fn ensure_migration_log_table(connection: &Connection) -> Result<(), AppError> {
    connection
        .execute_batch(
            r"
            CREATE TABLE IF NOT EXISTS app_migrations (
              id TEXT PRIMARY KEY,
              applied_at INTEGER NOT NULL
            );
            ",
        )
        .map_err(|err| AppError::Database(err.to_string()))
}

fn is_data_fix_applied(connection: &Connection) -> Result<bool, AppError> {
    connection
        .query_row(
            "SELECT 1 FROM app_migrations WHERE id = ?1 LIMIT 1",
            [DATA_FIX_MIGRATION_ID],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map(|row| row.is_some())
        .map_err(|err| AppError::Database(err.to_string()))
}

fn is_user_registration_extension_applied(connection: &Connection) -> Result<bool, AppError> {
    connection
        .query_row(
            "SELECT 1 FROM app_migrations WHERE id = ?1 LIMIT 1",
            [USER_REGISTRATION_MIGRATION_ID],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map(|row| row.is_some())
        .map_err(|err| AppError::Database(err.to_string()))
}

pub(crate) fn schema_sql() -> &'static str {
    include_str!("migrations/0001_schema.sql")
}

pub(crate) fn seed_sql() -> &'static str {
    include_str!("migrations/0002_seed.sql")
}

pub(crate) fn data_fix_sql() -> &'static str {
    include_str!("migrations/0003_legacy_offline_cleanup.sql")
}

pub(crate) fn user_registration_extension_sql() -> &'static str {
    include_str!("migrations/0004_user_registration_extension.sql")
}
