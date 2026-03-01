# Development Progress Log

Use this file as a required append-only task log after each completed task.

## Entry Template

```md
## YYYY-MM-DD - Task Title

- Scope:
- Changed files:
- Verification:
  - command:
  - result:
- Documentation updated:
- Next step:
```

## 2026-02-27 - Establish mandatory AI coding skill workflow

- Scope:
  - Added project-level skill workflow and enforced skill-first coding rules.
  - Added module size, task granularity, and mandatory progress/doc update rules.
  - Added deployment best-practice strategy based on official docs.
- Changed files:
  - `AGENTS.md`
  - `skills/project-aicode-workflow/SKILL.md`
  - `src/AGENTS.md`
  - `src-tauri/AGENTS.md`
  - `README.md`
  - `src-tauri/README.md`
  - `docs/deployment-strategy.md`
  - `docs/development-progress.md`
- Verification:
  - command: `git diff --name-only`
  - result: expected file set present.
- Documentation updated:
  - Updated `README.md` and `src-tauri/README.md` with workflow/doc links
  - Added `docs/deployment-strategy.md`
  - Added `docs/development-progress.md`
  - Updated root and module AGENTS rules
- Next step:
  - Apply this workflow to the next implementation task and append a new entry.

## 2026-02-27 - Add latest Tauri framework constraints for AI coding

- Scope:
  - Collected latest official Tauri v2 framework guidance and release baseline.
  - Added mandatory Tauri pre-edit thinking checklist and security/runtime constraints.
  - Bound these constraints into global and Rust-scoped AGENTS and project skill flow.
- Changed files:
  - `AGENTS.md`
  - `src/AGENTS.md`
  - `src-tauri/AGENTS.md`
  - `src-tauri/src/AGENTS.md`
  - `skills/project-aicode-workflow/SKILL.md`
  - `docs/tauri-framework-constraints.md`
  - `README.md`
  - `src-tauri/README.md`
  - `docs/development-progress.md`
- Verification:
  - command: `git diff --name-only`
  - result: Tauri constraints and rule-entry files are present in diff.
- Documentation updated:
  - Added `docs/tauri-framework-constraints.md` with source-backed mandatory checklist.
  - Updated AGENTS and README entries to force the constraints in AI code flow.
- Next step:
  - Enforce this checklist on the next `src-tauri` code change and verify with `cargo test`.

## 2026-02-27 10:08 - Add plan directory and timestamp sync workflow

- Scope:
  - Added a root `plan/` directory for per-task planning and time-sequenced progress tracking.
  - Updated project skill rules to require timestamped sync updates during implementation.
  - Added reusable plan template and task-level plan record.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1007-plan-folder-skill-sync.md`
- Changed files:
  - `skills/project-aicode-workflow/SKILL.md`
  - `plan/README.md`
  - `plan/TEMPLATE.md`
  - `plan/2026-02-27-1007-plan-folder-skill-sync.md`
  - `docs/development-progress.md`
- Verification:
  - command: `git status --short -- plan skills/project-aicode-workflow/SKILL.md docs/development-progress.md`
  - result: expected changes detected (`docs/development-progress.md`, `skills/project-aicode-workflow/SKILL.md`, and new `plan/` directory).
- Documentation updated:
  - Added `plan/README.md` and `plan/TEMPLATE.md`
  - Updated workflow rule in `skills/project-aicode-workflow/SKILL.md`
  - Logged this change in `docs/development-progress.md`
- Next step:
  - Use `plan/YYYY-MM-DD-HHMM-<task>.md` for each new task and append timestamp lines after each meaningful change.

## 2026-02-27 10:28 - Auth uses SQLite for user profile, device registry, and async routes

- Scope:
  - Introduced a new `src-tauri/src/db` module to own SQLite path config, schema bootstrap, and seed data.
  - Migrated auth user profile lookup and async (dynamic) route configuration from hardcoded Rust data to SQL queries.
  - Added device registry baseline table in SQLite (`device_registry`) as requested.
  - Updated Tauri startup to initialize DB in `setup` with app-data scoped DB file.
  - Tauri security boundary evaluation:
    - Capabilities/permissions: no capability expansion and no new plugin permissions were introduced.
    - Command exposure: no new commands were exposed; existing auth commands keep same names.
    - Runtime/state safety: no global mutable shared state introduced; DB path uses one-time initialization.
    - CSP/updater impact: no CSP or updater/signing configuration changes.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1017-auth-sqlite-storage.md`
- Changed files:
  - `src-tauri/Cargo.toml`
  - `src-tauri/Cargo.lock`
  - `src-tauri/.gitignore`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/core/error.rs`
  - `src-tauri/src/auth/services.rs`
  - `src-tauri/src/auth/commands.rs`
  - `src-tauri/src/auth/README.md`
  - `src-tauri/src/db/mod.rs`
  - `src-tauri/src/db/auth_repository.rs`
  - `src-tauri/src/db/README.md`
  - `plan/2026-02-27-1017-auth-sqlite-storage.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (`11 passed; 0 failed`).
- Documentation updated:
  - Added `src-tauri/src/db/README.md`
  - Updated `src-tauri/src/auth/README.md` with SQLite-backed source notes
  - Added this entry to `docs/development-progress.md`
- Next step:
  - Add device registration CRUD commands/services on top of `device_registry` when the API contract is ready.

## 2026-02-27 10:41 - Extract DB schema/seed SQL into migration files

- Scope:
  - Split inline SQLite schema and seed SQL out of `src-tauri/src/db/mod.rs`.
  - Added standalone migration scripts and loaded them from files during DB initialization.
  - Kept auth login and async route behavior unchanged.
  - Tauri security boundary evaluation:
    - Capabilities/permissions: unchanged (no capability expansion).
    - Command exposure: unchanged (no new commands).
    - Async/state safety: no new shared state; DB path init race handling improved for tests.
    - CSP/updater/version sync: unchanged.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1035-db-sql-files.md`
- Changed files:
  - `src-tauri/src/db/mod.rs`
  - `src-tauri/src/db/migrations/0001_schema.sql`
  - `src-tauri/src/db/migrations/0002_seed.sql`
  - `src-tauri/src/db/README.md`
  - `plan/2026-02-27-1035-db-sql-files.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (`12 passed; 0 failed`).
- Documentation updated:
  - Updated `src-tauri/src/db/README.md` with migration file loading notes.
  - Added this entry to `docs/development-progress.md`.
- Next step:
  - Optional: add migration version tracking table (e.g., `schema_migrations`) for future incremental DB upgrades.

## 2026-02-27 11:03 - Add mandatory DBHub MCP policy to AGENTS and project skill workflow

- Scope:
  - Added repository-level policy requiring DBHub MCP for database-related tasks.
  - Added Rust/Tauri scoped DBHub requirement in `src-tauri/AGENTS.md`.
  - Updated project workflow skill order to explicitly require DBHub MCP for schema/migration/seed/query work.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1101-dbhub-policy.md`
- Changed files:
  - `AGENTS.md`
  - `src-tauri/AGENTS.md`
  - `skills/project-aicode-workflow/SKILL.md`
  - `docs/development-progress.md`
- Verification:
  - command: `git diff --name-only -- AGENTS.md src-tauri/AGENTS.md skills/project-aicode-workflow/SKILL.md docs/development-progress.md`
  - result: expected docs/rules files present in diff.
- Documentation updated:
  - Updated root and Rust-scoped AGENTS instructions.
  - Updated `skills/project-aicode-workflow/SKILL.md`.
  - Appended this task entry.
- Next step:
  - Use DBHub MCP (`dbhub`) as the default DB inspection/SQL validation path in the next DB-related implementation task.

## 2026-02-27 11:10 - Document runtime database location in README

- Scope:
  - Added a dedicated README section that explains where the Tauri SQLite database is stored.
  - Documented the Windows path pattern and current local example path for quick troubleshooting.
  - Added source code references for path initialization and fallback path behavior.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1109-readme-db-location.md`
- Changed files:
  - `README.md`
  - `plan/2026-02-27-1109-readme-db-location.md`
  - `docs/development-progress.md`
- Verification:
  - command: `git diff --name-only -- README.md plan/2026-02-27-1109-readme-db-location.md docs/development-progress.md`
  - result: expected documentation files present in diff.
- Documentation updated:
  - Updated `README.md` with runtime SQLite path details.
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Optional: add the same DB location note to `src-tauri/README.md` for backend-focused onboarding.

## 2026-02-27 11:24 - Replace custom token strings with jsonwebtoken JWT

- Scope:
  - Replaced string-concatenated tokens with standard JWT generation via `jsonwebtoken`.
  - Added refresh token verification (signature/expiry/type) before issuing a new token pair.
  - Kept frontend IPC contract unchanged (`auth_login`, `auth_refresh_token` response fields remain the same).
  - Tauri security boundary evaluation:
    - Capabilities/permissions: unchanged (no capability file or permission expansion).
    - Command exposure: unchanged (no new command exported).
    - Runtime/state safety: unchanged shared-state model; JWT secret read via env/default and used in pure service functions.
    - CSP/updater/version sync: unchanged.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1117-auth-jwt-jsonwebtoken.md`
- Changed files:
  - `src-tauri/Cargo.toml`
  - `src-tauri/Cargo.lock`
  - `src-tauri/src/auth/services.rs`
  - `src-tauri/src/auth/commands.rs`
  - `src-tauri/src/auth/README.md`
  - `plan/2026-02-27-1117-auth-jwt-jsonwebtoken.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml refresh_rejects_ -- --nocapture`
  - result: passed (`2 passed; 0 failed`).
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (`14 passed; 0 failed`).
- Documentation updated:
  - Updated `src-tauri/src/auth/README.md` with JWT implementation and secret configuration notes.
  - Appended this task entry to `docs/development-progress.md`.
- Next step:
  - Move `PURE_ADMIN_JWT_SECRET` from dev fallback to environment-based configuration in packaging/CI for production builds.

## 2026-02-27 11:42 - Slim src for local-only desktop runtime

- Scope:
  - Removed dead `src/` modules and directives that were not referenced by runtime code.
  - Replaced runtime external links/resources in `src` with local/offline-safe content.
  - Trimmed npm dependencies and optimizeDeps entries corresponding to deleted code.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1138-local-thin-cleanup.md`
