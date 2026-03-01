//! 通知模块数据仓储层
//! 
//! 本模块负责通知数据的数据库操作：
//! - 表结构初始化
//! - 种子数据插入
//! - 通知项目查询
//! - 已读状态更新

// 引入 SQLx 查询类型
use sqlx::{Row, query, query_scalar};

// 引入应用错误类型
use crate::core::error::AppError;
// 引入数据库模块
use crate::db;
// 引入通知模型
use crate::notice::models::NoticeItem;

/// 初始化通知数据库
/// 
/// 创建通知表并插入默认种子数据
/// 
/// # 返回
/// * 成功返回 `Ok(())`
/// * 失败返回 `AppError`
pub fn init_notice_database() -> Result<(), AppError> {
    db::block_on(async {
        // 建立异步数据库连接
        let mut connection = db::connect_async().await?;

        // 创建通知项目表（如果不存在）
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

        // 插入默认种子数据（如果表为空）
        seed_default_items_if_needed(&mut connection).await?;
        Ok(())
    })
}

/// 获取未读通知列表
/// 
/// # 返回
/// * 未读通知项目列表
pub fn list_unread_notice_items() -> Result<Vec<NoticeItem>, AppError> {
    list_notice_items_by_read_state(false)
}

/// 获取已读通知列表
/// 
/// # 返回
/// * 已读通知项目列表
pub fn list_read_notice_items() -> Result<Vec<NoticeItem>, AppError> {
    list_notice_items_by_read_state(true)
}

/// 标记通知为已读
/// 
/// # 参数
/// * `id` - 通知项目 ID
/// 
/// # 返回
/// * 标记成功返回 true，项目不存在返回 false
pub fn mark_notice_item_read(id: u64) -> Result<bool, AppError> {
    // 将 u64 转换为 i64（PostgreSQL BIGINT）
    let id = i64::try_from(id)
        .map_err(|_| AppError::Validation("notice id out of range".to_string()))?;

    db::block_on(async move {
        let mut connection = db::connect_async().await?;
        
        // 执行更新：将 is_read 设置为 TRUE
        // 仅更新原本未读的项目
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

        // 返回是否影响了任何行
        Ok(result.rows_affected() > 0)
    })
}

/// 根据已读状态查询通知列表
/// 
/// # 参数
/// * `is_read` - 已读状态（false=未读，true=已读）
/// 
/// # 返回
/// * 通知项目列表
fn list_notice_items_by_read_state(is_read: bool) -> Result<Vec<NoticeItem>, AppError> {
    db::block_on(async move {
        let mut connection = db::connect_async().await?;
        
        // 查询指定已读状态的通知项目
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

        // 将查询结果转换为 NoticeItem 列表
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
                    .try_get::<Option<String>, _>(6)
                    .map_err(|err| AppError::Database(err.to_string()))?
                    .unwrap_or_default(),
                is_read: row
                    .try_get(7)
                    .map_err(|err| AppError::Database(err.to_string()))?,
            });
        }

        Ok(items)
    })
}

