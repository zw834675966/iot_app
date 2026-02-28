use std::sync::OnceLock;

use crate::core::error::AppError;

const DEFAULT_DATABASE_URL: &str =
    "postgres://postgres:EMSzw%4018627652962@127.0.0.1:5432/pure_admin_thin";
static DB_URL: OnceLock<String> = OnceLock::new();

#[allow(clippy::unnecessary_wraps)]
pub fn set_database_url(url: String) -> Result<(), AppError> {
    if DB_URL.get().is_some() {
        return Ok(());
    }

    if DB_URL.set(url).is_err() {
        return Ok(());
    }

    Ok(())
}

pub(crate) fn database_url() -> String {
    if let Some(url) = DB_URL.get() {
        return url.clone();
    }

    if let Ok(url) = std::env::var("PURE_ADMIN_DATABASE_URL") {
        if !url.trim().is_empty() {
            return url;
        }
    }

    DEFAULT_DATABASE_URL.to_string()
}