- Changed files:
  - `build/optimize.ts`
  - `package.json`
  - `pnpm-lock.yaml`
  - `src/main.ts`
  - `src/router/index.ts`
  - `src/directives/index.ts`
  - `src/layout/components/lay-footer/index.vue`
  - `src/layout/components/lay-notice/data.ts`
  - `src/README.md`
  - Deleted: `src/components/ReCol/index.ts`
  - Deleted: `src/components/RePureTableBar/index.ts`
  - Deleted: `src/components/RePureTableBar/src/bar.tsx`
  - Deleted: `src/directives/copy/index.ts`
  - Deleted: `src/directives/longpress/index.ts`
  - Deleted: `src/directives/optimize/index.ts`
  - Deleted: `src/plugins/echarts.ts`
  - Deleted: `src/utils/globalPolyfills.ts`
  - Deleted: `src/utils/http/index.ts`
  - Deleted: `src/utils/http/types.d.ts`
  - Deleted: `src/utils/localforage/index.ts`
  - Deleted: `src/utils/localforage/types.d.ts`
  - Deleted: `src/utils/preventDefault.ts`
  - Deleted: `src/utils/print.ts`
  - Deleted: `src/utils/propTypes.ts`
  - Deleted: `src/utils/sso.ts`
  - `plan/2026-02-27-1138-local-thin-cleanup.md`
  - `docs/development-progress.md`
- Verification:
  - command: `pnpm typecheck`
  - result: passed.
  - command: `pnpm lint`
  - result: passed.
- Documentation updated:
  - Updated `src/README.md` to match the slimmed local-only frontend baseline.
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Replace external avatar/icon seed values in `src-tauri/src/db/migrations/0002_seed.sql` with local/offline-safe defaults, then run `cargo test`.

## 2026-02-27 11:47 - Make DB seed values offline-safe

- Scope:
  - Replaced external avatar URLs in SQLite seed users with offline-safe empty values (frontend local avatar fallback).
  - Replaced dynamic route seed icon `ep:lollipop` with local offline-registered icon `ri/information-line`.
  - No Tauri command/capability/permission/CSP/updater changes.
  - Note: DBHub MCP server was not available in current session, so seed validation used repository tests as fallback.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1146-db-seed-offline-values.md`
- Changed files:
  - `src-tauri/src/db/migrations/0002_seed.sql`
  - `src-tauri/src/db/README.md`
  - `plan/2026-02-27-1146-db-seed-offline-values.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (`14 passed; 0 failed`).
- Documentation updated:
  - Updated `src-tauri/src/db/README.md` seed notes for offline-safe defaults.
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - If you also want strict local-only behavior on first run, add a one-time migration to normalize existing old DB rows that still contain external URLs.

## 2026-02-27 11:50 - Fix DBHub no-resource troubleshooting and Tauri DB location docs

- Scope:
  - Clarified Tauri local database runtime path wording in root `README.md`.
  - Added actionable troubleshooting for "no dbhub resources/tools" in `mcp.md`.
  - Added fallback verification path when MCP server is unavailable in the current session.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1150-readme-mcp-dbhub-fix.md`
- Changed files:
  - `README.md`
  - `mcp.md`
  - `plan/2026-02-27-1150-readme-mcp-dbhub-fix.md`
  - `docs/development-progress.md`
- Verification:
  - command: `git status --short -- README.md mcp.md plan/2026-02-27-1150-readme-mcp-dbhub-fix.md docs/development-progress.md`
  - result: expected status (`M README.md`, `M docs/development-progress.md`, `?? mcp.md`, `?? plan/2026-02-27-1150-readme-mcp-dbhub-fix.md`).
- Documentation updated:
  - Root `README.md` now includes a concrete PowerShell `Test-Path` self-check for the Tauri SQLite file.
  - `mcp.md` now contains a dedicated DBHub startup/connection troubleshooting section.
- Next step:
  - If needed, add the same troubleshooting snippet to `src-tauri/README.md` for backend onboarding consistency.

## 2026-02-27 - Remove frontend display of Copyright and link

- Scope:
  - Removed `lay-footer` component to stop displaying "Copyright 漏 2020-present 鑳芥簮绠＄悊绯荤粺" on the frontend.
  - Removed references to `LayFooter` in `lay-content/index.vue`.
  - Deleted the `src/layout/components/lay-footer` directory entirely.
- Changed files:
  - `src/layout/components/lay-content/index.vue`
  - Deleted: `src/layout/components/lay-footer/index.vue`
- Verification:
  - command: `pnpm typecheck && pnpm lint`
  - result: passed.
- Documentation updated:
  - Appended this task entry to `docs/development-progress.md`.
- Next step:
  - N/A

## 2026-02-27 11:58 - Add one-time legacy avatar/icon cleanup migration

- Scope:
  - Added one-time legacy data-fix migration to clean external avatar URLs and online icon values in existing SQLite databases.
  - Added migration guard table `app_migrations` so cleanup runs once per database.
  - Kept Tauri boundaries unchanged: no capabilities/permissions/commands/CSP/updater changes.
  - Note: DBHub MCP server was unavailable in this session, so SQL behavior verification used Rust tests fallback.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1156-one-time-avatar-icon-cleanup.md`
- Changed files:
  - `src-tauri/src/db/migrations/0003_legacy_offline_cleanup.sql`
  - `src-tauri/src/db/mod.rs`
  - `src-tauri/src/db/README.md`
  - `plan/2026-02-27-1156-one-time-avatar-icon-cleanup.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (`15 passed; 0 failed`).
- Documentation updated:
  - Updated `src-tauri/src/db/README.md` with one-time migration and tracking notes.
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - N/A

## 2026-02-27 12:24 - Remove login avatar icon

- Scope:
  - Removed the unnecessary avatar icon block from the login page UI.
  - Cleaned login static exports and login CSS avatar styles that became unused.
  - No Tauri/backend/database/capability/permission/CSP/updater changes.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1222-remove-login-avatar-icon.md`
- Changed files:
  - `src/views/login/index.vue`
  - `src/views/login/utils/static.ts`
  - `src/style/login.css`
  - `plan/2026-02-27-1222-remove-login-avatar-icon.md`
  - `docs/development-progress.md`
- Verification:
  - command: PowerShell assertion check for `<avatar class=\"avatar\" />` absence in `src/views/login/index.vue`
  - result: passed.
  - command: `pnpm typecheck`
  - result: passed.
  - command: `pnpm lint`
  - result: passed.
- Documentation updated:
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - N/A

## 2026-02-27 12:44 - Remove user avatar from top navigation

- Scope:
  - Removed user avatar rendering from top navigation user dropdown in vertical, horizontal, and mix layouts.
  - Removed now-unused avatar-related fields from `useNav` to prevent future navbar avatar usage.
  - Kept username display, logout dropdown behavior, and settings entry unchanged.
  - No Tauri/backend/database/capability/permission/CSP/updater changes.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1240-remove-navbar-user-avatar.md`
- Changed files:
  - `src/layout/components/lay-navbar/index.vue`
  - `src/layout/components/lay-sidebar/NavHorizontal.vue`
  - `src/layout/components/lay-sidebar/NavMix.vue`
  - `src/layout/hooks/useNav.ts`
  - `plan/2026-02-27-1240-remove-navbar-user-avatar.md`
  - `docs/development-progress.md`
- Verification:
  - command: nav-scope grep check for `userAvatar|avatarsStyle` in target navbar files
  - result: passed (no matches).
  - command: `pnpm typecheck`
  - result: passed.
  - command: `pnpm lint`
  - result: passed.
- Documentation updated:
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - N/A

## 2026-02-27 - Rewrite root README.md

- Scope:
  - Rewrote `README.md` to reflect the current state of the project as an offline-first Tauri desktop application ("能源管理系统").
  - Replaced outdated `vue-pure-admin` links and descriptions with correct architecture descriptions (Vue 3 + Tauri v2 + Rust + SQLite).
  - Maintained necessary technical architecture guidelines, database locations, and development commands.
  - Kept mandatory AI coding rules section intact.
- Changed files:
  - `README.md`
- Verification:
  - command: `git diff README.md`
  - result: Changes logically reflect the project state.
- Documentation updated:
  - `README.md` heavily updated.
  - Appended this task entry to `docs/development-progress.md`.
- Next step:
  - N/A

## 2026-02-27 12:50 - Remove avatar in notice panel items

- Scope:
  - Removed avatar rendering from notice panel list items.
  - Kept title, description, tag, and datetime display behavior unchanged.
  - No Tauri/backend/database/capability/permission/CSP/updater changes.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1248-remove-notice-panel-avatar.md`
