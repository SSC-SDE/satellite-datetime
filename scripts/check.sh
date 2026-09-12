#!/usr/bin/env bash
# Local pre-PR checks — mirrors pipeline dev + qa stages (see .github/workflows/pipeline.yml).
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> cargo fmt --check"
cargo fmt --all -- --check

echo "==> cargo test --all-features"
cargo test --all-features

echo "==> cargo test --no-default-features --lib"
cargo test --no-default-features --lib

echo "==> cargo run --example rfc3339_leap --features earth"
cargo run --example rfc3339_leap --features earth

echo "==> cargo clippy"
cargo clippy --all-features -- -D warnings

echo "==> cargo package --locked"
cargo package --locked

if command -v rustup >/dev/null 2>&1 && rustup target list --installed | grep -q '^thumbv7em-none-eabihf$'; then
  echo "==> cargo check --no-default-features --target thumbv7em-none-eabihf"
  cargo check --no-default-features --target thumbv7em-none-eabihf
else
  echo "==> skip thumbv7em (install with: rustup target add thumbv7em-none-eabihf)"
fi

echo "All checks passed."
