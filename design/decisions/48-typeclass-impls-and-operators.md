# Impl elaboration, slice 2: typeclass impls + operator desugaring

Settled 2026-07-12. Second impl slice: `impl T: Cap { … }` for ground
target types, plus the operator surface that consumes it. Grammar
already had the operator rows; this is elab + seeding only.

## Ground instance caps (no judge changes)

`cap Add { fn add(a: Self, b: Self): Self }` declares a family; an
`impl Number: Add { … }` elaborates to

```
def __impl_Add_Number = (bundle { … } : Add_Number)
```

and the judge driver seeds `Add_Number` as an ordinary *ground
instance cap*: every op signature with `Self := Number`, under
`Σ.Op("Add_Number", op)` / `Σ.Ops("Add_Number")`. The existing
bundle-vs-cap judgment and `SelC` lookup then check everything —
`check_V BundleV <- NamedTypeV` and `Σ` key by bare cap name today,
so instance caps are indistinguishable from written caps.

This is deliberate name-mangling (`{cap}_{target}`), the thing legacy
eventually replaced with structured type args. It is confined to two
places (the elab's def names, the driver's Σ keys), works only for
ground targets (`impl List[A]: Functor` errors — deferred), and gets
replaced by real cap type-args when generic impls land. The plain cap
(`Add`, with literal `Self` op types) is still seeded and inert.

## Operator desugaring (elab, type-directed syntactically)

Arithmetic desugars to instance-cap selections; boolean/comparison
operators desugar structurally onto the same tag protocol as if/else
(D-47) — `data Bool`/`data Ordering` in scope, no impl needed:

| surface | desugar |
|---|---|
| `a + - * / %  b` | `sel __impl_{Add,Sub,Mul,Div,Mod}_{T} . {add,sub,mul,div,mod_} (a, b)` |
| unary `-a` | `sel __impl_Neg_{T} . neg (a)` |
| `a == b` | `sel __impl_PartialEq_{T} . eq (a, b)` |
| `a != b` | `==` then a Bool-flipping case |
| `a < <= > >= b` | `sel __impl_PartialOrd_{T} . cmp (a, b)` then a case over `Ordering` |
| `a && b`, `a \|\| b` | lazy case on `a`'s Bool (rhs stays in its arm) |
| `!a` | Bool-flipping case |
| `a ** b` | elab error (legacy has no cap for it) |

**Dispatch is syntactic.** The elab resolves the operand type `T`
without the judge: literals (`Number`/`String`), annotated fn/lambda
params in scope, `(e : T)` annotations, data-ctor owners, known
fn/extern return types, cap-op return types, and operator results
recursively. Left operand first, then right; unresolvable or
un-impl'ed operands are a loud elab error telling the user to
annotate. Full inference-driven dispatch belongs to the spine-local
inference milestone; this covers annotated code, which the stdlib and
fixtures are.

**Not covered here**: `let`-annotation scope tracking (blocks fold in
reverse; params-only for now), operators inside impl-method bodies
(their params are not in the syntactic scope), value method dispatch
(`x.method()` — next slice), `Self`-typed direct cap calls
(`PartialEq.eq(a, b)` without an operator — use the operator).
