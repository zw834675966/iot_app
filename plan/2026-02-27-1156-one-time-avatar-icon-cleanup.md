# 2026-02-27 11:56 - One-time cleanup migration for legacy external avatar/online icon

- Objective: add a one-time DB data-fix migration that cleans legacy external avatar URLs and online icon values in existing SQLite databases.
- Scope:
  - add one SQL migration file under `src-tauri/src/db/migrations/`
  - update `src-tauri/src/db/mod.rs` to run one-time migration with migration log guard
  - update `src-tauri/src/db/README.md` and `docs/development-progress.md`
- Constraints review (Tauri):
  - capabilities/permissions: no change (DB local data patch only)
  - command exposure: no new command
  - async/state/CSP/updater: no change
  - version sync: no dependency change
- DBHub status:
  - DBHub MCP tools/resources are unavailable in this session; apply repository fallback (local SQL + cargo test).
- Verification plan:
  - `cargo test --manifest-path src-tauri/Cargo.toml`

## Progress
- [11:56:45] Created plan file and fixed scope to one-time legacy data cleanup migration (done)
- [11:56:51] Added SQL migration 0003 to clean legacy external avatars and online icons (done)
- [11:57:24] Wired one-time data-fix execution into db::init_database with app_migrations guard and added unit test for single-run behavior (done)
- [11:57:39] Updated db README with one-time legacy cleanup migration and app_migrations tracking notes (done)
- [11:58:19] Verified Rust backend changes with cargo test (15 passed; 0 failed) (done)
- [11:58:42] Synced final task summary into docs/development-progress.md (done)
