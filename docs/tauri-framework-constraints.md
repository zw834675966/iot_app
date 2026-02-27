# Tauri Framework Constraints (Latest Baseline)

Last verified: 2026-02-27  
Baseline release: `tauri-v2.10.2` (published 2026-02-04)

## Scope
This file is mandatory for any AI code change touching:
- `src-tauri/**`
- `src-tauri/tauri.conf.json`
- frontend IPC calls to Tauri (`@tauri-apps/api/core` invoke usage)

## Mandatory Pre-Edit Thinking (AI Checklist)
1. Capability boundary:
   - Which window/webview should get this capability?
   - Are we accidentally broadening permissions by merging capabilities?
2. Permission boundary:
   - Do we need a new or extended permission set (least privilege)?
3. Command exposure:
   - Are new commands strictly necessary and uniquely named?
   - Should command exposure be restricted via app manifest/capabilities?
4. Runtime model:
   - Is this command heavy and should be `async` to avoid UI freeze?
   - Are borrowed args in async signatures avoided or handled via `Result` workaround?
5. State safety:
   - Is shared state accessed via `manage` + `State` with proper mutex strategy?
6. Security posture:
   - Any CSP relaxation, remote script/content, or unsafe remote API exposure?
   - Any updater/signing/transport risk introduced?
7. Version sync:
   - Are JS and Rust Tauri dependencies still compatible per official guidance?

## Implementation Constraints (Must Follow)

### 1) Capabilities and Permissions First
- Use capability files under `src-tauri/capabilities/` to bind permissions to specific windows/webviews.
- Do not rely on broad defaults; explicitly list capability identifiers in `tauri.conf.json` when possible.
- Permissions should be scoped and composable; prefer permission sets over ad-hoc broad grants.
- Any remote API access in capability config must be explicit and minimal; avoid wildcard remote trust unless justified.

### 2) Command Design and Exposure
- Commands must be unique names across modules.
- Keep command handlers thin; delegate business logic to service modules.
- Prefer module-based command organization (`commands/*.rs`) over bloating `lib.rs`.
- For fallible commands return `Result<_, E>` with serializable error.
- Avoid exposing commands globally unless required; review command exposure boundaries.

### 3) Async and State Rules
- Heavy work should be `async`; avoid blocking the main thread.
- For async command args, avoid borrowed types (`&str`, some borrowed state patterns) unless using documented workaround.
- Use `Builder::manage(...)` for global state and access via `State`.
- Prefer `std::sync::Mutex` unless lock must be held across `await`; then use async mutex.
- Avoid unnecessary `Arc` wrapping for state managed by Tauri.

### 4) CSP and Remote Content
- Keep CSP strict and tailored; only allow trusted origins.
- Avoid CDN/remote scripts by default.
- Any CSP relaxation must be documented with reason and risk.

### 5) Updater and Signing
- Updater signatures are mandatory and cannot be disabled.
- Never commit signing private key; use runtime environment variables only:
  - `TAURI_SIGNING_PRIVATE_KEY`
  - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- Production updater endpoints should use HTTPS (unless explicitly running insecure mode for controlled dev use).
- Ensure updater artifact/signature and metadata format are valid for each target.

### 6) Version Compatibility
- Keep `@tauri-apps/api` and Rust crate `tauri` at compatible minor versions.
- For plugins, keep npm package and Rust crate versions synchronized as required by official docs.

## Required Verification For Tauri Changes
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `pnpm typecheck` (when IPC signatures or frontend invocation changed)
- `pnpm lint` (when frontend IPC wrappers changed)
- Manual smoke check: launch with `pnpm tauri:dev` for changed Tauri path

## Sources (Official)
- Tauri latest release API: https://api.github.com/repos/tauri-apps/tauri/releases/latest
- Capabilities: https://v2.tauri.app/security/capabilities/
- Permissions: https://v2.tauri.app/security/permissions/
- Calling Rust from Frontend: https://v2.tauri.app/develop/calling-rust/
- State Management: https://v2.tauri.app/develop/state-management/
- CSP: https://v2.tauri.app/security/csp/
- Updating Dependencies: https://v2.tauri.app/develop/updating-dependencies/
- Updater plugin: https://v2.tauri.app/plugin/updater/
