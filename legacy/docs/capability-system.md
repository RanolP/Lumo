# Lumo Capability System

## Core Idea

A **capability** (`cap`) is a named effect with a set of operations. Code
declares what capabilities it needs; callers provide implementations
("handlers"). Effects and handlers compose structurally.

This is Lumo's take on *algebraic effects* — pure CBPV up to the `perform`
boundary, abort-by-default past it, with `resume` as an explicit control
operator.

```
cap Cap { fn op(args): R; ... }     — declare the capability surface
bundle { fn op(args) { body }; ... } — construct a handler bundle (a value)
impl Cap { fn op(args) { body }; ... } — register a default bundle at module top
handle Cap with <bundle> in <body>   — install a bundle for the duration of body
perform Cap.op(args)                 — invoke the currently-installed handler's op
resume(v)                            — (only valid inside a handler body) run the rest of the performing code with v
```

## The central identity

> **`impl Cap { ... }` is sugar for a module-level bundle that gets handed
> to an implicit `handle Cap with <bundle> in main` at program start.**

That is: there is *no* separate "default implementation" concept. An `impl`
block is just a bundle value. At the program entry point, `main`'s
required caps are satisfied by wrapping its body in `handle` for each one,
using the matching `impl` as the bundle.

Consequence: **bundles used in `handle` and bundles created by `impl`
share the same calling convention and semantics.** If a handler method
needs `resume(v)` to feed a value back to the perform site, so does an
impl method.

## Runtime calling convention

Every bundle operation has this shape:

```
op: (__caps, user_args..., __k_perform) => __Ret
```

Two continuations are in scope:

- `__k_perform` — the rest of the program from the `perform` site, up to
  the enclosing `handle`. Passed explicitly as the last argument.
- `__k_handle` — the continuation of the whole `handle` expression. Closed
  over by the bundle factory when the `handle` is installed.

The body is CPS-lowered with `__k_handle` as its output continuation. Its
final value therefore flows **out of** the enclosing `handle` (abort).

`resume` is not a runtime binding — the compiler emits each call site
inline based on its *position* in the handler body's CPS form:

- **Tail position** (`resume(v)` is the body's last expression, the CPS
  continuation at the call site is `__k_handle`): emits as
  `__k_handle(__k_perform(v))`. `__k_perform(v)` returns a thunk; the
  outer `__trampoline` unwinds it iteratively. Stack-safe through any
  perform depth — each handler contributes only a bounded number of JS
  stack frames. This is the path stdlib `impl Cap { fn op(...) =
  resume(expr) }` patterns take.

- **Non-tail position** (e.g. `let _ = resume(a); rest`, `match … { …
  => resume(x), … }`): emits as `__trampoline(__k_perform(v))` at the
  call site. The synchronous trampoline drives `__k_perform(v)` to a
  concrete value before control returns to the rest of the handler body.
  Side effects in the resumed continuation run, and the resulting value
  is fed to the surrounding expression. Multi-shot patterns
  (`resume(true); resume(false)`, non-determinism, backtracking) work
  because each non-tail `resume` actually executes the captured
  continuation.

Stack cost trade-off: tail resume is O(1) regardless of perform depth.
Non-tail resume nests one synchronous `__trampoline` per call — fine for
shallow cases (the typical Coin/Choice/Backtrack handlers) but stack
growth proportional to total perform-chain depth × non-tail-resume
nesting, so deep parsers etc. should keep their stdlib impls in tail form.

`__k_perform` is a plain JS closure, so it can be re-invoked any number
of times with different values. The non-tail emission path is what makes
true multi-shot semantics work.

## Abort vs resume

Given:

```
handle Cap with bundle {
  fn op(x) { <body> }
} in <main_code>
```

Inside `<body>`:

- Falling off the end → value flows to `__k_handle` → the entire `handle`
  expression evaluates to that value (the surrounding computation that
  performed `Cap.op` never resumes). This is **abort**.
- Tail `resume(v)` → emits `__k_handle(__k_perform(v))`, returning the
  thunk → outer trampoline drives it → the performing computation
  continues from the `perform` site with `v` substituted. Stack-safe.
- Non-tail `resume(v)` (let-bound, combined with other expressions,
  used multiple times in sequence) → emits inline
  `__trampoline(__k_perform(v))` so side effects of the resumed
  continuation run before control returns. **Multi-shot works**: a
  handler can `resume(a); resume(b)` to enumerate two branches.

## Impl lowering reality check

Because impl = handler, **every impl method must explicitly `resume` if it
intends the perform site to continue.** Anything else is abort.

```lumo
impl IO {
  fn println(msg) {
    let _ = __println(msg);
    resume(Unit)               // otherwise main stops after the first println
  }
}

impl NumOps {
  fn num_add(a, b) = resume(__num_add(a, b))
  fn num_sub(a, b) = resume(__num_sub(a, b))
  …
}

impl Number: Add {
  fn add(self, other) = resume(NumOps.num_add(self, other))
}
```

If the compiler currently accepts `impl IO { fn println(msg) = __println(msg) }`
and silently threads `__println`'s value into `__k_perform`, that is a
leftover from before multi-shot — a special-case "tail expression is
implicitly `resume`" sugar that now contradicts the general semantics and
should be removed.

## Why the separation matters

With abort-by-default + explicit `resume`:

- `{ fn op(x) { 0 } }` aborts the perform; the whole `handle` expression
  evaluates to `0`. Same as `throw 0` in spirit.
- `{ fn op(x) { resume(x + 1) } }` (tail) maps-over the effect: the
  perform continues with `x + 1` instead of `x`.
- `{ fn op(x) { resume(true); resume(false) } }` (multi-shot) runs the
  performing code twice — once for each branch — enumerating
  combinations. Each resume executes the captured continuation with side
  effects.

If the compiler quietly converted tail expressions to `resume(e)`, the
first two would be indistinguishable. They must not be.

## Diagnostics

Because an impl method missing `resume` silently turns off the rest of the
program, the compiler should warn when an impl method body can reach a
non-`resume` tail (i.e. at least one code path falls off the end without
calling `resume`). Until that exists, writers of `impl` blocks must be
careful.

## Status of this document

- The multi-shot refactor (Stages A–D) put `resume` semantics in place for
  `handle` blocks.
- `impl Cap { ... }` is still lowered along the pre-multi-shot path, so
  stdlib impls appear to work via an *implicit* `__k(tail_expr)` — which
  is the sugar described above.
- Action item (tracked in `plans/unify-impl-and-handler.md`): unify the
  two paths, rewrite stdlib impls to use explicit `resume`, and add the
  diagnostic.
