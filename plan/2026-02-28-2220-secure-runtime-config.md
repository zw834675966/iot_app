# 2026-02-28-2220-secure-runtime-config

## Objective
- Replace hardcoded runtime secrets/settings with layered configuration (`config.toml` + environment variables) for backend runtime.

## Scope
- Add a typed runtime config loader in `src-tauri`.
- Route DB URL and JWT secret resolution to config loader.
- Remove hardcoded production-like fallback credentials from Rust source code.
- Add tests proving env overrides file/default values.
- Update docs and progress logs.

## Checklist
- [x] Add config crate dependencies and config module
- [x] Add failing tests for config precedence (env > file > default)
- [x] Refactor DB/JWT configuration reads to use app config
- [x] Run verification commands
- [x] Update docs (`README`, `development-progress`)

## Progress Timeline
- [22:20:05] Task started (in_progress)
- [22:21:44] Added config precedence tests and confirmed RED failure (`not implemented`) (in_progress)
- [22:24:31] Implemented `core/config` loader and switched DB/JWT reads to runtime config (in_progress)
- [22:25:47] Config precedence tests are GREEN (`3 passed`) (in_progress)
- [22:30:58] Added layered config files and ignored `src-tauri/config/local.toml` (in_progress)
- [22:31:41] Updated README/auth/core/runtime docs for config layering and env override keys (in_progress)
- [22:32:19] Verified full Rust suite (`37 passed; 0 failed`) (in_progress)
- [22:34:09] Re-ran full Rust suite after doc sync (`37 passed; 0 failed`) (completed)

## Verification
- command: `cargo test --manifest-path src-tauri/Cargo.toml core::config::tests:: -- --nocapture`
- result: passed (`3 passed; 0 failed`).
- command: `cargo test --manifest-path src-tauri/Cargo.toml`
- result: passed (`37 passed; 0 failed`; doctest `1 passed`).

## Completion
- status: completed
- follow-up: optional - move concrete local DB credentials from `src-tauri/config/default.toml` into `src-tauri/config/local.toml` or CI env secrets.
