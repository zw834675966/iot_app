# Database Access Policy (SeaORM / sqlx / Raw SQL / TimescaleDB)

Last updated: 2026-02-28
Applies to: `src-tauri/src/db/**`, `src-tauri/src/auth/**`, `src-tauri/src/notice/**`

## Goal
Define mandatory routing rules for:
- when to use SeaORM
- when to use sqlx or raw SQL
- when to write to PostgreSQL OLTP tables
- when to write to TimescaleDB time-series structures

This policy is mandatory for all new backend data-access code.

## Rule 1: SeaORM First for Standard CRUD (Default)
Use SeaORM as the default for standard business CRUD paths (about 80% cases).

Use SeaORM when all conditions are true:
1. Single-entity or simple relation CRUD (`create/read/update/delete`).
2. Query is mainly equality/range filters + ordering + pagination.
3. No database-specific extension function is required.
4. Result shape maps naturally to entity/DTO without complex aggregation.

Typical project examples:
- user account lifecycle: create/update/delete/enable/disable.
- role binding table write operations.
- device registration metadata CRUD.
- configuration publish state CRUD.

## Rule 2: sqlx / Raw SQL for Complex Query Paths
Use `sqlx` or raw SQL for advanced/DB-specific queries (about 20% cases).

Use sqlx/raw SQL when any condition is true:
1. Multi-table aggregation with `GROUP BY`/`STRING_AGG`/window functions.
2. Recursive CTE, lateral joins, materialized view reads.
3. TimescaleDB functions (`time_bucket`, `first`, `last`, `approx_percentile`, continuous aggregates).
4. Query plan/performance must be fully SQL-tuned and explain-analyzed.
5. Large batch ingest/update where handcrafted SQL is significantly better.

Project examples already matching this rule:
- auth route tree / permission aggregation queries.
- admin list/report style aggregate reads.
- migration/bootstrap SQL scripts.

## Rule 3: PostgreSQL vs TimescaleDB Write Routing

### 3.1 Write to PostgreSQL OLTP tables when data is transactional
Write to normal PostgreSQL tables for:
1. identity/auth: users, roles, permissions, tokens.
2. business state with frequent updates/deletes.
3. relational config and management metadata.
4. records requiring strict transactional consistency with user/admin operations.

Current OLTP examples in this project:
- `users`, `user_roles`, `permissions`, `user_permissions`
- `routes`, `route_roles`, `route_auths`
- `device_registry`, `notice_items`, `app_migrations`

### 3.2 Write to TimescaleDB when data is telemetry/event time-series
Write to TimescaleDB hypertables for data that is:
1. append-heavy, time-ordered, high-volume.
2. queried by time window, downsampling, retention.
3. mostly immutable after ingest (few updates/deletes).

Typical IoT examples:
- device telemetry points (temperature, power, pressure, etc.).
- high-frequency status/event streams.
- KPI time-series for dashboards and trend analytics.

## Rule 4: Repository Layer Boundary
1. Keep repository public APIs business-oriented; do not leak SQL details upward.
2. For each complex sqlx/raw query, add a one-line boundary comment:
   - why SeaORM is not used
   - which SQL feature/performance requirement requires raw SQL
3. Do not mix unrelated concerns in one repository file.
4. If a repository file exceeds ~450 lines, split by responsibility.

## Rule 5: Performance and Safety Checklist (Required in PR/Review)
For each new data-access change, reviewers must confirm:
1. Chosen path matches Rule 1/2.
2. Query uses parameter binding (no string-concatenated SQL).
3. Index coverage exists for filter/order columns.
4. For time-series writes, retention/compression policy impact is considered.
5. Error mapping stays inside `AppError::Database` / `AppError::Validation` boundaries.

## Quick Decision Matrix

| Scenario | Use | Write Target |
|---|---|---|
| User CRUD / role assignment | SeaORM | PostgreSQL OLTP |
| Login profile + permission aggregation | sqlx/raw SQL | PostgreSQL OLTP |
| Device metadata management | SeaORM | PostgreSQL OLTP |
| Telemetry ingest (high frequency) | sqlx/raw SQL (bulk) or tuned ingest path | TimescaleDB hypertable |
| 1m/5m/1h downsampling analytics | raw SQL with `time_bucket` or CAGG | TimescaleDB |
| Seed/migration/bootstrap | sqlx/raw SQL | PostgreSQL/TimescaleDB schema objects |

## Anti-Patterns (Disallowed)
1. Using raw SQL for trivial CRUD where SeaORM is sufficient.
2. Using SeaORM for extension-heavy analytics where SQL clarity/perf is degraded.
3. Writing high-frequency telemetry into OLTP entity tables.
4. Embedding SQL strings in service/command layer instead of repository.

## Migration Guidance for Existing Code
1. Keep existing stable complex SQL paths as-is unless there is a clear gain.
2. Move standard CRUD incrementally to SeaORM during touch/refactor windows.
3. Do not do broad rewrites without benchmark or correctness motivation.
