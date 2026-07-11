# Capability rows: engine value stays non-syntactic, one serialization

Settled 2026-07-11 (row design behind decision A of
`design/m2-type-brainstorm.html`).

The row's **semantic representation is D-25 unchanged**: a hash-keyed
multimap at the engine level — unification, absorption, and the `hash`
tactic operate on that, never on syntax. What M2 adds is exactly one
grammar production per boundary need (annotations from elab, `:infer`
printing, MIR round-trip), a wire format for the same value:

- **Marker mirrors the Lumo surface** (`CapAnnotation = '/' CapSet`):
  MIR writes `/ { … }`, reusing the node names CapSet / CapSig /
  CapRest per-language.
- **Entries are parameterized**: `CapSig = name TypeArgs?` — legacy
  caps carry type args (`Add[Number]`), so row matching unifies on
  args, not just names.
- **One row variable**: `..r` (`CapRest`), giving `/ { ..r, Console }`
  for cap-polymorphic HOFs. Without it, a HOF taking a capful thunk
  is unwritable and rows collapse back into the fixed-point inference
  D-39 defers.
- **Rows serialize on `F` only.** Defs are values, so every printed
  type is `U(TypeC)` and effects surface at the innermost `F`; elab
  output (strict CBPV, D-36/D-38) keeps performs under the final `F`.
  A latent-effect arrow (computation that performs before returning a
  lambda) can exist as an engine-internal value; *printing* one is an
  M2 bail (D-26). Extend the grammar only if a real fixture hits it.
- **Canonical print**: entries sorted, rest last — printing is the
  serialization of the multimap, so parse/print round-trips compare
  stably (D-32 canonicalize-then-compare).

Absent row = empty row = pure. There is still no row datatype in the
engine (D-25 holds); the grammar is the only place rows look like
syntax.
