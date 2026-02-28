use crate::core::error::{ApiResponse, AppResult};
use crate::notice::models::{NoticeItem, NoticeReadPayload};
use crate::notice::services::{
    get_read_notice_items, get_unread_notice_items, mark_notice_item_read,
};

#[tauri::command]
pub fn notice_get_unread_items() -> AppResult<Vec<NoticeItem>> {
    Ok(ApiResponse::ok(get_unread_notice_items()?))
}

#[tauri::command]
pub fn notice_get_read_items() -> AppResult<Vec<NoticeItem>> {
    Ok(ApiResponse::ok(get_read_notice_items()?))
}

#[tauri::command]
pub fn notice_mark_read(payload: NoticeReadPayload) -> AppResult<bool> {
    Ok(ApiResponse::ok(mark_notice_item_read(payload.id)?))
}
