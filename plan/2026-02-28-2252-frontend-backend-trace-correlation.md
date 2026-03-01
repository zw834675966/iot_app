# Task Plan: frontend-backend-trace-correlation

- Objective: Attach correlation ID at frontend invoke and align backend tracing request_id with that value.
- Scope: src/api invoke wrappers + src-tauri command signatures + tracing core.
- Constraints: no new Tauri capabilities/permissions; keep command exposure unchanged.

## Progress
- [22:52:03] Created plan and locked scope for end-to-end trace correlation (in_progress)
- [22:52:15] Added RED tests for frontend requestId precedence in core/tracing.rs (completed)
- [22:54:13] Implemented TraceContext + request_id resolution and updated Rust command signatures to accept optional trace context (completed)
- [22:55:12] Added frontend invokeWithTrace wrapper and routed user/routes/notice API calls through it (completed)
- [22:55:29] Documented trace.requestId frontend-backend contract in src-tauri/src/core/README.md (completed)
- [22:57:58] Verified RED->GREEN for trace requestId test and full Rust suite via cargo test --manifest-path src-tauri/Cargo.toml (passed)
- [22:57:58] Verified frontend contracts with pnpm typecheck and pnpm lint (passed)
- [22:57:58] Ran cargo fmt and re-verified cargo test after formatting (passed)
- [22:58:20] Synced completion summary into docs/development-progress.md (completed)
- [22:58:20] Task fully complete (completed)
- [22:59:01] Corrected development-progress markdown entry escaping to preserve trace/requestId literals (completed)
