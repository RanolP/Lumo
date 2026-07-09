# Lumo Project — Claude Instructions

## Repository layout (since 2026-07-09)

- **`legacy/`** — the entire previous implementation (Rust compiler crates,
  Lumo packages, langue 1, apps, docs, plans), archived intact. Read-only
  reference; do not extend it.
- **`design/`** — design documents for the fresh, DSL-driven rewrite.
  Start with `design/langue.md` (Langue 2: full language-definition DSL).
- New implementation code will live in a fresh Cargo workspace at the root
  (not created yet).

The old instructions about `crates/compiler/lumo.langue` and
`scripts/gen_langue.sh` apply only inside `legacy/` (paths now prefixed with
`legacy/`).

## Working rules for the rewrite

- The language definition (`.langue` sections) is the source of truth; Rust
  code is engines and generated output, never the definition.
- Design decisions go in `design/` before implementation.
