use std::collections::HashSet;

use crate::auth::models::{
    AdminChangeUserPasswordData, AdminChangeUserPasswordPayload, AdminDeleteUserPayload,
    AdminListUsersPayload, AdminManagedUserData, AdminRegisterUserPayload, AdminRegisteredUserData,
    AdminRenewUserAccountData, AdminRenewUserAccountPayload, AdminUpdateUserPayload,
};
use crate::core::error::AppError;
use crate::db::admin_repository;

const RESERVED_DEVICE_SCOPE_MESSAGE: &str = "RESERVED_API_NOT_IMPLEMENTED";
const PROTECTED_ADMIN_USERNAME: &str = "admin";
const ROLE_OPERATOR: &str = "operator";
const ROLE_TENANT: &str = "tenant";
const ROLE_MAINTAINER: &str = "maintainer";
const TERM_PERMANENT: &str = "permanent";
const TERM_DAYS: &str = "days";

pub fn register_user_by_admin(
    payload: AdminRegisterUserPayload,
    now_millis: u64,
) -> Result<AdminRegisteredUserData, AppError> {
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;

    let operator_username = payload.operator_username.trim().to_string();
    if operator_username.is_empty() {
        return Err(AppError::Validation(
            "operatorUsername is required".to_string(),
        ));
    }
    assert_operator_is_admin(&operator_username, now_millis)?;

    let username = payload.username.trim().to_string();
    if username.is_empty() {
        return Err(AppError::Validation("username is required".to_string()));
    }
    let password = payload.password.trim().to_string();
    if password.is_empty() {
        return Err(AppError::Validation("password is required".to_string()));
    }
    let nickname = payload.nickname.trim().to_string();
    if nickname.is_empty() {
        return Err(AppError::Validation("nickname is required".to_string()));
    }

    validate_phone(payload.phone.as_deref())?;
    let roles = normalize_roles(payload.roles)?;
    let (account_is_permanent, account_valid_days, account_expire_at) = build_account_term(
        payload.account_term_type.as_str(),
        payload.account_valid_days,
        now_millis,
    )?;

    let result = admin_repository::create_user(admin_repository::NewUserInput {
        username,
        password,
        nickname,
        phone: payload.phone,
        roles,
        account_is_permanent,
        account_valid_days,
        account_expire_at,
        created_by: operator_username,
        now_millis,
    })?;

    Ok(AdminRegisteredUserData {
        user_id: result.user_id,
        username: result.username,
        roles: result.roles,
        is_active: result.is_active,
        account_is_permanent: result.account_is_permanent,
        account_expire_at: result.account_expire_at,
    })
}

pub fn renew_user_account_by_admin(
    payload: AdminRenewUserAccountPayload,
    now_millis: u64,
) -> Result<AdminRenewUserAccountData, AppError> {
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;

    let operator_username = payload.operator_username.trim().to_string();
    if operator_username.is_empty() {
        return Err(AppError::Validation(
            "operatorUsername is required".to_string(),
        ));
    }
    assert_operator_is_admin(&operator_username, now_millis)?;

    if payload.user_id <= 0 {
        return Err(AppError::Validation("userId is required".to_string()));
    }
    assert_target_user_editable(payload.user_id)?;

    let renew_mode = payload.renew_mode.trim().to_ascii_lowercase();
    let (account_is_permanent, account_valid_days, account_expire_at) =
        if renew_mode == TERM_PERMANENT {
            (true, None, None)
        } else if renew_mode == TERM_DAYS {
            let renew_days = payload
                .renew_days
                .ok_or_else(|| AppError::Validation("renewDays is required".to_string()))?;
            if renew_days <= 0 {
                return Err(AppError::Validation(
                    "renewDays must be greater than 0".to_string(),
                ));
            }
            let millis = renew_days
                .checked_mul(24 * 60 * 60 * 1000)
                .ok_or_else(|| AppError::Validation("renewDays is too large".to_string()))?;
            let expire_at = now_millis
                .checked_add(millis)
                .ok_or_else(|| AppError::Validation("renewDays is too large".to_string()))?;
            (false, Some(renew_days), Some(expire_at))
        } else {
            return Err(AppError::Validation(
                "renewMode must be 'permanent' or 'days'".to_string(),
            ));
        };

    let result = admin_repository::renew_user_account(
        payload.user_id,
        account_is_permanent,
        account_valid_days,
        account_expire_at,
        now_millis,
    )?;

    Ok(AdminRenewUserAccountData {
        user_id: result.user_id,
        account_is_permanent: result.account_is_permanent,
        account_expire_at: result.account_expire_at,
        is_active: result.is_active,
    })
}

