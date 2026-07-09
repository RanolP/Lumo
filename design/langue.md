# Langue 2 — a full language-definition DSL

Status: draft for discussion (2026-07-09, revised 2026-07-10)
Decided so far: full-language-definition scope, legacy archived under `legacy/`,
tooling implemented in Rust, surface parser generated from the grammar,
arbitrary user-defined trees (no fixed pipeline shape), diagnostic templates
live in the DSL, name stays "Langue" (v2).

## 1. Motivation

The legacy compiler taught us where hand-written pipelines hurt:

- `lower_module` grew into a 5-phase dance (LIR rewrite, typecheck, patch
  type_args, resolve defaults, re-typecheck) where each phase patched state the
  previous one produced. The language definition lived implicitly in that
  choreography, not in any single place.
- The grammar was the one declarative part (`lumo.langue`) and it was also the
  most pleasant part to change: edit, regenerate, done. Everything after the
  CST — HIR lowering, scoping, typing — was hand-written Rust that had to be
  kept in sync by discipline.
- The empty `lumo2hir.langue` / `hir2lir.langue` placeholders in the legacy
  tree show we already wanted langue to describe translations, not just trees.

Langue 2 makes the whole language definition declarative: one artifact defines
Lumo's tokens, grammar, core trees, lowerings, binding structure, and typing
rules. The Rust code becomes *engines* that execute the definition, plus
explicitly-declared escape hatches — never the definition itself.

## 2. Scope

A language definition is a set of `.langue` files (one section each) under
`lumo/` (the definition of Lumo itself, the first customer):

```
lumo/
  tokens.langue      lexical structure
  grammar.langue     CST rules (langue 1, upgraded)
  core.langue        post-desugar tree definitions (replaces hir/lir .langue)
  lower.langue       CST -> Core and Core -> Core rewrite rules
  scope.langue       binding: scopes, declarations, references
  types.langue       kinds, type formers, bidirectional typing judgments
```

Out of scope for v1 (stays hand-written Rust, consuming the generated/loaded
definition): backends (TS/Rust emission), LTO, query/incremental engine,
diagnostics rendering, LSP.

## 3. Section designs

### 3.1 tokens

Lexer rules become declarative instead of the hand-written `crates/lexer`:

```
token Ident      = /[a-zA-Z_][a-zA-Z0-9_]*/ keywords('fn' 'data' 'cap' 'impl' ...)
token NumberLit  = /[0-9]+(\.[0-9]+)?/
token StringLit  = /"([^"\\]|\\.)*"/
trivia Whitespace = /[ \t\r\n]+/
trivia LineComment = /\/\/[^\n]*/
```

`keywords(...)` lists literals carved out of an ident-shaped token, so the
grammar can use `'fn'` directly and the lexer stays a single DFA.

### 3.2 grammar

Langue 1 carried over, with the pain points fixed in the language rather than
by convention:

- **Separated lists become a built-in**: `params:sep(Param, ',')` replaces the
  wrapper-node convention (`FooList`/`FooItems` with unlabeled children). The
  generator emits a proper list accessor. This deletes the biggest footgun in
  langue 1 (unlabeled `head`/`tail` walking).
- **Precedence/associativity annotations** for expression grammars, so the
  generated parser handles binary expressions without hand-written Pratt code:

  ```
  Expr = ...
    | infix BinExpr  { '|>' left, '+' '-' left @ 60, '*' '/' left @ 70 }
  ```
- `@token` declarations move to `tokens.langue`; `grammar.langue` only
  references them.

The generator still emits `syntax_kind.rs`, typed AST accessors, and a
rowan-style lossless tree. New: it emits the parser itself (legacy had
`#[parser(generate = true)]` for HIR/LIR already; the Lumo surface parser was
hand-written — v1 aims to generate it, with `extern` recovery hooks if needed).

### 3.3 core

**Decided: Langue does not fix a pipeline shape — a definition declares any
number of named trees.** The surface tree comes from `grammar.langue`; every
other tree is declared here as plain ADTs and the definition chooses how many
stages it wants (what HIR/LIR were, CBPV splits, backend-prep trees, ...):

```
tree Core {
  Expr {
    Lam(param: Binder, body: Expr)
    App(callee: Expr, arg: Expr)
    Let(binder: Binder, value: Expr, body: Expr)
    Perform(cap: CapRef, args: List(Expr))
    ...
  }
}
```

Generates the Rust enums + pretty-printer + a stable pattern-matching surface
for `lower` and `types` rules to target. Whether Lumo's own definition keeps
the CBPV value/computation distinction as two trees (or two sorts within one
tree) is a choice made *in* `core.langue`, not by Langue — deferred to when we
port the Lumo definition (M1).

### 3.4 lower

Rewrite rules between any two declared trees (including tree-to-same-tree
passes), with quasiquoted patterns on the source side and constructors on the
target side:

```
lower Surface -> Core {
  rule FnDecl { name, params, body } =>
    Let(name, foldr(params, lower(body), |p, acc| Lam(p, acc)))

  rule PipeExpr { lhs, rhs } =>          // a |> f  ==>  f(a)
    App(lower(rhs), lower(lhs))
}
```

