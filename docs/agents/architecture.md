# Architecture

## Layout

```text
.
├── proto/app.proto                 # Canonical gRPC API contract
├── crates/shared                   # Generated protobuf Rust types
├── crates/server                   # tonic server and database adapters
├── crates/frontend                 # Yew WebAssembly frontend
├── migrations/scylla/001_init.cql  # Scylla schema reference
├── migrations/clickhouse/001_init.sql
├── docker-compose.yml
├── Dockerfile.server
└── Dockerfile.frontend
```

## Call graph

```text
main -> gRPC module -> events module -> storage adapters
```

| File | Role |
| --- | --- |
| `crates/server/src/grpc.rs` | gRPC transport and request validation |
| `crates/server/src/events.rs` | `EventRepository`, cross-store write ordering, `OperationalEventStore` / `AnalyticsEventStore` traits |
| `crates/server/src/scylla.rs` | Scylla adapter and startup schema bootstrap |
| `crates/server/src/clickhouse.rs` | ClickHouse adapter and startup schema bootstrap |
| `crates/frontend/src/api.rs` | gRPC-Web client (frames, trailers, protobuf) |
| `crates/frontend/src/main.rs` | Yew UI; calls `api::health_check` and `api::create_event` |

## Storage seams

Production uses ScyllaDB for `OperationalEventStore` and ClickHouse for `AnalyticsEventStore`.

Tests in `events.rs` use in-memory adapters, so event ordering tests run without Docker databases.

`CreateEvent` writes to Scylla first, then ClickHouse. That ordering is implemented in `EventRepository::create_event`.
