//! 鉴权数据仓储模块
//! 
//! 本模块提供用户认证相关的数据查询功能：
//! - 根据用户名密码查询用户档案
//! - 查询并构建动态路由树
//! 
//! 采用仓储模式封装数据访问逻辑

// 引入标准库集合类型
use std::collections::HashMap;

// 引入 JSON 序列化相关类型
use serde_json::{Map, Value};

// 引入 SQLx 查询相关类型
use sqlx::{Row, query};

// 引入鉴权模型
use crate::auth::models::UserProfile;
// 引入应用错误类型
use crate::core::error::AppError;
// 引入数据库模块
use crate::db;

/// 路由行数据结构
/// 
/// 从数据库查询的原始路由数据
/// 包含路由的基本信息和关联的角色/权限
#[derive(Debug, Clone)]
struct RouteRow {
    id: i64,                      // 路由唯一标识
    parent_id: Option<i64>,        // 父路由 ID（用于树形结构）
    path: String,                 // 路由路径
    name: Option<String>,          // 路由名称
    component: Option<String>,    // 路由组件
    meta_title: String,           // 路由标题
    meta_icon: Option<String>,    // 路由图标
    meta_rank: Option<i64>,       // 路由排序
    roles: Vec<String>,           // 可访问的角色列表
    auths: Vec<String>,           // 操作权限列表
}

/// 路由节点数据结构
/// 
/// 在内存中构建树形结构时使用
/// 包含路由信息和子路由列表
#[derive(Debug, Clone)]
struct RouteNode {
    id: i64,                      // 路由唯一标识
    path: String,                 // 路由路径
    name: Option<String>,          // 路由名称
    component: Option<String>,    // 路由组件
    meta_title: String,           // 路由标题
    meta_icon: Option<String>,    // 路由图标
    meta_rank: Option<i64>,       // 路由排序
    roles: Vec<String>,           // 可访问的角色列表
    auths: Vec<String>,           // 操作权限列表
    children: Vec<RouteNode>,     // 子路由列表
}

