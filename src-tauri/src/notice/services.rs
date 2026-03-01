//! 通知模块业务逻辑层
//!
//! 本模块是通知功能的业务逻辑层
//! 负责协调命令层和仓储层之间的调用

// 引入应用错误类型
use crate::core::error::AppError;
// 引入通知数据模型
use crate::notice::models::NoticeItem;
// 引入通知仓储模块
use crate::notice::repository;

/// 获取未读通知列表
///
/// # 返回
/// * 未读通知项目列表
pub fn get_unread_notice_items() -> Result<Vec<NoticeItem>, AppError> {
    repository::list_unread_notice_items()
}

/// 获取已读通知列表
///
/// # 返回
/// * 已读通知项目列表
pub fn get_read_notice_items() -> Result<Vec<NoticeItem>, AppError> {
    repository::list_read_notice_items()
}

/// 标记通知为已读
///
/// # 参数
/// * `id` - 通知项目 ID
///
/// # 返回
/// * 标记成功返回 true，项目不存在返回 false
pub fn mark_notice_item_read(id: u64) -> Result<bool, AppError> {
    repository::mark_notice_item_read(id)
}
