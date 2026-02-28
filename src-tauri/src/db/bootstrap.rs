use std::fs;

use crate::core::error::AppError;

use super::{connect, migrations, path_store};

pub(crate) fn init_database() -> Result<(), AppError> {
    let db_path = path_store::database_path();
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|err| AppError::Database(err.to_string()))?;
    }

    let connection = connect()?;
    migrations::init_schema(&connection)?;
    migrations::init_seed_data(&connection)?;
    migrations::apply_one_time_data_fix(&connection)?;
    migrations::apply_user_registration_extension(&connection)?;

    Ok(())
}
