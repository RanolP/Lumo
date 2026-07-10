# M1 implementation plan — MIR + elaboration

Derived 2026-07-11 from PLAN.md M1 and decisions D-05/07/12/13/14/15/19/
21/24/28/30/31/32. Mirrors the M0 method: numbered steps, each
compilable + committed, `scripts/verify.sh` green at every step.

## Concrete elab syntax (derived, needs user confirmation)

The locked examples (design/langue.md §3.1–3.2) fix the shape; the
elided `...` parts are filled in as follows — **flag any deltas before
step 2 locks this as decision 35**:

```
from Lumo to MIR {
  FnDecl { name: $n, param_list: ParamList { params: [$p*] }, body: $b }
    ==> Lambda { params: [$p* to MIR], body: $b to MIR }
}

between MIR {
  Apply { fn: Lambda { param: $b, body: $e }, arg: $a } === $e[$b := $a]
}
```

- Pattern = `Node { field: Pattern, … }` | `$var` | `'literal'` |
  `[$x*]` (list capture under a labeled sep/rep field). Fields are the
  syn labels; omitted fields match anything.
- Construction = `Node { field: Construction, … }` | `$var` |
  `$var to Lang` (recursive elab, strict-subtree checked, D-28) |
  `$e[$b := $a]` (built-in subst, D-24) | `'literal'`.
- Node names qualify as `Lumo::FnDecl` when ambiguous (D-13).
- `from A to B` blocks with the same pair merge across files (D-05);
  `between L` groups merge per language (D-14).

## Steps

1. **MIR.syn.langue** — CBPV core as a syn language (D-15), so the
   existing M0 codegen gives MIR its parser/printer for free:
   values `Var Num Str Thunk Ctor`, computations `Ret Let Apply Lam
   Force Fix Case Perform Handle`, plus TypeV/TypeC sub-language.
   Gate: `langc check lumo/` clean; MIR corpus fixtures (`:parse(MIR)`)
   round-trip.
2. **.langue front end: elab items** — lex/parse `from`/`between`
   blocks, patterns, constructions, `$x`, `==>`, `===`, `[$x*]`,
   `to Lang`, subst. Record decision 35. Unit tests.
3. **Project model + check** — merge blocks by (from,to)/language;
   check: languages exist, node names/fields exist in the source and
   target grammars, metavars bound-before-use, `to` recursion only on
   strict-subtree captures (D-28), rule-conflict detection = same root
   kind with non-disjoint literal discriminants (D-13). Broken-input
   tests.
4. **Elab engine (langue-rt or new crate langue-elab)** — tree matcher
   over generated `SyntaxNode`s + construction renderer. v1 builds the
   target by *rendering MIR text and reparsing with MIR's generated
   parser* — simple, canonical by construction; optimize later.
5. **elab codegen** — `from` blocks → generated Rust match-functions on
   the engine (D-21); `langc gen` emits `crates/lumo-syntax/src/elab/`
   (or a lumo-elab crate). Gate: `elab Lumo -> MIR` of `fn id(x) = x`
   produces the expected MIR text.
6. **Scope simulation + recursion lowering (engine builtins, D-30/D-12)**
   — name resolution via Γ during elab; `use` hoisting → `λrequire.`
   lowering; SCC detection over top-level decls, cyclic groups through
   `fix`, acyclic through `let`. These are extern-declared engine passes
   (`extern pass scc_fix`, etc. — every extern named in .langue, D-01).
7. **between → egglog** — parse+store in M1; execution: compile rule
   groups to egglog programs with per-constructor cost 1 and
   min-tree-cost extract (D-31). If egglog integration balloons, park
   execution for M3 (PLAN M3 owns optimization) and keep only the
   compile-to-egglog-text step with golden tests.
8. **`:elab(A -> B)` fixtures (D-32)** — harness support:
   canonicalize-then-compare (parse expected with B's parser,
   pretty-print both, compare). Seed from legacy lowering behavior
   (`legacy/crates/compiler/src/lir.rs` is the reference semantics).
   Update the manifest to `main = parse Lumo | elab Lumo to MIR` (D-33).

## Risks

- Rule-conflict detection (D-13) can over/under-approximate — start
  strict (error on same root kind unless literal-disjoint), loosen with
  evidence.
- List captures `[$p*]` are the weakest part of the derived syntax —
  confirm with user before locking.
- Manifest gains a live elab stage; DCE roots now include MIR (already
  handled — stage languages are all live).
- Keep `-j 2` on every cargo invocation (WSL2 OOM).

## Handoff

M0 + parse-level legacy migration are complete and committed (through
`329240d`). A fresh session can execute this plan top-to-bottom;
`scripts/verify.sh` is the loop gate, `LANGC_UPDATE=1` blesses corpus
fixtures.