- Changed files:
  - `src/layout/components/lay-notice/components/NoticeItem.vue`
  - `plan/2026-02-27-1248-remove-notice-panel-avatar.md`
  - `docs/development-progress.md`
- Verification:
  - command: grep check for `el-avatar|noticeItem.avatar` in `NoticeItem.vue`
  - result: passed (no matches).
  - command: `pnpm typecheck`
  - result: passed.
  - command: `pnpm lint`
  - result: passed.
- Documentation updated:
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - N/A

## 2026-02-27 - Rewrite Rust backend documentation

- Scope:
  - Deep read `src-tauri` structure, code (`src-tauri/src/*/*.rs`), and SQL migrations (`src-tauri/src/db/migrations/*.sql`).
  - Rewrote `src-tauri/README.md` to clarify the current DDD architecture (Commands -> Services -> Repository).
  - Translated and expanded `src-tauri/src/auth/README.md` to document the SQLite DB adoption, JWT standard, and token lifecycle in Chinese.
  - Rewrote `src-tauri/src/core/README.md` emphasizing the unified `ApiResponse` and `AppError` handling.
  - Translated and formatted `src-tauri/src/db/README.md` explaining the new `migrations` folder approach and offline-safe constraints.
- Changed files:
  - `src-tauri/README.md`
  - `src-tauri/src/auth/README.md`
  - `src-tauri/src/core/README.md`
  - `src-tauri/src/db/README.md`
- Verification:
  - command: `git diff src-tauri`
  - result: Markdown formatting is clean, content correctly matches the current source code state.
- Documentation updated:
  - All Markdown files inside `src-tauri` updated.
  - Appended this task entry to `docs/development-progress.md`.
- Next step:
  - N/A

## 2026-02-27 13:12 - Migrate notice/message/todo to redb with read-state flow

- Scope:
  - Added a new Rust `notice` module backed by `redb` for notice center data (通知/消息/待办).
  - Added Tauri IPC commands to list unread items and mark a notice as read.
  - Replaced frontend hardcoded notice list with IPC-driven data loading.
  - Added per-item 已读 button in notice panel; clicking marks item as read and removes it from current list.
  - Added `isRead` persistence flag in database records so read items are excluded from later unread queries.
  - Tauri security boundary check: no capability/permission/CSP/updater changes; only command surface extended with two notice commands.
  - DBHub MCP note: mandatory dbhub usage was attempted, but server startup/handshake failed in this session; verification used code + tests fallback.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1304-redb-notice-center-read-state.md`
- Changed files:
  - `src-tauri/Cargo.toml`
  - `src-tauri/Cargo.lock`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/notice/mod.rs`
  - `src-tauri/src/notice/models.rs`
  - `src-tauri/src/notice/repository.rs`
  - `src-tauri/src/notice/services.rs`
  - `src-tauri/src/notice/commands.rs`
  - `src-tauri/src/notice/README.md`
  - `src/api/notice.ts`
  - `src/layout/components/lay-notice/data.ts`
  - `src/layout/components/lay-notice/index.vue`
  - `src/layout/components/lay-notice/components/NoticeList.vue`
  - `src/layout/components/lay-notice/components/NoticeItem.vue`
  - `plan/2026-02-27-1304-redb-notice-center-read-state.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml notice_ -- --nocapture`
  - result: passed (2 passed; 0 failed).
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (18 passed; 0 failed).
  - command: `pnpm typecheck`
  - result: passed.
  - command: `pnpm lint`
  - result: passed.
- Documentation updated:
  - Added `src-tauri/src/notice/README.md` for `redb` notice storage and IPC usage.
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Optional: add a dedicated “已读列表/回收站” tab if you need to review already read items.

## 2026-02-27 13:19 - Add read tab in notice panel with redb read-list query

- Scope:
  - Added backend read-list query for notice center (`isRead = true`) with new Tauri IPC command.
  - Added frontend `已读` tab to display read items.
  - Kept existing unread behavior: clicking `已读` removes item from unread display; item appears in read tab after reload.
  - Tauri security boundary evaluation:
    - Capabilities/permissions: unchanged.
    - Command exposure: extended with one read-only command (`notice_get_read_items`).
    - Async/state safety: unchanged shared state model.
    - CSP/updater/version sync: unchanged.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1319-notice-read-tab.md`
- Changed files:
  - `src-tauri/src/notice/repository.rs`
  - `src-tauri/src/notice/services.rs`
  - `src-tauri/src/notice/commands.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/notice/README.md`
  - `src/api/notice.ts`
  - `src/layout/components/lay-notice/data.ts`
  - `src/layout/components/lay-notice/index.vue`
  - `src/layout/components/lay-notice/components/NoticeList.vue`
  - `src/layout/components/lay-notice/components/NoticeItem.vue`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml mark_read_item_appears_in_read_list -- --nocapture`
  - result: passed (1 passed; 0 failed).
  - command: `cargo test --manifest-path src-tauri/Cargo.toml notice_ -- --nocapture`
  - result: passed (1 passed; 0 failed; filtered run).
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (19 passed; 0 failed).
  - command: `pnpm typecheck`
  - result: passed.
  - command: `pnpm lint`
  - result: passed.
- Documentation updated:
  - Updated `src-tauri/src/notice/README.md` with read-list command and behavior.
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Optional: add "全部已读" batch action in notice panel.

## 2026-02-27 13:31 - Install vuejs-ai skills + superpowers and enforce Vue skill routing constraints

- Scope:
  - Installed/updated `vuejs-ai/skills` with the official command and confirmed all Vue skills are available to Codex.
  - Followed superpowers official native-discovery install model (`~/.codex/superpowers` + `~/.agents/skills/superpowers` junction).
  - Added project constraints so Vue tasks must auto-route to the best matching Vue skill before editing.
  - Added a dedicated setup/routing guide for repeatable installation, verification, and maintenance.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1331-install-vue-superpowers-skills.md`
- Changed files:
  - `AGENTS.md`
  - `src/AGENTS.md`
  - `skills/project-aicode-workflow/SKILL.md`
  - `docs/ai-skills-usage.md`
  - `README.md`
  - `plan/2026-02-27-1331-install-vue-superpowers-skills.md`
  - `docs/development-progress.md`
- Verification:
  - command: `npx skills add vuejs-ai/skills --yes --global`
  - result: passed; installation summary reported 8 Vue skills installed.
  - command: `git -C "$env:USERPROFILE\\.codex\\superpowers" pull`
  - result: passed (`Already up to date.`).
  - command: `cmd /c mklink /J "$env:USERPROFILE\\.agents\\skills\\superpowers" "$env:USERPROFILE\\.codex\\superpowers\\skills"`
  - result: passed; junction recreated successfully.
  - command: `Get-ChildItem "$env:USERPROFILE\\.agents\\skills"`
  - result: passed; includes all Vue skills and `superpowers`.
  - command: `pnpm lint`
  - result: passed.
- Documentation updated:
  - Added `docs/ai-skills-usage.md` (official install + routing + maintenance).
  - Updated root and frontend AGENTS constraints and project workflow skill rules.
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Restart Codex CLI once so this session picks up any newly refreshed skill metadata.

## 2026-02-27 13:47 - Skills and MCP health check, plus DBHub MCP startup fix

- Scope:
  - Audited project-required skills baseline (`project-aicode-workflow`, `superpowers`, `vuejs-ai/skills`, and required routing skill files).
  - Re-ran official Vue skills installation command to ensure baseline is healthy.
  - Verified superpowers native discovery junction integrity.
  - Reproduced DBHub MCP handshake failure and fixed local MCP startup command.
  - Confirmed DBHub bridge handshake works with the project SQLite DSN.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1347-skills-mcp-health-check.md`
- Changed files:
  - `C:/Users/zw/.codex/config.toml`
  - `plan/2026-02-27-1347-skills-mcp-health-check.md`
  - `docs/development-progress.md`
- Verification:
  - command: `npx skills add vuejs-ai/skills --yes --global`
  - result: passed; installer reported 8 Vue skills installed.
  - command: `git -C "$env:USERPROFILE\\.codex\\superpowers" pull`
  - result: passed (`Already up to date.`).
  - command: `Get-Item "$env:USERPROFILE\\.agents\\skills\\superpowers" | Select-Object FullName,LinkType,Target`
  - result: passed; confirmed junction to `C:\Users\zw\.codex\superpowers\skills`.
  - command: `list_mcp_resources(server=\"dbhub\")`
  - result: reproduced failure in current session (`MCP startup failed: connection closed: initialize response`).
  - command: direct MCP framed probe against `C:/Users/zw/.codex/scripts/dbhub-stdio-bridge.cjs --dsn sqlite:///C:/Users/zw/AppData/Roaming/com.pureadmin.thin/db/pure-admin-thin.sqlite3`
  - result: passed; valid DBHub `initialize` response received.
  - command: direct MCP framed `tools/list` probe against same bridge and DSN
  - result: passed; returned `execute_sql` and `search_objects`.
- Documentation updated:
  - Added this task entry to `docs/development-progress.md`.
  - Added detailed task log at `plan/2026-02-27-1347-skills-mcp-health-check.md`.
- Next step:
  - Restart Codex CLI, then re-check DBHub MCP exposure in a fresh session.

## 2026-02-27 14:08 - Replace DBHub MCP with SQLite Tools MCP

- Scope:
  - Replaced local database MCP server from `dbhub` to `sqlite_tools` (`mcp-sqlite-tools` + stdio bridge).
  - Migrated repository-level DB MCP policy/docs from DBHub wording to SQLite Tools MCP wording.
  - Re-validated bridge-level MCP handshake and tool discovery against the new server.
