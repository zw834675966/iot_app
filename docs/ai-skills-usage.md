# AI Skills Setup and Routing

This project uses two external skill sources for Codex:
- `vuejs-ai/skills`: Vue-focused implementation and debugging skills
- `obra/superpowers`: process and execution workflow skills

## Official Sources

- Vue skills README: https://github.com/vuejs-ai/skills
- Superpowers install guide: https://raw.githubusercontent.com/obra/superpowers/refs/heads/main/.codex/INSTALL.md

## Installation Baseline (Windows)

### 1) Install/update Vue skills

```powershell
npx skills add vuejs-ai/skills --yes --global
```

### 2) Install/update superpowers with native discovery

```powershell
git -C "$env:USERPROFILE\.codex\superpowers" pull
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.agents\skills" | Out-Null
cmd /c rmdir "$env:USERPROFILE\.agents\skills\superpowers"
cmd /c mklink /J "$env:USERPROFILE\.agents\skills\superpowers" "$env:USERPROFILE\.codex\superpowers\skills"
```

If `~/.codex/superpowers` does not exist yet, clone first:

```powershell
git clone https://github.com/obra/superpowers.git "$env:USERPROFILE\.codex\superpowers"
```

### 3) Remove legacy bootstrap (if present)

Delete old blocks in `%USERPROFILE%\.codex\AGENTS.md` that reference `superpowers-codex bootstrap`.

### 4) Restart Codex

Restart the Codex CLI after install/update so skill discovery refreshes.

## Verification Commands

```powershell
Get-ChildItem "$env:USERPROFILE\.agents\skills" | Select-Object Name
Get-Item "$env:USERPROFILE\.agents\skills\superpowers" | Select-Object FullName,LinkType,Target
```

Expected:
- Vue skills exist in `%USERPROFILE%\.agents\skills\*`
- `superpowers` is a junction/symlink to `%USERPROFILE%\.codex\superpowers\skills`

## Vue Skill Auto-Routing Rules

For any Vue-related task, invoke the best-match skill first:

- Router, navigation guards, route lifecycle: `vue-router-best-practices`
- Pinia stores and state flow: `vue-pinia-best-practices`
- Options API (`data`, `methods`, `this`): `vue-options-api-best-practices`
- Component/unit/e2e test design: `vue-testing-best-practices`
- JSX/TSX in Vue: `vue-jsx-best-practices`
- Runtime warnings/errors/hydration/debugging: `vue-debug-guides`
- Reusable composable API design: `create-adaptable-composable`
- Generic Vue 3 + TS implementation: `vue-best-practices` (default fallback)

When a task spans multiple areas, invoke all relevant skills in most-specific-first order.

## Recommended Prompting Pattern

When a prompt is broad and you want stronger skill triggering, prepend:

```text
Use vue skill, <task>
```

## Maintenance

- Vue skills: rerun `npx skills add vuejs-ai/skills --yes --global`
- Superpowers: `git -C "$env:USERPROFILE\.codex\superpowers" pull`
- Restart Codex after updates