/// 插入默认种子数据（如果表为空）
/// 
/// # 参数
/// * `connection` - 数据库连接
async fn seed_default_items_if_needed(connection: &mut sqlx::PgConnection) -> Result<(), AppError> {
    // 检查表中是否有数据
    let count: i64 = query_scalar("SELECT COUNT(1) FROM notice_items")
        .fetch_one(&mut *connection)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    // 如果有数据则跳过
    if count > 0 {
        return Ok(());
    }

    // 获取默认通知项目
    let seeds = default_notice_items();
    
    // 逐条插入
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

/// 获取默认通知项目
/// 
/// # 返回
/// * 默认通知项目列表
fn default_notice_items() -> Vec<NoticeItem> {
    vec![
        NoticeItem {
            id: 1,
            item_type: "1".to_string(),
            title: "系统通知".to_string(),
            description: "欢迎使用能源管理系统。".to_string(),
            datetime: "今天".to_string(),
            status: None,
            extra: "".to_string(),
            is_read: false,
        },
        NoticeItem {
            id: 2,
            item_type: "2".to_string(),
            title: "告警消息".to_string(),
            description: "设备 A-100 温度超过阈值，请尽快处理。".to_string(),
            datetime: "今天".to_string(),
            status: None,
            extra: "".to_string(),
            is_read: false,
        },
        NoticeItem {
            id: 3,
            item_type: "2".to_string(),
            title: "运行消息".to_string(),
            description: "昨日能耗统计已生成。".to_string(),
            datetime: "昨天".to_string(),
            status: None,
            extra: "".to_string(),
            is_read: false,
        },
        NoticeItem {
            id: 4,
            item_type: "3".to_string(),
            title: "待办：巡检设备 B-02".to_string(),
            description: "请在本周内完成设备 B-02 巡检。".to_string(),
            datetime: String::new(),
            status: Some("warning".to_string()),
            extra: "进行中".to_string(),
            is_read: false,
        },
        NoticeItem {
            id: 5,
            item_type: "3".to_string(),
            title: "待办：处理工单 #2301".to_string(),
            description: "工单 #2301 需在今天 18:00 前处理完成。".to_string(),
            datetime: String::new(),
            status: Some("danger".to_string()),
            extra: "即将超时".to_string(),
            is_read: false,
        },
    ]
}

/// 通知模块测试
#[cfg(test)]
mod tests {
    // 引入同步原语
    use std::sync::{Mutex, Once};

    // 引入 SQLx 查询
    use sqlx::query;

    // 引入父模块所有项
    use super::*;

    /// 确保测试数据库已初始化
    /// 
    /// 使用 Once 确保只初始化一次
    fn ensure_db_ready() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            db::set_database_url(db::test_database_url()).expect("configure database url");
            db::init_database().expect("init auth db");
        });
    }

    /// 测试锁
    /// 
    /// 用于防止测试并发执行
    fn test_guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().expect("lock notice tests")
    }

    /// 重置通知表
    /// 
    /// 删除表以便测试重新初始化
    fn reset_notice_table() {
        db::block_on(async {
            let mut connection = db::connect_async().await.expect("connect db");
            query("DROP TABLE IF EXISTS notice_items")
                .execute(&mut connection)
                .await
                .expect("drop notice table");
        });
    }

    /// 测试：初始化并插入未读通知
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

    /// 测试：标记已读后项目从未读列表消失
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

    /// 测试：标记不存在的项目返回 false
    #[test]
    fn mark_read_returns_false_for_missing_item() {
        let _guard = test_guard();
        ensure_db_ready();
        reset_notice_table();

        init_notice_database().expect("init notice db");

        let changed = mark_notice_item_read(999_999).expect("mark missing");
        assert!(!changed);
    }

    /// 测试：标记已读后项目出现在已读列表
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

    /// 测试：读取已读通知时，NULL extra 字段不会导致解码失败
    #[test]
    fn read_items_allow_null_extra_column() {
        let _guard = test_guard();
        ensure_db_ready();
        reset_notice_table();

        init_notice_database().expect("init notice db");

        db::block_on(async {
            let mut connection = db::connect_async().await.expect("connect db");
            query(
                r"
                INSERT INTO notice_items (id, item_type, title, description, datetime, status, extra, is_read)
                VALUES ($1, $2, $3, $4, $5, $6, NULL, TRUE)
                ON CONFLICT (id) DO UPDATE SET
                  item_type = EXCLUDED.item_type,
                  title = EXCLUDED.title,
                  description = EXCLUDED.description,
                  datetime = EXCLUDED.datetime,
                  status = EXCLUDED.status,
                  extra = EXCLUDED.extra,
                  is_read = EXCLUDED.is_read
                ",
            )
            .bind(9_999_i64)
            .bind("2")
            .bind("null-extra")
            .bind("row with nullable extra")
            .bind("today")
            .bind(Option::<String>::None)
            .execute(&mut connection)
            .await
            .expect("insert nullable extra row");
        });

        let read_items = list_read_notice_items().expect("list read notices");
        let item = read_items
            .iter()
            .find(|item| item.id == 9_999)
            .expect("contains nullable extra row");
        assert_eq!(item.extra, "");
        assert!(item.is_read);
    }
}
