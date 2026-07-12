# Stdlib port, slice 1: the JS-target subset

Settled 2026-07-12. First port of the legacy stdlib
(`legacy/packages/`) onto the new pipeline, scoped to what today's
features carry: data/cap/extern decls, fns, match, blocks, and bare cap
impls (D-44). New sources live under root `packages/`, mirroring the
legacy package/`src#platform` layout.

**One compilation unit.** The pipeline is single-file; there is no
build system (legacy `lbs` is unported) and D-30's `use` lowers to JS
`require`, which neither the judge nor the single-unit story supports.
So the port drops all `use` decls and `lumo.toml`s, and
`packages/stdlib.manifest` lists the `.lumo` files in concatenation
order — decl order is binding order for the judge, so caps and impls
precede their users (libcore, then libstd, then the demo entry). The
gate (`crates/lumo-syntax/tests/stdlib.rs`) concatenates the manifest
and runs parse, `:infer`, and Lumo→JS compile over the unit.

**Host bindings live in the runtime prelude.** The legacy
`#[extern(name/operator/property)]` and `#[link]` attribute machinery
is not implemented in the new backend (D-43: the runtime is free
identifiers), so ported extern fns are plain `extern fn` decls and
`packages/runtime/js/prelude.js` is the source of truth for what each
one does in JS — each binding is a thunked n-ary function
(`const __num_add = () => (a, b) => a + b;`) matching the
`force f(args)` calling convention. Attribute-driven extern mapping
(inlining `+` at call sites, property reads, module links) is deferred
to a backend feature, not smuggled into the port. Lumo-side `Bool`
is the tagged `data` encoding; boolean-returning bindings construct
`{$: "true"|"false", args: []}`.

**`resume` is dropped, not ported.** Legacy default-impl methods wrap
every body in `resume(…)` (the legacy handler protocol). Under D-44,
default-impl methods return normally; the port deletes the wrapper and
keeps the body.

**Operators rewrite to cap ops.** The two legacy helper uses of `+`/`-`
(`libstd` process arg offsets) become `NumOps.add`/`NumOps.sub` calls,
statically resolved against `__impl_NumOps` (D-44). Operator syntax
stays deferred.

**Port-driven fixes and adaptations.** Running real programs surfaced
two backend gaps, fixed in `elab_externs.rs` rather than worked around:

- Duplicate `_` case binders (`.cons(_, _)`) emitted `((_, _) => …)`,
  which JS rejects — wildcard binders now uniquify to fresh names at
  emission.
- Lumo `f()` on a nullary fn emitted `f()()` (force + empty
  application), double-invoking. The judge reads an empty application
  as identity, so emission now collapses it over `force`; over `sel`
  the call stays (a nullary bundle clause is a 0-ary arrow that needs
  invoking). Consequence: a zero-arg call of a require-imported name
  emits one call, so D-30 hosts must export plain values, not thunks.

The port initially had to flatten `NumOps.cmp` (legacy nests the
eq-match inside the lt-match's `.false` arm; the D-28 guard bailed on
the re-entry) — resolved the same day by the amortized descent guard
(D-46), and the source is back to the legacy nested shape.

**Ported** (in manifest order): libcore prelude (extern `String`/
`Number`, `data Bool`), `Ordering`, `cap NumOps` + `impl NumOps`,
`cap StrOps` + `impl StrOps`, libstd `cap IO`/`FS`/`Process` + their
bare impls, `List[A]` with the free list fns, and a `hello` entry
(`fn main`) that exercises strings, numbers, lists, FS round-trip, and
process args end to end under node (`scripts/stdlib_smoke.sh`).

**Deferred, with their blocking features:**

- `Self`-typed operator/comparison caps (`Add`…`Not`, `PartialEq`,
  `PartialOrd`) and every typeclass impl (`impl Number: Add`,
  `impl String: Add`, `impl Bool: Not`) — need `impl T: Cap` +
  type-directed dispatch and a `Self` story. *(Landed same day —
  D-48; ported as ops.lumo, cmp caps, number_impls/string_impls,
  and `impl Bool: Not`.)*
- Inherent impls (`impl String { fn len(self) … }`) and value method
  dispatch (`"hi".len()`), including `impl[T] List[T]::map` — UFCS
  slices.
- The `langue` package (Tier 5) — needs if/else, value methods, and
  nested patterns.
- Multi-file/`use` resolution, `lumo.toml`, an `lbs` successor, and
  the `src#rs` backend files (no Rust backend in the new pipeline).
- Legacy row annotations like `list_length … / { NumOps }`: with
  static resolution the body needs no row, so ported signatures say
  `/ {}`; row-polymorphic stdlib signatures return when dynamic
  (handled) usage does.
