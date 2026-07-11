# M4 scope: direct-style JS emission

Settled 2026-07-12. JS is a language like any other (D-03): `JS.syn.langue`
defines a JavaScript *subset* large enough to carry compiled MIR; emission
is pretty-printing the JS tree; `elab MIR to JS` rules are the backend.

**Direct style, no CPS.** The legacy backend's CPS/trampoline machinery
existed to service `resume`, which is out of M2-M4 scope (D-39). Strict
CBPV maps to JS syntax-directedly:

| MIR | JS |
|---|---|
| `thunk { c }` | `() => c` |
| `force v` | `v()` |
| `ret v` | `v` (transparent) |
| `fn (x) => c` | `(x) => c` |
| `c(a, b)` | `c(a, b)` |
| `let x = c in b` | `((x) => b)(c)` |
| `fix f => c` | `(function f() { return c; })()` — `f` in scope is the self-thunk, `f()` re-enters |
| `.Tag(a)` | `({ $: "Tag", args: [a] })` |
| `roll` / `unroll` | transparent (legacy precedent) |
| `case v { arms }` | `((s) => s.$ === "Tag" ? ((b) => body)(s.args[0]) : … : __lumo_match_error(s))(v)` |
| `bundle { fn op(p) => c; }` | `({ op: (p) => c })` |
| `sel v.f` | `v.f` |
| `perform C` | `__lumo_perform("C")` |
| `handle C with h in b` | `__lumo_handle("C", h, () => b)` |
| `(v : T)` | `v` (types erase) |
| `def n = v` | `const n = v;` |

**Runtime = free identifiers.** Generated JS references
`__lumo_perform`, `__lumo_handle` (a dynamically-scoped handler stack:
handle pushes, perform reads the top handler for the cap or throws),
`__lumo_match_error`, and `require` (already free in MIR from D-30).
The host provides them; shipping a prelude file is deferred — the
fixture contract only needs the JS *text*.

**Grammar notes** (LL(1) without JS's cover grammar): `(…)` is always
`ParenExpr = '(' sep(Expr, ',')? ')'` (n-ary; doubles as a parameter
list), and `=>` is an ordinary right-associative praat infix operator —
`(x, y) => b`, `x => b`, and `() => b` all parse without lookahead.
Ternary is a praat postfix row carrying a `then ':' else` payload node.
Only what emission needs exists: no statements beyond `const`, no
blocks except the `function` body of `fix`, no classes, no `new`.

**Fixture contract** (D-32): `:parse(JS)` for the grammar,
`:elab(MIR -> JS)` per construct, and end-to-end `:elab(Lumo -> JS)`
through a handwritten chained driver (parse Lumo → elab to MIR →
reparse → elab to JS), wired in the corpus lookup — no new attribute.

**Legacy "golden parity", corrected.** Survey finding: the legacy
compiler has *no* full-text JS goldens — only 9 LTO fixtures with
substring assertions (their local intent already ported in M3) and
inline `contains` unit tests. M4 parity therefore means porting the
emission *behaviors* (identity/currying, force-then-apply, ctors +
match, cap ops through bundles, use/require, recursion via fix) as
end-to-end fixtures under the new conventions, not reproducing legacy
output text.

**Deferred**: `resume`/CPS (with D-39's resume); readability
post-passes (IIFE flattening, const collapsing, arity-based
uncurrying — legacy did these; our contract is correctness of the
tree, not prettiness); exports/`main` entry wrapper; TypeScript type
emission; shipping the runtime prelude; the M2 elab-gap backlog
(if/else, impl dispatch, nested patterns, mutual recursion) which
needs Lumo→MIR work before it ever reaches JS.
