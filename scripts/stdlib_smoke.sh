#!/usr/bin/env bash
# D-45 smoke: compile the ported stdlib unit and run it under node with
# the runtime prelude. Keep -j 2 — parallel rustc OOMs this machine.
set -euo pipefail
cd "$(dirname "$0")/.."

mkdir -p target
{
  cat packages/runtime/js/prelude.js
  cargo run -j 2 -q -p lumo-syntax --example compile_stdlib
  echo
  echo "main();"
} > target/stdlib-smoke.js
node target/stdlib-smoke.js
