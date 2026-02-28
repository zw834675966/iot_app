# 2026-02-28 11:38 - User registration management rename + admin user CRUD

## Objective
- Rename menu/page semantics from “页面管理/页面权限” to “用户注册管理”.
- Under admin account, show all registered users.
- Enforce operation rule:
  - target `admin` user: password change only
  - non-admin users: full CRUD (create/read/update/delete)

## Scope
- Rust DB repository/admin service/admin command extensions.
- Frontend API + page logic update for user list and CRUD actions.
- Route title rename via migration compensation.

## Checklist
- [x] Add route-title rename migration/compensation for existing DB
- [x] Add backend list/update/delete/password APIs with admin-only guards
- [x] Add backend tests for admin exception rule and CRUD paths
- [x] Update frontend API wrappers for new admin user-management commands
- [x] Refactor permission page into user-registration-management UI with table + actions
- [x] Run verification commands
- [x] Sync docs/progress

## Progress Timeline
- [11:38:00] Task started (in_progress)
- [12:03:00] Added backend user-management commands: list/update/delete/change-password with protected admin-user rule.
- [12:12:00] Added migration `0005_permission_page_to_user_registration.sql` and bootstrapped one-time compensation.
- [12:24:00] Refactored frontend `permission/page` to user registration management with user table + CRUD actions.
- [12:35:00] Verification passed: cargo test / pnpm typecheck / pnpm lint.

## Verification
- command: `cargo test --manifest-path src-tauri/Cargo.toml`
- result: passed (`30 passed; 0 failed`; doctest passed)
- command: `pnpm typecheck`
- result: passed
- command: `pnpm lint`
- result: passed

## Completion
- status: done
- follow-up: split current working-tree changes into feature-grouped commits before merge
