# 2026-02-27-1347-skills-mcp-health-check

## Objective
- Check whether the project-required skills and MCP setup are healthy, and fix any broken configuration.

## Scope
- Verify required skills baseline (`vuejs-ai/skills`, `superpowers`, project workflow skill files).
- Verify MCP tool health in current session.
- Fix DBHub MCP startup configuration if broken.

## Checklist
- [x] Verify required skill files and installation baselines.
- [x] Re-run official Vue skills install command.
- [x] Verify superpowers native discovery/junction status.
- [x] Reproduce DBHub MCP failure and isolate root cause.
- [x] Apply DBHub startup config fix.
- [x] Re-verify DBHub bridge handshake with project DSN.
- [x] Record follow-up action (Codex restart required to reload MCP config in-session).

## Progress Timeline
- [13:47:37] Task started: gathered AGENTS/workflow/docs requirements for skills and MCP checks (in_progress)
- [13:48:21] Verified required skill files exist for superpowers, Vue skill set, rust-router, actionbook, and project workflow skill (completed)
- [13:48:44] Reproduced MCP issue: `list_mcp_resources(server=dbhub)` failed during initialize handshake (completed)
- [13:49:10] Isolated root cause: `~/.codex/config.toml` used `cmd.exe /c node ...` wrapper for dbhub bridge, which blocks proper stdio MCP framing (completed)
- [13:49:28] Updated DBHub MCP config to direct `node` execution with bridge script args (completed)
- [13:49:44] Re-ran official Vue skills install command and confirmed 8 Vue skills installed successfully (completed)
- [13:49:54] Verified superpowers repository/junction baseline remains healthy (completed)
- [13:50:02] Re-validated DBHub bridge initialize handshake against project SQLite DSN via direct MCP framing probe (completed)
- [13:50:08] Confirmed current session still uses pre-change MCP process state; restart required for dbhub tool exposure refresh (completed)
- [13:50:52] Synced task summary into `docs/development-progress.md` (completed)
- [13:53:20] Verified DBHub MCP `tools/list` works through bridge (`execute_sql`, `search_objects`) with project DSN (completed)
- [13:53:39] Synced additional DBHub `tools/list` verification evidence into `docs/development-progress.md` (completed)

## Verification
- command: `npx skills add vuejs-ai/skills --yes --global`
- result: passed; installer reported 8 Vue skills installed.
- command: `git -C "$env:USERPROFILE\\.codex\\superpowers" pull`
- result: passed (`Already up to date.`).
- command: `Get-Item "$env:USERPROFILE\\.agents\\skills\\superpowers" | Select-Object FullName,LinkType,Target`
- result: passed; junction target is `C:\Users\zw\.codex\superpowers\skills`.
- command: `list_mcp_resources(server=\"dbhub\")`
- result: failed before fix in-session (`MCP startup failed: connection closed: initialize response`), reproduced.
- command: direct Node MCP probe to `C:/Users/zw/.codex/scripts/dbhub-stdio-bridge.cjs --dsn sqlite:///C:/Users/zw/AppData/Roaming/com.pureadmin.thin/db/pure-admin-thin.sqlite3`
- result: passed; returned valid `initialize` response from DBHub server.
- command: direct Node MCP probe with `tools/list` against same bridge/DSN
- result: passed; returned `execute_sql` and `search_objects`.

## Completion
- status: done (configuration fix applied; restart pending for current session pickup)
- follow-up: restart Codex CLI and re-check `list_mcp_resources(server=\"dbhub\")`/DBHub tools exposure in a fresh session.
