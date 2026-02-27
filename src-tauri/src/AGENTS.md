# Tauri Source-Level Rules

Applies to all files under `src-tauri/src/`.

## Mandatory References
- `../AGENTS.md`
- `../../docs/tauri-framework-constraints.md`

## Coding Constraints
- Keep Tauri commands thin and business logic in dedicated modules.
- Use explicit, serializable error types for command returns.
- Default to least-privilege assumptions; if a command needs wider access, document why.
- Prefer async commands for heavy work and avoid main-thread blocking patterns.

## Change Completion
- Run Rust tests for touched code path.
- Update `../../docs/development-progress.md` after task completion.
