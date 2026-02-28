use sqlx::{PgConnection, query, query_scalar, raw_sql};

use crate::core::error::AppError;

pub(crate) const DATA_FIX_MIGRATION_ID: &str = "0003_legacy_offline_cleanup";
pub(crate) const USER_REGISTRATION_MIGRATION_ID: &str = "0004_user_registration_extension";
pub(crate) const PERMISSION_ROUTE_RENAME_MIGRATION_ID: &str =
    "0005_permission_page_to_user_registration";
pub(crate) const HIDE_BUTTON_PERMISSION_ROUTE_MIGRATION_ID: &str =
    "0006_hide_button_permission_route";

pub(crate) async fn init_schema(connection: &mut PgConnection) -> Result<(), AppError> {
    raw_sql(schema_sql())
        .execute(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(())
}

pub(crate) async fn init_seed_data(connection: &mut PgConnection) -> Result<(), AppError> {
    raw_sql(seed_sql())
        .execute(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(())
}

pub(crate) async fn apply_one_time_data_fix(connection: &mut PgConnection) -> Result<(), AppError> {
    ensure_migration_log_table(connection).await?;
    if is_data_fix_applied(connection).await? {
        return Ok(());
    }

    raw_sql(data_fix_sql())
        .execute(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    query(
        r"
        INSERT INTO app_migrations (id, applied_at)
        VALUES ($1, EXTRACT(EPOCH FROM NOW())::BIGINT)
        ",
    )
    .bind(DATA_FIX_MIGRATION_ID)
    .execute(&mut *connection)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

pub(crate) async fn apply_user_registration_extension(
    connection: &mut PgConnection,
) -> Result<(), AppError> {
    ensure_migration_log_table(connection).await?;
    if is_user_registration_extension_applied(connection).await? {
        return Ok(());
    }

    raw_sql(user_registration_extension_sql())
        .execute(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    query(
        r"
        INSERT INTO app_migrations (id, applied_at)
        VALUES ($1, EXTRACT(EPOCH FROM NOW())::BIGINT)
        ",
    )
    .bind(USER_REGISTRATION_MIGRATION_ID)
    .execute(&mut *connection)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

pub(crate) async fn apply_permission_route_rename(
    connection: &mut PgConnection,
) -> Result<(), AppError> {
    ensure_migration_log_table(connection).await?;
    if is_permission_route_rename_applied(connection).await? {
        return Ok(());
    }

    raw_sql(permission_route_rename_sql())
        .execute(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    query(
        r"
        INSERT INTO app_migrations (id, applied_at)
        VALUES ($1, EXTRACT(EPOCH FROM NOW())::BIGINT)
        ",
    )
    .bind(PERMISSION_ROUTE_RENAME_MIGRATION_ID)
    .execute(&mut *connection)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

pub(crate) async fn apply_hide_button_permission_route(
    connection: &mut PgConnection,
) -> Result<(), AppError> {
    ensure_migration_log_table(connection).await?;
    if is_hide_button_permission_route_applied(connection).await? {
        return Ok(());
    }

    raw_sql(hide_button_permission_route_sql())
        .execute(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    query(
        r"
        INSERT INTO app_migrations (id, applied_at)
        VALUES ($1, EXTRACT(EPOCH FROM NOW())::BIGINT)
        ",
    )
    .bind(HIDE_BUTTON_PERMISSION_ROUTE_MIGRATION_ID)
    .execute(&mut *connection)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

async fn ensure_migration_log_table(connection: &mut PgConnection) -> Result<(), AppError> {
    query(
        r"
        CREATE TABLE IF NOT EXISTS app_migrations (
          id TEXT PRIMARY KEY,
          applied_at BIGINT NOT NULL
        );
        ",
    )
    .execute(&mut *connection)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(())
}

async fn is_data_fix_applied(connection: &mut PgConnection) -> Result<bool, AppError> {
    let row = query_scalar::<_, i32>("SELECT 1 FROM app_migrations WHERE id = $1 LIMIT 1")
        .bind(DATA_FIX_MIGRATION_ID)
        .fetch_optional(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(row.is_some())
}

async fn is_user_registration_extension_applied(
    connection: &mut PgConnection,
) -> Result<bool, AppError> {
    let row = query_scalar::<_, i32>("SELECT 1 FROM app_migrations WHERE id = $1 LIMIT 1")
        .bind(USER_REGISTRATION_MIGRATION_ID)
        .fetch_optional(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(row.is_some())
}

async fn is_permission_route_rename_applied(
    connection: &mut PgConnection,
) -> Result<bool, AppError> {
    let row = query_scalar::<_, i32>("SELECT 1 FROM app_migrations WHERE id = $1 LIMIT 1")
        .bind(PERMISSION_ROUTE_RENAME_MIGRATION_ID)
        .fetch_optional(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(row.is_some())
}

async fn is_hide_button_permission_route_applied(
    connection: &mut PgConnection,
) -> Result<bool, AppError> {
    let row = query_scalar::<_, i32>("SELECT 1 FROM app_migrations WHERE id = $1 LIMIT 1")
        .bind(HIDE_BUTTON_PERMISSION_ROUTE_MIGRATION_ID)
        .fetch_optional(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(row.is_some())
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

pub(crate) fn permission_route_rename_sql() -> &'static str {
    include_str!("migrations/0005_permission_page_to_user_registration.sql")
}

pub(crate) fn hide_button_permission_route_sql() -> &'static str {
    include_str!("migrations/0006_hide_button_permission_route.sql")
}
