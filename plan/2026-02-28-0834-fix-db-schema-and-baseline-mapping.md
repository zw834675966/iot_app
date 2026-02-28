# Task Plan - Fix DB schema SQL error and update baseline-browser-mapping

## Objective
- Fix Tauri startup failure caused by SQL migration syntax error.
- Update baseline-browser-mapping to latest dev dependency without breaking pnpm workflow.

## Scope
- src-tauri/src/db/migrations/0001_schema.sql
- package.json
- pnpm-lock.yaml
- docs/development-progress.md

## Progress
- [08:34:43] Created task plan and confirmed root-cause location in 0001_schema.sql (completed)
- [08:34:59] Fixed permissions table SQL syntax in 0001_schema.sql by adding closing ); (completed)
- [08:35:29] Updated dev dependency: baseline-browser-mapping -> latest via pnpm (completed)
- [08:36:23] Verification: cargo test hit pre-existing doctest failures in auth/services.rs; unit tests passed (not completed)
- [08:36:23] Verification: cargo test --lib passed (19 passed) confirming DB migration execution path is healthy (completed)
- [08:36:23] Verification: pnpm typecheck passed (completed)
