# Deployment Strategy (Vue + Tauri)

## Goal
Use a repeatable desktop-first release pipeline with optional static web deployment for preview/docs.

## Recommended Release Topology
1. Build frontend assets for Tauri:
   - `pnpm build:tauri` (relative base path for desktop asset loading).
2. Build desktop bundles:
   - `pnpm tauri build`.
3. Use CI matrix to build Windows/macOS/Linux artifacts.
4. Sign updater artifacts and publish release assets.
5. Serve updater metadata and signatures from release channel.

## CI/CD Baseline (Best Practice)
- Use `tauri-apps/tauri-action` for cross-platform release builds and artifact uploads.
- Build per OS target in matrix mode to avoid cross-compilation drift.
- Keep release automation in a dedicated publish workflow (trigger branch/tag based).

## Signing and Updater Baseline
- Keep updater signing enabled and provide keys through CI secrets.
- Use `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` in CI runtime (do not commit keys).
- Generate and publish updater metadata (`latest.json`) with release assets when updater is enabled.

## Web Deployment Baseline (Optional)
- Use static `dist/` output for web-hosted preview or docs environments.
- `vite preview` is for local verification only, not production runtime.
- For static host CI (GitHub Pages/Netlify/Vercel/Cloudflare), deploy built `dist/` artifacts only.

## Operational Notes
- Keep desktop release channel and web static deploy independent.
- Validate update path on at least one stable channel client before broad release.
- Align release notes with bundled artifact list and updater metadata.

## Sources
- Actionbook Rust skills repository: https://github.com/actionbook/rust-skills
- Actionbook Codex install notes: https://raw.githubusercontent.com/actionbook/rust-skills/main/.codex/INSTALL.md
- Tauri distribute docs: https://v2.tauri.app/distribute/
- Tauri updater plugin docs: https://v2.tauri.app/plugin/updater/
- Tauri GitHub Action README: https://raw.githubusercontent.com/tauri-apps/tauri-action/dev/README.md
- Vite static deployment docs: https://vite.dev/guide/static-deploy.html
