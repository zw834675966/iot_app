use std::path::PathBuf;
use std::sync::OnceLock;

use crate::core::error::AppError;

const DB_FILE_NAME: &str = "pure-admin-thin.sqlite3";
static DB_PATH: OnceLock<PathBuf> = OnceLock::new();

#[allow(clippy::unnecessary_wraps)]
pub fn set_database_path(path: PathBuf) -> Result<(), AppError> {
    if DB_PATH.get().is_some() {
        return Ok(());
    }

    if DB_PATH.set(path).is_err() {
        return Ok(());
    }

    Ok(())
}

pub(crate) fn database_path() -> PathBuf {
    if let Some(path) = DB_PATH.get() {
        return path.clone();
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("db")
        .join(DB_FILE_NAME)
}
