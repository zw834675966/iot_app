# Task Plan - Remove Notice Panel Avatar

- Objective: Remove avatar display from notice panel items.
- Scope: src/layout/components/lay-notice/components/NoticeItem.vue (and related notice data/types only if needed).
- Verification: RED check for el-avatar usage in notice item, then GREEN check; run pnpm typecheck and pnpm lint.

## Progress
- [12:48:53] Created task plan and identified notice avatar render point in NoticeItem component (completed)
- [12:49:20] Ran RED check: notice item still contains el-avatar (completed)
- [12:49:35] Removed el-avatar render block and avatar-only styles from notice item component (completed)
- [12:50:29] GREEN check passed: no `el-avatar` or `noticeItem.avatar` usage remains in NoticeItem component (completed)
- [12:50:29] Ran pnpm typecheck successfully (completed)
- [12:50:29] Ran pnpm lint successfully (completed)
- [12:50:52] Synced completion summary to docs/development-progress.md (completed)
- [12:51:36] Normalized development-progress entry formatting for this task (completed)
- [12:52:10] Fixed escaped-text issue in plan progress line (completed)