/// 根据用户名和密码查询用户档案
/// 
/// 执行多表关联查询，获取用户的基本信息、角色和权限
/// 
/// # 参数
/// * `username` - 用户名
/// * `password` - 密码（明文，会与数据库中的哈希值比对）
/// 
/// # 返回
/// * 成功返回 `Some(UserProfile)`
/// * 用户不存在或密码错误返回 `None`
/// * 数据库错误返回 `AppError`
pub fn find_user_profile(username: &str, password: &str) -> Result<Option<UserProfile>, AppError> {
    db::block_on(async {
        // 建立异步数据库连接
        let mut connection = db::connect_async().await?;

        // 使用 sqlx 执行复杂的多表关联查询
        // 查询用户基本信息、聚合角色和权限
        let row = query(
            r"
            SELECT
              u.avatar,
              u.username,
              u.nickname,
              COALESCE(STRING_AGG(DISTINCT ur.role, ','), '') AS roles,
              COALESCE(STRING_AGG(DISTINCT p.code, ','), '') AS permissions
            FROM users u
            LEFT JOIN user_roles ur ON ur.user_id = u.id
            LEFT JOIN user_permissions up ON up.user_id = u.id
            LEFT JOIN permissions p ON p.id = up.permission_id
            WHERE u.username = $1 AND u.password = $2 AND u.is_active = 1
            GROUP BY u.id, u.avatar, u.username, u.nickname
            LIMIT 1
            ",
        )
        .bind(username)
        .bind(password)
        .fetch_optional(&mut connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

        // 如果没有匹配的用户，返回 None
        let Some(row) = row else {
            return Ok(None);
        };

        // 提取查询结果到各个字段
        let avatar: String = row
            .try_get(0)
            .map_err(|err| AppError::Database(err.to_string()))?;
        let username: String = row
            .try_get(1)
            .map_err(|err| AppError::Database(err.to_string()))?;
        let nickname: String = row
            .try_get(2)
            .map_err(|err| AppError::Database(err.to_string()))?;
        let roles: String = row
            .try_get(3)
            .map_err(|err| AppError::Database(err.to_string()))?;
        let permissions: String = row
            .try_get(4)
            .map_err(|err| AppError::Database(err.to_string()))?;

        // 构建用户档案并返回
        Ok(Some(UserProfile {
            avatar,
            username,
            nickname,
            roles: split_csv(&roles),
            permissions: split_csv(&permissions),
        }))
    })
}

/// 查询所有动态路由并构建树形结构
/// 
/// 从数据库查询路由配置，根据 parent_id 构建树形结构
/// 返回前端 vue-router 所需的路由数组
/// 
/// # 返回
/// * 成功返回路由 JSON 数组
/// * 失败返回 `AppError`
pub fn find_async_routes() -> Result<Vec<Value>, AppError> {
    db::block_on(async {
        // 建立异步数据库连接
        let mut connection = db::connect_async().await?;

        // 查询所有路由及其关联的角色和权限
        // 使用 LEFT JOIN 确保即使没有角色/权限的路由也能返回
        let rows = query(
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
              COALESCE(STRING_AGG(DISTINCT rr.role, ','), '') AS roles,
              COALESCE(STRING_AGG(DISTINCT ra.auth, ','), '') AS auths
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
        .fetch_all(&mut connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

        // 将查询结果转换为 RouteRow 结构
        let mut route_rows = Vec::with_capacity(rows.len());
        for row in rows {
            let id: i64 = row
                .try_get(0)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let parent_id: Option<i64> = row
                .try_get(1)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let path: String = row
                .try_get(2)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let name: Option<String> = row
                .try_get(3)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let component: Option<String> = row
                .try_get(4)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let meta_title: String = row
                .try_get(5)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let meta_icon: Option<String> = row
                .try_get(6)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let meta_rank: Option<i32> = row
                .try_get(7)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let roles: String = row
                .try_get(8)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let auths: String = row
                .try_get(9)
                .map_err(|err| AppError::Database(err.to_string()))?;

            route_rows.push(RouteRow {
                id,
                parent_id,
                path,
                name,
                component,
                meta_title,
                meta_icon,
                meta_rank: meta_rank.map(i64::from),
                roles: split_csv(&roles),
                auths: split_csv(&auths),
            });
        }

        // 按父 ID 分组，构建 HashMap
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

        // 递归组装路由树，从根节点（parent_id = None）开始
        let tree = assemble_route_tree(None, &mut grouped);
        
        // 将路由树转换为 JSON 格式
        Ok(tree.into_iter().map(route_to_json).collect())
    })
}

/// 将逗号分隔的字符串拆分为字符串向量
/// 
/// 用于处理 SQL STRING_AGG 返回的逗号分隔值
/// 
/// # 参数
/// * `raw` - 原始逗号分隔字符串
/// 
/// # 返回
/// * 去除空白后的字符串向量
fn split_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

/// 递归组装路由树形结构
/// 
/// 根据 parent_id 递归构建父子路由的树形关系
/// 
/// # 参数
/// * `parent_id` - 父路由 ID（None 表示根节点）
/// * `grouped` - 按父 ID 分组的路由节点映射
/// 
/// # 返回
/// * 当前父节点下的所有子路由节点
fn assemble_route_tree(
    parent_id: Option<i64>,
    grouped: &mut HashMap<Option<i64>, Vec<RouteNode>>,
) -> Vec<RouteNode> {
    // 取出当前父节点的所有子路由
    let mut current = grouped.remove(&parent_id).unwrap_or_default();
    
    // 递归处理每个节点的子路由
    for node in &mut current {
        node.children = assemble_route_tree(Some(node.id), grouped);
    }
    current
}

/// 将路由节点转换为 JSON 格式
/// 
/// 转换为 vue-router 兼容的 JSON 格式
/// 
/// # 参数
/// * `node` - 路由节点
/// 
/// # 返回
/// * JSON 格式的路由对象
fn route_to_json(node: RouteNode) -> Value {
    let mut route = Map::new();
    route.insert("path".to_string(), Value::String(node.path));

    if let Some(name) = node.name {
        route.insert("name".to_string(), Value::String(name));
    }

    if let Some(component) = node.component {
        route.insert("component".to_string(), Value::String(component));
    }

    // 构建 meta 元数据对象
    let mut meta = Map::new();
    meta.insert("title".to_string(), Value::String(node.meta_title));

    if let Some(icon) = node.meta_icon {
        meta.insert("icon".to_string(), Value::String(icon));
    }

    if let Some(rank) = node.meta_rank {
        meta.insert("rank".to_string(), Value::Number(rank.into()));
    }

    // 添加角色限制
    if !node.roles.is_empty() {
        meta.insert(
            "roles".to_string(),
            Value::Array(node.roles.into_iter().map(Value::String).collect()),
        );
    }

    // 添加操作权限
    if !node.auths.is_empty() {
        meta.insert(
            "auths".to_string(),
            Value::Array(node.auths.into_iter().map(Value::String).collect()),
        );
    }

    route.insert("meta".to_string(), Value::Object(meta));

    // 递归添加子路由
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
            db::set_database_url(db::test_database_url()).expect("configure database url");
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
