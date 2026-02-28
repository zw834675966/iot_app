use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoticeItem {
    pub id: u64,
    #[serde(rename = "type")]
    pub item_type: String,
    pub title: String,
    pub datetime: String,
    pub description: String,
    pub status: Option<String>,
    pub extra: Option<String>,
    pub is_read: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoticeReadPayload {
    pub id: u64,
}
