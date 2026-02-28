# 2026-02-28 09:59 - Hide error route from active menu

## Objective
- Make the "异常页面" route passive: it should not be actively shown in navigation menus, and should only appear when an exception route is navigated to.

## Scope
- Update frontend router config for error route menu visibility.
- Keep exception pages (`/error/403`, `/error/404`, `/error/500`) reachable for guard-triggered navigation.
- No backend/Tauri/IPC/database change.

## Checklist
- [x] Locate route/menu visibility switch for error module
- [x] Apply minimal route config change
- [x] Run verification commands
- [x] Sync progress docs

## Progress Timeline
- [09:59:14] Task started (in_progress)
- [09:59:55] Enabled `meta.showLink: false` for `src/router/modules/error.ts` to hide "异常页面" from active menus (done)
- [10:01:12] Ran `pnpm typecheck` and `pnpm lint`; both passed (done)
- [10:01:50] Synced summary entry to `docs/development-progress.md` (done)

## Verification
- command: `pnpm typecheck`
- result: passed (`tsc` + `vue-tsc` no errors)
- command: `pnpm lint`
- result: passed (`eslint` + `prettier` + `stylelint`)

## Completion
- status: completed
- follow-up: Optional: set `meta.showLink: false` on `/error/*` children if tag-bar hiding is also required.
