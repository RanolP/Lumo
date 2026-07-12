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
  The numbered decision files (`design/decisions/`) migrated to the
  website RFCs 2026-07-13: `apps/website/src/content/rfcs/`, where
  D-NN = RFC 00NN.
- The new implementation lives in the root Cargo workspace (`crates/`,
  `lumo/`, `tests/`). The project website is `apps/website` (SolidJS +
  @solidjs/router): a promotional home with an embedded playground,
  `/playground` (Monaco over `crates/playground-wasm`), `/docs` and
  `/rfcs` (MDX under `src/content/`), and `/formalization` (generated
  from root `formalization.typ` by `pnpm typst:build`). GitHub Pages
  deploys it via `.github/workflows/website.yml`.
- **`packages/`** (root) — the ported stdlib (D-45) as `lumo.toml`
  packages built by `crates/lbs` (D-53: whole-program assembly,
  `lbs <check|build|run>`); host bindings in
  `packages/runtime/js/prelude.js`. Gates: `crates/lbs/tests/stdlib.rs`
  and `scripts/stdlib_smoke.sh` (builds + runs `packages/hello` under
  node).

## Working rules for the rewrite

- The language definition (`.langue` sections) is the source of truth; Rust
  code is engines and generated output, never the definition.
- Design decisions go in `apps/website/src/content/rfcs/` as numbered
  MDX RFCs before implementation (next number = highest + 1; keep the
  `export const title = "RFC 00NN — …"` header).
