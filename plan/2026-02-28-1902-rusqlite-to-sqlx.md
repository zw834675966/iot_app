# Task Plan - Replace `rusqlite` with `launchbadge/sqlx`

- Objective: Replace Rust SQLite dependency from `rusqlite` to `sqlx` in `src-tauri` and perform equivalent code migration with tests passing.
- Scope: `src-tauri/**` Rust backend + task documentation sync.
- Constraints:
  - Keep existing IPC command names and payload contracts unchanged.
  - Follow Tauri constraints with no capability/permission/CSP/updater expansion.

## Plan
1. Baseline and RED step: swap dependency to `sqlx` and run targeted Rust checks to capture expected failures.
2. Migrate DB layer (`db/mod.rs`, `bootstrap.rs`, `migrations.rs`) to async `sqlx` connection/pool usage.
3. Migrate repositories (`auth_repository.rs`, `admin_repository.rs`) from `rusqlite` APIs to `sqlx` query APIs.
4. Keep service/command call contracts unchanged via sync wrapper around internal async SQL execution.
5. Update and fix tests to use `sqlx` and pass.
6. Run verification commands and update docs/progress log.

## Progress Log
- [19:02:30] Created task plan file and locked migration scope (in_progress)
- [19:04:16] Updated Cargo.toml dependency from rusqlite to sqlx (+tokio runtime) for RED step (completed)
- [19:05:35] Ran RED verification: cargo check fails with unresolved rusqlite imports across db modules (completed)
- [19:07:32] Migrated db core modules (mod/bootstrap/migrations) to sqlx + async internals with sync wrappers (completed)
- [19:10:46] Replaced auth/admin repositories and db tests with sqlx query/transaction equivalents (completed)

- [19:11:16] Fixed file encoding issue by converting touched Rust files back to UTF-8 (completed)
- [19:11:47] Fixed sqlx transaction trait import and reached cargo check green (completed)
- [19:13:56] Enabled sqlx create_if_missing for file-based SQLite and fixed test init failures (completed)
- [19:13:56] Ran GREEN verification: cargo test passed (31 tests + doctests) (completed)
- [19:15:09] Updated README docs to reflect sqlx dependency (completed)
- [19:15:09] Ran final verification: cargo fmt + cargo test both passed (completed)
- [19:16:45] Synced task result into docs/development-progress.md (completed)
- [19:17:29] Clarified execution plan: preserved sync service/command contracts via db sync wrappers (completed)
