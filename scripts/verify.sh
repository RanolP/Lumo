#!/usr/bin/env bash
# The M0 verification loop: definition checks, generated-code freshness,
# and the full test suite (corpus included). Keep -j 2 — parallel rustc
# OOMs this machine.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "== langc check lumo =="
cargo run -j 2 -q -p langc -- check lumo

echo "== langc gen --check (committed generated code is current) =="
cargo run -j 2 -q -p langc -- gen lumo -o crates/lumo-syntax/src --check

echo "== cargo test --workspace =="
cargo test -j 2 --workspace --quiet

echo "verify: ALL GREEN"
