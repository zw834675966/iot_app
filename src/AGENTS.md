# Frontend Agent Rules

## Required Skills Before Editing
- First load `using-superpowers`.
- For feature/behavior changes, load `brainstorming` then `test-driven-development`.
- For UI/UX tasks, load `frontend-design`.
- If another listed skill applies, invoke it before edits.
- If changing Tauri IPC invocation or contracts (`@tauri-apps/api/core`), also follow `../docs/tauri-framework-constraints.md`.

## Scope and Module Rules
- Keep task scope small and verifiable in one context window.
- Keep each TS/Vue module around 400 lines; split when a file approaches ~450 lines.
- Prefer split by feature folder and responsibility (`views`, `components`, `composables`, `api`, `store`).

## Completion Rules
- Run `pnpm lint` and `pnpm typecheck` (or narrowed equivalent with clear rationale).
- Update affected developer docs and append a progress entry in `docs/development-progress.md`.
