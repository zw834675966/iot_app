# Casbin RBAC with PostgreSQL Policy Store

- Date: 2026-02-28
- Objective: Integrate Casbin-based RBAC into existing Tauri auth flow without breaking current logic; persist authorization policy in PostgreSQL.

## Scope
- Add `casbin-rs` and `sqlx-adapter` dependencies.
- Add RBAC service module and map existing roles to RBAC checks.
- Keep existing user-role source of truth in `user_roles`.
- Persist Casbin policy in PostgreSQL table `casbin_rule`.
- Replace hardcoded admin-only checks in admin services with RBAC enforcement.
- Update docs and development progress.

## Progress
- [21:58:44] Completed repository/constraints/skills/context scan and collected official casbin/sqlx-adapter API references (done)
- [22:01:28] Added Casbin/sqlx-adapter dependencies, introduced auth/rbac module, switched admin authorization to RBAC checks, and seeded PostgreSQL `casbin_rule` policies in schema/seed scripts (done)
- [22:02:21] Updated auth/db migration developer docs to include Casbin RBAC policy source and role-policy mapping (done)
- [22:05:33] Ran full Rust verification: `cargo test --manifest-path src-tauri/Cargo.toml` passed (34 unit tests + 1 doctest) (done)
- [22:06:29] Synced final task summary to `docs/development-progress.md` and completed deliverable packaging (done)
