# Database

## Source of truth

Schema files under `migrations/`:

- [migrations/scylla/001_init.cql](../../migrations/scylla/001_init.cql) — `app.events` keyspace and table
- [migrations/clickhouse/001_init.sql](../../migrations/clickhouse/001_init.sql) — `events_analytics` MergeTree table

Do not duplicate full DDL in docs; edit the migration files.

## Bootstrap policy

For local one-command usability, the server reads and applies those migration files at startup (`scylla.rs`, `clickhouse.rs`).

Replace startup bootstrap with real migration tooling before production.

## Write semantics

`CreateEvent` writes the full operational row to Scylla (including `payload`), then writes a slimmer analytics row to ClickHouse (no `payload` column). Both stores share the same generated `id` and `created_at`.

For stronger cross-store consistency in production, consider an outbox or event pipeline on top of the current write path.
