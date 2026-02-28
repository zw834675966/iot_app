# Task Plan - redb Notice Center with Read State

- Objective: Replace hardcoded notice/message/todo data with `redb`-backed storage and add "mark as read" behavior.
- Scope:
  - Rust/Tauri: add `redb` storage for notice items, expose IPC commands to list unread items and mark one as read.
  - Frontend/Vue: replace hardcoded notice data source with IPC fetch and add "已读" button per item.
  - Remove hardcoded list data from frontend source file.
- Constraints:
  - DBHub MCP required by policy, but current `dbhub` server is unavailable in this session (startup handshake failed). Use repository tests and static verification as fallback.
  - Tauri security checklist applies (capabilities/permissions/command exposure/CSP/updater/version sync).
- Verification: Rust tests + `pnpm typecheck` + `pnpm lint` + targeted grep checks for removed hardcoded notices.

## Progress
- [13:04:27] Created plan and recorded DBHub MCP unavailability with fallback verification path (completed)
- [13:09:50] Added Rust notice module with redb-backed storage, seed data, unread query, and mark-read commands wired into Tauri invoke handler (completed)
- [13:09:50] Ran RED/GREEN backend verification via `cargo test --manifest-path src-tauri/Cargo.toml notice_ -- --nocapture` (2 tests passed) (completed)
- [13:12:13] Replaced hardcoded frontend notice source with dynamic IPC fetch and per-item mark-read action, including notice API wrapper and list/item event flow (completed)
- [13:12:13] Added `src-tauri/src/notice/README.md` developer documentation for redb-backed notice module and commands (completed)
- [13:12:13] Ran `cargo test --manifest-path src-tauri/Cargo.toml` successfully (18 passed) (completed)
- [13:12:13] Ran `pnpm typecheck` successfully (completed)
- [13:12:13] Ran `pnpm lint` successfully (completed)
- [13:13:06] Synced final summary to `docs/development-progress.md` (completed)
