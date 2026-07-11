# M3 egglog execution: hybrid saturate/extract/reduce loop

Settled 2026-07-12. Decision 37 parked execution; this is how it runs.

**Engine**: the `egglog` crate, 2.0.x, `default-features = false` (pure
Rust, no native deps), embedded in `langue-rt` (engines live there,
D-01). Embedding surface: `EGraph::default()` +
`parse_and_run_program`; run limits are program text (`(run N)`);
`(extract root)` yields a `TermDag` that is walked directly into an
owned term — no s-expression re-parsing.

**Substitution is host-side.** egglog has no capture-avoiding
substitution; embedders either encode de Bruijn rulesets or evaluate on
the host. We take host-side, matching the M2 judge tactic (D-24). The
driver loop:

1. encode the input tree as an egglog expression, `(let root <expr>)`;
2. run the compiled `between` program + `(run N)` (bounded);
3. `(extract root)` → lowest-cost term;
4. if the extracted term contains `subst` nodes, reduce them host-side
   (innermost-first), `(union root <reduced>)`, go to 2 — bounded
   rounds; leftover `subst` at the bound is an error;
5. decode to surface text; reparse for the canonical output.

**`subst` is a high-cost constructor**, not an uninterpreted function:
`(constructor subst (Comp String Value) Comp :cost 1000)` (the proven
egglog-2.0 steering pattern; eggcc does exactly this). Extraction only
surfaces a `subst` node when its e-class has no subst-free alternative;
once the host reduces it and unions the result back, extraction picks
between the original form and the substituted form by true cost. This
changes the M1-compiled program format (D-19 golden re-blessed).

**Subst semantics v1 = the judge engine's**: replace `(VarV b)`
occurrences, no shadow-stopping — the grammar has no binder markers
(D-24 minimal). Documented limitation; fixtures avoid shadowing.
Deferred: binder markers or an alpha-uniqueness invariant from elab.

**Optional fields** (the M1 "optional fields are required" note):
- optional *list* fields encode absent as the empty Vec (semantically
  identical);
- a bare `ParenV` (no type) encodes as its inner value — parens are
  syntax; an annotated `ParenV` encodes as-is;
- any other absent optional (`FTypeC` row, `CapRest` name) hard-bails
  "unencodable" — types only appear inside annotations, out of M3
  fixture scope.

**Update to D-31's caveat**: egglog 2.0's built-in extractor is
Bellman-Ford over the hypergraph — effectively DAG cost with sharing,
so "tree cost double-counts shared subterms" is not expected to bite.
The duplicating-subst fixture is the watchpoint; whichever way it
lands is recorded there.

**Not ported from legacy LTO** (deferred to M4+): anything
interprocedural — resolution maps across defs, inline/clone heuristics,
DCE, `resume` stripping (no resume in M2 scope, D-39). Those need
def-level context beyond a single-term e-graph. M3 ports the *local*
core: perform-resolution under a visible handle, as a between rule.
