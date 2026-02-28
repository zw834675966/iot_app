# 2026-02-28 10:05 - Parent menu icon fallback for collapsed sidebar

## Objective
- Ensure parent directory items in navigation always have an icon so collapsed mode remains visually consistent.

## Scope
- Add frontend dynamic-route normalization fallback for parent menu icons.
- Keep existing backend data and route contract unchanged.
- No Tauri/backend/database/capability/permission/CSP/updater change.

## Checklist
- [x] Locate menu route transformation entrypoint
- [x] Implement parent icon fallback for dynamic routes
- [x] Run verification commands
- [x] Sync progress docs

## Progress Timeline
- [10:05:55] Task started (in_progress)
- [10:05:55] Added `DEFAULT_PARENT_MENU_ICON` and fallback assignment in `src/router/utils.ts` `addAsyncRoutes` for parent routes with missing `meta.icon` (done)
- [10:07:29] Fixed `RouteMeta` typing by providing fallback `title` when backend route meta is missing, then reran checks (done)
- [10:08:20] Synced task summary into `docs/development-progress.md` and completed task bookkeeping (done)

## Verification
- command: `pnpm typecheck`
- result: first run failed (`RouteMeta.title` missing when `meta` undefined); after fallback title fix, rerun passed.
- command: `pnpm lint`
- result: passed (`eslint` + `prettier` + `stylelint`).

## Completion
- status: completed
- follow-up: Optional: move default parent icon policy to backend route data if you want role/module-specific icon control.
