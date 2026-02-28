# Frontend Agent Rules

## Required Skills Before Editing

- First load `using-superpowers`.
- For feature/behavior changes, load `brainstorming` then `test-driven-development`.
- For UI/UX tasks, load `frontend-design`.
- For any Vue file/task, auto-match and invoke the best-fit `vuejs-ai/skills` skill before editing:
  - `vue-router-best-practices` for router/navigation/guard work
  - `vue-pinia-best-practices` for store/state management work
  - `vue-options-api-best-practices` for Options API components
  - `vue-testing-best-practices` for component/e2e test work
  - `vue-jsx-best-practices` for JSX/TSX usage
  - `vue-debug-guides` for Vue runtime warnings/errors/hydration issues
  - `create-adaptable-composable` for reusable composable API design
  - `vue-best-practices` as default/fallback for general Vue 3 + TS tasks
- If multiple Vue skills apply, invoke all relevant skills in most-specific-first order.
- If another listed skill applies, invoke it before edits.
- If changing Tauri IPC invocation or contracts (`@tauri-apps/api/core`), also follow `../docs/tauri-framework-constraints.md`.
- Follow `../docs/ai-skills-usage.md` for install/update/verification commands.

## Scope and Module Rules

- Keep task scope small and verifiable in one context window.
- Keep each TS/Vue module around 400 lines; split when a file approaches ~450 lines.
- Prefer split by feature folder and responsibility (`views`, `components`, `composables`, `api`, `store`).

## Completion Rules

- Run `pnpm lint` and `pnpm typecheck` (or narrowed equivalent with clear rationale).
- Update affected developer docs and append a progress entry in `docs/development-progress.md`.
