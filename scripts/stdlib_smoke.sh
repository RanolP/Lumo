#!/usr/bin/env bash
# D-45/D-53 smoke: build the stdlib-backed hello package with lbs and
# run it under node. Keep -j 2 — parallel rustc OOMs this machine.
set -euo pipefail
cd "$(dirname "$0")/.."

mkdir -p target  # hello's FS round-trip writes target/stdlib-smoke.txt
cargo run -j 2 -q -p lbs -- run packages/hello
