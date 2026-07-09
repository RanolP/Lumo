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

## 2. Project format

**Decided (2026-07-10): a language definition is a multi-file Langue project
with no import system.** Every `.langue` file under the project root — plus
the Langue stdlib — is concatenated into one global namespace (`cat`
semantics). Files are typed by a kind suffix, so a definition can be sliced
by *feature* (fn, pattern, cap, ...) instead of being forced into one
monolith per stage, without any wiring ceremony.

File kinds:

```
<name>.langue         project manifest: language name + options (no kind suffix)
<name>.syn.langue     lexical structure + CST grammar rules
<name>.tree.langue    tree declarations (post-desugar trees)
<name>.elab.langue    elaboration: rewrite rules between trees
<name>.scope.langue   binding: scopes, declarations, references
<name>.type.langue    kinds, type formers, bidirectional typing judgments
```

Resolution model:

- **Project-wide cat**: `langc` collects every kind-suffixed file under the
  root and merges them into a single definition. Neither file order nor item
  order carries meaning.
- **No forward declarations**: every item sees every other item — letrec-style
  global scope across all files and the stdlib.
- **stdlib included**: Langue ships standard definitions (helper combinators
  such as `foldr`/`map`, common token classes, builtin value shapes such as
  `List`) that participate in the same namespace as project files.
- **DCE instead of reachability errors**: unused items — stdlib or project —
  are dead-code-eliminated when the definition is loaded; nothing has to be
  "imported to count". An optional lint can surface unused *project* items.
- **Collisions**: two same-named items in the global namespace are an error.
  The designed exception is additive merging: multiple files contributing
  rules to the same judgment or constructors to the same tree, with
  exhaustiveness and overlap checked globally after the merge.

The Lumo definition (first customer) is expected to slice by feature:

```
lumo/
  lumo.langue
  surface/tokens.syn.langue
  surface/item.syn.langue
  surface/expr.syn.langue
  core.tree.langue
  elab/item.elab.langue
  elab/expr.elab.langue
  scope/lexical.scope.langue
  types/fn.type.langue
  types/cap.type.langue
  types/data.type.langue
```

Out of scope for v1 (stays hand-written Rust, consuming the generated/loaded
definition): backends (TS/Rust emission), LTO, query/incremental engine,
diagnostics rendering, LSP.

## 3. File-kind designs

### 3.1 syn: tokens

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

### 3.2 syn: grammar

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
- `@token` declarations become `token` items in a `.syn.langue` file (e.g.
  `tokens.syn.langue`); grammar rules reference tokens declared anywhere in
  the project — no import, no forward declaration.

The generator still emits `syntax_kind.rs`, typed AST accessors, and a
rowan-style lossless tree. New: it emits the parser itself (legacy had
`#[parser(generate = true)]` for HIR/LIR already; the Lumo surface parser was
hand-written — v1 aims to generate it, with `extern` recovery hooks if needed).

### 3.3 tree

**Decided: Langue does not fix a pipeline shape — a definition declares any
number of named trees.** The surface tree comes from the `.syn.langue` files;
every other tree is declared in `.tree.langue` files as plain ADTs, and the
definition chooses how many stages it wants (what HIR/LIR were, CBPV splits,
backend-prep trees, ...):

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
for elab and type rules to target. Whether Lumo's own definition keeps the
CBPV value/computation distinction as two trees (or two sorts within one tree)
is a choice made *in* its `.tree.langue` files, not by Langue — deferred to
when we port the Lumo definition (M1).

### 3.4 elab

Rewrite rules between any two declared trees (including tree-to-same-tree
passes), with quasiquoted patterns on the source side and constructors on the
target side:

```
elab Surface -> Core {
  rule FnDecl { name, params, body } =>
    Let(name, foldr(params, elab(body), |p, acc| Lam(p, acc)))

  rule PipeExpr { lhs, rhs } =>          // a |> f  ==>  f(a)
    App(elab(rhs), elab(lhs))
}
```

Semantics: syntax-directed, one rule per source node kind (checked for
exhaustiveness against the grammar), recursion explicit via `elab(...)`.
Helper combinators (`foldr`, `map`, fresh-name generation) come from the
Langue stdlib, which shares the project's global namespace and is DCE-ed like
everything else. A rule may call `extern fn` for genuinely procedural cases.

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

- **Generated Rust** for the hot, shape-defining parts: syn (token DFA,
  syntax kinds, AST accessors, parser) and tree (enums, printers). Same
  workflow as legacy langue: edit, run `langc gen`, commit generated code.
- **Interpreted rule tables** for elab/scope/types: `langc` compiles the rules
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

- **M0 — langc core**: `.langue` parser, project cat/merge/DCE model,
  `langc check`.
  Port Lumo's `.syn.langue` files; regenerate what legacy langue generated
  (syntax kinds, AST, lossless tree) and now also the surface parser.
- **M1 — tree + elab**: declare Core trees, write `Surface -> Core` rules,
  rewrite engine replaces hand-written HIR lowering.
- **M2 — scope**: scope facts + resolution engine.
- **M3 — type**: judgment DSL + bidirectional engine with Fomega/caps
  primitives; port the capability typing rules.
- **M4 — reconnect**: backends (start from legacy `ts.rs`/`rs.rs` knowledge,
  written fresh against Core), test fixtures ported from
  `legacy/crates/compiler/tests/fixtures/`.

Each milestone keeps `langc check` + golden-file tests green; fixtures are the
contract carried over from legacy.

## 7. Decisions and open questions

Decided (2026-07-10):

1. **Parser is generated** from the `.syn.langue` files, with precedence
   annotations and extern recovery hooks. Fall back to hand-written only if
   recovery quality proves insufficient at M0 exit.
2. **Arbitrary trees**: Langue fixes no pipeline shape; definitions declare
   any number of trees and elaborate between them. Lumo's CBPV split is a
   `.tree.langue` choice deferred to M1.
3. **Diagnostics live in the DSL** as message templates on rule premises.
4. **Name stays "Langue"** (v2); langue 1 grammar files remain a valid subset
   in spirit, modulo the `sep(...)`/precedence upgrades.
5. **Multi-file project format, no imports**: kind-suffixed files
   (`*.syn.langue`, `*.tree.langue`, `*.elab.langue`, `*.scope.langue`,
   `*.type.langue`) are concatenated project-wide together with the Langue
   stdlib into one global namespace — no `use`, no forward declarations,
   unused items DCE-ed. Same-kind files merge additively, checked globally;
   the suffix-less `<name>.langue` is a manifest (language name + options).

Still open:

5. **Incrementality**: design rule engines to be salsa-friendly from day one,
   or bolt incrementality on later? Legacy query engine suggests day-one
   awareness (pure rule tables help here).
