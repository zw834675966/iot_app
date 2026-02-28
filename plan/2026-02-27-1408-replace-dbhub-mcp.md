# 2026-02-27-1408-replace-dbhub-mcp

## Objective
- Replace DBHub MCP with another SQLite-capable MCP server and keep repository policy/docs aligned.

## Scope
- Switch local Codex MCP config from `dbhub` to a new SQLite MCP server.
- Add/adjust bridge script if protocol adaptation is required.
- Update repository policy/docs from DBHub-specific wording to the new MCP server.

## Checklist
- [x] Evaluate replacement candidates and confirm Node/Windows compatibility.
- [x] Implement local MCP config change.
- [x] Update repository policy/docs (`AGENTS`, skill workflow, `mcp.md`, db README).
- [x] Verify MCP handshake and basic tool discovery.
- [x] Sync task result into `docs/development-progress.md`.

## Progress Timeline
- [14:08:01] Task started; loaded required skills and workflow constraints (in_progress)
- [14:09:22] Added `sqlite-tools-stdio-bridge.cjs`, switched local MCP config from `dbhub` to `sqlite_tools`, and verified bridge-level MCP initialize + tools/list (completed)
- [14:10:57] Updated repository DB MCP policies/docs from `dbhub` to `sqlite_tools` (`AGENTS`, workflow SKILL, `mcp.md`, db README) (completed)
- [14:15:12] Re-verified local config and bridge file existence, then executed framed MCP probe against bridge (initialize + tools/list passed; 19 tools discovered) (completed)
- [14:15:46] Synced completion details into `docs/development-progress.md` (completed)

## Verification
- command: `Get-Content "$env:USERPROFILE/.codex/config.toml" -Raw`
- result: contains `[mcp_servers.sqlite_tools]` with Node + `sqlite-tools-stdio-bridge.cjs` and SQLite env vars.
- command: `@'...MCP framed probe JS...'@ | node -`
- result: passed (`initialize_ok=true`, `tools_count=19`, sample includes `execute_read_query`, `execute_write_query`, `execute_schema_query`, `list_tables`, `describe_table`).
- command: `rg -n "dbhub|sqlite_tools|SQLite Tools MCP|DBHub" AGENTS.md src-tauri/AGENTS.md skills/project-aicode-workflow/SKILL.md src-tauri/src/db/README.md mcp.md`
- result: repository policies/docs switched to `sqlite_tools`; no remaining DBHub policy references in these target files.
- command: `list_mcp_resources(server="sqlite_tools")`
- result: current session still reports `unknown MCP server 'sqlite_tools'`, indicating the runtime registry has not reloaded this new MCP config yet.

## Completion
- status: completed
- follow-up: restart Codex CLI once more, then re-check `list_mcp_resources(server="sqlite_tools")` / tool invocation in fresh session.
