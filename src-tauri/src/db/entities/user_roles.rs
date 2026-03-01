//! 用户角色关联实体定义模块
//!
//! 本模块定义 user_roles 表的 SeaORM 实体模型
//! 用于存储用户与角色的多对多关系

// 引入 SeaORM 实体 prelude
use sea_orm::entity::prelude::*;

/// 用户角色关联实体模型
///
/// 对应数据库中的 user_roles 表
/// 这是一个关联表，实现用户与角色的多对多关系
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user_roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)] // 复合主键：非自增
    pub user_id: i64, // 用户 ID（外键）
    #[sea_orm(primary_key, auto_increment = false)] // 复合主键：非自增
    pub role: String, // 角色名称
}

/// 用户角色实体关系定义
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",        // 关联到 users 表
        from = "Column::UserId",                     // 本表外键列
        to = "super::users::Column::Id",             // 目标表主键列
        on_update = "NoAction",                      // 更新时无动作
        on_delete = "Cascade"                        // 删除用户时级联删除角色关联
    )]
    Users, // 多对一：角色关联到用户
}

/// 实现 Related trait，建立与 users 的关联
impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

/// ActiveModel 行为实现
impl ActiveModelBehavior for ActiveModel {}
