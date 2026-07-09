# Unify `impl Cap` and `handle Cap with <bundle>` lowering

## Problem

Per [docs/capability-system.md](../docs/capability-system.md), `impl Cap { ... }`
is a bundle value that gets installed as the handler for a `handle Cap`
at program entry. Both forms should therefore share the same compilation
path and semantics.

The multi-shot refactor (Stages A–D) installed the new calling convention
(`__k_perform`, `__k_handle`, explicit `resume`) for `handle` blocks
inside `lower_handler_with_resume`. But `impl Cap { ... }` still takes
the pre-multi-shot path in `lower_impl_const`: it emits methods as plain
`(args, __k) => __thunk(() => __k(<tail_expr>))`, which implicitly
resumes on tail expressions.

That's two bugs:

1. **Inconsistency with the documented semantics.** An `impl` method
   written as `fn op() = e` silently means "resume with e", but a
   handler body written identically means "abort with e". Identical
   source, opposite behavior.
2. **Multi-shot unavailable for impls.** Impl methods cannot express
   patterns like non-determinism, backtracking, or zero-shot abort that
   the effect system is supposed to support — the implicit-resume sugar
   ties their hands.

## Design

### Single lowering path

Route `impl Cap { ... }` through `lower_handler_with_resume` instead of
having a separate branch in `lower_impl_const`. At the call site
(`emit_main_entry_wrapper` and any other place installing a default
impl), wrap the produced factory with the same `(k_handle) => bundle`
shape that `handle` uses.

### Main entry becomes an explicit handle chain

`main()` currently invokes `__main_cps(__caps_bundle, __identity)` after
constructing the bundle once. Replace with a chain of nested handles —
one per required cap — using each default impl as the bundle:

```js
export function main(): void {
  return __trampoline(
    handle_Cap_1(__impl_1, (__v) =>
      handle_Cap_2(__impl_2, (__v) =>
        …
        __main_cps(__caps_bundle, __identity)
      )
    )
  );
}
```

(Or, equivalently, build the caps bundle object where each value is the
factory applied to `__identity`, which is the single-step shortcut the
current runtime already uses.)

### Stdlib rewrite

Every stdlib impl method currently relying on the implicit-resume sugar
becomes explicit. Audit:

- `packages/libcore/src#js/ops.lumo` — all `NumOps` operations
- `packages/libcore/src#js/number.lumo` — the `impl Number: …` typeclass
  defaults
- `packages/libcore/src#js/string.lumo` — `impl String: Add`, `impl
  String: PartialEq`, `impl StrOps`
- `packages/libstd/src#js/io.lumo` — `impl IO`
- `packages/libstd/src#js.node/fs.lumo` — `impl FS`
- `packages/libstd/src#js.node/process.lumo` — `impl Process`

Mechanical transform: `fn op(args) = expr` → `fn op(args) = resume(expr)`
for pure extern delegates. For bodies that already sequence effectful
work, each branch that currently ends with a value needs an explicit
`resume(...)` wrapper.

Example: `impl Process { fn panic_with(msg) { let _err =
__console_error(msg); __exit_process(1) } }` has two statements; the
tail `__exit_process(1)` needs to become `resume(__exit_process(1))` so
the perform site sees the value. (In practice `__exit_process` never
returns, but the type system doesn't know that.)

### Diagnostic

Emit a warning when an impl method's body has a reachable tail that is
not a call to `resume`. This catches the common "forgot to resume"
mistake that otherwise produces a program that aborts silently after the
first perform.

Implementation: reuse the HIR/LIR walk already done for `value_method`
analysis; look for `resume(...)` in tail position of every control-flow
branch.

## Changes

### 1. `crates/compiler/src/backend/ts.rs` — unify lowering

- Delete the effectful branch in `lower_impl_const` that emits `(args,
  __k) => __thunk(() => __k(body))`. Route through
  `lower_handler_with_resume` with the impl bundle's entries.
- `lower_handler_with_resume` already produces a factory
  `(__k_handle) => { op: ... }`. Impl consts become a call to this
  factory.
- Update `emit_main_entry_wrapper` to invoke each default impl's factory
  with `__identity` as `__k_handle`.

### 2. Stdlib rewrite

Per the bullet list above. Each impl method's tail expression gets an
explicit `resume(...)` wrapper. Keep user-facing semantics identical.

### 3. Diagnostic

Add a HIR-level check `impl_methods_missing_resume` that walks each
`Item::Impl` method body and flags tails that aren't `resume(...)`.

### 4. Tests

Add to `crates/compiler/tests/e2e_hello.rs`:

- `impl_method_without_resume_aborts_execution` (Node e2e) — compile an
  impl missing resume, run, assert stdout shows abort (handle value)
  rather than continuation output.
- `impl_method_uses_resume_like_handler` — a user-defined `impl Cap` with
  explicit `resume`, runtime behaviour matches an equivalent `handle
  Cap with bundle { ... } in main_body`.

Add to `crates/compiler/tests/backend_ts.rs`:

- Structural test: `impl IO { ... }` compiles to the same bundle factory
  shape as `handle IO with bundle { ... } in ...`.

## Risk & Ordering

- This is a semantic break. Every stdlib impl currently in the repo needs
  the rewrite before tests go green.
- Order: (1) rewrite stdlib impls with explicit `resume`, (2) unify
  lowering, (3) add diagnostic, (4) add tests.
- Pitfall: `emit_main_entry_wrapper` transitively expands caps required
  by a default impl into the bundle. When that expansion crosses the
  new unified lowering, the calling convention changes. Verify with the
  Node e2e `hello_world_runs_on_node` and `multi_shot_resume_on_node`
  tests.

## Verification

1. `cargo test` — 250 passed (includes the two new tests above).
2. `cargo test --test e2e_hello -- --ignored` — Node e2e green.
3. `cd packages/langue && cargo run -p lbs -- build --target js.node` —
   rebuild, then `node dist/langue.js crates/compiler/lumo.langue out/`
   still prints `Parsed 59 rules from ...` and writes both `.rs` files.
4. Playground WASM rebuild finishes cleanly and the default "hello world"
   example runs.
