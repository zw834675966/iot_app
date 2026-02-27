# Rust Agent Rules

## Required Skills Before Editing
- First load `using-superpowers`.
- Rust work must start with `rust-router`, then load the matching Rust topic skills.
- Follow Actionbook Rust skills practices from `https://github.com/actionbook/rust-skills`.
- For bugfixes, apply `systematic-debugging`; for implementation, apply `test-driven-development`.
- Before any Tauri code/config change, load and apply `../docs/tauri-framework-constraints.md`.

## Mandatory Tauri Thinking Checklist (Every Task)
- Capability and permission impact: does this change require updates in `capabilities/*.json|toml` or `permissions/*.toml`?
- Command exposure impact: are we exposing extra commands, and should they be constrained via app manifest/capabilities?
- Runtime and concurrency impact: should command be `async`, and are state/locks used safely?
- Security impact: any CSP relaxation, remote content, updater endpoint, or key-handling risk?
- Version sync impact: are `@tauri-apps/*` and Rust crates still version-compatible?

## Scope and Module Rules
- Keep each task focused enough for one context window and one verification pass.
- Keep each Rust module around 400 lines; split when a file approaches ~450 lines.
- Separate command handlers, domain logic, models, and error handling into dedicated modules.

## Completion Rules
- Run `cargo test --manifest-path src-tauri/Cargo.toml` and targeted lint/format checks when relevant.
- Update Rust-facing developer docs and append a progress entry in `docs/development-progress.md`.
