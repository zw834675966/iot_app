use std::collections::HashMap;

use rusqlite::OptionalExtension;
use serde_json::{Map, Value};

use crate::auth::models::UserProfile;
use crate::core::error::AppError;
use crate::db;

#[derive(Debug, Clone)]
struct RouteRow {
    id: i64,
    parent_id: Option<i64>,
    path: String,
    name: Option<String>,
    component: Option<String>,
    meta_title: String,
    meta_icon: Option<String>,
    meta_rank: Option<i64>,
    roles: Vec<String>,
    auths: Vec<String>,
}

#[derive(Debug, Clone)]
struct RouteNode {
    id: i64,
    path: String,
    name: Option<String>,
    component: Option<String>,
    meta_title: String,
    meta_icon: Option<String>,
    meta_rank: Option<i64>,
    roles: Vec<String>,
    auths: Vec<String>,
    children: Vec<RouteNode>,
}

pub fn find_user_profile(username: &str, password: &str) -> Result<Option<UserProfile>, AppError> {
    let connection = db::connect()?;

    let row = connection
        .query_row(
            r"
            SELECT
              u.avatar,
              u.username,
              u.nickname,
              COALESCE(GROUP_CONCAT(DISTINCT ur.role), '') AS roles,
              COALESCE(GROUP_CONCAT(DISTINCT p.code), '') AS permissions
            FROM users u
            LEFT JOIN user_roles ur ON ur.user_id = u.id
            LEFT JOIN user_permissions up ON up.user_id = u.id
            LEFT JOIN permissions p ON p.id = up.permission_id
            WHERE u.username = ?1 AND u.password = ?2 AND u.is_active = 1
            GROUP BY u.id, u.avatar, u.username, u.nickname
            LIMIT 1
            ",
            [username, password],
            |row| {
                Ok(UserProfile {
                    avatar: row.get(0)?,
                    username: row.get(1)?,
                    nickname: row.get(2)?,
                    roles: split_csv(&row.get::<_, String>(3)?),
                    permissions: split_csv(&row.get::<_, String>(4)?),
                })
            },
        )
        .optional()
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(row)
}

pub fn find_async_routes() -> Result<Vec<Value>, AppError> {
    let connection = db::connect()?;

    let mut statement = connection
        .prepare(
            r"
            SELECT
              r.id,
              r.parent_id,
              r.path,
              r.name,
              r.component,
              r.meta_title,
              r.meta_icon,
              r.meta_rank,
              COALESCE(GROUP_CONCAT(DISTINCT rr.role), '') AS roles,
              COALESCE(GROUP_CONCAT(DISTINCT ra.auth), '') AS auths
            FROM routes r
            LEFT JOIN route_roles rr ON rr.route_id = r.id
            LEFT JOIN route_auths ra ON ra.route_id = r.id
            GROUP BY
              r.id,
              r.parent_id,
              r.path,
              r.name,
              r.component,
              r.meta_title,
              r.meta_icon,
              r.meta_rank
            ORDER BY COALESCE(r.parent_id, 0), COALESCE(r.meta_rank, 0), r.id
            ",
        )
        .map_err(|err| AppError::Database(err.to_string()))?;

    let rows = statement
        .query_map([], |row| {
            Ok(RouteRow {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                path: row.get(2)?,
                name: row.get(3)?,
                component: row.get(4)?,
                meta_title: row.get(5)?,
                meta_icon: row.get(6)?,
                meta_rank: row.get(7)?,
                roles: split_csv(&row.get::<_, String>(8)?),
                auths: split_csv(&row.get::<_, String>(9)?),
            })
        })
        .map_err(|err| AppError::Database(err.to_string()))?;

    let route_rows: Result<Vec<_>, _> = rows.collect();
    let route_rows = route_rows.map_err(|err| AppError::Database(err.to_string()))?;

    let mut grouped: HashMap<Option<i64>, Vec<RouteNode>> = HashMap::new();
    for row in route_rows {
        grouped.entry(row.parent_id).or_default().push(RouteNode {
            id: row.id,
            path: row.path,
            name: row.name,
            component: row.component,
            meta_title: row.meta_title,
            meta_icon: row.meta_icon,
            meta_rank: row.meta_rank,
            roles: row.roles,
            auths: row.auths,
            children: Vec::new(),
        });
    }

    let tree = assemble_route_tree(None, &mut grouped);
    Ok(tree.into_iter().map(route_to_json).collect())
}

fn split_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn assemble_route_tree(
    parent_id: Option<i64>,
    grouped: &mut HashMap<Option<i64>, Vec<RouteNode>>,
) -> Vec<RouteNode> {
    let mut current = grouped.remove(&parent_id).unwrap_or_default();
    for node in &mut current {
        node.children = assemble_route_tree(Some(node.id), grouped);
    }
    current
}

fn route_to_json(node: RouteNode) -> Value {
    let mut route = Map::new();
    route.insert("path".to_string(), Value::String(node.path));

    if let Some(name) = node.name {
        route.insert("name".to_string(), Value::String(name));
    }

    if let Some(component) = node.component {
        route.insert("component".to_string(), Value::String(component));
    }

    let mut meta = Map::new();
    meta.insert("title".to_string(), Value::String(node.meta_title));

    if let Some(icon) = node.meta_icon {
        meta.insert("icon".to_string(), Value::String(icon));
    }

    if let Some(rank) = node.meta_rank {
        meta.insert("rank".to_string(), Value::Number(rank.into()));
    }

    if !node.roles.is_empty() {
        meta.insert(
            "roles".to_string(),
            Value::Array(node.roles.into_iter().map(Value::String).collect()),
        );
    }

    if !node.auths.is_empty() {
        meta.insert(
            "auths".to_string(),
            Value::Array(node.auths.into_iter().map(Value::String).collect()),
        );
    }

    route.insert("meta".to_string(), Value::Object(meta));

    if !node.children.is_empty() {
        route.insert(
            "children".to_string(),
            Value::Array(node.children.into_iter().map(route_to_json).collect()),
        );
    }

    Value::Object(route)
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use super::*;
    use crate::db;

    fn ensure_db_ready() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let test_db = std::env::temp_dir().join("pure-admin-thin-auth-tests.sqlite3");
            let _ = std::fs::remove_file(&test_db);
            db::set_database_path(test_db).expect("configure database path");
            db::init_database().expect("init db");
        });
    }

    #[test]
    fn finds_admin_profile() {
        ensure_db_ready();
        let user = find_user_profile("admin", "admin123")
            .expect("query user")
            .expect("admin should exist");
        assert_eq!(user.username, "admin");
        assert_eq!(user.roles, vec!["admin".to_string()]);
    }

    #[test]
    fn returns_none_for_invalid_credentials() {
        ensure_db_ready();
        let user = find_user_profile("admin", "wrong-password").expect("query user");
        assert!(user.is_none());
    }

    #[test]
    fn finds_permission_route_root() {
        ensure_db_ready();
        let routes = find_async_routes().expect("query routes");
        let root_path = routes[0]
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or_default();
        assert_eq!(root_path, "/permission");
    }
}
