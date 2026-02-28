# Task Plan - Install vuejs-ai skills and superpowers constraints

- Objective: Ensure `vuejs-ai/skills` and `obra/superpowers` are installed per official docs, then add project constraints for automatic Vue skill routing and superpowers usage.
- Scope:
  - Verify/install `vuejs-ai/skills` into Codex skill discovery paths.
  - Verify/install/update superpowers via native discovery/junction workflow.
  - Update project instructions (`AGENTS.md`, `src/AGENTS.md`, `skills/project-aicode-workflow/SKILL.md`, docs) to enforce Vue skill routing + superpowers usage.
- Constraints:
  - Follow official docs from vuejs-ai README and superpowers INSTALL.md.
  - Keep changes documentation-focused (no business code behavior changes).
- Verification: install command outputs + path checks + markdown/rules diff checks.

## Progress
- [13:31:47] Created task plan and collected official installation docs (completed)
- [13:32:44] Ran official install/update commands: vuejs-ai skills installed globally; superpowers repo updated; junction refreshed (completed)
- [13:34:28] Updated AGENTS/workflow/frontend rules and added docs/ai-skills-usage.md with official install+routing guidance (completed)
- [13:35:03] Verified skill install state and ran pnpm lint for docs/rules changes (completed)
- [13:35:28] Synced summary to docs/development-progress.md (completed)
