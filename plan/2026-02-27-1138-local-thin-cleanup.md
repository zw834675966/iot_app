# 2026-02-27 11:38 - Local-only frontend slimming

- Objective: remove unused src/ code and dependencies that are unnecessary for local desktop runtime; replace obvious external links/resources in src.
- Scope:
  - delete dead code modules confirmed by reference scan
  - trim npm dependencies tied to deleted modules
  - replace src external URLs with local/offline-safe assets
  - update progress docs
- Verification plan:
  - pnpm typecheck
  - pnpm lint (if time permits)

## Progress
- [11:39:11] Created task plan file and locked scope (done)
- [11:40:42] Removed dead modules/directives and cleaned core references (done)
- [11:41:17] Replaced runtime external URLs in src footer/notice with offline-safe local content (done)
- [11:41:17] Trimmed npm dependencies and optimizeDeps entries tied to removed modules (done)
- [11:42:22] Updated lockfile and passed validation: pnpm typecheck + pnpm lint (done)
- [11:42:58] Synced summary into docs/development-progress.md (done)
- [11:43:47] Fixed markdown formatting in development-progress entry (done)
