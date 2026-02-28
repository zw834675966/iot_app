# 2026-02-27-1017-auth-sqlite-storage

## Objective
- Replace hardcoded auth user profile and async route data with SQLite-backed storage.
- Add a dedicated db module/folder to manage schema bootstrap and query access.
- Include a device registry table in SQLite as the baseline for device registration data.

## Scope
- Add src-tauri/src/db/ for SQLite connection, schema initialization, and seed data.
- Refactor src-tauri/src/auth/services.rs to query user profile and routes from SQLite.
- Update auth command flow/tests to use database-backed services.
- Update docs and development progress log.

## Checklist
- [x] Create task plan and timestamp timeline
- [x] Add SQLite dependency and db module
- [x] Move hardcoded users/routes into SQLite seed data
- [x] Refactor auth services/commands to read from SQLite
- [x] Add/adjust Rust tests for DB-backed behavior
- [x] Run verification (cargo test --manifest-path src-tauri/Cargo.toml)
- [x] Sync docs (src-tauri/src/auth/README.md, docs/development-progress.md)

## Progress Timeline
- [10:17:42] Created task plan file and initialized scope/checklist (done)
- [10:19:41] Added failing test login_rejects_unknown_user in auth command tests (done)
- [10:20:21] Ran RED check for login_rejects_unknown_user; failed as expected on hardcoded fallback user (done)
- [10:23:19] Added src-tauri/src/db module with SQLite schema, seed data, and auth/device query support (done)
- [10:23:19] Refactored auth services/commands to read user profile and async routes from SQLite (done)
- [10:23:19] Added startup DB initialization in src-tauri/src/lib.rs and Database error variant (done)
- [10:23:58] Added idempotent database path configuration to avoid test/runtime reconfiguration conflicts (done)
- [10:24:35] Updated auth/db module docs to reflect SQLite-backed user/route/device data flow (done)
- [10:24:54] Added SQLite artifact ignore rule in src-tauri/.gitignore (done)
- [10:26:01] Ran full cargo test; found DB path reconfiguration failure across test modules (done)
- [10:26:01] Made DB path setup idempotent and removed unused route field warning (done)
- [10:26:59] Re-ran full cargo test; all tests passed (done)

## Verification
- command: cargo test --manifest-path src-tauri/Cargo.toml
- result: passed, 11 passed; 0 failed.

## Completion
- status: done
- follow-up: add auth/device CRUD commands on top of the new SQLite tables when needed.
- [10:28:52] Appended task completion entry to docs/development-progress.md (done)
