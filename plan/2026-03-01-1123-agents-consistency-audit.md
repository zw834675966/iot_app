# 2026-03-01 11:23 - AGENTS consistency audit and correction

## Objective
Audit all repository `AGENTS.md` files against current project layout and tooling, then correct any stale or incorrect guidance.

## Scope
- Review: `AGENTS.md`, `src/AGENTS.md`, `src-tauri/AGENTS.md`, `src-tauri/src/AGENTS.md`.
- Validate referenced directories/commands/tooling against repository state.
- Update only mismatched documentation statements.

## Progress
- [11:23:39] Created plan file and started AGENTS consistency audit (completed)
- [11:24:10] Audited all AGENTS files and identified root AGENTS mismatches (`mock/`, `src/types/`, commitlint enforcement) (completed)
- [11:24:40] Updated root `AGENTS.md` to align with current repo layout and tooling (completed)
- [11:24:58] Synced progress documentation entry in `docs/development-progress.md` (completed)
- [11:25:15] Ran verification command `git status --short` for changed files review (completed)

## Verification
- `git status --short -- AGENTS.md src/AGENTS.md src-tauri/AGENTS.md src-tauri/src/AGENTS.md`

## Result
All AGENTS guidance reflects current repository structure and tool setup.
- [11:24:34] Verified AGENTS change scope and confirmed only root `AGENTS.md` required edits (completed)