pub fn list_users_by_admin(
    payload: AdminListUsersPayload,
    now_millis: u64,
) -> Result<Vec<AdminManagedUserData>, AppError> {
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;
    let operator_username = payload.operator_username.trim().to_string();
    if operator_username.is_empty() {
        return Err(AppError::Validation(
            "operatorUsername is required".to_string(),
        ));
    }
    assert_operator_is_admin(&operator_username, now_millis)?;
    let records = admin_repository::list_users()?;
    Ok(records.into_iter().map(map_managed_user_record).collect())
}

pub fn update_user_by_admin(
    payload: AdminUpdateUserPayload,
    now_millis: u64,
) -> Result<AdminManagedUserData, AppError> {
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;
    let operator_username = payload.operator_username.trim().to_string();
    if operator_username.is_empty() {
        return Err(AppError::Validation(
            "operatorUsername is required".to_string(),
        ));
    }
    assert_operator_is_admin(&operator_username, now_millis)?;

    if payload.user_id <= 0 {
        return Err(AppError::Validation("userId is required".to_string()));
    }
    assert_target_user_editable(payload.user_id)?;

    let username = payload.username.trim().to_string();
    if username.is_empty() {
        return Err(AppError::Validation("username is required".to_string()));
    }
    let nickname = payload.nickname.trim().to_string();
    if nickname.is_empty() {
        return Err(AppError::Validation("nickname is required".to_string()));
    }

    validate_phone(payload.phone.as_deref())?;
    let roles = normalize_roles(payload.roles)?;
    let (account_is_permanent, account_valid_days, account_expire_at) = build_account_term(
        payload.account_term_type.as_str(),
        payload.account_valid_days,
        now_millis,
    )?;

    let record = admin_repository::update_user(admin_repository::UpdateUserInput {
        user_id: payload.user_id,
        username,
        nickname,
        phone: payload.phone,
        roles,
        is_active: payload.is_active,
        account_is_permanent,
        account_valid_days,
        account_expire_at,
        now_millis,
    })?;
    Ok(map_managed_user_record(record))
}

pub fn delete_user_by_admin(
    payload: AdminDeleteUserPayload,
    now_millis: u64,
) -> Result<bool, AppError> {
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;
    let operator_username = payload.operator_username.trim().to_string();
    if operator_username.is_empty() {
        return Err(AppError::Validation(
            "operatorUsername is required".to_string(),
        ));
    }
    assert_operator_is_admin(&operator_username, now_millis)?;
    if payload.user_id <= 0 {
        return Err(AppError::Validation("userId is required".to_string()));
    }
    assert_target_user_editable(payload.user_id)?;
    let deleted = admin_repository::delete_user(payload.user_id)?;
    if !deleted {
        return Err(AppError::Validation("user not found".to_string()));
    }
    Ok(true)
}

pub fn change_user_password_by_admin(
    payload: AdminChangeUserPasswordPayload,
    now_millis: u64,
) -> Result<AdminChangeUserPasswordData, AppError> {
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;
    let operator_username = payload.operator_username.trim().to_string();
    if operator_username.is_empty() {
        return Err(AppError::Validation(
            "operatorUsername is required".to_string(),
        ));
    }
    assert_operator_is_admin(&operator_username, now_millis)?;
    if payload.user_id <= 0 {
        return Err(AppError::Validation("userId is required".to_string()));
    }
    let password = payload.password.trim().to_string();
    if password.is_empty() {
        return Err(AppError::Validation("password is required".to_string()));
    }

    let record = admin_repository::update_user_password(payload.user_id, &password, now_millis)?;
    Ok(AdminChangeUserPasswordData {
        user_id: record.user_id,
        username: record.username,
    })
}

pub fn ensure_user_available_with_message(
    username: &str,
    error_message: &str,
    now_millis: u64,
) -> Result<(), AppError> {
    let now_millis =
        i64::try_from(now_millis).map_err(|_| AppError::Validation(error_message.to_string()))?;
    let status = admin_repository::find_user_login_state(username)?;
    let Some(status) = status else {
        return Err(AppError::Validation(error_message.to_string()));
    };

    if !status.is_active {
        return Err(AppError::Validation(error_message.to_string()));
    }

    let expired = !status.account_is_permanent
        && status
            .account_expire_at
            .is_some_and(|expire_at| expire_at <= now_millis);
    if expired {
        admin_repository::deactivate_user_by_username(username, now_millis)?;
        return Err(AppError::Validation(error_message.to_string()));
    }

    Ok(())
}

