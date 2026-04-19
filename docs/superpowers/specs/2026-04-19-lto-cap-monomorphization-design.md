# LTO via Capability Monomorphization

**Status:** design approved, plan pending
**Date:** 2026-04-19

## Goal

Eliminate CPS overhead for computations whose capability dependencies are statically known. When a function's `perform` calls can be resolved to default impls at compile time, and those impls are themselves dependency-free (no remaining unresolved performs), emit a monomorphized clone of the function with direct calls — and drop the CPS plumbing entirely for that clone.

Today every cap-using function is wrapped in CPS: takes `__caps` as first arg, takes a `__k` continuation as last arg, dispatches every `perform` through `__caps.Cap_Type` at runtime, and runs under `__trampoline`. For arithmetic-heavy code (`1 + 2 + 3` performs `Add[Number]` three times), this is pure overhead — there is exactly one possible impl.

## Non-goals

- Closure specialization (passing fns as values, monomorphizing higher-order callees per call site). Out of scope; documented as v2.
- Inlining through user `handle { ... } in expr` blocks. Default impls only for v1.
- Cross-module LTO across separately-compiled artifacts. Lumo currently merges all sources before `lower_module`; the design assumes whole-program merge persists.
- A `#[inline(never)]` opt-out. YAGNI; add later if profiling motivates.

## Decisions

| ID | Decision |
|---|---|
| Q1 | Hybrid emission: clone (B) and call-site inline (C) both used; `#[inline(always)]` forces C. |
| Q2 | "Dependency-free" = body has no remaining unresolved perform after substituting current resolutions; iterate to fixed point. |
| Q3 | Heuristic for B vs C without attribute: small body **and** single call site → C; otherwise B. |
| Q4 | Sources of static knowledge: default impls only (`impl T: Cap` typeclass + `impl Cap` platform). |
| Q5 | Original CPS function dropped via call-graph DCE when no remaining caller needs it. |
| Q6 | No clone-count guardrail. |
| Q7 | Extend attribute grammar to allow positional args so `#[inline(always)]` parses. |

## Architecture

A new module `crates/compiler/src/lto/mod.rs` runs as Phase 4 of `lower_module` in `query/mod.rs`, after the existing typecheck/patch passes and before returning the LIR to the backend.

Pipeline becomes:

```
0.  rewrite_value_method_calls            (existing)
1.  typecheck::infer_caps_for_file        (existing)
2.  patch_perform_type_args               (existing)
3.  fill_default_type_args                (existing)
3'. typecheck::infer_caps_for_file        (existing re-typecheck)
3''.typecheck::apply_inferred_caps        (existing)
4.  lto::optimize                         (NEW)
4'. typecheck::infer_caps_for_file        (NEW — clones changed cap requirements)
4''.typecheck::apply_inferred_caps        (NEW)
```

`lto::optimize(&mut lir::File)` runs five sub-phases:

1. Build the **cap-resolution map** — `(cap, type_args) → ImplResolution`.
2. Build the **call graph** — `fn name → call sites`.
3. Run the **fixed-point dependency-free analysis** — yields the dep-free set + per-Perform-site resolution.
4. Apply the **emission transformation** — clone fns / inline at call sites per heuristic D.
5. Run **DCE** — drop original CPS fns with zero remaining users.

The re-typecheck after LTO is required because clones have new mangled names and rewritten Performs change cap requirements. The backend (`ts.rs`) needs **no changes for the happy path**: clones whose bodies no longer contain Performs or calls to effectful fns naturally fall through the existing `fn_caps`-emptiness branch and emit without CPS.

`lto/` sits as a sibling to `query/`, `typecheck/`, `backend/`. Submodules:

- `lto/mod.rs` — entry point and orchestration.
- `lto/resolution.rs` — cap-resolution map (lifts `collect_default_impls` from `ts.rs`).
- `lto/call_graph.rs` — call graph construction.
- `lto/dep_free.rs` — fixed-point analysis.
- `lto/emit.rs` — clone / inline transformation.
- `lto/dce.rs` — call-graph-based dead-fn removal.

## Components

