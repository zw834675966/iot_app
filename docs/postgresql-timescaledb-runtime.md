# PostgreSQL + TimescaleDB Runtime Notes (src-tauri)

This backend now uses PostgreSQL (`sqlx` with postgres driver) for both auth/admin data and notice center data.

## Access policy

For ORM/query/write-routing constraints, treat this as mandatory:

- [`docs/database-access-policy.md`](./database-access-policy.md)

## Connection URL

The app reads database URL in this order:

1. `PURE_ADMIN_DATABASE_URL` environment variable
2. fallback default in code: `postgres://postgres:EMSzw%4018627652962@127.0.0.1:5432/pure_admin_thin`

For tests, default is:

- `postgres://postgres:EMSzw%4018627652962@127.0.0.1:5432/pure_admin_thin_test`

Override with:

- `PURE_ADMIN_TEST_DATABASE_URL`

## Required Extensions

TimescaleDB is required at database level. Run once per database:

```sql
CREATE EXTENSION IF NOT EXISTS timescaledb;
```

## Suggested local bootstrap

```powershell
$env:PGPASSWORD='EMSzw@18627652962'
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
