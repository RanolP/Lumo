# Langue 2 implementation plan

Derived from the locked decisions in `design/decisions/`; the design
snapshot is `design/langue.md`. Every step keeps `langc check` + the
fixture suite green.

## Completed (details in git history and design/decisions/)

- **M0** (2026-07-11) — langc core: project model, salsa, syn codegen,
  fixture harness, full legacy surface grammar, parse-level legacy
  source gates, `langc check`. Recovery reviewed acceptable (D-02).
- **M1** (2026-07-11) — MIR (strict CBPV) + generated `elab Lumo -> MIR`
  with extern rules/passes, recursion lowering, scope simulation,
  between→egglog emission, `:elab` corpus.
- **M2** (2026-07-12) — relational judge engine + judgment codegen,
  rows/forall/spine-local bidirectional (~70 rules), `:infer(Lumo)`
  corpus, ~40 legacy type cases green.
- **M3** (2026-07-12) — egglog 2.0 saturate/extract/reduce loop (D-42),
  `:optimize(MIR)` corpus, handle/perform local resolution, 14 fixtures.
- **M4** (2026-07-12) — `JS.syn.langue`, `elab MIR to JS`, end-to-end
  `:elab(Lumo -> JS)` run under node with the D-43 runtime.
- **Post-M4 features** (2026-07-12) — D-44 bare cap impls; D-45 stdlib
  port (JS subset, COMPLETE); D-46 nested matches; D-47 if/else; D-48
  typeclass impls + operators; D-49 inherent impls + UFCS; D-50 generic
  inherent impls; D-51 capability passing (Effekt style, no
  continuations); D-52 mutual recursion (module fixpoint); D-53 build
  system slice 1 (`crates/lbs`, manifest-driven, `dist/{name}.js`).

## Backlog (deferred, by area)

Type system / judge (D-39 buckets and later):
- resume + non-tail-resumptive handlers (exceptions/generators) — needs
  a delimited-control slice (D-51); abortive handlers first.
- inference depth: unannotated lambdas/fix, cap_inference of bare defs.
- bounds (bounded binders), assoc_types, exhaustiveness checking.
- nested patterns in match arms.
- generic typeclass impls + structured cap type-args (Σ keys that unify
  on args without rigidifying skolems; retires `{Cap}_{T}` mangling).
- span-carrying judge errors (D-53).

Elab / dispatch:
- typeclass methods via dot; self-less static methods; Self-typed
  direct cap calls; let-annotation scope in the dispatch table;
  operators inside impl-method bodies; `**`.
- first-class use of row'd fns (re-arrangement) and of mutual-group
  members (eta-expansion through the module); generic mutual groups;
  groups spanning impls (D-51/D-52).

Optimization (M3 deferrals):
- interprocedural LTO: cross-def resolution maps, inline/clone
  heuristics, DCE, resume stripping — needs def-level context.
- binder-aware subst (needs binder markers or alpha-uniqueness from
  elab, D-42); generated optimize driver; DAG-aware extraction (only if
  sharing gets mis-ranked, D-31/D-42).
- `:optimize` between the pipe's elab stages (compile_driver goes
  straight through today).

Backend / build system (M4 + D-53 deferrals):
- real `use` scoping/namespaces (global namespace stands),
  demand-driven loading, separate compilation, `rs` targets, out-dir
  config, extern-mapping attributes.
- readability post-passes (IIFE flattening, const collapsing,
  uncurrying); TypeScript type emission; exports beyond the bin
  `main();` wrapper.

Parser (M0 known limits, revisit only if real grammars hit them):
- recovery inside a nested construct skips only up to its own follow
  set; `sep()` interiors don't resync.

## Cross-cutting rules

- The definition is the source of truth; Rust is engines + generated
  output, never the definition (D-01).
- All three kinds are code-generated — edit, `langc gen`, commit; no
  interpreted rule tables (D-21).
- Documents stay snapshots; new decisions get a new numbered file in
  `design/decisions/` (D-10).
- stdlib and built-in tactics start minimal and grow only on proven need
  (D-24, D-29).
