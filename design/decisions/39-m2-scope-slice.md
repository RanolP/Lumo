# M2 scope: core + data/match + capability rows, judgments on MIR

Settled 2026-07-11 (decision A of `design/m2-type-brainstorm.html`).

**No LIR.** M2 judgments target MIR directly — the pipeline stays
`parse Lumo | elab Lumo to MIR | judge`. PLAN.md's `LIR.type.langue`
wording predates D-36; a second lowering earns its keep only when a
concrete need appears (resolves the MIR-vs-LIR question left open at M1
close).

**In scope** (legacy `fixtures/type/` buckets): basics, annotation,
hof, recursion, if_else, extern, data, iso_recursive, match, cap,
cap_row — core Fω-slice inference (D-40), data/match typing, and
capability-row checking (D-41).

**Deferred, explicitly**: resume, bounds, assoc_types, cap_inference,
and match *exhaustiveness*. Each needs machinery beyond relational
rules — extern tactics, dictionary passing, or fixed-point iteration —
and pulling any of them into M2 dilutes its purpose: validating the
judgment surface (D-17/D-23) on a real language. Their fixture
expectations migrate when their machinery lands.

**Fixture wiring** (scope-independent): `:infer(Lumo)` runs the full
pipeline (parse, elab, judge) and prints `name : Type` lines with the
MIR type sub-language's printer (D-32). A judgment failure is a generic
bail per D-26.
