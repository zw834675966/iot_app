use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    QueryFilter, QueryOrder, TransactionTrait,
};

use crate::core::error::AppError;
use crate::db;
use crate::db::entities::{user_roles, users};

use super::{
    ManagedUserRecord, NewUserInput, RegisteredUserRecord, UpdateUserInput, UserLoginState,
    map_user_mutation_error, normalize_unique_roles, trim_optional_phone,
};

pub(super) fn create_user(input: NewUserInput) -> Result<RegisteredUserRecord, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;
        let transaction = connection.begin().await.map_err(map_db_error)?;

        let normalized_phone = trim_optional_phone(input.phone);
        let unique_roles = normalize_unique_roles(input.roles);

        let user = users::ActiveModel {
            username: Set(input.username),
            password: Set(input.password),
            nickname: Set(input.nickname),
            avatar: Set(String::new()),
            is_active: Set(1),
            phone: Set(normalized_phone),
            account_is_permanent: Set(i32::from(input.account_is_permanent)),
            account_valid_days: Set(input.account_valid_days),
            account_expire_at: Set(input.account_expire_at),
            created_at: Set(Some(input.now_millis)),
            updated_at: Set(Some(input.now_millis)),
            created_by: Set(Some(input.created_by)),
            ..Default::default()
        };

        let inserted = user.insert(&transaction).await.map_err(map_user_db_error)?;

        for role in &unique_roles {
            user_roles::ActiveModel {
                user_id: Set(inserted.id),
                role: Set(role.clone()),
            }
            .insert(&transaction)
            .await
            .map_err(map_db_error)?;
        }

        transaction.commit().await.map_err(map_db_error)?;

        Ok(RegisteredUserRecord {
            user_id: inserted.id,
            username: inserted.username,
            roles: unique_roles,
            is_active: inserted.is_active == 1,
            account_is_permanent: inserted.account_is_permanent == 1,
            account_expire_at: inserted.account_expire_at,
        })
    })
}

pub(super) fn renew_user_account(
    user_id: i64,
    account_is_permanent: bool,
    account_valid_days: Option<i64>,
    account_expire_at: Option<i64>,
    now_millis: i64,
) -> Result<RegisteredUserRecord, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;

        let existing = users::Entity::find_by_id(user_id)
            .one(&connection)
            .await
            .map_err(map_db_error)?
            .ok_or_else(|| AppError::Validation("user not found".to_string()))?;

        let mut active: users::ActiveModel = existing.into();
        active.account_is_permanent = Set(i32::from(account_is_permanent));
        active.account_valid_days = Set(account_valid_days);
        active.account_expire_at = Set(account_expire_at);
        active.is_active = Set(1);
        active.updated_at = Set(Some(now_millis));
        active
            .update(&connection)
            .await
            .map_err(map_user_db_error)?;

        load_registered_user_record(&connection, user_id).await
    })
}

pub(super) fn find_user_login_state(username: &str) -> Result<Option<UserLoginState>, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;
        let user = users::Entity::find()
            .filter(users::Column::Username.eq(username))
            .one(&connection)
            .await
            .map_err(map_db_error)?;

        Ok(user.map(|record| UserLoginState {
            is_active: record.is_active == 1,
            account_is_permanent: record.account_is_permanent == 1,
            account_expire_at: record.account_expire_at,
        }))
    })
}

pub(super) fn deactivate_user_by_username(username: &str, now_millis: i64) -> Result<(), AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;
        users::Entity::update_many()
            .col_expr(users::Column::IsActive, Expr::value(0))
            .col_expr(users::Column::UpdatedAt, Expr::value(now_millis))
            .filter(users::Column::Username.eq(username))
            .exec(&connection)
            .await
            .map_err(map_db_error)?;
        Ok(())
    })
}

pub(super) fn deactivate_expired_users(now_millis: i64) -> Result<usize, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;
        let result = users::Entity::update_many()
            .col_expr(users::Column::IsActive, Expr::value(0))
            .col_expr(users::Column::UpdatedAt, Expr::value(now_millis))
            .filter(users::Column::IsActive.eq(1))
            .filter(users::Column::AccountIsPermanent.eq(0))
            .filter(users::Column::AccountExpireAt.is_not_null())
            .filter(users::Column::AccountExpireAt.lte(now_millis))
            .exec(&connection)
            .await
            .map_err(map_db_error)?;

        Ok(usize::try_from(result.rows_affected).unwrap_or(usize::MAX))
    })
}

