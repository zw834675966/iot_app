//! 实体模块预导出
//!
//! 本模块重新导出常用的实体类型，方便其他模块使用

// 导出 user_roles 实体为 UserRoles
pub use super::user_roles::Entity as UserRoles;
// 导出 users 实体为 Users
pub use super::users::Entity as Users;
