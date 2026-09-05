#!/usr/bin/env bash
# Local pre-PR checks — mirrors .github/workflows/ci.yml (except MSRV job and example).
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

echo "All checks passed."
