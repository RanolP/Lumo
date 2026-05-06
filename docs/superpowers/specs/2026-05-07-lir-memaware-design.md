# LIR / LIR-Memaware Split — Design Spec

**Date:** 2026-05-07
**Status:** Approved

## Motivation

Lumo's long-term execution model is Functional But In-Place (FBIP), following the Linear
Resource Calculus from Perceus (Koka). Every value is semantically immutable and linearly
owned; when the runtime can prove a value is uniquely referenced (refcount = 1), it reuses
the allocation in place instead of copying.

To support this, memory operations (`dup`, `drop`, uniqueness tests) must become explicit
nodes in the IR before reaching a backend. The current `crates/lir` has no such nodes —
they must be introduced in a separate, lower IR.

This spec defines the split into `lir` (pure functional) and `lir-memaware` (memory-explicit),
plus the elaboration pass that bridges them.

## Naming

| crate | role |
|-------|------|
| `crates/lir` | Unchanged. Pure functional IR. HIR lowers into this. Typecheck and query passes operate here. |
| `crates/lir-memaware` | Strict superset of `lir`. Adds `Dup`, `Drop`, `IsUnique` nodes. Backends target this. |

The name `Pure` is used inside `lir-memaware` to wrap any original `lir::Expr` node,
making the boundary visually clear: `Pure(_)` = functional world, everything else = memory world.

## `lir-memaware` Expression Type

```rust
pub enum Expr {
    /// Any node from the pure functional IR — unchanged semantics.
    Pure(lir::Expr),

    /// Increment the refcount of a value (clone if RC > 1).
    /// Emitted at every use of a binding after the first.
    Dup { id: ExprId, expr: Box<Expr> },

    /// Release a binding before evaluating body.
    /// Emitted when a bound value goes unused before end of scope.
    Drop { id: ExprId, name: String, body: Box<Expr> },

    /// FBIP branch: if `expr` is uniquely owned, take `unique_branch`;
    /// otherwise copy and take `shared_branch`.
    /// NOT inserted by the elaboration pass — reserved for a later FBIP optimisation pass.
    IsUnique {
        id: ExprId,
        expr: Box<Expr>,
        unique_branch: Box<Expr>,
        shared_branch: Box<Expr>,
    },
}
```

All other `lir-memaware` types (`File`, `Item`, `FnDecl`, `Param`, …) are thin wrappers
that re-use `lir`'s types everywhere except where `Expr` appears.

## Elaboration Pass

**Location:** `crates/compiler/src/elaborate.rs` (new file)
**Signature:** `fn elaborate(file: &lir::File) -> lir_memaware::File`

The pass walks the `lir::Expr` tree and inserts `Dup` and `Drop` nodes. It does **not**
insert `IsUnique` — that is a separate, later optimisation pass.

### Algorithm (single tree walk)

1. **Usage count** — for each let-binding in scope, count the number of syntactic references
   to the bound name in its continuation subtree.
2. **Dup insertion** — if a binding is used N ≥ 2 times, wrap each of the first N−1 uses
   in `Dup`. The final (consuming) use is left bare — it takes ownership. This matches
   Perceus convention: every non-consuming use increments the refcount.
3. **Drop insertion** — if a binding's usage count is 0 (bound but never used), insert
   `Drop { name, body: continuation }` immediately after the binding site.

### Conservative correctness

The JS backend today treats `Dup` and `Drop` as no-ops (GC handles memory). This means
the elaborated output is always correct regardless of whether the usage counts are perfectly
tight. Over-counting `Dup` is safe (wastes a clone); under-counting `Drop` is safe (GC
collects later). Precision can be improved later without changing the spec.

## Backend Treatment

| node | JS backend (today) | JS backend (with FBIP) |
|------|--------------------|------------------------|
| `Pure(e)` | emit as before | unchanged |
| `Dup { expr }` | emit `expr` (no-op) | emit RC increment |
| `Drop { name, body }` | emit `body` (no-op) | emit RC decrement / free |
| `IsUnique { … }` | always take `shared_branch` | RC == 1 → `unique_branch` |

The Rust backend follows the same conservative approach today.

## Pipeline After This Change

```
HIR
 └─ lir::lower          → lir::File
     ├─ typecheck        (operates on lir::File — unchanged)
     ├─ query passes     (operates on lir::File — unchanged)
     └─ elaborate        → lir_memaware::File
         └─ backend      (ts.rs / rs.rs target lir_memaware::File)
```

Typecheck and query passes remain on `lir` — they need no memory semantics.

## Relation to `mut` / `let` Parameter Qualifiers

Mutable Value Semantics (`mut` = inout, `let` = out parameters) is a separate feature
that builds on top of this IR split. The `mut`-param tuple-return transform will be
specified in a follow-up design doc after this split lands.

## Out of Scope

- `IsUnique` insertion pass (FBIP optimisation) — future work
- Reference counting in the JS runtime — future work
- Linearity enforcement in the typechecker — future work
- `mut` / `let` parameter qualifiers — follow-up spec
