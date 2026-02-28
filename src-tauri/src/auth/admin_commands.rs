use crate::auth::admin_services;
use crate::auth::models::{
    AdminChangeUserPasswordData, AdminChangeUserPasswordPayload, AdminDeleteUserPayload,
    AdminListUsersPayload, AdminManagedUserData, AdminRegisterUserPayload, AdminRegisteredUserData,
    AdminRenewUserAccountData, AdminRenewUserAccountPayload, AdminUpdateUserPayload,
    UserDeviceScopeGetPayload, UserDeviceScopeReservedData, UserDeviceScopeSnapshot,
    UserDeviceScopeUpsertPayload,
};
use crate::auth::services::now_millis;
use crate::core::error::{ApiResponse, AppError, AppResult};

#[tauri::command]
pub fn auth_admin_register_user(
    payload: AdminRegisterUserPayload,
) -> AppResult<AdminRegisteredUserData> {
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
pub fn auth_admin_list_users(
    payload: AdminListUsersPayload,
) -> AppResult<Vec<AdminManagedUserData>> {
    let data = admin_services::list_users_by_admin(payload, now_millis())?;
    Ok(ApiResponse::ok(data))
}

#[tauri::command]
pub fn auth_admin_update_user(payload: AdminUpdateUserPayload) -> AppResult<AdminManagedUserData> {
    let data = admin_services::update_user_by_admin(payload, now_millis())?;
    Ok(ApiResponse::ok(data))
}

#[tauri::command]
pub fn auth_admin_delete_user(payload: AdminDeleteUserPayload) -> AppResult<bool> {
    let data = admin_services::delete_user_by_admin(payload, now_millis())?;
    Ok(ApiResponse::ok(data))
}

#[tauri::command]
pub fn auth_admin_change_user_password(
    payload: AdminChangeUserPasswordPayload,
) -> AppResult<AdminChangeUserPasswordData> {
    let data = admin_services::change_user_password_by_admin(payload, now_millis())?;
    Ok(ApiResponse::ok(data))
}

#[tauri::command]
pub fn user_device_scope_get(
    _payload: UserDeviceScopeGetPayload,
) -> AppResult<UserDeviceScopeReservedData> {
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
        assert_eq!(
            err,
            AppError::Validation("forbidden: admin only".to_string())
        );
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
    fn admin_cannot_renew_protected_admin_user() {
        ensure_test_db_ready();
        let payload = AdminRenewUserAccountPayload {
            operator_username: "admin".to_string(),
            user_id: 1,
            renew_mode: "days".to_string(),
            renew_days: Some(7),
        };
        let err = auth_admin_renew_user_account(payload).expect_err("expect protected user check");
        assert_eq!(
            err,
            AppError::Validation("admin user only supports password change".to_string())
        );
    }

    #[test]
    fn admin_can_list_users() {
        ensure_test_db_ready();
        let payload = AdminListUsersPayload {
            operator_username: "admin".to_string(),
        };
        let result = auth_admin_list_users(payload).expect("list users");
        assert!(!result.data.is_empty());
        assert!(result.data.iter().any(|item| item.username == "admin"));
    }

    #[test]
    fn admin_cannot_update_protected_admin_profile() {
        ensure_test_db_ready();
        let payload = AdminUpdateUserPayload {
            operator_username: "admin".to_string(),
            user_id: 1,
            username: "admin".to_string(),
            nickname: "admin".to_string(),
            phone: None,
            roles: vec!["operator".to_string()],
            is_active: true,
            account_term_type: "permanent".to_string(),
            account_valid_days: None,
        };
        let err = auth_admin_update_user(payload).expect_err("expect protected user check");
        assert_eq!(
            err,
            AppError::Validation("admin user only supports password change".to_string())
        );
    }

    #[test]
    fn admin_can_update_and_delete_non_admin_user() {
        ensure_test_db_ready();
        let register_payload = AdminRegisterUserPayload {
            operator_username: "admin".to_string(),
            username: "tenant_for_crud".to_string(),
            password: "admin123".to_string(),
            nickname: "crud target".to_string(),
            phone: Some("13800138001".to_string()),
            roles: vec!["tenant".to_string()],
            account_term_type: "days".to_string(),
            account_valid_days: Some(30),
        };
        let registered = auth_admin_register_user(register_payload).expect("register user");

        let update_payload = AdminUpdateUserPayload {
            operator_username: "admin".to_string(),
            user_id: registered.data.user_id,
            username: "tenant_for_crud_renamed".to_string(),
            nickname: "crud target renamed".to_string(),
            phone: Some("13800138002".to_string()),
            roles: vec!["maintainer".to_string(), "operator".to_string()],
            is_active: true,
            account_term_type: "permanent".to_string(),
            account_valid_days: None,
        };
        let updated = auth_admin_update_user(update_payload).expect("update user");
        assert_eq!(updated.data.user_id, registered.data.user_id);
        assert_eq!(updated.data.username, "tenant_for_crud_renamed");
        assert!(updated.data.roles.contains(&"maintainer".to_string()));
        assert!(updated.data.roles.contains(&"operator".to_string()));
        assert!(updated.data.account_is_permanent);

        let delete_payload = AdminDeleteUserPayload {
            operator_username: "admin".to_string(),
            user_id: updated.data.user_id,
        };
        let deleted = auth_admin_delete_user(delete_payload).expect("delete user");
        assert!(deleted.data);
    }

    #[test]
    fn admin_can_change_password_for_protected_admin_user() {
        ensure_test_db_ready();
        let payload = AdminChangeUserPasswordPayload {
            operator_username: "admin".to_string(),
            user_id: 1,
            password: "admin123".to_string(),
        };
        let result = auth_admin_change_user_password(payload).expect("change password");
        assert_eq!(result.data.username, "admin");
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