pub fn run_startup_expiration_compensation(now_millis: u64) -> Result<usize, AppError> {
    let now_millis = i64::try_from(now_millis)
        .map_err(|_| AppError::Validation("invalid current timestamp".to_string()))?;
    admin_repository::deactivate_expired_users(now_millis)
}

pub fn reserved_device_scope_message() -> &'static str {
    RESERVED_DEVICE_SCOPE_MESSAGE
}

fn assert_operator_is_admin(operator_username: &str, now_millis: i64) -> Result<(), AppError> {
    if !admin_repository::is_admin_user(operator_username, now_millis)? {
        return Err(AppError::Validation("forbidden: admin only".to_string()));
    }
    Ok(())
}

fn normalize_roles(raw_roles: Vec<String>) -> Result<Vec<String>, AppError> {
    let mut normalized = HashSet::new();
    for role in raw_roles {
        let role = role.trim().to_ascii_lowercase();
        if role.is_empty() {
            continue;
        }
        if !is_allowed_role(&role) {
            return Err(AppError::Validation(format!("invalid role: {role}")));
        }
        normalized.insert(role);
    }

    if normalized.is_empty() {
        return Err(AppError::Validation("roles is required".to_string()));
    }

    let mut roles: Vec<String> = normalized.into_iter().collect();
    roles.sort();
    Ok(roles)
}

fn is_allowed_role(role: &str) -> bool {
    matches!(role, ROLE_OPERATOR | ROLE_TENANT | ROLE_MAINTAINER)
}

fn build_account_term(
    account_term_type: &str,
    account_valid_days: Option<i64>,
    now_millis: i64,
) -> Result<(bool, Option<i64>, Option<i64>), AppError> {
    let term = account_term_type.trim().to_ascii_lowercase();
    if term == TERM_PERMANENT {
        return Ok((true, None, None));
    }

    if term != TERM_DAYS {
        return Err(AppError::Validation(
            "accountTermType must be 'permanent' or 'days'".to_string(),
        ));
    }

    let days = account_valid_days
        .ok_or_else(|| AppError::Validation("accountValidDays is required".to_string()))?;
    if days <= 0 {
        return Err(AppError::Validation(
            "accountValidDays must be greater than 0".to_string(),
        ));
    }

    let millis = days
        .checked_mul(24 * 60 * 60 * 1000)
        .ok_or_else(|| AppError::Validation("accountValidDays is too large".to_string()))?;
    let expire_at = now_millis
        .checked_add(millis)
        .ok_or_else(|| AppError::Validation("accountValidDays is too large".to_string()))?;
    Ok((false, Some(days), Some(expire_at)))
}

fn validate_phone(phone: Option<&str>) -> Result<(), AppError> {
    let Some(phone) = phone else {
        return Ok(());
    };
    let trimmed = phone.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    if trimmed.len() < 6 || trimmed.len() > 20 {
        return Err(AppError::Validation("invalid phone format".to_string()));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_digit() || c == '+' || c == '-' || c == ' ')
    {
        return Err(AppError::Validation("invalid phone format".to_string()));
    }
    Ok(())
}

fn assert_target_user_editable(user_id: i64) -> Result<(), AppError> {
    let username = admin_repository::find_username_by_user_id(user_id)?
        .ok_or_else(|| AppError::Validation("user not found".to_string()))?;
    if username.eq_ignore_ascii_case(PROTECTED_ADMIN_USERNAME) {
        return Err(AppError::Validation(
            "admin user only supports password change".to_string(),
        ));
    }
    Ok(())
}

fn map_managed_user_record(record: admin_repository::ManagedUserRecord) -> AdminManagedUserData {
    AdminManagedUserData {
        user_id: record.user_id,
        username: record.username,
        nickname: record.nickname,
        phone: record.phone,
        roles: record.roles,
        is_active: record.is_active,
        account_is_permanent: record.account_is_permanent,
        account_valid_days: record.account_valid_days,
        account_expire_at: record.account_expire_at,
        created_at: record.created_at,
        updated_at: record.updated_at,
        created_by: record.created_by,
    }
}
