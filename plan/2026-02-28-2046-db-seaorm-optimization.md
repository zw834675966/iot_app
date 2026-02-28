# Task Plan - DB SeaORM Optimization

- Date: 2026-02-28
- Scope: `src-tauri/src/db` hybrid database access optimization (SeaORM for regular CRUD, sqlx/raw SQL for complex queries)

## Steps
1. Add characterization/guard tests for DB behaviors touched by repository refactor.
2. Introduce SeaORM runtime integration and entity modules for core tables (`users`, `user_roles`).
3. Refactor admin repository regular CRUD paths to SeaORM; retain complex aggregate queries in sqlx.
4. Keep auth repository complex route/profile queries on sqlx and add raw SQL boundary comments.
5. Run Rust tests and update docs/progress.

## Progress
- [20:46:00] Created task plan and scoped refactor boundaries (done)
- [20:46:55] Added failing DB test for SeaORM connection/query baseline (done)
- [20:47:24] Executed RED test; failed as expected (missing sea_orm crate and connect_orm_async) (done)
- [20:51:56] Added SeaORM dependency, connection entrypoint, and entity models for users/user_roles (done)
- [20:51:56] Refactored admin repository into SeaORM CRUD module + sqlx complex query module (done)
- [20:54:57] Verified GREEN test opens_seaorm_connection_for_postgres now passes (done)
- [20:54:57] Full cargo test blocked by unrelated deleted file src-tauri/src/auth/commands.rs in working tree (blocked)
- [20:55:51] Reviewed SeaORM docs (EntityTrait, ActiveModelTrait, ConnectionTrait, Statement/raw SQL) and mapped 80/20 usage boundary (done)
- [21:11:13] Restored src-tauri/src/auth/commands.rs and reran full cargo test (done)
- [21:11:13] Fixed test-only compatibility to set_database_url in auth commands tests (done)
- [21:11:13] Made admin command registration test usernames unique to avoid rerun collisions (done)
- [21:11:13] Full cargo test --manifest-path src-tauri/Cargo.toml passed (done)
