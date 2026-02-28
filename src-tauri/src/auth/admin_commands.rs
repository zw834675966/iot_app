use crate::auth::admin_services;
use crate::auth::models::{
    AdminRegisterUserPayload, AdminRegisteredUserData, AdminRenewUserAccountData,
    AdminRenewUserAccountPayload, UserDeviceScopeGetPayload, UserDeviceScopeReservedData,
    UserDeviceScopeSnapshot, UserDeviceScopeUpsertPayload,
};
use crate::auth::services::now_millis;
use crate::core::error::{ApiResponse, AppError, AppResult};

#[tauri::command]
pub fn auth_admin_register_user(payload: AdminRegisterUserPayload) -> AppResult<AdminRegisteredUserData> {
    let data = admin_services::register_user_by_admin(payload, now_millis())?;
    Ok(ApiResponse::ok(data))
}

#[tauri::command]
pub fn auth_admin_renew_user_account(
    payload: AdminRenewUserAccountPayload,
) -> AppResult<AdminRenewUserAccountData> {
    let data = admin_services::renew_user_account_by_admin(payload, now_millis())?;
    Ok(ApiResponse::ok(data))
}

#[tauri::command]
pub fn user_device_scope_get(_payload: UserDeviceScopeGetPayload) -> AppResult<UserDeviceScopeReservedData> {
    Ok(ApiResponse::ok(UserDeviceScopeReservedData {
        implemented: false,
        message: admin_services::reserved_device_scope_message().to_string(),
        scope: UserDeviceScopeSnapshot {
            all_areas: false,
            all_floors: false,
            all_devices: false,
            areas: vec![],
            floors: vec![],
            devices: vec![],
        },
    }))
}

#[tauri::command]
pub fn user_device_scope_upsert(_payload: UserDeviceScopeUpsertPayload) -> AppResult<bool> {
    Err(AppError::Validation(
        admin_services::reserved_device_scope_message().to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use crate::db;

    use super::*;

    fn ensure_test_db_ready() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let test_db = std::env::temp_dir().join("pure-admin-thin-auth-tests.sqlite3");
            let _ = std::fs::remove_file(&test_db);
            db::set_database_path(test_db).expect("configure database path");
            db::init_database().expect("init database");
        });
    }

    #[test]
    fn admin_can_register_user_with_multiple_roles() {
        ensure_test_db_ready();
        let payload = AdminRegisterUserPayload {
            operator_username: "admin".to_string(),
            username: "tenant_multi_role".to_string(),
            password: "admin123".to_string(),
            nickname: "多角色用户".to_string(),
            phone: Some("13800138000".to_string()),
            roles: vec!["tenant".to_string(), "operator".to_string()],
            account_term_type: "days".to_string(),
            account_valid_days: Some(30),
        };

        let result = auth_admin_register_user(payload).expect("register user");
        assert_eq!(result.data.username, "tenant_multi_role");
        assert!(result.data.roles.contains(&"tenant".to_string()));
        assert!(result.data.roles.contains(&"operator".to_string()));
        assert!(!result.data.account_is_permanent);
        assert!(result.data.account_expire_at.is_some());
    }

    #[test]
    fn non_admin_cannot_register_user() {
        ensure_test_db_ready();
        let payload = AdminRegisterUserPayload {
            operator_username: "common".to_string(),
            username: "tenant_forbidden".to_string(),
            password: "admin123".to_string(),
            nickname: "禁止注册".to_string(),
            phone: None,
            roles: vec!["tenant".to_string()],
            account_term_type: "permanent".to_string(),
            account_valid_days: None,
        };

        let err = auth_admin_register_user(payload).expect_err("expect forbidden");
        assert_eq!(err, AppError::Validation("forbidden: admin only".to_string()));
    }

    #[test]
    fn admin_can_renew_user_account() {
        ensure_test_db_ready();
        let register_payload = AdminRegisterUserPayload {
            operator_username: "admin".to_string(),
            username: "tenant_for_renew".to_string(),
            password: "admin123".to_string(),
            nickname: "续期用户".to_string(),
            phone: None,
            roles: vec!["tenant".to_string()],
            account_term_type: "days".to_string(),
            account_valid_days: Some(7),
        };
        let register_result =
            auth_admin_register_user(register_payload).expect("register user for renew");

        let renew_payload = AdminRenewUserAccountPayload {
            operator_username: "admin".to_string(),
            user_id: register_result.data.user_id,
            renew_mode: "days".to_string(),
            renew_days: Some(90),
        };
        let renewed = auth_admin_renew_user_account(renew_payload).expect("renew account");
        assert_eq!(renewed.data.user_id, register_result.data.user_id);
        assert!(!renewed.data.account_is_permanent);
        assert!(renewed.data.account_expire_at.is_some());
        assert!(renewed.data.is_active);
    }

    #[test]
    fn reserved_upsert_returns_not_implemented() {
        ensure_test_db_ready();
        let payload = UserDeviceScopeUpsertPayload {
            user_id: 1,
            all_areas: false,
            all_floors: false,
            all_devices: false,
            areas: vec![],
            floors: vec![],
            devices: vec![],
        };
        let err = user_device_scope_upsert(payload).expect_err("expect reserved message");
        assert_eq!(
            err,
            AppError::Validation("RESERVED_API_NOT_IMPLEMENTED".to_string())
        );
    }
}
