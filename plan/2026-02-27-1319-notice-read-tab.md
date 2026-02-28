# Task Plan - Add Read Tab for Notice Center

- Objective: Add a dedicated read list tab in notice center while keeping current unread workflow.
- Scope:
  - Rust/Tauri: expose IPC to list read notice items from redb.
  - Frontend/Vue: add `已读` tab and hide read-action button for read-list items.
  - Keep current behavior: clicking `已读` on unread item removes it from unread display.
- Constraints:
  - Tauri security checklist applies; no capability/permission/CSP/updater changes expected.
  - DBHub MCP policy is SQL-focused; this task uses redb KV store and Rust tests for validation.
- Verification: `cargo test --manifest-path src-tauri/Cargo.toml notice_ -- --nocapture` + `pnpm typecheck` + `pnpm lint`.

## Progress
- [13:19:00] Created task plan and confirmed scope (completed)
- [13:20:06] Added RED test for read-list retrieval in repository module (completed)
- [13:20:33] Verified RED failure: missing `list_read_notice_items_at` (completed)
- [13:22:03] Implemented backend read-list API and passed targeted test (completed)
- [13:23:58] Updated frontend notice tabs to include read list and hide read button in read tab (completed)
- [13:25:00] Ran verification: cargo notice tests + pnpm typecheck + pnpm lint (completed)
- [13:25:42] Synced module/doc updates and appended development progress entry (completed)
- [13:27:31] Ran full backend verification: `cargo test --manifest-path src-tauri/Cargo.toml` (completed)