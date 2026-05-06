# `mut` / `let` Parameter Qualifiers — Design Spec

**Date:** 2026-05-07
**Status:** Approved

## Motivation

Lumo's values are semantically immutable. Functions that need to update a value (e.g. advancing an
iterator, accumulating into a list) must return the updated value explicitly. Without special syntax
this forces callers to manually thread the updated binding through every call, which is verbose and
error-prone.

`mut` and `let` parameter qualifiers provide explicit in-out and out-only parameter passing at the
surface syntax level. They desugar completely to tuple returns before HIR — no special handling is
needed anywhere below the LST→HIR boundary.

This builds on the `lir-memaware` split (2026-05-07) which established the FBIP / linearity
infrastructure.

## Semantics

| Qualifier | Caller provides | Function contract | Caller receives |
|-----------|----------------|-------------------|-----------------|
| *(none)*  | value           | read-only          | nothing extra   |
| `mut`     | existing binding | read + write back  | updated binding (rebound) |
| `let`     | nothing         | write (produce)    | new binding     |

### `mut` — inout

```
fn advance(mut cursor: Number): Bool
```

The caller passes an existing binding. The function reads the current value and writes a new one
back. After the call, the original binding is consumed and replaced by the updated value.

```
let done = advance(mut pos)
-- pos is now the updated cursor; old pos is gone
```

### `let` — out (output-only)

```
fn split(xs: List[A], let left: List[A], let right: List[A]): Unit
```

The caller provides no value for `let` params — the function produces them from scratch. After the
call, new bindings are introduced at the call site.

```
let _ = split(xs, let left, let right)
-- left and right are now in scope
```

## Syntax

### Parameter declarations

```
Param          = qualifier:ParamQualifier? name:Ident ':' ty:TypeExpr
ParamQualifier = 'mut' | 'let'
```

Qualifiers are valid on any `Param` in a `ParamList`, including `self`:

```
fn next(mut self: List[A]): Option[A]
```

Qualifiers are **not** valid on lambda params or extern fn params.

### Call-site arguments

```
Arg    = MutArg | LetArg | Expr
MutArg = 'mut' name:Ident
LetArg = 'let' name:Ident
```

- `MutArg` must name an existing in-scope binding — bare `Ident` only, no expressions.
- `LetArg` introduces a new binding that comes into scope after the call.
- A `mut` arg at the call site must correspond to a `mut` param in the callee declaration (and vice versa). A `let` arg corresponds to a `let` param. Mismatches are a type error.

## Synthetic Tuple Types

`mut`/`let` params desugar to tuple returns at the HIR boundary. Since Lumo has no built-in tuple
type, the compiler generates anonymous product types on demand:

```
data __Tuple2[A, B]       { .mk(A, B)          }
data __Tuple3[A, B, C]    { .mk(A, B, C)        }
data __Tuple4[A, B, C, D] { .mk(A, B, C, D)     }
-- ... up to whatever arity is needed
```

These are emitted as regular `DataDecl` HIR items at the top of the file, deduplicated by arity.
If a file has no qualified params, no `__TupleN` types are emitted.

**Arity formula:** `1 + count(mut params) + count(let params)`

If arity is 1 (no qualified params), no tuple is needed and the function is emitted unchanged.

**Tuple slot order:** `[original_return_type, qualified_params_in_declaration_order]`

Example:

```
fn foo(a: T, mut b: U, c: V, let d: W): R
--                    ^^^^^          ^^^^
-- arity = 1 + 1 + 1 = 3
-- return type becomes: __Tuple3[R, U, W]
```

## Desugaring Rules

Desugaring is performed during `hir::lower_lossless`, before any HIR node is emitted.
After desugaring, HIR contains no `mut`/`let` qualifiers anywhere.

### Function declaration

- `mut x: T` params remain as regular input params.
- `let x: T` params are removed from the input param list entirely.
- The return type is replaced by `__TupleN[original_return, mut_outputs..., let_outputs...]`
  in declaration order of qualified params.
- Within the function body:
  - `mut b` is in scope as an ordinary binding initialized to the caller's value. The programmer
    may rebind it with `let b = ...` to produce the updated value.
  - `let d` is **not** in scope as an input. The programmer must bind it with `let d = ...`
    somewhere in the body before the return expression. It is a compile error if `d` is not in
    scope at the return point.
