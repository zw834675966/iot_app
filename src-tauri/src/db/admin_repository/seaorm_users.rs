//! SeaORM 用户管理模块
//! 
//! 本模块使用 SeaORM ORM 框架执行用户相关的数据库操作
//! 包括用户的增删改查、角色管理等

// 引入 SeaORM 查询表达式
use sea_orm::sea_query::Expr;
// 引入 SeaORM 核心 trait
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    QueryFilter, QueryOrder, TransactionTrait,
};

// 引入应用错误类型
use crate::core::error::AppError;
// 引入数据库模块
use crate::db;
// 引入实体模型
use crate::db::entities::{user_roles, users};

// 引入父模块的数据结构
use super::{
    ManagedUserRecord, NewUserInput, RegisteredUserRecord, UpdateUserInput, UserLoginState,
    map_user_mutation_error, normalize_unique_roles, trim_optional_phone,
};

/// 创建新用户
/// 
/// 在事务中执行：
/// 1. 创建用户记录
/// 2. 创建用户角色关联记录
/// 
/// # 参数
/// * `input` - 新用户输入数据
/// 
/// # 返回
/// * 成功返回已注册用户记录
pub(super) fn create_user(input: NewUserInput) -> Result<RegisteredUserRecord, AppError> {
    db::block_on(async move {
        // 建立 SeaORM 数据库连接
        let connection = db::connect_orm_async().await?;
        
        // 开启数据库事务（确保用户和角色的一致性）
        let transaction = connection.begin().await.map_err(map_db_error)?;

        // 规范化手机号
        let normalized_phone = trim_optional_phone(input.phone);
        
        // 规范化角色列表（去重排序）
        let unique_roles = normalize_unique_roles(input.roles);

        // 构建用户 ActiveModel
        let user = users::ActiveModel {
            username: Set(input.username),
            password: Set(input.password),
            nickname: Set(input.nickname),
            avatar: Set(String::new()), // 默认空头像
            is_active: Set(1),          // 默认激活
            phone: Set(normalized_phone),
            account_is_permanent: Set(i32::from(input.account_is_permanent)),
            account_valid_days: Set(input.account_valid_days),
            account_expire_at: Set(input.account_expire_at),
            created_at: Set(Some(input.now_millis)),
            updated_at: Set(Some(input.now_millis)),
            created_by: Set(Some(input.created_by)),
            ..Default::default()
        };

        // 插入用户记录
        let inserted = user.insert(&transaction).await.map_err(map_user_db_error)?;

        // 为用户创建角色关联记录
        for role in &unique_roles {
            user_roles::ActiveModel {
                user_id: Set(inserted.id),
                role: Set(role.clone()),
            }
            .insert(&transaction)
            .await
            .map_err(map_db_error)?;
        }

        // 提交事务
        transaction.commit().await.map_err(map_db_error)?;

        // 返回创建成功的用户记录
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

/// 续期用户账号
/// 
/// # 参数
/// * `user_id` - 用户 ID
/// * `account_is_permanent` - 是否永久账号
/// * `account_valid_days` - 有效天数
/// * `account_expire_at` - 过期时间戳
/// * `now_millis` - 当前时间戳
/// 
/// # 返回
/// * 成功返回已注册用户记录
pub(super) fn renew_user_account(
    user_id: i64,
    account_is_permanent: bool,
    account_valid_days: Option<i64>,
    account_expire_at: Option<i64>,
    now_millis: i64,
) -> Result<RegisteredUserRecord, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;

        // 查询现有用户
        let existing = users::Entity::find_by_id(user_id)
            .one(&connection)
            .await
            .map_err(map_db_error)?
            .ok_or_else(|| AppError::Validation("user not found".to_string()))?;

        // 构建更新模型
        let mut active: users::ActiveModel = existing.into();
        active.account_is_permanent = Set(i32::from(account_is_permanent));
        active.account_valid_days = Set(account_valid_days);
        active.account_expire_at = Set(account_expire_at);
        active.is_active = Set(1); // 续期时激活账号
        active.updated_at = Set(Some(now_millis));
        
        // 执行更新
        active
            .update(&connection)
            .await
            .map_err(map_user_db_error)?;

        // 加载更新后的用户记录
        load_registered_user_record(&connection, user_id).await
    })
}

