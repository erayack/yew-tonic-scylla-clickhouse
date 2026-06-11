# API and contracts

## Canonical contract

`proto/app.proto` defines `app.v1.AppService`. Generated types live in `crates/shared`.

## RPC behavior

| RPC | Behavior |
| --- | --- |
| `HealthCheck` | Returns `status = "ok"` when the server process is running (no database probe). |
| `CreateEvent` | Accepts `name` and `payload`; rejects empty/whitespace-only `name`; writes Scylla first, then ClickHouse; returns a UUID string. |

Name trimming/validation lives in `crates/server/src/grpc.rs`. Persistence ordering lives in `crates/server/src/events.rs`.

## Frontend transport

The browser calls tonic via gRPC-Web frames. There is no JSON compatibility API in the starter.

- `crates/frontend/src/api.rs` — gRPC-Web client
- `crates/frontend/src/main.rs` — Yew UI

## Where things live today

| Change | Starting point |
| --- | --- |
| Add or change RPCs | `proto/app.proto`, then `crates/server/src/grpc.rs` |
| Change write ordering or storage seams | `crates/server/src/events.rs` |
| Change browser protocol details | `crates/frontend/src/api.rs` |
| Change UI | `crates/frontend/src/main.rs` |