### Cap-resolution map (`lto/resolution.rs`)

```rust
pub struct ResolutionMap {
    impls: HashMap<(String, Vec<String>), ImplResolution>,
}

pub struct ImplResolution {
    pub impl_const: String,                        // e.g. "__impl_Number_Add"
    pub methods: HashMap<String, MethodInfo>,
}

pub struct MethodInfo {
    pub params: Vec<lir::Param>,
    pub body: lir::Expr,
    pub performs: Vec<PerformSite>,                // Performs inside the method body
}

pub struct PerformSite {
    pub cap: String,
    pub type_args: Vec<String>,
}
```

Built from two sources, both already present in `ts.rs:228` (`collect_default_impls`):

- **Typeclass defaults** (`impl T: Cap { ... }`) → key `(cap, [T])`.
- **Platform defaults** (`impl Cap { ... }`) → key `(cap, [cap])`.

The existing function in `ts.rs` is moved here; `ts.rs` re-exports it so the runtime dispatch path keeps working unchanged.

Multiple impls for the same key are treated as **ambiguous** and excluded from the map — Performs targeting an ambiguous binding cannot be statically resolved and stay CPS.

### Call graph (`lto/call_graph.rs`)

```rust
pub struct CallGraph {
    pub edges: HashMap<String, Vec<CallSite>>,     // caller fn → its call sites
    pub callers: HashMap<String, Vec<String>>,     // callee fn → its callers (for DCE)
}

pub struct CallSite {
    pub callee: CallTarget,
    pub span: Span,
    pub cap_bindings: Vec<(String, Vec<String>)>,  // resolved bindings active here
}

pub enum CallTarget {
    Fn(String),                                    // direct fn call
    ImplMethod { impl_const: String, method: String }, // direct impl method call
    Indirect,                                      // lambda / fn-typed param / unknown
}
```

Built by walking each `FnDecl.value` and recording every `lir::Expr::Apply` and `Member-Apply` chain.

For v1, `cap_bindings` at a call site is computed from the callee's `fn_caps` (looked up via the typechecker output) without any type-arg substitution — Lumo's generic fns don't currently propagate type parameters through caps in a way that requires substitution. If/when generic-cap-propagation lands, this is the seam to extend.

Indirect calls (`CallTarget::Indirect`) opaque-edge a fn out of the eligible set. The fn containing them stays CPS.

### Dependency-free analysis (`lto/dep_free.rs`)

A `(fn_or_method, cap_binding_tuple)` pair is **dependency-free** if every `Perform` in its body, after substituting bindings, resolves to another dependency-free pair via the resolution map, and every direct call lands in a dependency-free target.

Algorithm (worklist):

1. **Seed**: every impl method whose body contains zero Performs and zero calls to fns with non-empty `fn_caps` is marked `DepFree` under the empty binding.
2. **Iterate**: for each pair currently marked `Pending`:
   - Walk every `Perform { cap, type_args }` → resolve via the map → check the target method+binding is `DepFree`.
   - Walk every direct call → check the `(callee, callee_binding)` is `DepFree`.
   - Indirect calls → mark `Blocked` immediately.
   - All checks pass → mark `DepFree`, enqueue dependents (callers) for re-check.
3. **Stop** when an iteration adds no new `DepFree` markers.

Termination: the set of `(fn, binding)` pairs is finite (bounded by source-text mentions), and the dep-free set grows monotonically.

**Recursion** (self-call): the recursive position is treated optimistically — give the pair a tentative `DepFree` mark while analyzing its body, then verify nothing else blocks it.

**Mutual recursion**: same trick at SCC granularity. Compute SCCs of the call graph; an SCC is `DepFree` iff every Perform inside resolves to a target outside the SCC and every cross-SCC call lands in a `DepFree` target.

Output:

```rust
pub struct DepFreeAnalysis {
    pub status: HashMap<(String, Vec<String>), DepFreeStatus>,
    pub perform_resolution: HashMap<PerformId, ResolvedRef>,  // per-site resolution
}

pub enum DepFreeStatus { Pending, DepFree, Blocked }

pub struct ResolvedRef {
    pub impl_const: String,
    pub method: String,
}
```

