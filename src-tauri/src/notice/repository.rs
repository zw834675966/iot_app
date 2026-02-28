use redb::{Database, ReadableTable, ReadableTableMetadata, TableDefinition};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::core::error::AppError;
use crate::notice::models::NoticeItem;

const DB_FILE_NAME: &str = "pure-admin-thin-notice.redb";
const NEXT_ID_KEY: &str = "next_id";
const NOTICE_ITEMS_TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("notice_items");
const NOTICE_META_TABLE: TableDefinition<&str, u64> = TableDefinition::new("notice_meta");

static NOTICE_DB_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn set_notice_database_path(path: PathBuf) -> Result<(), AppError> {
    if NOTICE_DB_PATH.get().is_some() {
        return Ok(());
    }

    if NOTICE_DB_PATH.set(path).is_err() {
        return Ok(());
    }

    Ok(())
}

pub fn init_notice_database() -> Result<(), AppError> {
    let path = database_path();
    init_notice_database_at(&path)
}

pub fn list_unread_notice_items() -> Result<Vec<NoticeItem>, AppError> {
    let path = database_path();
    list_unread_notice_items_at(&path)
}

pub fn list_read_notice_items() -> Result<Vec<NoticeItem>, AppError> {
    let path = database_path();
    list_read_notice_items_at(&path)
}

pub fn mark_notice_item_read(id: u64) -> Result<bool, AppError> {
    let path = database_path();
    mark_notice_item_read_at(&path, id)
}

fn init_notice_database_at(path: &Path) -> Result<(), AppError> {
    let db = open_database(path)?;
    ensure_tables(&db)?;
    seed_default_items_if_needed(&db)?;
    Ok(())
}

fn list_unread_notice_items_at(path: &Path) -> Result<Vec<NoticeItem>, AppError> {
    list_notice_items_by_read_state(path, false)
}

fn list_read_notice_items_at(path: &Path) -> Result<Vec<NoticeItem>, AppError> {
    list_notice_items_by_read_state(path, true)
}

fn list_notice_items_by_read_state(
    path: &Path,
    is_read: bool,
) -> Result<Vec<NoticeItem>, AppError> {
    let db = open_database(path)?;
    let read_txn = db.begin_read().map_err(redb_error)?;
    let table = read_txn
        .open_table(NOTICE_ITEMS_TABLE)
        .map_err(redb_error)?;

    let mut items = Vec::new();
    for entry in table.iter().map_err(redb_error)? {
        let (_, value) = entry.map_err(redb_error)?;
        let item = decode_notice_item(value.value())?;
        if item.is_read == is_read {
            items.push(item);
        }
    }

    items.sort_by_key(|item| item.id);
    Ok(items)
}

fn mark_notice_item_read_at(path: &Path, id: u64) -> Result<bool, AppError> {
    let db = open_database(path)?;
    let write_txn = db.begin_write().map_err(redb_error)?;
    let mut table = write_txn
        .open_table(NOTICE_ITEMS_TABLE)
        .map_err(redb_error)?;

    let encoded = if let Some(raw) = table.get(id).map_err(redb_error)? {
        let mut item = decode_notice_item(raw.value())?;
        drop(raw);
        if item.is_read {
            None
        } else {
            item.is_read = true;
            Some(encode_notice_item(&item)?)
        }
    } else {
        return Ok(false);
    };

    if let Some(value) = encoded {
        table.insert(id, value.as_slice()).map_err(redb_error)?;
    }

    drop(table);
    write_txn.commit().map_err(redb_error)?;
    Ok(true)
}

fn open_database(path: &Path) -> Result<Database, AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| AppError::Database(format!("create notice db dir failed: {err}")))?;
    }
    Database::create(path).map_err(redb_error)
}

fn ensure_tables(db: &Database) -> Result<(), AppError> {
    let write_txn = db.begin_write().map_err(redb_error)?;
    {
        write_txn
            .open_table(NOTICE_ITEMS_TABLE)
            .map_err(redb_error)?;
        write_txn
            .open_table(NOTICE_META_TABLE)
            .map_err(redb_error)?;
    }
    write_txn.commit().map_err(redb_error)?;
    Ok(())
}

fn seed_default_items_if_needed(db: &Database) -> Result<(), AppError> {
    let read_txn = db.begin_read().map_err(redb_error)?;
    let table = read_txn
        .open_table(NOTICE_ITEMS_TABLE)
        .map_err(redb_error)?;
    let has_data = table.len().map_err(redb_error)? > 0;
    drop(table);
    drop(read_txn);
    if has_data {
        return Ok(());
    }

    let seeds = default_notice_items();
    let next_id = seeds.iter().map(|item| item.id).max().unwrap_or(0) + 1;

    let write_txn = db.begin_write().map_err(redb_error)?;
    {
        let mut table = write_txn
            .open_table(NOTICE_ITEMS_TABLE)
            .map_err(redb_error)?;
        for item in &seeds {
            let encoded = encode_notice_item(item)?;
            table
                .insert(item.id, encoded.as_slice())
                .map_err(redb_error)?;
        }
        let mut meta = write_txn
            .open_table(NOTICE_META_TABLE)
            .map_err(redb_error)?;
        meta.insert(NEXT_ID_KEY, next_id).map_err(redb_error)?;
    }
    write_txn.commit().map_err(redb_error)?;
    Ok(())
}

