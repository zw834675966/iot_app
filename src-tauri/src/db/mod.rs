//! PostgreSQL database access entrypoints.
pub mod admin_repository;
pub mod auth_repository;
pub mod entities;

mod bootstrap;
mod migrations;
mod path_store;

use std::future::Future;
use std::sync::OnceLock;

use sea_orm::{Database, DatabaseConnection};
use sqlx::postgres::PgConnectOptions;
use sqlx::{Connection as _, PgConnection};
use tokio::runtime::{Builder, Runtime};

use crate::core::error::AppError;

/// Configure database URL (usually called once in Tauri setup).
pub fn set_database_url(url: String) -> Result<(), AppError> {
    path_store::set_database_url(url)
}

/// Resolve current database URL from override/environment/default.
pub fn database_url() -> String {
    path_store::database_url()
}

#[cfg(test)]
pub fn test_database_url() -> String {
    std::env::var("PURE_ADMIN_TEST_DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("PURE_ADMIN_DATABASE_URL")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| {
            "postgres://postgres:EMSzw%4018627652962@127.0.0.1:5432/pure_admin_thin_test"
                .to_string()
        })
}

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("create sqlx runtime")
    })
}

pub(crate) fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    runtime().block_on(future)
}

/// Open PostgreSQL connection (synchronous wrapper).
pub fn connect() -> Result<PgConnection, AppError> {
    block_on(connect_async())
}

/// Open SeaORM connection (synchronous wrapper).
pub fn connect_orm() -> Result<DatabaseConnection, AppError> {
    block_on(connect_orm_async())
}

/// Open PostgreSQL connection.
pub async fn connect_async() -> Result<PgConnection, AppError> {
    let url = path_store::database_url();
    let options = url
        .parse::<PgConnectOptions>()
        .map_err(|err| AppError::Database(format!("invalid postgres url: {err}")))?;

    PgConnection::connect_with(&options)
        .await
        .map_err(|err| AppError::Database(err.to_string()))
}

/// Open SeaORM connection for entity-based CRUD.
pub async fn connect_orm_async() -> Result<DatabaseConnection, AppError> {
    let url = path_store::database_url();
    Database::connect(url)
        .await
        .map_err(|err| AppError::Database(err.to_string()))
}

/// Initialize database schema and seed data (synchronous wrapper).
pub fn init_database() -> Result<(), AppError> {
    block_on(init_database_async())
}

/// Initialize database schema and seed data.
pub async fn init_database_async() -> Result<(), AppError> {
    bootstrap::init_database().await
}

#[cfg(test)]
mod tests;
