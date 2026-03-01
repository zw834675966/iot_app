use casbin::{CoreApi, DefaultModel, Enforcer};
use sqlx_adapter::SqlxAdapter;

use crate::core::error::AppError;
use crate::db;
use crate::db::admin_repository;

pub const RESOURCE_USER: &str = "user";
pub const RESOURCE_DEVICE: &str = "device";
pub const RESOURCE_CONTROL: &str = "control";
pub const RESOURCE_DASHBOARD: &str = "dashboard";

pub const ACTION_MANAGE: &str = "manage";
pub const ACTION_CREATE: &str = "create";
pub const ACTION_ISSUE: &str = "issue";
pub const ACTION_VIEW: &str = "view";

const ENFORCER_POOL_SIZE: u32 = 4;

const RBAC_MODEL_CONF: &str = r#"
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = r.sub == p.sub && r.obj == p.obj && r.act == p.act
"#;

pub fn ensure_user_allowed(
    username: &str,
    resource: &str,
    action: &str,
    now_millis: i64,
    forbidden_message: &str,
) -> Result<(), AppError> {
    let roles = admin_repository::find_effective_roles(username, now_millis)?;
    if roles.is_empty() {
        return Err(AppError::Validation(forbidden_message.to_string()));
    }

    let allowed = db::block_on(async {
        let enforcer = build_enforcer().await?;
        for role in &roles {
            let granted = enforcer
                .enforce((role.as_str(), resource, action))
                .map_err(|err| AppError::Database(format!("evaluate rbac policy failed: {err}")))?;
            if granted {
                return Ok::<bool, AppError>(true);
            }
        }
        Ok(false)
    })?;

    if allowed {
        Ok(())
    } else {
        Err(AppError::Validation(forbidden_message.to_string()))
    }
}

async fn build_enforcer() -> Result<Enforcer, AppError> {
    let model = DefaultModel::from_str(RBAC_MODEL_CONF)
        .await
        .map_err(|err| AppError::Database(format!("build rbac model failed: {err}")))?;
    let adapter = SqlxAdapter::new(db::database_url(), ENFORCER_POOL_SIZE)
        .await
        .map_err(|err| AppError::Database(format!("open rbac policy adapter failed: {err}")))?;
    Enforcer::new(model, adapter)
        .await
        .map_err(|err| AppError::Database(format!("initialize rbac enforcer failed: {err}")))
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use super::*;
    use crate::db;

    fn ensure_db_ready() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            db::set_database_url(db::test_database_url()).expect("configure database url");
            db::init_database().expect("init db");
        });
    }

    #[test]
    fn admin_can_manage_users() {
        ensure_db_ready();
        let result = ensure_user_allowed(
            "admin",
            RESOURCE_USER,
            ACTION_MANAGE,
            1,
            "forbidden: admin only",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn common_user_cannot_manage_users() {
        ensure_db_ready();
        let err = ensure_user_allowed(
            "common",
            RESOURCE_USER,
            ACTION_MANAGE,
            1,
            "forbidden: admin only",
        )
        .expect_err("common user should not pass admin management policy");
        assert_eq!(
            err,
            AppError::Validation("forbidden: admin only".to_string())
        );
    }
}
