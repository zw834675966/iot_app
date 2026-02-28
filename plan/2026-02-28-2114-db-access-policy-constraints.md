# Task Plan - DB Access Policy Constraints

- Date: 2026-02-28
- Scope: Add enforceable project constraints for SeaORM vs sqlx/raw SQL and PostgreSQL vs timeseries write routing.

## Steps
1. Create a dedicated policy doc under `docs/` with decision matrix and examples.
2. Link and enforce the policy from `AGENTS.md` as mandatory constraints.
3. Cross-link runtime/storage docs so the policy is discoverable during DB work.
4. Update development progress log.
5. Run lightweight verification (link/path checks).

## Progress
- [21:14:00] Created task plan and defined documentation touch points (done)
- [21:16:20] Added new policy doc docs/database-access-policy.md with SeaORM/sqlx and OLTP/TSDB routing rules (done)
- [21:16:20] Added mandatory policy entry in AGENTS.md (done)
- [21:16:20] Linked runtime notes to policy doc (done)
- [21:16:44] Synced final summary into docs/development-progress.md (done)