`PerformId` is the `Span` of the Perform node — `lir::Expr::Perform` is already wrapped in `Spanned<…>` and spans are unique per source location.

### Emission transformation (`lto/emit.rs`)

For each `(fn, binding)` pair marked `DepFree`, decide between **clone (B)** and **call-site inline (C)** by heuristic D:

```
if fn has #[inline(always)]                                        → C (forced)
else if body_size(fn) <= INLINE_SIZE_THRESHOLD
        AND call_graph.callers(fn).len() == 1                      → C
else                                                               → B
```

`INLINE_SIZE_THRESHOLD` defaults to a small constant (start at 16 LIR nodes; tune later). Body size is a simple LIR walk count.

**Clone (B)**:

1. Mint a clone name: `<fn>__<cap1>_<args1>__<cap2>_<args2>...` — same convention as `cap_runtime_name` in `ts.rs:54-ish`. Two-underscore separator between fn name and bindings.
2. Deep-clone the `FnDecl`, rename to the mint name.
3. Rewrite every `Perform { cap, type_args }` in the clone's body using `perform_resolution` → replace with `Apply(Member(Ident(impl_const), method), [args...])`.
4. Clear the clone's `cap` annotation (`FnDecl.cap = None`).
5. Insert the clone into `lir::File.items`.
6. At every call site whose `cap_bindings` match this binding, rewrite the callee identifier to the clone name.

**Call-site inline (C)**:

1. At the call site, replace the `Apply(Ident(fn), args)` with the fn's body, with parameters substituted by the actual args.
2. Within the inlined body, also rewrite Performs using the resolution map.
3. Don't emit a standalone clone — the caller absorbs the body.
4. Param substitution must alpha-rename any conflicting locals in the body to avoid shadowing the caller's bindings.

When a fn ends up with both clone-eligible call sites and CPS-only call sites, both forms coexist: the clone services the resolvable callers; the original CPS body services the rest.

### DCE (`lto/dce.rs`)

After emission, walk the call graph from entry points (`main` and any exported fn). Any `FnDecl` not reachable is dropped. This includes the original CPS body of a fn whose every caller now points to a clone.

Special cases:

- `main` is always retained.
- `extern fn` declarations are always retained (they expose external names).
- Impl methods are reachable iff their containing impl const is reachable; if a clone uses an impl method directly, the impl const becomes reachable through that edge.

### `#[inline(always)]` attribute

Three pieces of plumbing:

1. **Grammar**: `crates/compiler/lumo.langue` and `crates/lst/src/parser.rs` — extend `AttributeArgs` to accept a positional flag. Current rule:
   ```
   AttributeArgs = '(' args:AttributeArg* ')'
   AttributeArg  = key:Ident '=' value:Expr
   ```
   New rule:
   ```
   AttributeArgs = '(' (positional:Ident | args:AttributeArg)* ')'
   ```
   Parser change: in `parse_attribute`, if a `(` is followed by an `Ident` not followed by `=`, parse it as a positional flag.
2. **Allow on regular fns**: `parse_attributes` currently rejects attributes on non-`extern` items (`parser.rs:330-355`). Permit them on `FnDecl`.
3. **HIR/LIR plumbing**: add `attrs: Vec<Attribute>` (or a typed `inline_hint: InlineHint`) to `hir::FnDecl` and `lir::FnDecl`. The existing `lir::FnDecl.inline: bool` field is currently used only by the `inline_always_calls` backend pass for compiler-internal fns; reuse it by setting `inline = true` when `#[inline(always)]` is parsed.
4. **Validation**: at the start of `lto::optimize`, error if `#[inline(always)]` is set on a fn that is **not** dep-free under any of its call sites' bindings — there's no way to satisfy the contract. Error message: `` "fn `foo` is marked #[inline(always)] but has unresolved capability `Bar[Baz]`; remove the attribute or provide a default impl" ``.

## Data flow

