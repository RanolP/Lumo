# Cost model: per-constructor constants, min-tree-cost extraction

The widely adopted model, adopted as-is:

1. Costs are an optional integer annotation on node declarations in the
   grammar, **default 1** — compiles 1:1 onto egglog `:cost`.
2. The v1 extractor is egglog's built-in minimal-tree-cost `extract`
   (fixpoint bottom-up, optimal for additive tree cost). No ILP /
   DAG-aware extraction in v1.
3. Dynamic costs stay an escape hatch (egglog `set-cost` / Rust
   `CostModel`), not a DSL feature.
4. Known caveat, accepted for v1: tree cost double-counts shared
   subterms, so duplicating (subst-style) rewrites can be mis-ranked —
   revisit with DAG-aware extraction if it bites.

Same family in the wild: egg `AstSize` + greedy extractor, Cranelift's
per-opcode constant tiers, Herbie's AstSize.
