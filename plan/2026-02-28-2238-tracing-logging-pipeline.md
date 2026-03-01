# Task Plan: tracing-logging-pipeline

- Objective: Add tracing-based logging with leveled console output and daily rolling file persistence.
- Scope: src-tauri runtime config + tracing setup + command entry tracing.
- Constraints: Follow Tauri security boundaries; avoid capability/permission expansion.

## Progress
- [22:38:34] Created task plan and captured implementation scope (in_progress)
- [22:38:57] Added RED tests for logging config file/env parsing in src-tauri/src/core/config.rs (completed)
- [22:40:16] Added RuntimeConfig.logging with env overrides and validation in core/config.rs (completed)
- [22:41:07] Added core/tracing.rs with subscriber initialization, daily rolling appender, and traced command wrapper; exported module in core/mod.rs (completed)
- [22:41:19] Added tracing/tracing-subscriber/tracing-appender dependencies in src-tauri/Cargo.toml (completed)
- [22:42:27] Wired tracing init in lib.rs and wrapped auth/admin/notice Tauri commands with traced request spans (completed)
- [22:42:43] Added [logging] defaults to src-tauri/config/default.toml and local.example.toml (completed)
- [22:42:59] Updated src-tauri/src/core/README.md with tracing architecture and logging config/env documentation (completed)
- [22:44:46] Fixed unstable tracing unit test assertion for EnvFilter construction (completed)
- [22:45:26] Refactored tracing filter parsing test to avoid RUST_LOG environment interference (completed)
- [22:46:14] Verified Rust tests: cargo test --manifest-path src-tauri/Cargo.toml (passed: 41 unit tests + 1 doctest)
- [22:46:14] Tauri security review: no capability/permission expansion, no new command exposure, no CSP/updater/version changes (completed)
- [22:46:14] Task implementation complete; documentation/progress sync pending final summary (in_progress)
- [22:46:41] Synced task outcome to docs/development-progress.md (completed)
- [22:47:11] Ran cargo fmt --manifest-path src-tauri/Cargo.toml to normalize formatting (completed)
- [22:48:25] Re-ran cargo test after formatting/doc fix (passed: 41 unit tests + 1 doctest) (completed)
- [22:48:25] Task fully complete (completed)
