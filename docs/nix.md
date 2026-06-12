# Nix

Reproducible local tooling via the pinned flake dev shell. Commit both `flake.nix` and `flake.lock` so everyone gets the same Nix inputs.

If flakes are enabled in your Nix config:

```bash
nix develop
```

If your Nix install has flakes disabled by default:

```bash
nix --extra-experimental-features 'nix-command flakes' develop
```

To enable flakes permanently, add this to `~/.config/nix/nix.conf`:

```conf
experimental-features = nix-command flakes
```

With [direnv](https://direnv.net/), run `direnv allow` once — `.envrc` loads the flake automatically.

The shell includes Rust (with `wasm32-unknown-unknown`), `protoc`, `trunk`, and OpenSSL. It also sets the local-development env vars from [README.md](../README.md), so you can run `cargo run -p server` and `trunk serve` without exporting them yourself.

Validation commands: [docs/agents/validation.md](agents/validation.md).
