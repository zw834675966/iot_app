use std::collections::HashSet;

use rusqlite::{params, OptionalExtension};

use crate::core::error::AppError;
use crate::db;

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

pub fn create_user(input: NewUserInput) -> Result<RegisteredUserRecord, AppError> {
    let mut connection = db::connect()?;
    let transaction = connection
        .transaction()
        .map_err(|err| AppError::Database(err.to_string()))?;

    let phone = input.phone.and_then(|v| {
        let trimmed = v.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    transaction
        .execute(
            r"
            INSERT INTO users (
              username,
              password,
              nickname,
              avatar,
              is_active,
              phone,
              account_is_permanent,
              account_valid_days,
              account_expire_at,
              created_at,
              updated_at,
              created_by
            ) VALUES (?1, ?2, ?3, '', 1, ?4, ?5, ?6, ?7, ?8, ?8, ?9)
            ",
            params![
                input.username,
                input.password,
                input.nickname,
                phone,
                if input.account_is_permanent { 1 } else { 0 },
                input.account_valid_days,
                input.account_expire_at,
                input.now_millis,
                input.created_by
            ],
        )
        .map_err(map_insert_user_error)?;

    let user_id = transaction.last_insert_rowid();

    let unique_roles: HashSet<String> = input
        .roles
        .into_iter()
        .map(|role| role.trim().to_string())
        .filter(|role| !role.is_empty())
        .collect();

    for role in &unique_roles {
        transaction
            .execute(
                "INSERT INTO user_roles (user_id, role) VALUES (?1, ?2)",
                params![user_id, role],
            )
            .map_err(|err| AppError::Database(err.to_string()))?;
    }

    transaction
        .commit()
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(RegisteredUserRecord {
        user_id,
        username: db::connect()?
            .query_row(
                "SELECT username FROM users WHERE id = ?1 LIMIT 1",
                [user_id],
                |row| row.get(0),
            )
            .map_err(|err| AppError::Database(err.to_string()))?,
        roles: unique_roles.into_iter().collect(),
        is_active: true,
        account_is_permanent: input.account_is_permanent,
        account_expire_at: input.account_expire_at,
    })
}

pub fn renew_user_account(
    user_id: i64,
    account_is_permanent: bool,
    account_valid_days: Option<i64>,
    account_expire_at: Option<i64>,
    now_millis: i64,
) -> Result<RegisteredUserRecord, AppError> {
    let connection = db::connect()?;
    let affected = connection
        .execute(
            r"
            UPDATE users
            SET
              account_is_permanent = ?2,
              account_valid_days = ?3,
              account_expire_at = ?4,
              is_active = 1,
              updated_at = ?5
            WHERE id = ?1
            ",
            params![
                user_id,
                if account_is_permanent { 1 } else { 0 },
                account_valid_days,
                account_expire_at,
                now_millis
            ],
        )
        .map_err(|err| AppError::Database(err.to_string()))?;

    if affected == 0 {
        return Err(AppError::Validation("user not found".to_string()));
    }

    load_registered_user_record(user_id)
}

pub fn is_admin_user(username: &str, now_millis: i64) -> Result<bool, AppError> {
    let connection = db::connect()?;
    connection
        .query_row(
            r"
            SELECT 1
            FROM users u
            INNER JOIN user_roles ur ON ur.user_id = u.id
            WHERE u.username = ?1
              AND u.is_active = 1
              AND ur.role = 'admin'
              AND (
                COALESCE(u.account_is_permanent, 1) = 1
                OR u.account_expire_at IS NULL
                OR u.account_expire_at > ?2
              )
            LIMIT 1
            ",
            params![username, now_millis],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map(|row| row.is_some())
        .map_err(|err| AppError::Database(err.to_string()))
}

pub fn find_user_login_state(username: &str) -> Result<Option<UserLoginState>, AppError> {
    let connection = db::connect()?;
    connection
        .query_row(
            r"
            SELECT
              u.is_active,
              COALESCE(u.account_is_permanent, 1) AS account_is_permanent,
              u.account_expire_at
            FROM users u
            WHERE u.username = ?1
            LIMIT 1
            ",
            [username],
            |row| {
                let is_active: i64 = row.get(0)?;
                let account_is_permanent: i64 = row.get(1)?;
                Ok(UserLoginState {
                    is_active: is_active == 1,
                    account_is_permanent: account_is_permanent == 1,
                    account_expire_at: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(|err| AppError::Database(err.to_string()))
}

pub fn deactivate_user_by_username(username: &str, now_millis: i64) -> Result<(), AppError> {
    let connection = db::connect()?;
    connection
        .execute(
            "UPDATE users SET is_active = 0, updated_at = ?2 WHERE username = ?1",
            params![username, now_millis],
        )
        .map_err(|err| AppError::Database(err.to_string()))?;
    Ok(())
}

pub fn deactivate_expired_users(now_millis: i64) -> Result<usize, AppError> {
    let connection = db::connect()?;
    connection
        .execute(
            r"
            UPDATE users
            SET is_active = 0,
                updated_at = ?1
            WHERE is_active = 1
              AND COALESCE(account_is_permanent, 1) = 0
              AND account_expire_at IS NOT NULL
              AND account_expire_at <= ?1
            ",
            [now_millis],
        )
        .map_err(|err| AppError::Database(err.to_string()))
}

fn load_registered_user_record(user_id: i64) -> Result<RegisteredUserRecord, AppError> {
    let connection = db::connect()?;
    let user = connection
        .query_row(
            r"
            SELECT
              username,
              is_active,
              COALESCE(account_is_permanent, 1) AS account_is_permanent,
              account_expire_at
            FROM users
            WHERE id = ?1
            LIMIT 1
            ",
            [user_id],
            |row| {
                let is_active: i64 = row.get(1)?;
                let account_is_permanent: i64 = row.get(2)?;
                Ok((
                    row.get::<_, String>(0)?,
                    is_active == 1,
                    account_is_permanent == 1,
                    row.get::<_, Option<i64>>(3)?,
                ))
            },
        )
        .optional()
        .map_err(|err| AppError::Database(err.to_string()))?
        .ok_or_else(|| AppError::Validation("user not found".to_string()))?;

    let mut statement = connection
        .prepare(
            r"
            SELECT role
            FROM user_roles
            WHERE user_id = ?1
            ORDER BY role ASC
            ",
        )
        .map_err(|err| AppError::Database(err.to_string()))?;
    let rows = statement
        .query_map([user_id], |row| row.get::<_, String>(0))
        .map_err(|err| AppError::Database(err.to_string()))?;
    let roles: Result<Vec<String>, _> = rows.collect();
    let roles = roles.map_err(|err| AppError::Database(err.to_string()))?;

    Ok(RegisteredUserRecord {
        user_id,
        username: user.0,
        roles,
        is_active: user.1,
        account_is_permanent: user.2,
        account_expire_at: user.3,
    })
}

fn map_insert_user_error(err: rusqlite::Error) -> AppError {
    match err {
        rusqlite::Error::SqliteFailure(_, Some(message)) => {
            if message.to_lowercase().contains("users.username") {
                return AppError::Validation("username already exists".to_string());
            }
            AppError::Database(message)
        }
        other => AppError::Database(other.to_string()),
    }
}