```
┌─────────────────┐
│ lower_module    │
│   (query)       │
└────────┬────────┘
         │  &mut lir::File (post Phase 3'')
         ▼
┌─────────────────────────────────────────────┐
│ lto::optimize                               │
│                                             │
│  ┌──────────────┐   ┌──────────────────┐    │
│  │ resolution   │   │ call_graph       │    │
│  │  ::build     │   │  ::build         │    │
│  └──────┬───────┘   └────────┬─────────┘    │
│         │                    │              │
│         └──────┐    ┌────────┘              │
│                ▼    ▼                       │
│         ┌─────────────────┐                 │
│         │ dep_free::run   │                 │
│         │  (worklist)     │                 │
│         └────────┬────────┘                 │
│                  │ DepFreeAnalysis          │
│                  ▼                          │
│         ┌─────────────────┐                 │
│         │ emit::transform │  (B vs C)       │
│         └────────┬────────┘                 │
│                  │ mutates lir::File        │
│                  ▼                          │
│         ┌─────────────────┐                 │
│         │ dce::sweep      │                 │
│         └────────┬────────┘                 │
└──────────────────┼──────────────────────────┘
                   │
                   ▼
         re-typecheck → backend
```

## Error handling

- **Ambiguous resolution** (multiple impls for same key) → exclude from resolution map; affected Performs are unresolvable; fn stays CPS.
- **`#[inline(always)]` on non-dep-free fn** → compile error with the message above.
- **Cycle in dep-free analysis** → resolved by SCC handling; never an error.
- **Inline expansion produces malformed LIR** (e.g. lambda with captured-name conflict) → alpha-rename pass on the inlined body before substitution. This is an internal invariant — if violated, panic with an internal-compiler-error message.

## Testing

New fixture directory: `crates/compiler/tests/fixtures/lto/`. Each fixture is a Lumo source file paired with the expected post-LTO LIR (or expected JS output).

Coverage targets:

1. **Trivial leaf inlining** — `1 + 2` (single Perform of `Add[Number]`) → no CPS, direct primitive op in output.
2. **Two-level chain** — `fn double(x) = x + x` called from main → clone `double__Add_Number` exists, no CPS.
3. **Fixed-point unlock** — fn `f` performs cap `A`, `A`'s impl performs cap `B`, `B`'s impl is dep-free. After one round `B`'s impl is dep-free; after two rounds `A`'s impl is dep-free; `f` becomes eligible.
4. **Mixed eligibility** — fn called from one site with resolvable caps and one with a `handle` block → both clone and original CPS coexist; clone reachable from one caller, original from the other.
5. **Recursion** — `fn fact(n) = if n == 0 then 1 else n * fact(n - 1)` → clone is dep-free; clone references itself by mint name.
6. **Mutual recursion** — `fn even(n) = if n == 0 then true else odd(n - 1)` + `odd` → both clones dep-free, mutually self-referential.
7. **Indirect call blocks** — `fn map(xs, cb) = ...` is not eligible; documented in the fixture.
8. **`#[inline(always)]` happy path** — annotated fn is inlined at call site; standalone clone not emitted.
9. **`#[inline(always)]` error path** — annotation on a fn with unresolvable caps emits the documented compile error.
10. **DCE** — fn called only via clones is removed from output; its impl const stays reachable through the clone.

Existing test suites (`backend_ts.rs`, `e2e_hello.rs`, fixture-driven syntax/type tests) must remain green; LTO is purely additive at the LIR level and the backend's behavior on non-eligible fns is unchanged.

## Open questions / future work

- **Closure specialization (v2)**: monomorphize higher-order callees per `(callee, lambda-shape)` at the call site. Unlocks `xs.map(fn x => x + 1).sum()` chains.
- **`handle` block inlining**: requires a purity analysis on handler bodies (zero-or-once `resume`, no side effects beyond the resume target). Significant scope; defer.
- **`INLINE_SIZE_THRESHOLD` tuning**: the initial value of 16 LIR nodes is a guess. Once integrated, profile real Lumo programs and adjust.
- **Cross-module LTO**: not relevant while whole-program merge is the only mode. Revisit if Lumo grows separate compilation.
- **`#[inline(never)]`**: a runtime-dispatch escape hatch. Skip until profiling shows a need.