pub(super) fn update_user(input: UpdateUserInput) -> Result<ManagedUserRecord, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;
        let transaction = connection.begin().await.map_err(map_db_error)?;

        let existing = users::Entity::find_by_id(input.user_id)
            .one(&transaction)
            .await
            .map_err(map_db_error)?
            .ok_or_else(|| AppError::Validation("user not found".to_string()))?;

        let mut active: users::ActiveModel = existing.into();
        active.username = Set(input.username);
        active.nickname = Set(input.nickname);
        active.phone = Set(trim_optional_phone(input.phone));
        active.is_active = Set(i32::from(input.is_active));
        active.account_is_permanent = Set(i32::from(input.account_is_permanent));
        active.account_valid_days = Set(input.account_valid_days);
        active.account_expire_at = Set(input.account_expire_at);
        active.updated_at = Set(Some(input.now_millis));
        active
            .update(&transaction)
            .await
            .map_err(map_user_db_error)?;

        let unique_roles = normalize_unique_roles(input.roles);
        user_roles::Entity::delete_many()
            .filter(user_roles::Column::UserId.eq(input.user_id))
            .exec(&transaction)
            .await
            .map_err(map_db_error)?;

        for role in &unique_roles {
            user_roles::ActiveModel {
                user_id: Set(input.user_id),
                role: Set(role.clone()),
            }
            .insert(&transaction)
            .await
            .map_err(map_db_error)?;
        }

        transaction.commit().await.map_err(map_db_error)?;

        load_managed_user_record(&connection, input.user_id).await
    })
}

pub(super) fn delete_user(user_id: i64) -> Result<bool, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;
        let result = users::Entity::delete_by_id(user_id)
            .exec(&connection)
            .await
            .map_err(map_db_error)?;

        Ok(result.rows_affected > 0)
    })
}

pub(super) fn update_user_password(
    user_id: i64,
    password: &str,
    now_millis: i64,
) -> Result<ManagedUserRecord, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;

        let existing = users::Entity::find_by_id(user_id)
            .one(&connection)
            .await
            .map_err(map_db_error)?
            .ok_or_else(|| AppError::Validation("user not found".to_string()))?;

        let mut active: users::ActiveModel = existing.into();
        active.password = Set(password.to_string());
        active.updated_at = Set(Some(now_millis));
        active
            .update(&connection)
            .await
            .map_err(map_user_db_error)?;

        load_managed_user_record(&connection, user_id).await
    })
}

pub(super) fn find_username_by_user_id(user_id: i64) -> Result<Option<String>, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;
        let record = users::Entity::find_by_id(user_id)
            .one(&connection)
            .await
            .map_err(map_db_error)?;
        Ok(record.map(|model| model.username))
    })
}

async fn load_registered_user_record<C>(
    connection: &C,
    user_id: i64,
) -> Result<RegisteredUserRecord, AppError>
where
    C: ConnectionTrait,
{
    let user = users::Entity::find_by_id(user_id)
        .one(connection)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| AppError::Validation("user not found".to_string()))?;

    let roles = load_roles(connection, user_id).await?;

    Ok(RegisteredUserRecord {
        user_id: user.id,
        username: user.username,
        roles,
        is_active: user.is_active == 1,
        account_is_permanent: user.account_is_permanent == 1,
        account_expire_at: user.account_expire_at,
    })
}

async fn load_managed_user_record<C>(
    connection: &C,
    user_id: i64,
) -> Result<ManagedUserRecord, AppError>
where
    C: ConnectionTrait,
{
    let user = users::Entity::find_by_id(user_id)
        .one(connection)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| AppError::Validation("user not found".to_string()))?;

    let roles = load_roles(connection, user_id).await?;

    Ok(ManagedUserRecord {
        user_id: user.id,
        username: user.username,
        nickname: user.nickname,
        phone: user.phone,
        roles,
        is_active: user.is_active == 1,
        account_is_permanent: user.account_is_permanent == 1,
        account_valid_days: user.account_valid_days,
        account_expire_at: user.account_expire_at,
        created_at: user.created_at,
        updated_at: user.updated_at,
        created_by: user.created_by,
    })
}

async fn load_roles<C>(connection: &C, user_id: i64) -> Result<Vec<String>, AppError>
where
    C: ConnectionTrait,
{
    let role_rows = user_roles::Entity::find()
        .filter(user_roles::Column::UserId.eq(user_id))
        .order_by_asc(user_roles::Column::Role)
        .all(connection)
        .await
        .map_err(map_db_error)?;

    Ok(role_rows.into_iter().map(|row| row.role).collect())
}

fn map_db_error(err: DbErr) -> AppError {
    AppError::Database(err.to_string())
}

fn map_user_db_error(err: DbErr) -> AppError {
    map_user_mutation_error(err.to_string())
}
