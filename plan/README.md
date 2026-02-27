# Plan Directory

This directory stores task plans and timestamped progress logs.

## Naming
- Create one plan file per task:
  - `YYYY-MM-DD-HHMM-<task>.md`

## Required Sync Rule
- After each meaningful change, append a timestamp line in the active plan file:
  - `- [HH:mm:ss] <what changed> (<status>)`
- Keep entries append-only and ordered by time.

## Completion Rule
- When task implementation is done, summarize the final result in `docs/development-progress.md`.
