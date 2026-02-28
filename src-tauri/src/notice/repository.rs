use sqlx::{Row, query, query_scalar};

use crate::core::error::AppError;
use crate::db;
use crate::notice::models::NoticeItem;

pub fn init_notice_database() -> Result<(), AppError> {
    db::block_on(async {
        let mut connection = db::connect_async().await?;

        query(
            r"
            CREATE TABLE IF NOT EXISTS notice_items (
              id BIGINT PRIMARY KEY,
              item_type TEXT NOT NULL,
              title TEXT NOT NULL,
              description TEXT NOT NULL,
              datetime TEXT NOT NULL,
              status TEXT,
              extra TEXT,
              is_read BOOLEAN NOT NULL DEFAULT FALSE
            )
            ",
        )
        .execute(&mut connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

        seed_default_items_if_needed(&mut connection).await?;
        Ok(())
    })
}

pub fn list_unread_notice_items() -> Result<Vec<NoticeItem>, AppError> {
    list_notice_items_by_read_state(false)
}

pub fn list_read_notice_items() -> Result<Vec<NoticeItem>, AppError> {
    list_notice_items_by_read_state(true)
}

pub fn mark_notice_item_read(id: u64) -> Result<bool, AppError> {
    let id = i64::try_from(id)
        .map_err(|_| AppError::Validation("notice id out of range".to_string()))?;

    db::block_on(async move {
        let mut connection = db::connect_async().await?;
        let result = query(
            r"
            UPDATE notice_items
            SET is_read = TRUE
            WHERE id = $1
              AND is_read = FALSE
            ",
        )
        .bind(id)
        .execute(&mut connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

        Ok(result.rows_affected() > 0)
    })
}

fn list_notice_items_by_read_state(is_read: bool) -> Result<Vec<NoticeItem>, AppError> {
    db::block_on(async move {
        let mut connection = db::connect_async().await?;
        let rows = query(
            r"
            SELECT id, item_type, title, description, datetime, status, extra, is_read
            FROM notice_items
            WHERE is_read = $1
            ORDER BY id ASC
            ",
        )
        .bind(is_read)
        .fetch_all(&mut connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            let id: i64 = row
                .try_get(0)
                .map_err(|err| AppError::Database(err.to_string()))?;
            let id = u64::try_from(id)
                .map_err(|_| AppError::Database("notice id is negative".to_string()))?;

            items.push(NoticeItem {
                id,
                item_type: row
                    .try_get(1)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                title: row
                    .try_get(2)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                description: row
                    .try_get(3)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                datetime: row
                    .try_get(4)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                status: row
                    .try_get(5)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                extra: row
                    .try_get(6)
                    .map_err(|err| AppError::Database(err.to_string()))?,
                is_read: row
                    .try_get(7)
                    .map_err(|err| AppError::Database(err.to_string()))?,
            });
        }

        Ok(items)
    })
}

async fn seed_default_items_if_needed(connection: &mut sqlx::PgConnection) -> Result<(), AppError> {
    let count: i64 = query_scalar("SELECT COUNT(1) FROM notice_items")
        .fetch_one(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    if count > 0 {
        return Ok(());
    }

    let seeds = default_notice_items();
    for item in &seeds {
        query(
            r"
            INSERT INTO notice_items (id, item_type, title, description, datetime, status, extra, is_read)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            ",
        )
        .bind(i64::try_from(item.id).map_err(|_| AppError::Validation("notice id out of range".to_string()))?)
        .bind(&item.item_type)
        .bind(&item.title)
        .bind(&item.description)
        .bind(&item.datetime)
        .bind(&item.status)
        .bind(&item.extra)
        .bind(item.is_read)
        .execute(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;
    }

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

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, Once};

    use sqlx::query;

    use super::*;

    fn ensure_db_ready() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            db::set_database_url(db::test_database_url()).expect("configure database url");
            db::init_database().expect("init auth db");
        });
    }

    fn test_guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().expect("lock notice tests")
    }

    fn reset_notice_table() {
        db::block_on(async {
            let mut connection = db::connect_async().await.expect("connect db");
            query("DROP TABLE IF EXISTS notice_items")
                .execute(&mut connection)
                .await
                .expect("drop notice table");
        });
    }

    #[test]
    fn initializes_and_seeds_unread_notice_items() {
        let _guard = test_guard();
        ensure_db_ready();
        reset_notice_table();

        init_notice_database().expect("init notice db");

        let items = list_unread_notice_items().expect("list unread notices");
        assert!(items.iter().any(|item| item.item_type == "1"));
        assert!(items.iter().any(|item| item.item_type == "2"));
        assert!(items.iter().any(|item| item.item_type == "3"));
        assert!(items.iter().all(|item| !item.is_read));
    }

    #[test]
    fn mark_read_sets_is_read_and_hides_item() {
        let _guard = test_guard();
        ensure_db_ready();
        reset_notice_table();

        init_notice_database().expect("init notice db");

        let first_id = list_unread_notice_items()
            .expect("list unread notices")
            .first()
            .map(|item| item.id)
            .expect("at least one unread item");

        let changed = mark_notice_item_read(first_id).expect("mark read");
        assert!(changed);

        let unread = list_unread_notice_items().expect("list unread notices again");
        assert!(unread.iter().all(|item| item.id != first_id));
    }

    #[test]
    fn mark_read_returns_false_for_missing_item() {
        let _guard = test_guard();
        ensure_db_ready();
        reset_notice_table();

        init_notice_database().expect("init notice db");

        let changed = mark_notice_item_read(999_999).expect("mark missing");
        assert!(!changed);
    }

    #[test]
    fn mark_read_item_appears_in_read_list() {
        let _guard = test_guard();
        ensure_db_ready();
        reset_notice_table();

        init_notice_database().expect("init notice db");

        let target_id = list_unread_notice_items()
            .expect("list unread notices")
            .first()
            .map(|item| item.id)
            .expect("at least one unread item");
        mark_notice_item_read(target_id).expect("mark read");

        let read_items = list_read_notice_items().expect("list read notices");
        assert!(read_items.iter().any(|item| item.id == target_id));
        assert!(read_items.iter().all(|item| item.is_read));
    }
}