Semantics: syntax-directed, one rule per source node kind (checked for
exhaustiveness against the grammar), recursion explicit via `lower(...)`.
Helper combinators (`foldr`, `map`, fresh-name generation) are built into the
rewrite engine. A rule may call `extern fn` for genuinely procedural cases.

### 3.5 scope

Scope-graph style declarations (Statix-inspired, but much smaller):

```
scope {
  File introduces module_scope
  FnDecl declares name in enclosing        // visible module-wide (letrec)
  Param declares name in body
  LetExpr declares binder in body           // sequential, shadows
  MatchArm declares pattern_binders in body
  Ident resolves in lexical scope
  UseDecl imports path into enclosing
}
```

The engine builds the scope graph from these facts during a single CST walk
and answers resolution queries. Output feeds both `types` and the LSP
(go-to-def falls out for free).

### 3.6 types

The most ambitious section, and deliberately the most constrained. The
committed type-system decisions (System Fomega kinds, spine-local bidirectional
inference, capability rows) are *engine primitives*, not user-definable — the
DSL declares the syntax-directed rules on top of them:

```
types {
  judgment infer(e: Expr) -> Type
  judgment check(e: Expr, t: Type)

  rule infer App(f, a):
    infer(f) ~> Fn(param, ret | caps)
      else error "cannot call a non-function of type {0}" at f
    check(a, param)
      else error "expected {param}, found {0}" at a
    -----------------------------------
    ret | caps

  rule check Lam(x, body) against Fn(param, ret | caps):
    bind x : param
    check(body, ret)

  rule infer Perform(cap, args): ...
}
```

Built into the engine (not expressible, only invocable): unification (`~>`),
instantiation/generalization, kind checking, row operations for capabilities,
fresh metavariables. The DSL owns which rule fires for which node, what the
premises are, and what errors each failed premise reports — **decided:
diagnostic message templates live in the DSL** (each premise may carry an
`else error "..." at node` clause; the engine fills type/name holes and maps
`at` to spans). If a rule cannot be expressed, it is written as `extern rule`
in Rust against a stable API — the definition file still names it, so coverage
is visible.

### 3.7 extern hatches

Every section can declare `extern` items implemented in Rust and registered by
name. The rule: the `.langue` files must mention every extern, so `grep extern`
over the definition shows exactly where the declarative story has holes.

## 4. Execution model (proposed)

Hybrid, per section:

- **Generated Rust** for the hot, shape-defining parts: tokens (DFA), grammar
  (syntax kinds, AST accessors, parser), core (enums, printers). Same workflow
  as legacy langue: edit, run `langc gen`, commit generated code.
- **Interpreted rule tables** for lower/scope/types: `langc` compiles the rules
  to a compact checked IR that generic Rust engines execute. Rationale:
  changing a typing rule should not require recompiling generated Rust, and
  these rules are dense in semantics but not performance-critical enough to
  need codegen in v1. If profiling disagrees later, codegen them then.

`langc` itself is one Rust crate (`crates/langc`) with a library API so the
compiler embeds the loaded definition, plus a CLI for generation and checking
(`langc check` validates exhaustiveness, unknown labels, extern coverage).

## 5. Bootstrapping note

Langue 2 files need their own parser. `langc` hand-writes exactly one parser —
for the `.langue` format itself — and everything downstream is generated. If
we later want langue-in-langue, the format is simple enough to self-describe,
but that is explicitly not a v1 goal (the legacy Lumo-hosted langue rewrite
stays archived as prior art).

## 6. Milestones

- **M0 — langc core**: `.langue` parser, section model, `langc check`.
  Port `tokens` + `grammar` for Lumo; regenerate what legacy langue generated
  (syntax kinds, AST, lossless tree) and now also the surface parser.
- **M1 — core + lower**: declare Core trees, write `Surface -> Core` rules,
  rewrite engine replaces hand-written HIR lowering.
- **M2 — scope**: scope facts + resolution engine.
- **M3 — types**: judgment DSL + bidirectional engine with Fomega/caps
  primitives; port the capability typing rules.
- **M4 — reconnect**: backends (start from legacy `ts.rs`/`rs.rs` knowledge,
  written fresh against Core), test fixtures ported from
  `legacy/crates/compiler/tests/fixtures/`.

Each milestone keeps `langc check` + golden-file tests green; fixtures are the
contract carried over from legacy.

## 7. Decisions and open questions

Decided (2026-07-10):

1. **Parser is generated** from `grammar.langue`, with precedence annotations
   and extern recovery hooks. Fall back to hand-written only if recovery
   quality proves insufficient at M0 exit.
2. **Arbitrary trees**: Langue fixes no pipeline shape; definitions declare
   any number of trees and lower between them. Lumo's CBPV split is a
   `core.langue` choice deferred to M1.
3. **Diagnostics live in the DSL** as message templates on rule premises.
4. **Name stays "Langue"** (v2); langue 1 grammar files remain a valid subset
   in spirit, modulo the `sep(...)`/precedence upgrades.

Still open:

5. **Incrementality**: design rule engines to be salsa-friendly from day one,
   or bolt incrementality on later? Legacy query engine suggests day-one
   awareness (pure rule tables help here).