fn default_notice_items() -> Vec<NoticeItem> {
    vec![
        NoticeItem {
            id: 1,
            item_type: "1".to_string(),
            title: "系统通知".to_string(),
            description: "欢迎使用能源管理系统。".to_string(),
            datetime: "今天".to_string(),
            status: None,
            extra: None,
            is_read: false,
        },
        NoticeItem {
            id: 2,
            item_type: "2".to_string(),
            title: "告警消息".to_string(),
            description: "设备 A-100 温度超过阈值，请尽快处理。".to_string(),
            datetime: "今天".to_string(),
            status: None,
            extra: None,
            is_read: false,
        },
        NoticeItem {
            id: 3,
            item_type: "2".to_string(),
            title: "运行消息".to_string(),
            description: "昨日能耗统计已生成。".to_string(),
            datetime: "昨天".to_string(),
            status: None,
            extra: None,
            is_read: false,
        },
        NoticeItem {
            id: 4,
            item_type: "3".to_string(),
            title: "待办：巡检设备 B-02".to_string(),
            description: "请在本周内完成设备 B-02 巡检。".to_string(),
            datetime: String::new(),
            status: Some("warning".to_string()),
            extra: Some("进行中".to_string()),
            is_read: false,
        },
        NoticeItem {
            id: 5,
            item_type: "3".to_string(),
            title: "待办：处理工单 #2301".to_string(),
            description: "工单 #2301 需在今天 18:00 前处理完成。".to_string(),
            datetime: String::new(),
            status: Some("danger".to_string()),
            extra: Some("即将超时".to_string()),
            is_read: false,
        },
    ]
}

fn encode_notice_item(item: &NoticeItem) -> Result<Vec<u8>, AppError> {
    serde_json::to_vec(item)
        .map_err(|err| AppError::Database(format!("encode notice item failed: {err}")))
}

fn decode_notice_item(bytes: &[u8]) -> Result<NoticeItem, AppError> {
    serde_json::from_slice(bytes)
        .map_err(|err| AppError::Database(format!("decode notice item failed: {err}")))
}

fn database_path() -> PathBuf {
    if let Some(path) = NOTICE_DB_PATH.get() {
        return path.clone();
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("db")
        .join(DB_FILE_NAME)
}

fn redb_error<E: std::fmt::Display>(err: E) -> AppError {
    AppError::Database(format!("notice redb error: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db_path(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("{name}-{nanos}.redb"))
    }

    #[test]
    fn initializes_and_seeds_unread_notice_items() {
        let path = temp_db_path("notice-seed");
        init_notice_database_at(&path).expect("init notice db");

        let items = list_unread_notice_items_at(&path).expect("list unread notices");
        assert!(items.iter().any(|item| item.item_type == "1"));
        assert!(items.iter().any(|item| item.item_type == "2"));
        assert!(items.iter().any(|item| item.item_type == "3"));
        assert!(items.iter().all(|item| !item.is_read));
    }

    #[test]
    fn mark_read_sets_is_read_and_hides_item() {
        let path = temp_db_path("notice-mark-read");
        init_notice_database_at(&path).expect("init notice db");

        let first_id = list_unread_notice_items_at(&path)
            .expect("list unread notices")
            .first()
            .map(|item| item.id)
            .expect("at least one unread item");

        let changed = mark_notice_item_read_at(&path, first_id).expect("mark read");
        assert!(changed);

        let unread = list_unread_notice_items_at(&path).expect("list unread notices again");
        assert!(unread.iter().all(|item| item.id != first_id));
    }

    #[test]
    fn mark_read_returns_false_for_missing_item() {
        let path = temp_db_path("notice-missing");
        init_notice_database_at(&path).expect("init notice db");

        let changed = mark_notice_item_read_at(&path, u64::MAX).expect("mark missing");
        assert!(!changed);
    }

    #[test]
    fn mark_read_item_appears_in_read_list() {
        let path = temp_db_path("notice-read-list");
        init_notice_database_at(&path).expect("init notice db");

        let target_id = list_unread_notice_items_at(&path)
            .expect("list unread notices")
            .first()
            .map(|item| item.id)
            .expect("at least one unread item");
        mark_notice_item_read_at(&path, target_id).expect("mark read");

        let read_items = list_read_notice_items_at(&path).expect("list read notices");
        assert!(read_items.iter().any(|item| item.id == target_id));
        assert!(read_items.iter().all(|item| item.is_read));
    }
}
