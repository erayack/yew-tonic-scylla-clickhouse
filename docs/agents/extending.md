# Extending the template

Typical first steps after creating a repo from this template:

1. Rename crate package names in the three `Cargo.toml` files if desired.
2. Replace `proto/app.proto` with your domain API; types regenerate via the `crates/shared` build script.
3. Implement new RPCs in `crates/server/src/grpc.rs`.
4. Add or change multi-store write logic in `crates/server/src/events.rs` (or replace/rename that module as your domain grows).
5. Update `migrations/scylla/` and `migrations/clickhouse/`; keep startup bootstrap in sync until you switch to dedicated migration tooling.
6. Update the Yew UI in `crates/frontend/src/main.rs` and browser API calls in `crates/frontend/src/api.rs`.

See [architecture.md](architecture.md) for the current module map.
