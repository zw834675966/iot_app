# Task Plan: fix-tauri-log-double-init

- Objective: Fix tauri dev panic caused by double logger initialization.
- Scope: remove tauri-plugin-log runtime initialization when tracing is active.
- Constraints: no capability/permission changes; keep command exposure unchanged.

## Progress
- [23:01:33] Root cause confirmed from tauri dev logs and lib.rs setup hook (in_progress)
- [23:02:04] Removed tauri-plugin-log setup from lib.rs to prevent duplicate logger initialization (completed)
- [23:02:04] Removed unused direct dependencies log/tauri-plugin-log from src-tauri/Cargo.toml (completed)
- [23:02:20] Updated core README with rule to avoid tracing + tauri-plugin-log double initialization (completed)
- [23:05:31] Verified formatting and Rust tests via cargo fmt --check and cargo test --manifest-path src-tauri/Cargo.toml (passed)
- [23:05:31] Runtime smoke probe: cargo run/tauri:dev were timeout-limited in CLI session; no duplicate-logger panic observed before timeout (partial)
- [23:05:54] Synced completion summary to docs/development-progress.md (completed)
- [23:05:54] Task complete (completed)
