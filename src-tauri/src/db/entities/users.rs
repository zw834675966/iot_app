//! 用户实体定义模块
//!
//! 本模块定义 users 表的 SeaORM 实体模型

// 引入 SeaORM 实体 prelude
use sea_orm::entity::prelude::*;

/// 用户实体模型
///
/// 对应数据库中的 users 表
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)] // 主键
    pub id: i64, // 用户 ID
    pub username: String,                // 用户名（唯一）
    pub password: String,                // 密码（哈希存储）
    pub nickname: String,                // 昵称
    pub avatar: String,                  // 头像 URL
    pub is_active: i32,                  // 是否激活（1=激活，0=停用）
    pub phone: Option<String>,           // 手机号（可选）
    pub account_is_permanent: i32,       // 是否永久账号（1=永久，0=有时限）
    pub account_valid_days: Option<i64>, // 有效天数
    pub account_expire_at: Option<i64>,  // 过期时间戳（毫秒）
    pub created_at: Option<i64>,         // 创建时间戳
    pub updated_at: Option<i64>,         // 更新时间戳
    pub created_by: Option<String>,      // 创建者
}

/// 用户实体关系定义
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_roles::Entity")]
    UserRoles, // 一对多：用户可以有多个角色
}

/// 实现 Related trait，建立与 user_roles 的关联
impl Related<super::user_roles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserRoles.def()
    }
}

/// ActiveModel 行为实现
impl ActiveModelBehavior for ActiveModel {}
