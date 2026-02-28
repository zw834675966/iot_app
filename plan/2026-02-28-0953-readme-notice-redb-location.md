# 2026-02-28 09:53 - Add notice redb location to root README

- Objective: document where the notice center data is stored, and distinguish it from SQLite auth/system storage.
- Scope:
  - update `README.md` only for user-facing database location guidance
  - no runtime code change
  - no Tauri capability/permission/CSP/updater change

## Progress
- [09:53:49] Added a new root README section for notice center `redb` file location and PowerShell quick-check command (done)
- [09:54:34] Appended task summary, scope, and verification evidence to `docs/development-progress.md` (done)
- [09:54:34] Verified root README contains `pure-admin-thin-notice.redb` location and quick-check entries via `rg` (done)
