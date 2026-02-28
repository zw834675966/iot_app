use sqlx::{Row, query, query_scalar};

use crate::core::error::AppError;
use crate::db;

use super::{ManagedUserRecord, split_csv_sorted};

pub(super) fn is_admin_user(username: &str, now_millis: i64) -> Result<bool, AppError> {
    db::block_on(async {
        let mut connection = db::connect_async().await?;
        let row = query_scalar::<_, i32>(
            r"
            SELECT 1
            FROM users u
            INNER JOIN user_roles ur ON ur.user_id = u.id
            WHERE u.username = $1
              AND u.is_active = 1
              AND ur.role = 'admin'
              AND (
                COALESCE(u.account_is_permanent, 1) = 1
                OR u.account_expire_at IS NULL
                OR u.account_expire_at > $2
              )
            LIMIT 1
            ",
        )
        .bind(username)
        .bind(now_millis)
        .fetch_optional(&mut connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

        Ok(row.is_some())
    })
}

pub(super) fn list_users() -> Result<Vec<ManagedUserRecord>, AppError> {
    db::block_on(async {
        let mut connection = db::connect_async().await?;
        let rows = query(
            r"
            SELECT
              u.id,
              u.username,
              u.nickname,
              u.phone,
              u.is_active,
              COALESCE(u.account_is_permanent, 1) AS account_is_permanent,
              u.account_valid_days,
              u.account_expire_at,
              u.created_at,
              u.updated_at,
              u.created_by,
              COALESCE(STRING_AGG(DISTINCT ur.role, ','), '') AS roles
            FROM users u
            LEFT JOIN user_roles ur ON ur.user_id = u.id
            GROUP BY
              u.id,
              u.username,
              u.nickname,
              u.phone,
              u.is_active,
              u.account_is_permanent,
              u.account_valid_days,
              u.account_expire_at,
              u.created_at,
              u.updated_at,
              u.created_by
            ORDER BY u.id ASC
            ",
        )
        .fetch_all(&mut connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            let is_active: i32 = row
                .try_get(4)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let account_is_permanent: i32 = row
                .try_get(5)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let roles_csv: String = row
                .try_get(11)
                .map_err(|err| AppError::Database(err.to_string()))?;

            records.push(ManagedUserRecord {
                user_id: row
                    .try_get(0)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                username: row
                    .try_get(1)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                nickname: row
                    .try_get(2)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                phone: row
                    .try_get(3)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                roles: split_csv_sorted(&roles_csv),
                is_active: is_active == 1,
                account_is_permanent: account_is_permanent == 1,
                account_valid_days: row
                    .try_get(6)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                account_expire_at: row
                    .try_get(7)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                created_at: row
                    .try_get(8)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                updated_at: row
                    .try_get(9)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                created_by: row
                    .try_get(10)
                    .map_err(|err| AppError::Database(err.to_string()))?,
            });
        }

        Ok(records)
    })
}
