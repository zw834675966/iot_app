# Notice Module (redb)

This module provides the Tauri-side notice center data source using `redb` and exposes IPC commands for frontend consumption.

## Scope

- Stores `通知` / `消息` / `待办` items in local `redb`.
- Supports unread filtering (`isRead = false`).
- Supports read filtering (`isRead = true`).
- Supports "mark as read" updates (`isRead = true`).

## Runtime DB Location

- File name: `pure-admin-thin-notice.redb`
- Resolved at startup in [`lib.rs`](d:/rust/iot/pure-admin-thin/src-tauri/src/lib.rs), under app data directory `.../db/`.

## Commands

- `notice_get_unread_items`
  - Return all unread notice items.
- `notice_get_read_items`
  - Return all read notice items.
- `notice_mark_read`
  - Input payload: `{ id: number }`
  - Marks one record as read and returns whether the item existed.

## Data Model

See [`models.rs`](d:/rust/iot/pure-admin-thin/src-tauri/src/notice/models.rs):

- `id`
- `type` (`"1"` notice, `"2"` message, `"3"` todo)
- `title`
- `datetime`
- `description`
- `status` (optional)
- `extra` (optional)
- `isRead` (boolean)

## Behavior Notes

- Default seed data is written only when table is empty.
- Marking as read does not physically delete records; it flips `isRead`.
- Frontend unread tabs query unread records, and read tab queries read records.