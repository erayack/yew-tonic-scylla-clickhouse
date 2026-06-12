# Yew + tonic + ScyllaDB + ClickHouse Template

A GitHub template for a Rust full-stack app: Yew (WASM), tonic (gRPC), gRPC-Web, ScyllaDB, and ClickHouse. One vertical slice submits an event, validates it, writes to Scylla then ClickHouse, and returns the event ID.

## Prerequisites

Docker-only quick start: Docker and Docker Compose.

Local app development: Rust stable, the `wasm32-unknown-unknown` target, and Trunk:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
```

```bash
nix develop
```

If flakes are not enabled globally yet:

```bash
nix --extra-experimental-features 'nix-command flakes' develop
```

## Quick start

```bash
cp .env.example .env
docker compose up --build
```

Open <http://127.0.0.1:8080>.

The frontend is compiled with `FRONTEND_BACKEND_URL=http://127.0.0.1:50051` in Docker, because browser code runs on the host and cannot resolve Docker service names such as `server`.

## Local development

Run databases in Docker:

```bash
docker compose up scylla clickhouse
```

Server:

```bash
export SERVER_BIND_ADDR=0.0.0.0:50051
export SCYLLA_URI=127.0.0.1:9042
export CLICKHOUSE_URL=http://127.0.0.1:8123
export CLICKHOUSE_DATABASE=default
export CLICKHOUSE_USER=default
export CLICKHOUSE_PASSWORD=
cargo run -p server
```

Frontend:

```bash
cd crates/frontend
FRONTEND_BACKEND_URL=http://127.0.0.1:50051 trunk serve
```

## Ports

| Service | Host port | Container port | Purpose |
| --- | ---: | ---: | --- |
| Frontend | 8080 | 80 | Yew static app |
| Server | 50051 | 50051 | tonic gRPC + gRPC-Web |
| ScyllaDB | 9042 | 9042 | CQL |
| ClickHouse HTTP | 8123 | 8123 | HTTP API |
| ClickHouse native | 9000 | 9000 | Native protocol |

## Environment variables

| Variable | Docker default | Local default/example | Purpose |
| --- | --- | --- | --- |
| `SERVER_BIND_ADDR` | `0.0.0.0:50051` | `0.0.0.0:50051` | Backend listen address |
| `SCYLLA_URI` | `scylla:9042` | `127.0.0.1:9042` | Scylla contact point |
| `CLICKHOUSE_URL` | `http://clickhouse:8123` | `http://127.0.0.1:8123` | ClickHouse HTTP URL |
| `CLICKHOUSE_DATABASE` | `default` | `default` | ClickHouse database |
| `CLICKHOUSE_USER` | `default` | `default` | ClickHouse user |
| `CLICKHOUSE_PASSWORD` | empty | empty | ClickHouse password |
| `FRONTEND_BACKEND_URL` | `http://127.0.0.1:50051` | `http://127.0.0.1:50051` | Browser-visible backend URL |

## Further reading

- [AGENTS.md](AGENTS.md) — architecture, API, and validation