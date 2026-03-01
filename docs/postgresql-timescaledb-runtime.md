# PostgreSQL + TimescaleDB Runtime Notes (src-tauri)

This backend now uses PostgreSQL (`sqlx` with postgres driver) for both auth/admin data and notice center data.

## Access policy

For ORM/query/write-routing constraints, treat this as mandatory:

- [`docs/database-access-policy.md`](./database-access-policy.md)

## Connection URL

The app reads database URL in this order:

1. `src-tauri/config/default.toml`
2. `src-tauri/config/local.toml` (optional local override)
3. Environment variables (highest priority):
   - `PURE_ADMIN_DATABASE_URL` / `PURE_ADMIN_DATABASE__URL`
   - `PURE_ADMIN_TEST_DATABASE_URL` / `PURE_ADMIN_DATABASE__TEST_URL`
   - `PURE_ADMIN_JWT_SECRET` / `PURE_ADMIN_AUTH__JWT_SECRET`
   - `PURE_ADMIN_SERVER_PORT` / `PURE_ADMIN_SERVER__PORT`

For tests, database URL resolution is:

1. `PURE_ADMIN_TEST_DATABASE_URL` (if set)
2. `database.test_url` from config files
3. `database.url` from config files

## Required Extensions

TimescaleDB is required at database level. Run once per database:

```sql
CREATE EXTENSION IF NOT EXISTS timescaledb;
```

## Suggested local bootstrap

```powershell
$env:PGPASSWORD='<YOUR_POSTGRES_PASSWORD>'
psql -h 127.0.0.1 -U postgres -d postgres -c "CREATE DATABASE pure_admin_thin"
psql -h 127.0.0.1 -U postgres -d postgres -c "CREATE DATABASE pure_admin_thin_test"
psql -h 127.0.0.1 -U postgres -d pure_admin_thin -c "CREATE EXTENSION IF NOT EXISTS timescaledb"
psql -h 127.0.0.1 -U postgres -d pure_admin_thin_test -c "CREATE EXTENSION IF NOT EXISTS timescaledb"
```

## Migration runtime behavior

- SQL migration scripts are executed through `sqlx::raw_sql`.
- `db::init_database` uses PostgreSQL advisory lock to avoid concurrent initialization races in parallel test runs.

## Storage changes

- Removed local sqlite file storage path usage in backend.
- Removed `redb` notice storage; notice items are now in PostgreSQL table `notice_items`.
