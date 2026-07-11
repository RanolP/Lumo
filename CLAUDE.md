# Lumo Project — Claude Instructions

## Repository layout (since 2026-07-09)

- **`legacy/`** — the previous implementation, archived as read-only
  reference; do not extend it. Pruned 2026-07-12: parts reimplemented by
  the rewrite (lexer/lst/span/hir/lir/lir-memaware/types/simple-ts-ast
  crates, apps/lumoc, scripts/) were deleted — recover via git history if
  needed. What remains is still load-bearing or unported:
  `crates/compiler` (test-fixture gates + caps/LTO/query reference),
  `packages/` (parse-gate sources + unported stdlib), `crates/{lbs,lsp,
  playground-wasm}`, `apps/playground`, `docs/`, `plans/`. The legacy
  workspace no longer builds.
- **`design/`** — design documents for the fresh, DSL-driven rewrite.
  Start with `design/langue.md` (Langue 2: full language-definition DSL).
- The new implementation lives in the root Cargo workspace (`crates/`,
  `lumo/`, `tests/`).

## Working rules for the rewrite

- The language definition (`.langue` sections) is the source of truth; Rust
  code is engines and generated output, never the definition.
- Design decisions go in `design/` before implementation.
