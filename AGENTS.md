# Repository Guidelines

## Mandatory AI Coding Skill Workflow
- Before any code edit, the agent MUST load and follow [`skills/project-aicode-workflow/SKILL.md`](skills/project-aicode-workflow/SKILL.md).
- Always run the process skill `using-superpowers` first.
- Rust work MUST use the installed Actionbook Rust skill set from `https://github.com/actionbook/rust-skills` and then pick the matching Rust skill (`rust-router` first, then topic skill).
- TypeScript/Vue work MUST use skills before editing code (`brainstorming` before design changes, `test-driven-development` before feature or bugfix implementation, `frontend-design` for UI design tasks).
- If any additional skill matches the task scope, it MUST be invoked (do not skip applicable skills).

## Tauri Framework Constraints (Mandatory)
- Any change touching `src-tauri/**`, `src/api/**` (Tauri IPC calls), or `src-tauri/tauri.conf.json` MUST follow [`docs/tauri-framework-constraints.md`](docs/tauri-framework-constraints.md).
- Treat `docs/tauri-framework-constraints.md` as the authoritative Tauri v2 coding/security checklist for AI code decisions.
- Every Tauri-related task must explicitly evaluate security boundaries (capabilities, permissions, command exposure, CSP/updater impact) before editing code.

## Task Scope Control
- Keep each task within one AI-code context window: one clear objective, a small file set, and verifiable output.
- If work is too large, split into sequential sub-tasks with explicit checkpoints.
- Every completed task must update developer documentation and progress tracking (see `docs/development-progress.md`).

## Module Size Policy
- Target each code module to stay around 400 lines.
- If a file grows beyond roughly 450 lines, split by responsibility (domain/service/view/composable/utils).
- Keep engineering tree boundaries clear: avoid mixing UI, domain logic, API adapters, and persistence logic in the same module.

## Project Structure & Module Organization
This repository combines a Vue 3 + TypeScript admin UI with a Tauri desktop shell.

- `src/`: frontend application code (`views/`, `layout/`, `router/`, `store/modules/`, `api/`, `components/`, `utils/`, `style/`)
- `src-tauri/`: Rust desktop host (`src/main.rs`, `src/lib.rs`, `tauri.conf.json`)
- `mock/`: mock API handlers for local development
- `build/`: Vite build helpers/plugins
- `public/`: static assets and runtime config (`platform-config.json`)
- `docs/`: project documentation and integration notes

## Build, Test, and Development Commands
- `pnpm install`: install dependencies (Node `^20.19.0 || >=22.13.0`, pnpm `>=9`)
- `pnpm dev`: run Vite dev server
- `pnpm tauri:dev`: run desktop app in development mode
- `pnpm build`: create production web build in `dist/`
- `pnpm build:tauri`: build frontend assets with relative base path for Tauri packaging
- `pnpm typecheck`: run `tsc` and `vue-tsc` checks
- `pnpm lint`: run ESLint, Prettier, and Stylelint
- `cargo test --manifest-path src-tauri/Cargo.toml`: run Rust tests

## Coding Style & Naming Conventions
- Follow `.editorconfig`: UTF-8, LF, 2-space indentation, trailing whitespace trimmed.
- Prettier config uses double quotes, no trailing commas, and minimal arrow parentheses.
- Vue components are typically organized by feature folder; many route/page entries use `index.vue`.
- Composables follow `useX.ts` (example: `src/layout/hooks/useLayout.ts`).
- Keep shared utilities and API wrappers typed; prefer explicit exported types in `src/types/`.

## Testing Guidelines
- No dedicated JS unit test runner is currently configured; use `pnpm typecheck` and `pnpm lint` as required quality gates.
- For UI changes, manually verify affected routes/pages with `pnpm dev`.
- For Tauri/Rust changes, add unit tests in `src-tauri/src` (`#[cfg(test)]`) and run `cargo test`.

## Commit & Pull Request Guidelines
- Conventional commits are enforced by commitlint (`feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `release`, etc.).
- Keep commit subjects short, imperative, and under 108 characters.
- PRs should include:
  - clear summary and scope
  - linked issue/ticket
  - screenshots for UI changes
  - verification steps and results (`pnpm lint`, `pnpm typecheck`, and Rust checks when relevant)

## Security & Configuration Tips
- Never commit secrets to `.env*` files.
- Review runtime config changes in `public/platform-config.json` and Tauri settings in `src-tauri/tauri.conf.json` carefully.
