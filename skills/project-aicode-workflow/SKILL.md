---
name: project-aicode-workflow
description: Use when coding in this repository to enforce mandatory skill loading, task sizing, module split limits, deployment discipline, and documentation/progress updates.
---

# Project AI Code Workflow

## Mandatory Skill Order
1. Run `using-superpowers` before any analysis or edit.
2. Run `brainstorming` before creative design or behavior changes.
3. Select implementation skills by language:
   - Rust: start with `rust-router`, then required Rust topic skills. Use Actionbook Rust skills guidance from `https://github.com/actionbook/rust-skills`.
   - TypeScript/Vue: use `test-driven-development` before feature/bugfix edits and `frontend-design` for UI work.
   - Vue tasks MUST invoke the installed `vuejs-ai/skills` by topic match (`vue-router-best-practices`, `vue-pinia-best-practices`, `vue-options-api-best-practices`, `vue-testing-best-practices`, `vue-jsx-best-practices`, `vue-debug-guides`, `create-adaptable-composable`, default `vue-best-practices`).
4. If another skill is clearly relevant, invoke it before code edits.
5. If the task touches Tauri backend/config or IPC contract, load and follow `docs/tauri-framework-constraints.md`.
6. If the task involves database schema/migration/seed/query work, use SQLite Tools MCP (`sqlite_tools`) to inspect schema and validate SQL behavior before finalizing edits.
7. Keep Codex skill sources healthy per `docs/ai-skills-usage.md`:
   - superpowers via native discovery (`~/.agents/skills/superpowers` -> `~/.codex/superpowers/skills`)
   - vuejs-ai skills via `npx skills add vuejs-ai/skills --yes --global`

## Task Granularity Rules
- Keep each task within one AI context: one objective, bounded scope, and clear verification commands.
- Split larger goals into smaller tasks with checkpoints and independent verification.
- Avoid bundling unrelated refactors into the same task.

## Plan and Timestamp Sync (Required)
1. Ensure `plan/` exists at repository root before editing.
2. For each task, create or update one plan file in `plan/` named:
   - `YYYY-MM-DD-HHMM-<task>.md`
3. After every meaningful file change, append a progress line with local timestamp:
   - format: `- [HH:mm:ss] <what changed> (<status>)`
4. Keep the plan file synchronized during implementation, not only at task end.
5. At completion, sync the final summary from the plan file to `docs/development-progress.md`.

## Module Size and Structure Rules
- Target module size: around 400 lines.
- Soft limit: 400 lines. Hard split trigger: around 450 lines.
- Split by responsibility:
  - UI/view
  - state/composable
  - API adapter
  - domain/service logic
  - shared utility/types

## Completion Gates (Required)
1. Run relevant checks (`pnpm lint`, `pnpm typecheck`, `cargo test --manifest-path src-tauri/Cargo.toml`, or narrowed equivalents).
2. Update developer documentation for behavior or workflow changes.
3. Append a new entry to `docs/development-progress.md` with:
   - date and time
   - task title
   - related plan file in `plan/`
   - changed files
   - verification commands and result
   - next follow-up item (if any)
4. Only then mark the task complete.

## Tauri-Specific Required Reasoning
- For each Tauri task, explicitly evaluate: capabilities/permissions, command exposure, async/state safety, CSP/updater risk, and version sync.

## Deployment Baseline
- Follow `docs/deployment-strategy.md` for release and update flow.
- Keep release flow aligned with official Tauri signing/updater guidance and CI automation guidance.
