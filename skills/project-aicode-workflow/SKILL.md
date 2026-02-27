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
4. If another skill is clearly relevant, invoke it before code edits.
5. If the task touches Tauri backend/config or IPC contract, load and follow `docs/tauri-framework-constraints.md`.

## Task Granularity Rules
- Keep each task within one AI context: one objective, bounded scope, and clear verification commands.
- Split larger goals into smaller tasks with checkpoints and independent verification.
- Avoid bundling unrelated refactors into the same task.

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
   - date
   - task title
   - changed files
   - verification commands and result
   - next follow-up item (if any)
4. Only then mark the task complete.

## Tauri-Specific Required Reasoning
- For each Tauri task, explicitly evaluate: capabilities/permissions, command exposure, async/state safety, CSP/updater risk, and version sync.

## Deployment Baseline
- Follow `docs/deployment-strategy.md` for release and update flow.
- Keep release flow aligned with official Tauri signing/updater guidance and CI automation guidance.
