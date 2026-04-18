# Multi-shot Resume Support

## Problem

Handler bodies currently use single-`__k` calling convention that conflates
two semantic continuations:

- `k_perform` — what `resume` should invoke (the rest of the handled body)
- `k_handle` — where the handler's result should flow (the code after the
  whole `handle ... in ...` expression)

Consequences:

1. `resume` is bound as `() => __k` → `resume(x)` returns a thunk, not a
   concrete value. Handler body can't compose multiple resume results.
2. Handler body is lowered with `lower_expr` (non-CPS), so nested Performs
   inside the handler don't thread continuations correctly.
3. Handler's implicit return feeds `__k` = `k_perform` — always resumes. No
   way to express "abort" semantics.

Code pointer: [ts.rs:2453](crates/compiler/src/backend/ts.rs#L2453).

## Goal

Handler can:

- Call `resume(v)` zero, one, or many times
- Each `resume(v)` returns the concrete value of running `k_perform(v)` to
  completion via trampoline
- Combine those results in normal effectful code (other Performs allowed)
- The handler body's final value is the `handle` expression's result (abort
  by default, not implicit resume)

## Stages

### Stage A — synchronous resume (isolated, low risk)

Change binding in `lower_handler_with_resume`:

```diff
-resume = () => __k
+resume = (v) => __trampoline(__k(v))
```

`resume(x)` now returns the trampoline-driven concrete value.

**Still broken after Stage A**: nested Performs in handler body; implicit
return still flows to `k_perform`.

### Stage B — CPS-lower handler body

In `lower_handler_with_resume`, replace `lower_expr(&entry.body, ctx)` with
`lower_cps_expr(&entry.body, <continuation>, handled_caps, ctx)`.

The continuation is Stage-C-dependent: in Stage B alone we can use
`identity_k` and accept that implicit return still hits `k_perform` (keeping
current semantics until Stage C changes them).

### Stage C — split k_perform vs k_handle (the real semantic change)

1. **Handle-site IIFE** closes over `k_handle`:

   ```
   ((__caps) => body_cps)(Object.assign({}, __caps, { <key>: handler_factory(k_handle) }))
   ```

   where `handler_factory` takes `k_handle` and returns the handler bundle.

2. **Handler method** now constructs `resume` and feeds its body's result to
   the captured `k_handle`:

   ```
   op: (__caps_local, args..., __k_perform) => __thunk(() => {
     const resume = (v) => __trampoline(__k_perform(v));
     // CPS-lower body with continuation = k_handle
     return <body_cps with __k = k_handle>;
   })
   ```

3. **Perform site** unchanged: still passes `k_perform` as the final arg.

### Stage D — tests

Existing tests assert current Option-A (implicit resume) behavior:

- [backend_ts.rs](crates/compiler/tests/backend_ts.rs): `ts_backend_handle_always_uses_cps`, `ts_backend_handle_with_resume_uses_cps`, `ts_backend_cps_handle_with_let_perform`, `ts_backend_mixed_resume_entries`
- [e2e_hello.rs](crates/compiler/tests/e2e_hello.rs): `arithmetic_operator_desugars_to_cap_call`

Update those to match the new algebraic-effects semantics (handler body
result = handle expression result; resume is explicit). Add new tests for
multi-shot.

## Implementation Order

1. Stage A (branch, ship, verify single-shot semantics still work)
2. Stage B (branch, verify nested Performs in handler body)
3. Stage C (semantic break; update test suite in the same commit)
4. Stage D — add multi-shot test:

   ```lumo
   cap Choice { fn pick(): Number }

   fn main(): Number =
     handle Choice with bundle {
       fn pick() { resume(1) + resume(2) }
     } in perform Choice.pick()
   // expect: 3
   ```

## Open Questions

- Should `resume` be zero-arg (`resume()` for Unit-returning performs) or
  always one-arg?  Today's LIR shape suggests one-arg.
- How does multi-shot interact with mutable state introduced by the LRC
  planned in formalization.typ? Deferred.
