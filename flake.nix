{
  description = "Yew + tonic + ScyllaDB + ClickHouse Rust template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rustfmt" "clippy" ];
        targets = [ "wasm32-unknown-unknown" ];
      };

      darwinLibs = pkgs.lib.optionals pkgs.stdenv.isDarwin (
        with pkgs.darwin.apple_sdk.frameworks; [ Security SystemConfiguration ]
      );
    in {
      devShells.default = pkgs.mkShell {
        packages = with pkgs; [
          rustToolchain
          protobuf
          trunk
          pkg-config
          openssl
        ] ++ darwinLibs;

        shellHook = ''
          export PROTOC=${pkgs.protobuf}/bin/protoc
          export PROTOC_INCLUDE=${pkgs.protobuf}/include
          export FRONTEND_BACKEND_URL=''${FRONTEND_BACKEND_URL:-http://127.0.0.1:50051}
          export SERVER_BIND_ADDR=''${SERVER_BIND_ADDR:-0.0.0.0:50051}
          export SCYLLA_URI=''${SCYLLA_URI:-127.0.0.1:9042}
          export CLICKHOUSE_URL=''${CLICKHOUSE_URL:-http://127.0.0.1:8123}
          export CLICKHOUSE_DATABASE=''${CLICKHOUSE_DATABASE:-default}
          export CLICKHOUSE_USER=''${CLICKHOUSE_USER:-default}
          export CLICKHOUSE_PASSWORD=''${CLICKHOUSE_PASSWORD:-}
        '';
      };
    });
}