- At the return point, the compiler wraps the return expression in `__TupleN.mk(...)`.
  The tuple slots for `mut` params use whatever binding of that name is in scope at the
  return point. The slots for `let` params use the binding introduced by the programmer.

```
-- source:
fn foo(a: T, mut b: U, let d: W): R = body

-- desugared HIR:
data __Tuple3[A, B, C] { .mk(A, B, C) }
fn foo(a: T, b: U): __Tuple3[R, U, W] = __Tuple3.mk(body, b, d)
--                   inputs: mut stays ^        result  mut  let
-- `d` must be bound inside `body` for this to compile
```

### Call site

```
-- source:
let r = foo(x, mut y)

-- desugared HIR:
let __t = foo(x, y) in
let r   = match __t { .mk(v, _) => v } in
let y   = match __t { .mk(_, v) => v } in
...
```

```
-- source:
let r = bar(let w)

-- desugared HIR:
let __t = bar() in
let r   = match __t { .mk(v, _) => v } in
let w   = match __t { .mk(_, v) => v } in
...
```

The fresh name `__t` is compiler-generated and guaranteed not to clash with user bindings.
Slot extraction order mirrors the tuple slot order defined above.

If the original return type is `Unit` and the caller discards it, only the qualified-param
slots are extracted. The Unit slot is still present in the tuple but can be pattern-matched away.

## Linearity Check

The linearity check runs **before** call-site desugaring, using the original LST nodes where
qualifier information is still present. It is a local, syntactic check — not a full linear type
system.

**Rules enforced:**

1. A `mut x` argument must be a bare identifier — not a complex expression.
   - Error: `"mut argument must be a simple binding name, not an expression"`
2. A binding passed as `mut x` must not appear anywhere else in the same call's argument list
   (neither as `mut x` again nor as a plain `x`).
   - Error: `"binding 'x' is passed as mut here but also appears at argument position N — a mut binding can only be used once per call"`
3. A `mut` argument must correspond to a `mut` parameter in the callee; a plain argument must not
   correspond to a `mut` parameter.
   - Error: `"parameter 'b' is declared mut but called without mut"` / `"argument passed as mut but parameter 'b' is not mut"`

**Out of scope for this iteration:**

- Cross-call linearity (using `x` after passing it as `mut` in a previous call without rebinding)
- Linearity in closures / captured bindings
- Full Linear Resource Calculus enforcement (deferred to a future LRC pass)

## Interaction with `impl` Methods

`self` can be `mut`. This is the primary motivation — enabling stateful iteration:

```
cap Iterator {
  fn next(mut self: Iterator): Option[Item]
}

impl[A] List[A]: Iterator {
  fn next(mut self: List[A]): Option[A] =
    match self {
      .nil        => Option.none,
      .cons(h, t) => let self = t in Option.some(h)
      --             ^^^^^^^^^^^ rebind self before return
    }
}
```

`mut self` desugars identically to any other `mut` param — the return type gains an extra `self`
slot. At the call site, `xs.next(mut xs)` is the explicit form (or a future sugar `xs.next!` —
out of scope here).

## Pipeline Impact

| Layer | Change |
|-------|--------|
| `crates/compiler/lumo.langue` | Add `ParamQualifier`; update `Param` rule; add `MutArg`, `LetArg`, update `Arg` |
| `scripts/gen_langue.sh` | Regenerate `crates/lst/src/syntax_kind.rs`, `ast.rs`, `lossless.rs` |
| `crates/lst/src/parser.rs` | Parse optional qualifier before param name; parse `mut`/`let` in arg position |
| `crates/hir/src/lib.rs` | Linearity check + full desugaring; emit `__TupleN` DataDecls on demand |
| `crates/lir/` and below | No changes |
| `crates/lir-memaware/` | No changes |
| Backends (`ts.rs`, `rs.rs`) | No changes |
| Typechecker | No changes (sees plain ADT + plain functions) |

## Out of Scope

- Tuple type as a first-class language feature (the `__TupleN` types are compiler-internal)
- Qualifier on lambda params
- Qualifier on `extern fn` params
- Cross-call / cross-closure linearity enforcement
- `mut self` syntactic sugar (`xs.next!` shorthand)
- `let`-param in cap operation declarations (deferred)
