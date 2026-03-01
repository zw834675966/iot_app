//! 通知模块 IPC 命令层
//!
//! 本模块定义前端可调用的 Tauri 命令接口

// 引入核心错误类型
use crate::core::error::{ApiResponse, AppResult};
// 引入链路追踪相关类型
use crate::core::tracing::{execute_traced_command, TraceContext};
// 引入通知数据模型
use crate::notice::models::{NoticeItem, NoticeReadPayload};
// 引入通知服务层函数
use crate::notice::services::{
    get_read_notice_items, get_unread_notice_items, mark_notice_item_read,
};

/// 获取未读通知列表
///
/// # 返回
/// * 所有未读的通知项目列表
#[tauri::command]
pub fn notice_get_unread_items(trace: Option<TraceContext>) -> AppResult<Vec<NoticeItem>> {
    execute_traced_command("notice_get_unread_items", trace, || {
        Ok(ApiResponse::ok(get_unread_notice_items()?))
    })
}

/// 获取已读通知列表
///
/// # 返回
/// * 所有已读的通知项目列表
#[tauri::command]
pub fn notice_get_read_items(trace: Option<TraceContext>) -> AppResult<Vec<NoticeItem>> {
    execute_traced_command("notice_get_read_items", trace, || {
        Ok(ApiResponse::ok(get_read_notice_items()?))
    })
}

/// 标记通知为已读
///
/// # 参数
/// * `payload` - 包含通知 ID 的请求体
///
/// # 返回
/// * 标记成功返回 true，项目不存在返回 false
#[tauri::command]
pub fn notice_mark_read(
    payload: NoticeReadPayload,
    trace: Option<TraceContext>,
) -> AppResult<bool> {
    execute_traced_command("notice_mark_read", trace, || {
        Ok(ApiResponse::ok(mark_notice_item_read(payload.id)?))
    })
}
