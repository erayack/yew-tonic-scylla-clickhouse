# Validation

## Rust

```bash
cargo fmt --check
cargo check --workspace
cargo check -p frontend --target wasm32-unknown-unknown
cargo test -p server
cargo test -p frontend
```

Event ordering tests run without Docker (`cargo test -p server events`).

## Docker

```bash
docker compose config
docker compose build server frontend
```

Full stack smoke test (after `cp .env.example .env`):

```bash
docker compose up --build
```

Open <http://127.0.0.1:8080> and submit an event.

## Local split dev

See [README.md](../../README.md) for database-only Docker, server env vars, and `trunk serve`.
