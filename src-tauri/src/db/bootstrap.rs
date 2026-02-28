use sqlx::query;

use crate::core::error::AppError;

use super::{connect_async, migrations};

pub(crate) async fn init_database() -> Result<(), AppError> {
    let mut connection = connect_async().await?;
    let advisory_lock_key: i64 = 2026022801;

    query("SELECT pg_advisory_lock($1)")
        .bind(advisory_lock_key)
        .execute(&mut connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let init_result = async {
        migrations::init_schema(&mut connection).await?;
        migrations::init_seed_data(&mut connection).await?;

        migrations::apply_one_time_data_fix(&mut connection).await?;
        migrations::apply_user_registration_extension(&mut connection).await?;
        migrations::apply_permission_route_rename(&mut connection).await?;
        migrations::apply_hide_button_permission_route(&mut connection).await?;

        Ok::<(), AppError>(())
    }
    .await;

    let _ = query("SELECT pg_advisory_unlock($1)")
        .bind(advisory_lock_key)
        .execute(&mut connection)
        .await;

    init_result
}
