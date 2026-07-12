# Lumo Project — Claude Instructions

## Repository layout (since 2026-07-09)

- **`crates/lumo-syntax/tests/data/`** — real-source parse-gate corpora
  inherited from the pre-rewrite implementation, read by the crate's
  `parse_gates.rs`: `fixtures/` (case sources; the unmigrated
  expectation buckets — resume, bounds, assoc_types, cap_inference,
  exhaustiveness — also live here as future-feature reference) and
  `packages/**/*.lumo`. The old `legacy/` tree itself was pruned
  2026-07-12–13; comments citing `legacy/...` paths refer to git
  history.
- **`design/`** — design documents for the fresh, DSL-driven rewrite.
  Start with `design/langue.md` (Langue 2: full language-definition DSL).
- The new implementation lives in the root Cargo workspace (`crates/`,
  `lumo/`, `tests/`). The browser playground is `apps/playground`
  (SolidJS + Monaco) over `crates/playground-wasm`; GitHub Pages deploys
  it via `.github/workflows/playground.yml`.
- **`packages/`** (root) — the ported stdlib (D-45) as `lumo.toml`
  packages built by `crates/lbs` (D-53: whole-program assembly,
  `lbs <check|build|run>`); host bindings in
  `packages/runtime/js/prelude.js`. Gates: `crates/lbs/tests/stdlib.rs`
  and `scripts/stdlib_smoke.sh` (builds + runs `packages/hello` under
  node).

## Working rules for the rewrite

- The language definition (`.langue` sections) is the source of truth; Rust
  code is engines and generated output, never the definition.
- Design decisions go in `design/` before implementation.
