# 通知模块 (PostgreSQL)

> 本模块提供消息通知中心的数据访问和 IPC 命令接口，使用 PostgreSQL 数据库存储通知数据。

## 功能范围

- 存储 `通知` / `消息` / `待办` 项目
- 支持未读筛选（`is_read = false`）
- 支持已读筛选（`is_read = true`）
- 支持标记已读操作（`is_read = true`）

## 目录结构

```
src-tauri/src/notice/
├── mod.rs         # 模块入口
├── commands.rs    # Tauri IPC 命令层
├── models.rs      # 数据模型定义
├── services.rs    # 业务逻辑层
├── repository.rs  # 数据仓储层
└── README.md     # 本文档
```

## 数据表结构

```sql
CREATE TABLE IF NOT EXISTS notice_items (
  id BIGINT PRIMARY KEY,
  item_type TEXT NOT NULL,    -- 类型：1-通知，2-消息，3-待办
  title TEXT NOT NULL,        -- 标题
  description TEXT NOT NULL,  -- 描述
  datetime TEXT NOT NULL,     -- 时间戳
  status TEXT,                -- 状态（如 warning, danger）
  extra TEXT,                 -- 额外信息
  is_read BOOLEAN NOT NULL DEFAULT FALSE  -- 是否已读
)
```

## IPC 命令

| 命令名称                  | 说明             | 返回类型          |
| ------------------------- | ---------------- | ----------------- |
| `notice_get_unread_items` | 获取所有未读通知 | `Vec<NoticeItem>` |
| `notice_get_read_items`   | 获取所有已读通知 | `Vec<NoticeItem>` |
| `notice_mark_read`        | 标记通知为已读   | `boolean`         |

### notice_mark_read

请求载荷：

```json
{
  "id": 1
}
```

返回：`true` 表示标记成功，`false` 表示项目不存在。

## 数据模型

参见 [`models.rs`](d:/rust/iot/pure-admin-thin/src-tauri/src/notice/models.rs)：

| 字段          | 类型      | 说明                                     |
| ------------- | --------- | ---------------------------------------- |
| `id`          | `u64`     | 通知唯一标识                             |
| `type`        | `string`  | 类型：`"1"` 通知，`"2"` 消息，`"3"` 待办 |
| `title`       | `string`  | 通知标题                                 |
| `datetime`    | `string`  | 时间戳/日期                              |
| `description` | `string`  | 通知描述                                 |
| `status`      | `string?` | 状态（如 warning, danger）               |
| `extra`       | `string`  | 额外信息（如"进行中"、"即将超时"）       |
| `isRead`      | `boolean` | 是否已读                                 |

## 模块层次

```
commands.rs (Tauri IPC 命令层)
       │
       ▼
services.rs (业务逻辑层)
       │
       ▼
repository.rs (数据仓储层)
       │
       ▼
PostgreSQL 数据库
```

## 行为说明

1. **种子数据**：默认数据仅在表为空时插入
2. **标记已读**：不会物理删除记录，仅翻转 `is_read` 字段
3. **查询分离**：前端未读标签页查询未读记录，已读标签页查询已读记录
4. **初始化**：应用启动时调用 `init_notice_database()` 初始化表结构和种子数据

## 相关文档

- [数据库模块](../db/README.md)
- [鉴权模块](../auth/README.md)
- [Tauri 框架约束](../../docs/tauri-framework-constraints.md)