/// 查询用户登录状态
/// 
/// # 参数
/// * `username` - 用户名
/// 
/// # 返回
/// * 登录状态（如果用户存在）
pub(super) fn find_user_login_state(username: &str) -> Result<Option<UserLoginState>, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;
        
        // 按用户名查询用户
        let user = users::Entity::find()
            .filter(users::Column::Username.eq(username))
            .one(&connection)
            .await
            .map_err(map_db_error)?;

        // 转换并返回登录状态
        Ok(user.map(|record| UserLoginState {
            is_active: record.is_active == 1,
            account_is_permanent: record.account_is_permanent == 1,
            account_expire_at: record.account_expire_at,
        }))
    })
}

/// 停用指定用户
/// 
/// # 参数
/// * `username` - 用户名
/// * `now_millis` - 当前时间戳
pub(super) fn deactivate_user_by_username(username: &str, now_millis: i64) -> Result<(), AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;
        
        // 批量更新：设置 is_active = 0, updated_at = now
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

/// 停用所有已过期的用户
/// 
/// # 参数
/// * `now_millis` - 当前时间戳
/// 
/// # 返回
/// * 成功停用的用户数量
pub(super) fn deactivate_expired_users(now_millis: i64) -> Result<usize, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;
        
        // 查询并更新：只更新非永久账号且已过期的用户
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

/// 更新用户信息
/// 
/// # 参数
/// * `input` - 用户更新输入数据
/// 
/// # 返回
/// * 更新后的用户记录
pub(super) fn update_user(input: UpdateUserInput) -> Result<ManagedUserRecord, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;
        
        // 开启事务
        let transaction = connection.begin().await.map_err(map_db_error)?;

        // 查询现有用户
        let existing = users::Entity::find_by_id(input.user_id)
            .one(&transaction)
            .await
            .map_err(map_db_error)?
            .ok_or_else(|| AppError::Validation("user not found".to_string()))?;

        // 构建更新模型
        let mut active: users::ActiveModel = existing.into();
        active.username = Set(input.username);
        active.nickname = Set(input.nickname);
        active.phone = Set(trim_optional_phone(input.phone));
        active.is_active = Set(i32::from(input.is_active));
        active.account_is_permanent = Set(i32::from(input.account_is_permanent));
        active.account_valid_days = Set(input.account_valid_days);
        active.account_expire_at = Set(input.account_expire_at);
        active.updated_at = Set(Some(input.now_millis));
        
        // 执行更新
        active
            .update(&transaction)
            .await
            .map_err(map_user_db_error)?;

        // 规范化角色列表
        let unique_roles = normalize_unique_roles(input.roles);
        
        // 删除旧角色关联
        user_roles::Entity::delete_many()
            .filter(user_roles::Column::UserId.eq(input.user_id))
            .exec(&transaction)
            .await
            .map_err(map_db_error)?;

        // 插入新角色关联
        for role in &unique_roles {
            user_roles::ActiveModel {
                user_id: Set(input.user_id),
                role: Set(role.clone()),
            }
            .insert(&transaction)
            .await
            .map_err(map_db_error)?;
        }

        // 提交事务
        transaction.commit().await.map_err(map_db_error)?;

        // 加载更新后的用户记录
        load_managed_user_record(&connection, input.user_id).await
    })
}

/// 删除用户
/// 
/// # 参数
/// * `user_id` - 用户 ID
/// 
/// # 返回
/// * 删除成功返回 true
pub(super) fn delete_user(user_id: i64) -> Result<bool, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;
        
        // 根据 ID 删除用户（关联的角色会自动因 CASCADE 而删除）
        let result = users::Entity::delete_by_id(user_id)
            .exec(&connection)
            .await
            .map_err(map_db_error)?;

        Ok(result.rows_affected > 0)
    })
}

