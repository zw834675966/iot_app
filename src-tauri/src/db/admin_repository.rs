use std::collections::HashSet;

use crate::core::error::AppError;

#[path = "admin_repository/seaorm_users.rs"]
mod seaorm_users;
#[path = "admin_repository/sqlx_reports.rs"]
mod sqlx_reports;

pub struct NewUserInput {
    pub username: String,
    pub password: String,
    pub nickname: String,
    pub phone: Option<String>,
    pub roles: Vec<String>,
    pub account_is_permanent: bool,
    pub account_valid_days: Option<i64>,
    pub account_expire_at: Option<i64>,
    pub created_by: String,
    pub now_millis: i64,
}

#[derive(Debug, Clone)]
pub struct RegisteredUserRecord {
    pub user_id: i64,
    pub username: String,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub account_is_permanent: bool,
    pub account_expire_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct UserLoginState {
    pub is_active: bool,
    pub account_is_permanent: bool,
    pub account_expire_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ManagedUserRecord {
    pub user_id: i64,
    pub username: String,
    pub nickname: String,
    pub phone: Option<String>,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub account_is_permanent: bool,
    pub account_valid_days: Option<i64>,
    pub account_expire_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub created_by: Option<String>,
}

pub struct UpdateUserInput {
    pub user_id: i64,
    pub username: String,
    pub nickname: String,
    pub phone: Option<String>,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub account_is_permanent: bool,
    pub account_valid_days: Option<i64>,
    pub account_expire_at: Option<i64>,
    pub now_millis: i64,
}

pub fn create_user(input: NewUserInput) -> Result<RegisteredUserRecord, AppError> {
    seaorm_users::create_user(input)
}

pub fn renew_user_account(
    user_id: i64,
    account_is_permanent: bool,
    account_valid_days: Option<i64>,
    account_expire_at: Option<i64>,
    now_millis: i64,
) -> Result<RegisteredUserRecord, AppError> {
    seaorm_users::renew_user_account(
        user_id,
        account_is_permanent,
        account_valid_days,
        account_expire_at,
        now_millis,
    )
}

pub fn is_admin_user(username: &str, now_millis: i64) -> Result<bool, AppError> {
    sqlx_reports::is_admin_user(username, now_millis)
}

pub fn find_user_login_state(username: &str) -> Result<Option<UserLoginState>, AppError> {
    seaorm_users::find_user_login_state(username)
}

pub fn deactivate_user_by_username(username: &str, now_millis: i64) -> Result<(), AppError> {
    seaorm_users::deactivate_user_by_username(username, now_millis)
}

pub fn deactivate_expired_users(now_millis: i64) -> Result<usize, AppError> {
    seaorm_users::deactivate_expired_users(now_millis)
}

pub fn list_users() -> Result<Vec<ManagedUserRecord>, AppError> {
    sqlx_reports::list_users()
}

pub fn update_user(input: UpdateUserInput) -> Result<ManagedUserRecord, AppError> {
    seaorm_users::update_user(input)
}

pub fn delete_user(user_id: i64) -> Result<bool, AppError> {
    seaorm_users::delete_user(user_id)
}

pub fn update_user_password(
    user_id: i64,
    password: &str,
    now_millis: i64,
) -> Result<ManagedUserRecord, AppError> {
    seaorm_users::update_user_password(user_id, password, now_millis)
}

pub fn find_username_by_user_id(user_id: i64) -> Result<Option<String>, AppError> {
    seaorm_users::find_username_by_user_id(user_id)
}

pub(super) fn normalize_unique_roles(raw_roles: Vec<String>) -> Vec<String> {
    let unique_roles: HashSet<String> = raw_roles
        .into_iter()
        .map(|role| role.trim().to_string())
        .filter(|role| !role.is_empty())
        .collect();
    let mut roles: Vec<String> = unique_roles.into_iter().collect();
    roles.sort();
    roles
}

pub(super) fn split_csv_sorted(raw: &str) -> Vec<String> {
    let mut values: Vec<String> = raw
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect();
    values.sort();
    values.dedup();
    values
}

pub(super) fn trim_optional_phone(phone: Option<String>) -> Option<String> {
    phone.and_then(|raw| {
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

pub(super) fn map_user_mutation_error(message: String) -> AppError {
    let lowered = message.to_lowercase();
    if lowered.contains("users.username")
        || lowered.contains("users_username_key")
        || lowered.contains("duplicate key value violates unique constraint")
    {
        return AppError::Validation("username already exists".to_string());
    }

    AppError::Database(message)
}
