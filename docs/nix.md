# Nix

Reproducible local tooling via the flake dev shell.

```bash
nix develop
```

With [direnv](https://direnv.net/), run `direnv allow` once — `.envrc` loads the flake automatically.

The shell includes Rust (with `wasm32-unknown-unknown`), `protoc`, `trunk`, and OpenSSL. It also sets the local-development env vars from [README.md](../README.md), so you can run `cargo run -p server` and `trunk serve` without exporting them yourself.

Validation commands: [docs/agents/validation.md](agents/validation.md).
