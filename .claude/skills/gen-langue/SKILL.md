---
name: gen-langue
description: Update Lumo .langue grammar files and regenerate syntax_kind.rs/ast.rs. Use when changing syntax, adding new AST nodes, or updating the compiler grammar.
allowed-tools: Bash(scripts/gen_langue.sh *) Read Edit Write Glob Grep
argument-hint: "[compiler|hir|lir|all]"
---

# Langue Grammar Update Workflow

When the Lumo language syntax changes, update the `.langue` grammar files and regenerate the typed AST code.

## Workflow

1. **Edit the `.langue` file** to reflect the new syntax:
   - `crates/compiler/lumo.langue` — surface syntax (CST)
   - `crates/hir/hir.langue` — HIR (desugared, no operators/if-else)
   - `crates/lir/lir.langue` — LIR (CBPV-normalized, single-param lambda)

2. **Regenerate** the typed code:
   ```bash
   scripts/gen_langue.sh          # all targets
   scripts/gen_langue.sh compiler  # only compiler
   scripts/gen_langue.sh hir       # only HIR
   scripts/gen_langue.sh lir       # only LIR
   ```

3. **Verify** the generated output matches the Rust data structures:
   - Compare `syntax_kind.rs` enum variants against `crates/lst/src/parser.rs` (surface), `crates/hir/src/lib.rs` (HIR), or `crates/lir/src/lib.rs` (LIR)
   - Check `ast.rs` accessor methods match struct fields

## Key rules

- The `.langue` files are the **source of truth** for grammar shape. Edit them first, then generate.
- The generated `syntax_kind.rs` and `ast.rs` are **not yet integrated** into the compiler build (Phase 2). They are reference outputs.
- The Lumo-bootstrapped `packages/langue/dist/langue.js` is the generator. If you change the langue tool itself, rebuild with `lbs build --target js packages/langue`.

## Grammar layers

| Layer | File | Rust source | Notes |
|-------|------|------------|-------|
| Surface | `crates/compiler/lumo.langue` | `crates/lst/src/parser.rs` | Full syntax: operators, if/else, assign, attributes |
| HIR | `crates/hir/hir.langue` | `crates/hir/src/lib.rs` | Desugared: no operators, no if/else, explicit produce/perform |
| LIR | `crates/lir/lir.langue` | `crates/lir/src/lib.rs` | CBPV: single-param lambda, roll/unroll, ctor |

## What $ARGUMENTS means

- `compiler` — regenerate only `crates/compiler/{syntax_kind,ast}.rs`
- `hir` — regenerate only `crates/hir/{syntax_kind,ast}.rs`
- `lir` — regenerate only `crates/lir/{syntax_kind,ast}.rs`
- `all` or empty — regenerate all three
