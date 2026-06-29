#!/usr/bin/env bash
set -euo pipefail

echo "[bootstrap] rust version"
rustc --version
cargo --version

echo "[bootstrap] workspace check"
cargo check --workspace

echo "[bootstrap] workspace test"
cargo test --workspace

echo "[bootstrap] lint"
cargo clippy --workspace --all-targets -- -D warnings

echo "[bootstrap] fmt check"
cargo fmt --all -- --check

echo "[bootstrap] done"
