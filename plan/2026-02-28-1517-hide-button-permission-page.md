# Task Plan: Hide button-permission page

## Objective
Hide the "按钮权限" page from dynamic routes by database migration so it applies to both new and existing local databases.

## Steps
1. Add failing test proving `/permission/button` route still exists (RED).
2. Implement migration to remove button-permission route tree and wire it into startup migrations.
3. Update progress docs for this task.
4. Re-run verification (GREEN) and sync progress docs.

## Progress Log
- [15:17:03] Loaded required skills and traced route source to SQLite dynamic route seed/migrations (in_progress).
- [15:17:03] Checked `sqlite_tools` MCP availability; startup handshake failed in this session, will use test-based SQL verification fallback (in_progress).
- [15:20:44] Added RED test `hides_button_permission_routes` in `src-tauri/src/db/tests.rs` expecting button-permission routes to be absent (in_progress).
- [15:22:52] Added migration `0006_hide_button_permission_route.sql`, wired it in `migrations.rs`/`bootstrap.rs`, and updated DB tests to exercise the new one-time migration (in_progress).
- [15:24:10] Verified RED->GREEN with `cargo test --manifest-path src-tauri/Cargo.toml hides_button_permission_routes -- --nocapture` and full `cargo test --manifest-path src-tauri/Cargo.toml` (both passed) (completed).
- [15:25:36] Synced completion summary into `docs/development-progress.md` (completed).
