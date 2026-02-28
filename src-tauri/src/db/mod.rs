//! SQLite database access entrypoints.
pub mod admin_repository;
pub mod auth_repository;

mod bootstrap;
mod migrations;
mod path_store;

use rusqlite::Connection;
use std::path::PathBuf;

use crate::core::error::AppError;

/// Configure database file path (usually called once in Tauri setup).
pub fn set_database_path(path: PathBuf) -> Result<(), AppError> {
    path_store::set_database_path(path)
}

/// Open SQLite connection.
pub fn connect() -> Result<Connection, AppError> {
    Connection::open(path_store::database_path()).map_err(|err| AppError::Database(err.to_string()))
}

/// Initialize database schema and seed data.
pub fn init_database() -> Result<(), AppError> {
    bootstrap::init_database()
}

#[cfg(test)]
mod tests;