- Related plan file in `plan/`:
  - `plan/2026-02-27-1408-replace-dbhub-mcp.md`
- Changed files:
  - `C:/Users/zw/.codex/config.toml`
  - `C:/Users/zw/.codex/scripts/sqlite-tools-stdio-bridge.cjs`
  - `AGENTS.md`
  - `src-tauri/AGENTS.md`
  - `skills/project-aicode-workflow/SKILL.md`
  - `src-tauri/src/db/README.md`
  - `mcp.md`
  - `plan/2026-02-27-1408-replace-dbhub-mcp.md`
  - `docs/development-progress.md`
- Verification:
  - command: `Get-Content "$env:USERPROFILE/.codex/config.toml" -Raw`
  - result: passed; contains `[mcp_servers.sqlite_tools]` with bridge path and SQLite env config.
  - command: direct framed MCP probe to `sqlite-tools-stdio-bridge.cjs` (initialize + `tools/list`)
  - result: passed (`initialize_ok=true`, `tools_count=19`; includes `execute_read_query`, `execute_write_query`, `execute_schema_query`, `list_tables`, `describe_table`).
  - command: `rg -n "dbhub|sqlite_tools|SQLite Tools MCP|DBHub" AGENTS.md src-tauri/AGENTS.md skills/project-aicode-workflow/SKILL.md src-tauri/src/db/README.md mcp.md`
  - result: passed; DB policies/docs switched to `sqlite_tools` in target files.
  - command: `list_mcp_resources(server="sqlite_tools")`
  - result: current session still reports `unknown MCP server 'sqlite_tools'` (fresh Codex session reload still required).
- Documentation updated:
  - Updated DB MCP policy docs (`AGENTS.md`, `src-tauri/AGENTS.md`, workflow `SKILL.md`, `src-tauri/src/db/README.md`, `mcp.md`).
  - Added this task entry and synchronized plan status.
- Next step:
  - Restart Codex CLI once, then re-check `sqlite_tools` MCP visibility in the new session.

## 2026-02-28 08:37 - Fix SQLite schema syntax crash and refresh baseline-browser-mapping

- Scope:
  - Fixed startup crash caused by malformed SQL in migration `0001_schema.sql` (`permissions` table statement was not closed).
  - Updated `baseline-browser-mapping` to latest as a direct dev dependency for baseline data freshness.
  - Tauri security boundary evaluation:
    - Capabilities/permissions: unchanged.
    - Command exposure: unchanged.
    - Async/state safety: unchanged.
    - CSP/updater/version sync: unchanged.
  - SQLite Tools MCP note: `sqlite_tools` was not exposed in this session, so verification used Rust test execution fallback.
- Related plan file in `plan/`:
  - `plan/2026-02-28-0834-fix-db-schema-and-baseline-mapping.md`
