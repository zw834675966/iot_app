//! 通知模块数据模型
//!
//! 本模块定义通知数据的结构体

// 引入序列化相关 trait
use serde::{Deserialize, Serialize};

/// 通知项目数据结构
///
/// 表示一条通知/消息/待办记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoticeItem {
    pub id: u64, // 通知唯一标识
    #[serde(rename = "type")]
    pub item_type: String, // 通知类型：1-通知，2-消息，3-待办
    pub title: String, // 通知标题
    pub datetime: String, // 时间戳/日期
    pub description: String, // 通知描述
    pub status: Option<String>, // 状态（如 warning, danger）
    pub extra: String, // 额外信息（如"进行中"、"即将超时"）
    pub is_read: bool, // 是否已读
}

/// 标记已读请求载荷
///
/// 用于将通知标记为已读的请求参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoticeReadPayload {
    pub id: u64, // 通知 ID
}