/// 更新用户密码
/// 
/// # 参数
/// * `user_id` - 用户 ID
/// * `password` - 新密码
/// * `now_millis` - 当前时间戳
/// 
/// # 返回
/// * 更新后的用户记录
pub(super) fn update_user_password(
    user_id: i64,
    password: &str,
    now_millis: i64,
) -> Result<ManagedUserRecord, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;

        // 查询现有用户
        let existing = users::Entity::find_by_id(user_id)
            .one(&connection)
            .await
            .map_err(map_db_error)?
            .ok_or_else(|| AppError::Validation("user not found".to_string()))?;

        // 构建更新模型
        let mut active: users::ActiveModel = existing.into();
        active.password = Set(password.to_string());
        active.updated_at = Set(Some(now_millis));
        
        // 执行更新
        active
            .update(&connection)
            .await
            .map_err(map_user_db_error)?;

        // 加载更新后的用户记录
        load_managed_user_record(&connection, user_id).await
    })
}

/// 根据用户 ID 查询用户名
/// 
/// # 参数
/// * `user_id` - 用户 ID
/// 
/// # 返回
/// * 用户名（如果存在）
pub(super) fn find_username_by_user_id(user_id: i64) -> Result<Option<String>, AppError> {
    db::block_on(async move {
        let connection = db::connect_orm_async().await?;
        
        // 根据 ID 查询用户
        let record = users::Entity::find_by_id(user_id)
            .one(&connection)
            .await
            .map_err(map_db_error)?;
        
        // 返回用户名
        Ok(record.map(|model| model.username))
    })
}

/// 加载已注册用户记录
/// 
/// # 参数
/// * `connection` - 数据库连接
/// * `user_id` - 用户 ID
/// 
/// # 返回
/// * 已注册用户记录
async fn load_registered_user_record<C>(
    connection: &C,
    user_id: i64,
) -> Result<RegisteredUserRecord, AppError>
where
    C: ConnectionTrait,
{
    // 查询用户
    let user = users::Entity::find_by_id(user_id)
        .one(connection)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| AppError::Validation("user not found".to_string()))?;

    // 查询用户角色
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

/// 加载可管理的用户记录
/// 
/// # 参数
/// * `connection` - 数据库连接
/// * `user_id` - 用户 ID
/// 
/// # 返回
/// * 可管理的用户记录
async fn load_managed_user_record<C>(
    connection: &C,
    user_id: i64,
) -> Result<ManagedUserRecord, AppError>
where
    C: ConnectionTrait,
{
    // 查询用户
    let user = users::Entity::find_by_id(user_id)
        .one(connection)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| AppError::Validation("user not found".to_string()))?;

    // 查询用户角色
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

/// 加载用户角色列表
/// 
/// # 参数
/// * `connection` - 数据库连接
/// * `user_id` - 用户 ID
/// 
/// # 返回
/// * 角色列表（按角色名升序排序）
async fn load_roles<C>(connection: &C, user_id: i64) -> Result<Vec<String>, AppError>
where
    C: ConnectionTrait,
{
    // 查询用户角色
    let role_rows = user_roles::Entity::find()
        .filter(user_roles::Column::UserId.eq(user_id))
        .order_by_asc(user_roles::Column::Role)
        .all(connection)
        .await
        .map_err(map_db_error)?;

    // 转换为字符串列表
    Ok(role_rows.into_iter().map(|row| row.role).collect())
}

/// 将数据库错误映射为应用错误
/// 
/// # 参数
/// * `err` - 数据库错误
/// 
/// # 返回
/// * 应用错误
fn map_db_error(err: DbErr) -> AppError {
    AppError::Database(err.to_string())
}

/// 将用户相关的数据库错误映射为应用错误
/// 
/// # 参数
/// * `err` - 数据库错误
/// 
/// # 返回
/// * 应用错误（可能包含用户名重复等特定错误）
fn map_user_db_error(err: DbErr) -> AppError {
    map_user_mutation_error(err.to_string())
}
