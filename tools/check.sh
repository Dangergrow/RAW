#!/usr/bin/env bash
set -euo pipefail

echo "==> fmt"
cargo fmt --all

echo "==> clippy"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> test (online)"
cargo test --workspace

if [[ -f .cargo/config.toml ]]; then
  echo "==> test (offline)"
  cargo test --workspace --offline
fi
