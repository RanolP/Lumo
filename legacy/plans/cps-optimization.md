# CPS Optimization Pass

## Current Overhead Sources

Every effectful call site currently produces:

1. A fresh continuation closure per operation: `(__cps_v_N) => { ... }`
2. A `__thunk(() => body)` wrapping per effectful function body
3. Uniform `__caps` forwarding even when unused by the callee
4. `Perform` always goes through `__caps.X.op(__caps, args..., __k)` dispatch

The generated code in [packages/langue/dist/langue.js](packages/langue/dist/langue.js)
allocates dozens of closures per parser step. Functional correctness is fine;
the constant factor is what a CPS optimization pass should attack.

## Proposed Passes (ordered by ROI)

### Pass 1 — Eta reduction on continuations

Pattern detection on tsast after CPS lowering, before indent printing:

```js
// Before
(__cps_v) => f(args..., __cps_v)

// After (when __cps_v occurs exactly once as last arg of a tail call)
f(args..., /*inherited k*/)
```

And identity continuations:

```js
// Before
foo(__caps, x, (__v) => __v)

// After
foo(__caps, x, <outer_k>)   // or wrap in __trampoline if outer_k missing
```

Expected impact: halves closure allocations in deep CPS chains (langue hot
loops are the obvious benchmark).

Implementation sketch:

- New pass in [simple-ts-ast/src/pass.rs](crates/simple-ts-ast/src/pass.rs)
- Walk `Expr::Arrow { params: [single], body: FunctionBody::Expr(Call { args })}`
- If the single param occurs exactly once, as the last element of `args`,
  and nowhere else in body, rewrite the call's `args.last()` to the caller's
  own `__k` and drop the arrow.
- Needs use-count analysis on identifiers.

### Pass 2 — Tail-call `__thunk` elision

Pattern: effectful function body is exactly `__thunk(() => effectful_call(...))`.

Since `effectful_call` itself returns a thunk, the outer `__thunk` is
redundant (bounce twice when once suffices). Drop the outer `__thunk` and
return the inner call directly.

```js
// Before
function foo(__caps, x, __k) {
  return __thunk(() => g(__caps, x, __k));
}

// After
function foo(__caps, x, __k) {
  return g(__caps, x, __k);
}
```

Expected impact: one allocation + one bounce per passthrough fn.

Implementation:

- Post-pass in [backend/ts.rs](crates/compiler/src/backend/ts.rs) or tsast
- Simple structural match on `Function { body: Expr(Call(__thunk, [Arrow([], Expr(Call(...)))])) }`
- Verify the inner call is "known to return a thunk" (either a user fn with
  caps, or an impl method in CPS form).

### Pass 3 — Passthrough-only functions stay thunk-less

Functions that have `cap` annotations but don't themselves `perform` — they
just forward `__caps` to effectful callees — never need their own `__thunk`.
The callee already produces one.

Detection: body contains zero `lir::Expr::Perform` nodes, only
`Apply*(Force(Ident(fn)), ...)` where `fn` has caps.

Apply to both user fns (`lower_fn_decl`) and impl methods (`lower_impl_const`).

### Pass 4 — `Object.assign` elision at handle

`((__caps) => body)(Object.assign({}, __caps, { K: h }))` can drop
`Object.assign({}, ...)` when the outer `__caps` is known; sometimes just
inline as a `const __caps_inner = { ...__caps_outer, K: h };` block. Cosmetic
but reduces one allocation per handle.

### Pass 5 (stretch) — Purity inference + direct-style compilation

For a chain of functions where:

- All caps resolve statically to default impls (no user `handle` intercepts)
- The chain's entry is a fn whose Performs are all of the above caps

…we could compile that chain in direct style (no CPS, no thunks, no
continuations). Bridge back to CPS only at explicit `handle` boundaries.

Requires a call-graph analysis + intraprocedural "CPS required?" flag.
Biggest win but also biggest complexity. Worth deferring until passes 1-3
stabilize and we have benchmarks.

## Suggested Implementation Order

1. Pass 1 (eta reduction) — pure tsast transformation, isolated, high impact
2. Pass 2 (tail `__thunk` elision) — small backend/ts.rs change, visible gen-size drop
3. Pass 3 (passthrough elision) — generalization of Pass 2
4. Pass 4 — cleanup pass, do alongside the Multi-shot Stage C handle-site IIFE refactor
5. Pass 5 — defer; needs benchmarks + more architecture discussion

## Benchmarks to Track

- `packages/langue/dist/langue.js` line count (closure sites roughly = non-trivial arrows)
- Wall-clock for `node langue.js crates/compiler/lumo.langue out/`
- Number of `__thunk(` occurrences in generated JS

## Interaction With Other Work

- Must happen **after** multi-shot resume changes settle (the handler IIFE
  shape will change in Stage C).
- Opt 1 (eta reduction) should be safe to ship independently — doesn't
  touch cap semantics.
