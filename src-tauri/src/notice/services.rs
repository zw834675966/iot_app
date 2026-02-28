use crate::core::error::AppError;
use crate::notice::models::NoticeItem;
use crate::notice::repository;

pub fn get_unread_notice_items() -> Result<Vec<NoticeItem>, AppError> {
    repository::list_unread_notice_items()
}

pub fn get_read_notice_items() -> Result<Vec<NoticeItem>, AppError> {
    repository::list_read_notice_items()
}

pub fn mark_notice_item_read(id: u64) -> Result<bool, AppError> {
    repository::mark_notice_item_read(id)
}
