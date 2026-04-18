#!/usr/bin/env bash
# Regenerate syntax_kind.rs and ast.rs from .langue grammar files
# using the Lumo-bootstrapped langue tool (packages/langue/dist/langue.js).
#
# Usage:
#   scripts/gen_langue.sh          # regenerate all targets
#   scripts/gen_langue.sh compiler  # regenerate only compiler

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LANGUE="$REPO_ROOT/packages/langue/dist/langue.js"

if ! command -v node &>/dev/null; then
  echo "error: node not found in PATH" >&2
  exit 1
fi

if [[ ! -f "$LANGUE" ]]; then
  echo "error: langue.js not found at $LANGUE" >&2
  echo "  Build it first: lbs build --target js packages/langue" >&2
  exit 1
fi

gen() {
  local name="$1" input="$2" output_dir="$3"
  echo "generating $name..."
  node "$LANGUE" "$input" "$output_dir"
}

targets="${1:-all}"

case "$targets" in
  compiler|all)
    gen "compiler" "$REPO_ROOT/crates/compiler/lumo.langue" "$REPO_ROOT/crates/compiler/src"
    ;;&
  hir|all)
    gen "hir" "$REPO_ROOT/crates/hir/hir.langue" "$REPO_ROOT/crates/hir/src"
    ;;&
  lir|all)
    gen "lir" "$REPO_ROOT/crates/lir/lir.langue" "$REPO_ROOT/crates/lir/src"
    ;;&
  all) ;;
  compiler|hir|lir) ;;
  *)
    echo "unknown target: $targets" >&2
    echo "usage: $0 [compiler|hir|lir|all]" >&2
    exit 1
    ;;
esac

echo "done."
