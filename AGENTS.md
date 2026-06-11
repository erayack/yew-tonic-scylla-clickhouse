# Agent guide

Rust full-stack template: Yew WASM frontend, tonic gRPC server, ScyllaDB operational store, ClickHouse analytics store.

**Tooling:** Cargo workspace. Frontend builds with [Trunk](https://trunk-rs.github.io/trunk/) for `wasm32-unknown-unknown`.

## Commands

```bash
cargo check --workspace
cargo check -p frontend --target wasm32-unknown-unknown
cargo test -p server
cargo test -p frontend
```

Frontend dev server:

```bash
cd crates/frontend
FRONTEND_BACKEND_URL=http://127.0.0.1:50051 trunk serve
```

## Reference

| Topic | Doc |
| --- | --- |
| Layout and architecture | [docs/agents/architecture.md](docs/agents/architecture.md) |
| gRPC API and frontend boundary | [docs/agents/api.md](docs/agents/api.md) |
| Database schema and migrations | [docs/agents/database.md](docs/agents/database.md) |
| Extending this template | [docs/agents/extending.md](docs/agents/extending.md) |
| Validation and smoke tests | [docs/agents/validation.md](docs/agents/validation.md) |

Human setup (Docker, ports, env vars): [README.md](README.md)