- Changed files:
  - `src-tauri/src/db/migrations/0001_schema.sql`
  - `package.json`
  - `pnpm-lock.yaml`
  - `plan/2026-02-28-0834-fix-db-schema-and-baseline-mapping.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: unit tests passed, but doctests failed in pre-existing `src/auth/services.rs` doc blocks.
  - command: `cargo test --manifest-path src-tauri/Cargo.toml --lib`
  - result: passed (`19 passed; 0 failed`).
  - command: `pnpm typecheck`
  - result: passed.
- Documentation updated:
  - Added this task entry to `docs/development-progress.md`.
  - Added synchronized task timeline in `plan/2026-02-28-0834-fix-db-schema-and-baseline-mapping.md`.
- Next step:
  - Optional: clean up failing doctest blocks in `src-tauri/src/auth/services.rs` so full `cargo test` becomes green.

## 2026-02-28 08:47 - Refactor db mod to interface-only entrypoints

- Scope:
  - Extracted implementation logic out of `src-tauri/src/db/mod.rs`.
  - Kept `mod.rs` as thin entrypoint that only wires modules and forwards interface calls.
  - Split responsibilities into path management, migration execution, and bootstrap orchestration modules.
  - Moved db unit tests into dedicated `src-tauri/src/db/tests.rs`.
  - Tauri security boundary evaluation:
    - Capabilities/permissions: unchanged.
    - Command exposure: unchanged.
    - Async/state safety: unchanged (`OnceLock` path state kept, no new shared mutable globals).
    - CSP/updater/version sync: unchanged.
- Related plan file in `plan/`:
  - `plan/2026-02-28-0845-db-mod-thin-interface.md`
- Changed files:
  - `src-tauri/src/db/mod.rs`
  - `src-tauri/src/db/path_store.rs`
  - `src-tauri/src/db/migrations.rs`
  - `src-tauri/src/db/bootstrap.rs`
  - `src-tauri/src/db/tests.rs`
  - `plan/2026-02-28-0845-db-mod-thin-interface.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml --lib`
  - result: passed (`19 passed; 0 failed`).
- Documentation updated:
  - Added this task entry to `docs/development-progress.md`.
  - Synced timeline in `plan/2026-02-28-0845-db-mod-thin-interface.md`.
- Next step:
  - Optional: if you want stricter encapsulation, move migration tests into `migrations.rs` internal test module and remove `pub(crate)` exposure of helper functions.

## 2026-02-28 09:53 - Add notice redb database location to root README

- Scope:
  - Clarified that notice center data (通知/消息/待办) is stored in a separate local `redb` file instead of `pure-admin-thin.sqlite3`.
  - Added runtime path and Windows sample path for notice database in root `README.md`.
  - Added a PowerShell quick-check command for the notice database file.
  - Tauri security boundary evaluation:
    - Capabilities/permissions: unchanged.
    - Command exposure: unchanged.
    - Async/state safety: unchanged.
    - CSP/updater/version sync: unchanged.
- Related plan file in `plan/`:
  - `plan/2026-02-28-0953-readme-notice-redb-location.md`
- Changed files:
  - `README.md`
  - `plan/2026-02-28-0953-readme-notice-redb-location.md`
  - `docs/development-progress.md`
- Verification:
  - command: `rg -n "通知中心 redb|pure-admin-thin-notice.redb" README.md`
  - result: passed; root README now contains notice center redb location and quick-check guidance.
- Documentation updated:
  - Updated root `README.md` database section to include notice center redb storage details.
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Optional: align the same redb location note into `src-tauri/README.md` runtime storage section for backend-only onboarding.

## 2026-02-28 09:59 - Hide exception page from active menu routes

- Scope:
  - Updated error route module to make "异常页面" passive-only by setting `meta.showLink: false`.
  - Kept exception pages routable (`/error/403`, `/error/404`, `/error/500`) for guard-triggered and manual error navigation.
  - No backend/Tauri/database/capability/permission/CSP/updater changes.
- Related plan file in `plan/`:
  - `plan/2026-02-28-0959-hide-error-route-menu.md`
- Changed files:
  - `src/router/modules/error.ts`
  - `plan/2026-02-28-0959-hide-error-route-menu.md`
  - `docs/development-progress.md`
- Verification:
  - command: `pnpm typecheck`
  - result: passed.
  - command: `pnpm lint`
  - result: passed.
  - command: `rg -n "showLink:\\s*false" src/router/modules/error.ts`
  - result: passed; error route meta now explicitly hides menu entry.
- Documentation updated:
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Optional: if you also want error pages hidden from tags view when opened, set `meta.showLink: false` on each error child route.

## 2026-02-28 10:05 - Add parent menu icon fallback for collapsed sidebar consistency

- Scope:
  - Added a dynamic-route normalization fallback so any parent directory route (route with children) gets a default icon when `meta.icon` is missing.
  - Kept backend database data unchanged; fix is frontend-side and takes effect on current returned routes.
  - No Tauri/backend/database/capability/permission/CSP/updater changes.
- Related plan file in `plan/`:
  - `plan/2026-02-28-1005-parent-menu-icon-fallback.md`
- Changed files:
  - `src/router/utils.ts`
  - `plan/2026-02-28-1005-parent-menu-icon-fallback.md`
  - `docs/development-progress.md`
- Verification:
  - command: `pnpm typecheck`
  - result: first run failed due `RouteMeta.title` typing on empty `meta`; after adding fallback `title`, rerun passed.
  - command: `pnpm lint`
  - result: passed.
- Documentation updated:
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Optional: if you want different icons by directory type, add a backend-provided icon mapping policy instead of using a single frontend fallback icon.

## 2026-02-28 10:30 - Draft admin registration + device-reserve operation document (no code changes)

- Scope:
  - Produced a review-first operations/design document for admin-only user registration, account validity lifecycle, and reserved user-device config interface.
  - Kept this task documentation-only; no frontend/backend logic changes were applied.
  - Captured SQLite-first deployment order, rollback strategy, and acceptance checklist.
- Related plan file in `plan/`:
  - `plan/2026-02-28-1030-admin-register-device-reserve-doc.md`
- Changed files:
  - `docs/admin-user-registration-and-device-reserve-operation.md`
  - `plan/2026-02-28-1030-admin-register-device-reserve-doc.md`
  - `docs/development-progress.md`
- Verification:
  - command: `rg -n "Admin 用户注册|到期自动注销|设备配置" docs/admin-user-registration-and-device-reserve-operation.md`
  - result: passed; target sections exist and align with requested scope.
  - command: `git status --short -- docs/admin-user-registration-and-device-reserve-operation.md plan/2026-02-28-1030-admin-register-device-reserve-doc.md docs/development-progress.md`
  - result: passed; only documentation/plan files for this task are touched.
  - command: `list_mcp_resources(server=\"sqlite_tools\")`
  - result: failed in current session (`MCP startup failed: ... initialize response`); SQL validation must be rerun in a healthy MCP session before code implementation.
- Documentation updated:
  - Added `docs/admin-user-registration-and-device-reserve-operation.md`.
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - After your review approval, execute implementation in phases: DB migration -> backend commands -> frontend page logic.

## 2026-02-28 10:32 - Finalize confirmed decisions and add reserved API contract doc

- Scope:
  - Applied confirmed business decisions to the operation document:
    - roles: multi-select
    - phone: optional (no unique requirement)
    - account renewal: supported
    - expiration handling: lazy check + startup compensation
    - device config: interface contract only (no table/code implementation in this phase)
  - Added one standalone API contract document for admin registration/renew and reserved device-scope APIs.
  - Documentation-only change; no frontend/backend runtime code changes.
- Related plan file in `plan/`:
  - `plan/2026-02-28-1030-admin-register-device-reserve-doc.md`
- Changed files:
  - `docs/admin-user-registration-and-device-reserve-operation.md`
  - `docs/admin-user-device-reserve-api-contract.md`
  - `plan/2026-02-28-1030-admin-register-device-reserve-doc.md`
  - `docs/development-progress.md`
- Verification:
  - command: `rg -n "多选|可选|续期|惰性 \\+ 启动补偿|仅保留接口契约" docs/admin-user-registration-and-device-reserve-operation.md`
  - result: passed.
  - command: `rg -n "roles|phone|renew|RESERVED_API_NOT_IMPLEMENTED|user_device_scope_get|user_device_scope_upsert" docs/admin-user-device-reserve-api-contract.md`
  - result: passed.
  - command: `git status --short -- docs/admin-user-registration-and-device-reserve-operation.md docs/admin-user-device-reserve-api-contract.md plan/2026-02-28-1030-admin-register-device-reserve-doc.md docs/development-progress.md`
  - result: passed (documentation/plan files updated).
- Documentation updated:
  - Updated operation document to confirmed version.
  - Added reserved API contract document.
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Start phased implementation after your approval: schema migration -> auth/admin commands -> frontend pages -> reserved API stubs.

## 2026-02-28 11:28 - Implement admin registration + renewal + reserved device scope API (with full verification)

- Scope:
  - Implemented admin-only user registration flow with multi-role support, optional phone, account term (permanent/days), and renewal capability.
  - Added lazy expiration check at login/refresh path and startup compensation deactivation.
  - Added reserved device-scope API contract handlers (`get/upsert`) with explicit not-implemented marker.
  - Updated frontend user page and API wrapper to call new admin registration/renew APIs and reserved device-scope APIs.
  - Fixed historical doc-comment parsing issues by rebuilding `auth/services.rs` and `db/auth_repository.rs` into clean, equivalent implementations so full doctest path is green.
  - Tauri security boundary evaluation:
    - Capabilities/permissions: unchanged.
    - Command exposure: only explicit new admin commands added in `invoke_handler`, no wildcard exposure.
    - Async/state safety: unchanged shared-state model; startup compensation runs once during setup.
    - CSP/updater/version sync: unchanged.
- Related plan file in `plan/`:
  - `plan/2026-02-28-1105-admin-register-implementation.md`
- Changed files:
  - `src-tauri/src/db/migrations/0004_user_registration_extension.sql`
  - `src-tauri/src/db/migrations.rs`
  - `src-tauri/src/db/bootstrap.rs`
  - `src-tauri/src/db/tests.rs`
  - `src-tauri/src/db/admin_repository.rs`
  - `src-tauri/src/auth/admin_services.rs`
  - `src-tauri/src/auth/admin_commands.rs`
  - `src-tauri/src/auth/models.rs`
  - `src-tauri/src/auth/commands.rs`
  - `src-tauri/src/auth/services.rs`
  - `src-tauri/src/auth/mod.rs`
  - `src-tauri/src/lib.rs`
  - `src/api/user.ts`
  - `src/views/permission/page/index.vue`
  - `plan/2026-02-28-1105-admin-register-implementation.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (`24 passed; 0 failed`; doctest `1 passed`).
  - command: `pnpm typecheck`
  - result: passed.
  - command: `pnpm lint`
  - result: passed.
- Documentation updated:
  - Synced implementation checklist/timeline in `plan/2026-02-28-1105-admin-register-implementation.md`.
  - Added this completion entry to `docs/development-progress.md`.
- Next step:
  - Re-run SQL-level validation in a healthy `sqlite_tools` MCP session to confirm live schema introspection consistency with migration 0004.

## 2026-02-28 12:35 - Upgrade "页面管理" to "用户注册管理" with admin user CRUD and protected admin rule

- Scope:
  - Renamed permission page semantics to user registration management:
    - seed title changed to `用户注册管理`
    - added one-time migration compensation `0005_permission_page_to_user_registration.sql` for existing databases
  - Added admin-only user management APIs:
    - list users: `auth_admin_list_users`
    - update user profile/roles/status/term: `auth_admin_update_user`
    - delete user: `auth_admin_delete_user`
    - change user password: `auth_admin_change_user_password`
  - Enforced protected admin-user policy:
    - target username `admin` is not editable/deletable by generic CRUD
    - `admin` password change remains allowed
  - Refactored frontend `permission/page`:
    - page title and behavior switched to user registration management
    - added registered user table with full info display
    - non-admin users support edit/delete, admin user only supports password change
    - retained reserved device-scope interface section
  - Tauri security boundary evaluation:
    - capability/permission/CSP/updater settings unchanged
    - only explicit new invoke commands registered; no wildcard exposure
    - all new management operations remain admin-gated in service layer
- Related plan file in `plan/`:
  - `plan/2026-02-28-1138-user-register-management-crud.md`
- Changed files:
  - `src-tauri/src/db/migrations/0002_seed.sql`
  - `src-tauri/src/db/migrations/0005_permission_page_to_user_registration.sql`
  - `src-tauri/src/db/migrations.rs`
  - `src-tauri/src/db/bootstrap.rs`
  - `src-tauri/src/db/tests.rs`
  - `src-tauri/src/db/admin_repository.rs`
  - `src-tauri/src/auth/models.rs`
  - `src-tauri/src/auth/admin_services.rs`
  - `src-tauri/src/auth/admin_commands.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/auth/commands.rs`
  - `src-tauri/src/db/migrations/README.md`
  - `src/api/user.ts`
  - `src/views/permission/page/index.vue`
  - `plan/2026-02-28-1138-user-register-management-crud.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (`30 passed; 0 failed`; doctest passed)
  - command: `pnpm typecheck`
  - result: passed
  - command: `pnpm lint`
  - result: passed
- Documentation updated:
  - Updated plan execution file `plan/2026-02-28-1138-user-register-management-crud.md`
  - Added this task entry to `docs/development-progress.md`
- Next step:
  - Split and commit by feature groups (DB migration, backend CRUD APIs, frontend page/API).

## 2026-02-28 14:44 - Reorder permission page sections and collapse management blocks

- Scope:
  - Moved `已注册用户信息` section to the first position on `用户注册管理` page.
  - Converted `用户注册管理` and `用户设备配置（预留）` from always-expanded cards to collapsible sections.
  - Kept existing admin permission behavior and handlers unchanged.
- Related plan file in `plan/`:
  - `plan/2026-02-28-1437-permission-page-collapse-order.md`
- Changed files:
  - `src/views/permission/page/index.vue`
  - `plan/2026-02-28-1437-permission-page-collapse-order.md`
  - `docs/development-progress.md`
- Verification:
  - command: `pnpm typecheck`
  - result: failed due pre-existing baseline issues (`src/components/ReIcon/index.ts` missing `./src/iconifyIconOnline`, `src/router/index.ts` missing `@/utils/progress`).
  - command: `pnpm exec eslint src/views/permission/page/index.vue`
  - result: passed.
- Documentation updated:
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Repair baseline missing modules, then rerun global `pnpm typecheck`.

## 2026-02-28 15:06 - Fix missing `@/utils/progress` module resolution in lay-tag

- Scope:
  - Investigated Vite import-analysis failure in `src/layout/components/lay-tag/index.vue`.
  - Restored `@/utils/progress` module path with a local compatibility implementation that does not depend on `nprogress`.
  - Kept existing call sites unchanged (`NProgress.start()` / `NProgress.done()`).
- Related plan file in `plan/`:
  - `plan/2026-02-28-1501-fix-progress-import.md`
- Changed files:
  - `src/utils/progress/index.ts`
  - `plan/2026-02-28-1501-fix-progress-import.md`
  - `docs/development-progress.md`
- Verification:
  - command: `pnpm exec vite build`
  - result: passed.
  - command: `pnpm typecheck`
  - result: passed.
- Documentation updated:
  - Added this task entry to `docs/development-progress.md`.
  - Added synchronized timeline to `plan/2026-02-28-1501-fix-progress-import.md`.
- Next step:
  - Optional: if you want the visible top loading bar back, re-introduce `nprogress` dependency and replace the current no-op compatibility shim.

## 2026-02-28 15:25 - Hide button-permission page from dynamic routes

- Scope:
  - Added one-time DB migration to remove the `/permission/button` route tree (including children) from dynamic route data.
  - Wired the migration into startup DB initialization so existing local databases also hide this page.
  - Added/updated DB tests to cover migration SQL load and route removal behavior.
  - Tauri security boundary evaluation:
    - Capabilities/permissions: unchanged.
    - Command exposure: unchanged.
    - Async/state safety: unchanged.
    - CSP/updater/version sync: unchanged.
  - SQLite Tools MCP note: `sqlite_tools` server handshake failed in this session, so SQL behavior validation used Rust DB tests fallback.
- Related plan file in `plan/`:
  - `plan/2026-02-28-1517-hide-button-permission-page.md`
- Changed files:
  - `src-tauri/src/db/migrations/0006_hide_button_permission_route.sql`
  - `src-tauri/src/db/migrations.rs`
  - `src-tauri/src/db/bootstrap.rs`
  - `src-tauri/src/db/tests.rs`
  - `plan/2026-02-28-1517-hide-button-permission-page.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml hides_button_permission_routes -- --nocapture`
  - result: failed first (RED, `left: 3 right: 0`), then passed after migration implementation.
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (`31 passed; 0 failed`; doctest passed).
- Documentation updated:
  - Added this task entry to `docs/development-progress.md`.
  - Added synchronized timeline to `plan/2026-02-28-1517-hide-button-permission-page.md`.
- Next step:
  - Restart app (`pnpm tauri:dev`) so local DB runs migration 0006 and menu updates immediately.

## 2026-02-28 19:16 - Replace rusqlite with sqlx and migrate DB access layer

- Scope:
  - Replaced `src-tauri` SQLite dependency from `rusqlite` to `sqlx` (`launchbadge/sqlx`) and added explicit Tokio runtime support.
  - Migrated DB core modules (`mod/bootstrap/migrations`) to `sqlx` execution path while preserving existing sync-facing APIs.
  - Migrated `auth_repository` and `admin_repository` SQL execution/transactions from `rusqlite` calls to `sqlx::query`/`query_scalar`.
  - Migrated DB tests from `rusqlite::Connection` to `sqlx::SqliteConnection` and kept previous behavior assertions.
  - Tauri security boundary evaluation:
    - Capabilities/permissions: unchanged.
    - Command exposure: unchanged (no new command registration).
    - Async/state safety: no shared-state expansion; DB runtime encapsulated in `db` module.
    - CSP/updater/version sync: unchanged.
  - SQLite Tools MCP note: attempted `list_mcp_resources(server="sqlite_tools")` and current session failed to initialize MCP; SQL behavior was verified via Rust tests fallback.
- Related plan file in `plan/`:
  - `plan/2026-02-28-1902-rusqlite-to-sqlx.md`
- Changed files:
  - `README.md`
  - `src-tauri/Cargo.toml`
  - `src-tauri/Cargo.lock`
  - `src-tauri/README.md`
  - `src-tauri/src/db/mod.rs`
  - `src-tauri/src/db/bootstrap.rs`
  - `src-tauri/src/db/migrations.rs`
  - `src-tauri/src/db/auth_repository.rs`
  - `src-tauri/src/db/admin_repository.rs`
  - `src-tauri/src/db/tests.rs`
  - `plan/2026-02-28-1902-rusqlite-to-sqlx.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo check --manifest-path src-tauri/Cargo.toml`
  - result: passed.
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (`31 passed; 0 failed`; doctest `1 passed`).
  - command: `list_mcp_resources(server="sqlite_tools")`
  - result: failed in this session (`MCP startup failed: handshaking ... connection closed`).
- Documentation updated:
  - Updated dependency wording from `rusqlite` to `sqlx` in root/backend README.
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Run `pnpm tauri:dev` for one manual startup smoke check against an existing local DB file.

## 2026-02-28 19:39 - Upgrade Rust tauri-cli to latest stable 2.10.0

- Scope:
  - Verified latest stable `tauri-cli` version from official online source.
  - Upgraded global Rust CLI from `2.9.4` to `2.10.0`.
  - Re-validated local Tauri toolchain version reporting after upgrade.
  - No project runtime/business code changes.
- Related plan file in `plan/`:
  - `plan/2026-02-28-1933-upgrade-rust-tauri-cli.md`
- Changed files:
  - `plan/2026-02-28-1933-upgrade-rust-tauri-cli.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo install --list | rg "tauri-cli|cargo-tauri"`
  - result: `tauri-cli v2.10.0`.
  - command: `cargo tauri --version`
  - result: `tauri-cli 2.10.0`.
  - command: `pnpm tauri info`
  - result: `tauri-cli 🦀: 2.10.0`.
- Documentation updated:
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Optional: run `pnpm tauri:dev` for one full interactive smoke check.

## 2026-02-28 19:02 - Fix redb begin_read missing trait import

- Scope:
  - Fixed a compilation error in `src/notice/repository.rs` where `begin_read` could not be found due to missing `ReadableDatabase` trait import.
  - Fixed cascading type inference errors (`E0282`) for `redb::Table::iter()` and `value.value()` resolving automatically after the trait was correctly imported.
  - Fixed a clippy warning `cast_possible_truncation` in `src/db/admin_repository.rs` by using `try_into().unwrap_or(usize::MAX)`.
- Changed files:
  - `src-tauri/src/notice/repository.rs`
  - `src-tauri/src/db/admin_repository.rs`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (31 passed; 0 failed).
  - command: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
  - result: passed (no warnings).
- Documentation updated:
  - Appended this task entry to `docs/development-progress.md`.
- Next step:
  - N/A

## 2026-02-28 20:16 - Switch src-tauri database stack to PostgreSQL 17 + TimescaleDB 2.19

- Scope:
  - Replaced backend storage stack from `sqlite + redb` to PostgreSQL (`sqlx-postgres`) and unified notice storage into PostgreSQL table `notice_items`.
  - Migrated DB connection/config model from local file path to database URL (`PURE_ADMIN_DATABASE_URL` + test URL fallback).
  - Converted SQL migrations and repositories to PostgreSQL syntax (`$n` placeholders, `STRING_AGG`, `RETURNING`, `raw_sql` script execution).
  - Added PostgreSQL advisory lock in DB bootstrap to prevent concurrent migration races during parallel tests.
  - Tauri security boundary evaluation:
    - capabilities/permissions: unchanged.
    - command exposure: unchanged (no new Tauri commands introduced for this migration).
    - runtime/state safety: DB init now serialized at DB level via advisory lock.
    - CSP/updater/version sync: unchanged.
  - SQLite Tools MCP note: server tooling remained unavailable in session; SQL behavior validated through full Rust test suite.
- Related plan file in `plan/`:
  - `plan/2026-02-28-1947-postgres-timescaledb-migration.md`
- Changed files:
  - `src-tauri/Cargo.toml`
  - `src-tauri/Cargo.lock`
  - `src-tauri/src/db/mod.rs`
  - `src-tauri/src/db/path_store.rs`
  - `src-tauri/src/db/bootstrap.rs`
  - `src-tauri/src/db/migrations.rs`
  - `src-tauri/src/db/migrations/0001_schema.sql`
  - `src-tauri/src/db/migrations/0002_seed.sql`
  - `src-tauri/src/db/migrations/0003_legacy_offline_cleanup.sql`
  - `src-tauri/src/db/migrations/0004_user_registration_extension.sql`
  - `src-tauri/src/db/migrations/0005_permission_page_to_user_registration.sql`
  - `src-tauri/src/db/migrations/0006_hide_button_permission_route.sql`
  - `src-tauri/src/db/admin_repository.rs`
  - `src-tauri/src/db/auth_repository.rs`
  - `src-tauri/src/db/tests.rs`
  - `src-tauri/src/notice/repository.rs`
  - `src-tauri/src/notice/mod.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/auth/commands.rs`
  - `src-tauri/src/auth/admin_commands.rs`
  - `docs/postgresql-timescaledb-runtime.md`
  - `plan/2026-02-28-1947-postgres-timescaledb-migration.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (`31 passed; 0 failed`; doctest `1 passed`).
- Documentation updated:
  - Added `docs/postgresql-timescaledb-runtime.md`.
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Optional: move DB URL credentials to environment-only config for production packaging, and remove hardcoded fallback URL.

## 2026-02-28 20:22 - Sync root README with PostgreSQL + TimescaleDB runtime

- Scope:
  - Updated root documentation to match the current backend storage architecture.
  - Removed outdated SQLite/redb runtime-path guidance from root README and replaced it with PostgreSQL/TimescaleDB connection guidance.
- Related plan file in `plan/`:
  - `plan/2026-02-28-2020-readme-postgres-sync.md`
- Changed files:
  - `README.md`
  - `plan/2026-02-28-2020-readme-postgres-sync.md`
  - `docs/development-progress.md`
- Verification:
  - command: `rg -n "sqlite|SQLite|redb|pure-admin-thin\.sqlite3|pure-admin-thin-notice\.redb|PostgreSQL|TimescaleDB" README.md`
  - result: passed; root README now describes PostgreSQL/TimescaleDB and no longer references sqlite/redb runtime file paths.
- Documentation updated:
  - Rewrote root `README.md` sections: stack, features, DB config, local bootstrap and validation commands.
  - Added this entry to `docs/development-progress.md`.
- Next step:
  - Optional: synchronize `src-tauri/README.md` wording to remove any remaining SQLite/redb historical phrasing.

## 2026-02-28 20:55 - DB hybrid optimization: SeaORM CRUD + sqlx complex query boundary

- Scope:
  - Introduced SeaORM into src-tauri for regular entity-based CRUD paths.
  - Split oversized dmin_repository into focused modules:
    - SeaORM CRUD module for standard user lifecycle operations.
    - sqlx module retained for aggregate-heavy reporting/authorization checks.
  - Added explicit SQL-boundary comments in auth repository for complex query paths.
  - Added a new DB test to validate SeaORM connection/query flow.
  - Tauri security boundary evaluation:
    - capabilities/permissions: unchanged.
    - command exposure: unchanged.
    - async/state safety: unchanged command surface; DB access remains backend-only.
    - CSP/updater/version sync: unchanged.
- Related plan file in plan/:
  - plan/2026-02-28-2046-db-seaorm-optimization.md
- Changed files:
  - src-tauri/Cargo.toml
  - src-tauri/src/db/mod.rs
  - src-tauri/src/db/tests.rs
  - src-tauri/src/db/admin_repository.rs
  - src-tauri/src/db/admin_repository/seaorm_users.rs
  - src-tauri/src/db/admin_repository/sqlx_reports.rs
  - src-tauri/src/db/entities/mod.rs
  - src-tauri/src/db/entities/prelude.rs
  - src-tauri/src/db/entities/users.rs
  - src-tauri/src/db/entities/user_roles.rs
  - src-tauri/src/db/auth_repository.rs
  - plan/2026-02-28-2046-db-seaorm-optimization.md
  - docs/development-progress.md
- Verification:
  - command: cargo test --manifest-path src-tauri/Cargo.toml opens_seaorm_connection_for_postgres -- --nocapture
  - result: passed (1 passed; 0 failed).
  - command: cargo test --manifest-path src-tauri/Cargo.toml
  - result: blocked by unrelated worktree state (src-tauri/src/auth/commands.rs currently deleted), not by this DB refactor.
- Documentation updated:
  - Added this entry to docs/development-progress.md.
- Next step:
  - Resolve the worktree deletion state for src-tauri/src/auth/commands.rs, then rerun full test suite.

## 2026-02-28 21:11 - Restore auth commands test wiring and complete full Rust verification

- Scope:
  - Restored src-tauri/src/auth/commands.rs into working tree and resolved compile mismatch to current DB API.
  - Updated auth/admin test setup to use db::set_database_url(db::test_database_url()).
  - Made admin registration test usernames unique per run to avoid PostgreSQL unique-key collisions on repeated test executions.
  - Tauri security boundary evaluation:
    - capabilities/permissions: unchanged.
    - command exposure: unchanged.
    - async/state safety: unchanged.
    - CSP/updater/version sync: unchanged.
- Related plan file in plan/:
  - plan/2026-02-28-2046-db-seaorm-optimization.md
- Changed files:
  - src-tauri/src/auth/commands.rs
  - src-tauri/src/auth/admin_commands.rs
  - plan/2026-02-28-2046-db-seaorm-optimization.md
  - docs/development-progress.md
- Verification:
  - command: cargo test --manifest-path src-tauri/Cargo.toml
  - result: passed (32 passed; 0 failed; doctest 1 passed).
- Documentation updated:
  - Added this entry to docs/development-progress.md.
- Next step:
  - N/A

## 2026-02-28 21:16 - Add database access routing constraints (SeaORM/sqlx + OLTP/Timeseries)

- Scope:
  - Added mandatory, project-level rules for choosing SeaORM vs sqlx/raw SQL.
  - Added explicit write-routing constraints for PostgreSQL OLTP data vs TimescaleDB time-series data.
  - Linked runtime notes to the new policy for discoverability.
  - Tauri security boundary evaluation:
    - capabilities/permissions: unchanged.
    - command exposure: unchanged.
    - async/state safety: unchanged.
    - CSP/updater/version sync: unchanged.
- Related plan file in plan/:
  - plan/2026-02-28-2114-db-access-policy-constraints.md
- Changed files:
  - AGENTS.md
  - docs/database-access-policy.md
  - docs/postgresql-timescaledb-runtime.md
  - plan/2026-02-28-2114-db-access-policy-constraints.md
  - docs/development-progress.md
- Verification:
  - command:
    g -n "Database Access Routing Policy|database-access-policy|SeaORM|sqlx|TimescaleDB" AGENTS.md docs/database-access-policy.md docs/postgresql-timescaledb-runtime.md
  - result: passed; all mandatory policy entries and cross-links are present.
- Documentation updated:
  - Added new policy doc docs/database-access-policy.md.
  - Added mandatory policy section in AGENTS.md.
  - Added runtime doc link to the policy.
- Next step:
  - Optional: add a lightweight CI grep/check script to fail PRs when new complex SQL lacks a boundary comment.

## 2026-02-28 22:06 - Introduce Casbin RBAC with PostgreSQL policy store

- Scope:
  - Integrated `casbin-rs` with `sqlx-adapter` for RBAC policy enforcement.
  - Added new auth domain module `src-tauri/src/auth/rbac.rs` to centralize RBAC decisions.
  - Replaced hardcoded admin check in admin domain service with Casbin authorization (`user/manage`).
  - Added PostgreSQL schema/seed support for policy table `casbin_rule` and default policy rules.
  - Extended managed-user role normalization to allow `guest`.
  - Tauri security boundary evaluation:
    - capabilities/permissions: unchanged.
    - command exposure: unchanged (no new Tauri commands).
    - async/state safety: no new shared Tauri global state introduced.
    - CSP/updater/version sync: unchanged.
- Related plan file in plan/:
  - plan/2026-02-28-2158-casbin-rbac-postgres.md
- Changed files:
  - src-tauri/Cargo.toml
  - src-tauri/Cargo.lock
  - src-tauri/src/auth/mod.rs
  - src-tauri/src/auth/rbac.rs
  - src-tauri/src/auth/admin_services.rs
  - src-tauri/src/db/admin_repository.rs
  - src-tauri/src/db/admin_repository/sqlx_reports.rs
  - src-tauri/src/db/migrations/0001_schema.sql
  - src-tauri/src/db/migrations/0002_seed.sql
  - src-tauri/src/db/tests.rs
  - src-tauri/src/auth/README.md
  - src-tauri/src/db/migrations/README.md
  - plan/2026-02-28-2158-casbin-rbac-postgres.md
  - docs/development-progress.md
- Verification:
  - command: cargo test --manifest-path src-tauri/Cargo.toml
  - result: passed (34 passed; 0 failed; doctest 1 passed).
- Documentation updated:
  - Added RBAC/Casbin usage note to `src-tauri/src/auth/README.md`.
  - Added `casbin_rule` and RBAC seed details to `src-tauri/src/db/migrations/README.md`.
  - Added this task entry to `docs/development-progress.md`.
- Next step:
  - Optional: add explicit RBAC checks to future device-control IPC commands once those command endpoints are implemented.

## 2026-02-28 22:32 - Introduce layered runtime config (config.toml + env) for DB/JWT/port

- Scope:
  - Added a new backend runtime config loader (`core/config.rs`) using `config` crate + `.env` loading.
  - Moved DB URL and JWT secret resolution out of hardcoded Rust constants into layered config (`default.toml`, optional `local.toml`, env overrides).
  - Added config precedence tests (RED->GREEN) covering file override and env override behavior.
  - Added backend config templates and ignored local secret file (`src-tauri/config/local.toml`).
  - Tauri security boundary evaluation:
    - capabilities/permissions: unchanged.
    - command exposure: unchanged.
    - async/state safety: unchanged; config is read-only via `OnceLock`.
    - CSP/updater/version sync: unchanged.
- Related plan file in `plan/`:
  - `plan/2026-02-28-2220-secure-runtime-config.md`
- Changed files:
  - `src-tauri/Cargo.toml`
  - `src-tauri/Cargo.lock`
  - `src-tauri/src/core/mod.rs`
  - `src-tauri/src/core/config.rs`
  - `src-tauri/src/core/README.md`
  - `src-tauri/src/db/path_store.rs`
  - `src-tauri/src/db/mod.rs`
  - `src-tauri/src/auth/services.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/config/default.toml`
  - `src-tauri/config/local.example.toml`
  - `src-tauri/.gitignore`
  - `README.md`
  - `docs/postgresql-timescaledb-runtime.md`
  - `src-tauri/src/auth/README.md`
  - `plan/2026-02-28-2220-secure-runtime-config.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml core::config::tests:: -- --nocapture`
  - result: passed (`3 passed; 0 failed`).
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (`37 passed; 0 failed`; doctest `1 passed`).
- Documentation updated:
  - Updated root `README.md` runtime configuration section.
  - Updated `docs/postgresql-timescaledb-runtime.md` connection resolution and env keys.
  - Updated `src-tauri/src/auth/README.md` JWT key management notes.
  - Updated `src-tauri/src/core/README.md` to include config module.
- Next step:
  - Optional: move concrete local DB credentials from `src-tauri/config/default.toml` to `src-tauri/config/local.toml` + environment secrets in CI/release environments.

## 2026-02-28 22:46 - Add tracing-based leveled logging and request chain tracing

- Scope:
  - Introduced `tracing + tracing-subscriber + tracing-appender` for backend observability.
  - Added runtime logging config ([logging] level/directory) from TOML + env overrides.
  - Wrapped Tauri command entrypoints with request span context (`request_id`, `command`) and result-level logging (INFO/WARN/ERROR).
  - Enabled daily rolling log file output while keeping console output enabled.
  - Tauri security boundary evaluation:
    - capabilities/permissions: unchanged (no expansion)
    - command exposure: unchanged (no new IPC command)
    - runtime/state safety: command wrappers are lightweight and synchronous, no new shared mutable state
    - CSP/updater/version sync: unchanged
- Related plan file in plan/:
  - plan/2026-02-28-2238-tracing-logging-pipeline.md
- Changed files:
  - src-tauri/Cargo.toml
  - src-tauri/Cargo.lock
  - src-tauri/src/core/mod.rs
  - src-tauri/src/core/tracing.rs
  - src-tauri/src/core/config.rs
  - src-tauri/src/lib.rs
  - src-tauri/src/auth/commands.rs
  - src-tauri/src/auth/admin_commands.rs
  - src-tauri/src/notice/commands.rs
  - src-tauri/config/default.toml
  - src-tauri/config/local.example.toml
  - src-tauri/src/core/README.md
  - plan/2026-02-28-2238-tracing-logging-pipeline.md
  - docs/development-progress.md
- Verification:
  - command: cargo test --manifest-path src-tauri/Cargo.toml
  - result: passed (41 passed; 0 failed + doctest 1 passed).
- Documentation updated:
  - Updated src-tauri/src/core/README.md with tracing architecture and logging config/env docs.
  - Appended this task entry to docs/development-progress.md.
- Next step:
  - Optional: add frontend-side correlation id propagation (IPC payload/header level) to correlate UI actions with backend spans.

## 2026-02-28 22:58 - Add frontend-backend trace correlation ID alignment for Tauri invoke

- Scope:
  - Added a frontend `invokeWithTrace` wrapper that attaches `trace.requestId` to every Tauri invoke call.
  - Updated auth/notice/routes API modules to route all invokes through the tracing wrapper.
  - Extended backend tracing pipeline to accept optional frontend trace context and prioritize incoming `requestId` over generated IDs.
  - Updated Tauri command signatures to accept optional `trace` argument without changing command names or exposure.
  - Tauri security boundary evaluation:
    - capabilities/permissions: unchanged (no new capability or permission grants)
    - command exposure: unchanged (no new commands, only optional input field)
    - runtime/state safety: unchanged concurrency model; request id resolution is lock-free atomic fallback
    - CSP/updater/version sync: unchanged
- Related plan file in `plan/`:
  - `plan/2026-02-28-2252-frontend-backend-trace-correlation.md`
- Changed files:
  - `src/api/tauriInvoke.ts`
  - `src/api/user.ts`
  - `src/api/routes.ts`
  - `src/api/notice.ts`
  - `src-tauri/src/core/tracing.rs`
  - `src-tauri/src/auth/commands.rs`
  - `src-tauri/src/auth/admin_commands.rs`
  - `src-tauri/src/notice/commands.rs`
  - `src-tauri/src/core/README.md`
  - `plan/2026-02-28-2252-frontend-backend-trace-correlation.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (`43 passed; 0 failed` + doctest `1 passed`).
  - command: `pnpm typecheck`
  - result: passed.
  - command: `pnpm lint`
  - result: passed.
- Documentation updated:
  - Updated `src-tauri/src/core/README.md` with frontend `trace.requestId` contract.
  - Appended this task entry to `docs/development-progress.md`.
- Next step:
  - Optional: return `requestId` in command success/error envelopes to enable UI log panels to deep-link to backend log files.

## 2026-02-28 23:05 - Fix Tauri panic from duplicate logger initialization

- Scope:
  - Removed auri-plugin-log initialization in src-tauri/src/lib.rs setup hook.
  - Kept racing as the single logging/tracing pipeline to avoid logger re-initialization conflicts.
  - Removed direct dependencies log and auri-plugin-log from src-tauri/Cargo.toml.
  - Updated core logging docs to explicitly forbid simultaneous racing + auri-plugin-log initialization.
  - Tauri security boundary evaluation:
    - capabilities/permissions: unchanged.
    - command exposure: unchanged.
    - runtime/state safety: unchanged; only startup logging initialization path adjusted.
    - CSP/updater/version sync: unchanged.
- Related plan file in plan/:
  - plan/2026-02-28-2301-fix-tauri-log-double-init.md
- Changed files:
  - src-tauri/src/lib.rs
  - src-tauri/Cargo.toml
  - src-tauri/Cargo.lock
  - src-tauri/src/core/README.md
  - plan/2026-02-28-2301-fix-tauri-log-double-init.md
  - docs/development-progress.md
- Verification:
  - command: cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
  - result: passed.
  - command: cargo test --manifest-path src-tauri/Cargo.toml
  - result: passed (43 passed; 0 failed + doctest 1 passed).
  - command: cargo run --manifest-path src-tauri/Cargo.toml --no-default-features --color always -- / pnpm tauri:dev
  - result: timeout-limited in this CLI session; no immediate duplicate-logger panic observed before timeout.
- Documentation updated:
  - Updated src-tauri/src/core/README.md runtime constraint note.
  - Appended this task entry to docs/development-progress.md.
- Next step:
  - Re-run pnpm tauri:dev in your local terminal and confirm startup no longer panics.

## 2026-02-28 - Add Chinese comments to db and notice modules

- Scope:
  - Added line-by-line Chinese comments to all Rust files in src-tauri/src/db/ and src-tauri/src/notice/
  - Refactored README.md documents for both modules (PostgreSQL version)
  - Fixed duplicate definition compilation error in db/tests.rs
- Changed files:
  - src-tauri/src/db/mod.rs
  - src-tauri/src/db/bootstrap.rs
  - src-tauri/src/db/migrations.rs
  - src-tauri/src/db/path_store.rs
  - src-tauri/src/db/auth_repository.rs
  - src-tauri/src/db/admin_repository.rs
  - src-tauri/src/db/admin_repository/seaorm_users.rs
  - src-tauri/src/db/admin_repository/sqlx_reports.rs
  - src-tauri/src/db/entities/mod.rs
  - src-tauri/src/db/entities/users.rs
  - src-tauri/src/db/entities/user_roles.rs
  - src-tauri/src/db/entities/prelude.rs
  - src-tauri/src/db/tests.rs
  - src-tauri/src/db/README.md
  - src-tauri/src/notice/mod.rs
  - src-tauri/src/notice/commands.rs
  - src-tauri/src/notice/models.rs
  - src-tauri/src/notice/repository.rs
  - src-tauri/src/notice/services.rs
  - src-tauri/src/notice/README.md
- Verification:
  - command: cargo check --manifest-path src-tauri/Cargo.toml
  - result: passed (compilation successful after fixing duplicate ensure_db_ready)
- Documentation updated:
  - Refactored src-tauri/src/db/README.md to PostgreSQL version
  - Refactored src-tauri/src/notice/README.md to PostgreSQL version
  - Appended this task entry to docs/development-progress.md
- Next step:

## 2026-03-01 10:33 - Add Chinese line comments to lib.rs and backend README

- Scope:
  - Added line-by-line Chinese comments in `src-tauri/src/lib.rs` for startup flow clarity.
  - Added Chinese technical README for `src-tauri/src/` backend overview.
  - Tauri security boundary evaluation:
    - capabilities/permissions: unchanged (no capability expansion).
    - command exposure: unchanged (no new commands).
    - runtime/state safety: unchanged (startup flow only annotated).
    - CSP/updater/version sync: unchanged.
- Related plan file in `plan/`:
  - `plan/2026-03-01-1029-lib-rs-cn-comments.md`
- Changed files:
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/README.md`
  - `plan/2026-03-01-1029-lib-rs-cn-comments.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (43 passed; 0 failed; doctest 1 passed).
- Documentation updated:
  - Added `src-tauri/src/README.md`.
- Next step:
  - N/A

## 2026-03-01 10:48 - Fix notice_get_read_items decode failure on NULL extra

- Scope:
  - Fixed notice read-list decoding when `notice_items.extra` is `NULL` in PostgreSQL rows.
  - Added regression test covering read-list retrieval with a nullable `extra` value.
  - Tauri security boundary evaluation:
    - capabilities/permissions: unchanged.
    - command exposure: unchanged.
    - async/state safety: unchanged.
    - CSP/updater/version sync: unchanged.
- Related plan file in `plan/`:
  - `plan/2026-03-01-1046-notice-read-null-extra-fix.md`
- Changed files:
  - `src-tauri/src/notice/repository.rs`
  - `plan/2026-03-01-1046-notice-read-null-extra-fix.md`
  - `docs/development-progress.md`
- Verification:
  - command: `cargo test --manifest-path src-tauri/Cargo.toml notice::repository::tests::read_items_allow_null_extra_column -- --nocapture`
  - result: passed (1 passed; 0 failed).
  - command: `cargo test --manifest-path src-tauri/Cargo.toml`
  - result: passed (44 passed; 0 failed; doctest 1 passed).
- Documentation updated:
  - Added this progress entry.
- Next step:
  - N/A

## 2026-03-01 11:14 - Ignore runtime log files from Git tracking

- Scope:
  - Prevented runtime `.log` files from repeatedly dirtying the working tree.
  - Added ignore patterns for both root-level and `src-tauri` log directories.
  - Removed existing tracked log files from Git index while keeping local files.
- Related plan file in `plan/`:
  - `plan/2026-03-01-1112-ignore-runtime-log-tracking.md`
- Changed files:
  - `.gitignore`
  - `src-tauri/.gitignore`
  - `plan/2026-03-01-1112-ignore-runtime-log-tracking.md`
  - `docs/development-progress.md`
- Verification:
  - command: `git status --short --branch`
  - result: pending (run after commit/push)
- Documentation updated:
  - Added this progress entry.
- Next step:
  - N/A
- Verification:
  - command: `git status --short --branch` (post-commit)
  - result: pending
