# PostgreSQL + TimescaleDB Migration Plan

- Task: Replace sqlite + redb storage in `src-tauri` with PostgreSQL 17 + TimescaleDB 2.19
- Scope: Rust backend (`src-tauri/**`) plus developer docs/progress tracking
- Constraints: Tauri security checklist preserved, no new command exposure, DB URL from environment

## Steps
1. Switch dependency/runtime from sqlite/redb to postgres/sqlx-postgres.
2. Convert migrations and repository SQL to PostgreSQL syntax.
3. Replace notice redb repository with postgres table implementation.
4. Adapt tests and bootstrap configuration for postgres URL.
5. Run verification and sync docs (`docs/development-progress.md`).

## Progress
- [19:47:24] Created task plan and confirmed migration scope (done)
- [19:49:42] Replaced DB path store with PostgreSQL URL resolver and env fallback (done)
- [19:50:35] Switched db module from SqliteConnection to PgConnection and added URL config helpers (done)
- [19:50:49] Removed sqlite file bootstrap logic and kept migration pipeline over postgres connection (done)
- [19:52:04] Migrated migration executor from SqliteConnection to PgConnection and updated migration log SQL for PostgreSQL (done)
- [19:53:58] Rewrote migrations 0001-0006 to PostgreSQL syntax and enabled TimescaleDB extension bootstrap (done)
- [19:54:44] Replaced Cargo dependencies: sqlx sqlite->postgres and removed redb crate (done)
- [19:57:42] Replaced notice redb repository with PostgreSQL table-backed implementation and updated repository tests (done)
- [19:58:08] Updated Tauri setup to configure PostgreSQL URL and removed notice redb path bootstrap (done)
- [19:59:10] Updated Rust test initializers to use PostgreSQL test URL instead of sqlite temp files (done)
- [20:01:22] Rebuilt db/tests.rs for PostgreSQL (isolated schema per test, postgres placeholders, information_schema checks) (done)
- [20:02:14] Removed obsolete notice redb path API export and kept notice module on PostgreSQL-only initialization (done)
- [20:03:03] Converted repository SQL placeholders/aggregations for PostgreSQL and replaced user insert id retrieval with RETURNING (done)
- [20:03:18] Applied rustfmt on src-tauri after migration edits (done)
- [20:05:16] Fixed PostgreSQL test borrow semantics (&mut reborrows) and removed unused Tauri import warning (done)
- [20:05:35] Re-ran rustfmt after fixing db test borrow issues (done)
- [20:07:02] Provisioned local PostgreSQL databases and TimescaleDB extension; aligned default/test DB URLs with provided postgres credentials (done)
- [20:07:18] Re-ran rustfmt after default URL adjustments (done)
- [20:08:20] Removed UTF-8 BOM from migration SQL files to fix PostgreSQL parser error on include_str! scripts (done)
- [20:09:20] Switched migration script execution to sqlx::raw_sql for PostgreSQL multi-statement compatibility (done)
- [20:09:36] Re-ran rustfmt after raw_sql migration executor update (done)
- [20:10:37] Removed extension DDL from schema script to prevent parallel test race; fixed migration existence checks to decode SELECT 1 as INT4 (done)
- [20:10:45] Updated PostgreSQL db test expectation for route title to UTF-8 Chinese literal (done)
- [20:13:12] Normalized Postgres integer decoding paths (admin flags/meta_rank) and added pg_advisory_lock around init migrations to prevent concurrent bootstrap races (done)
- [20:13:20] Rewrote db/tests.rs with clean UTF-8-safe assertions and legacy-title checks to avoid locale/encoding noise (done)
- [20:13:33] Re-ran rustfmt after bootstrap lock and repository decode updates (done)
- [20:13:58] Recreated pure_admin_thin/pure_admin_thin_test databases and re-enabled TimescaleDB extension for clean validation baseline (done)
- [20:14:46] Fixed remaining SELECT 1 decoder mismatch in admin repository (INT4 -> i32) for admin command path (done)
- [20:16:06] Added developer runtime note doc for PostgreSQL + TimescaleDB configuration and migration behavior (done)
- [20:16:58] Synced completion summary into docs/development-progress.md with verification evidence and changed file list (done)
- [20:18:24] Verified local DB engine versions: PostgreSQL 17.8, TimescaleDB extension 2.25.0 (note: not 2.19.x) (done)
