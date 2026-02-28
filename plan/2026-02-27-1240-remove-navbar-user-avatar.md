# Task Plan - Remove Navbar User Avatar

- Objective: Remove user avatar display from top navigation/user menu UI.
- Scope: src/layout/components/lay-navbar/index.vue, src/layout/components/lay-sidebar/NavHorizontal.vue, src/layout/components/lay-sidebar/NavMix.vue, and related useNav fields if no longer needed.
- Verification: RED check for remaining userAvatar image bindings, then GREEN check; run pnpm typecheck and pnpm lint.

## Progress
- [12:40:35] Created task plan and identified avatar render locations in navbar/horizontal/mix navigation (completed)
- [12:43:35] Removed avatar image rendering and related nav-field bindings from navbar/horizontal/mix layouts and useNav (completed)
- [12:43:35] GREEN check passed for nav scope: no userAvatar/avatarsStyle references remain in target nav files (completed)
- [12:44:26] Ran pnpm typecheck successfully (completed)
- [12:44:26] Ran pnpm lint successfully (completed)
- [12:44:49] Synced completion summary to docs/development-progress.md (completed)
- [12:45:40] Normalized plan/progress markdown formatting for this task entry (completed)
- [12:46:46] RED baseline was confirmed before edits via grep: nav user-avatar references existed in target components (completed)
