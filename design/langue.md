# Langue 2 — a full language-definition DSL

Status: draft for discussion (2026-07-09, revised 2026-07-10)
Decided so far: full-language-definition scope, legacy archived under
`legacy/`, tooling implemented in Rust, surface parser generated from the
grammar, arbitrary user-defined trees (no fixed pipeline shape), diagnostic
templates live in the DSL, name stays "Langue" (v2), multi-file project with
no imports (cat + stdlib + DCE), and the four architecture pillars of
section 2.

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

## 2. Core architecture

Four pillars. Everything else in this document is a consequence of these.

### 2.1 Salsa-like incremental queries

**Decided: day one** (closes the former open question). Every derived
artifact is a memoized query over inputs, at both layers:

- **Definition layer (langc)**: `parse(file)` → `merged_definition(project)`
  → `rule_tables(kind)` → generated code. Editing one `.type.langue` file
  re-derives only the type tables.
- **Compiled-language layer**: the compiler that Langue produces is itself
  query-structured — `cst(file)`, `tree(file, "Core")`, `resolve(node)`,
  `infer(node)` — continuing what the legacy query engine started.

Rule tables are pure values, which is exactly what memoization wants: the
cat/merge/DCE project model (section 3) produces one immutable definition
value per revision. Candidate runtime: the `salsa` crate, or a purpose-built
equivalent if salsa's model fights the reasoning engine.

### 2.2 Ungrammar-like notation

The `.langue` notation keeps langue 1's ungrammar lineage: a file describes
the **shape of trees**, never a parsing algorithm. Labels become accessors,
alternatives become node kinds, and the parser is *derived* from the shape
(plus precedence annotations) rather than written. This philosophy extends
beyond syntax: tree declarations, elab rules, scope facts, and typing rules
are all shape-first, algorithm-free declarations that engines interpret.

### 2.3 E-graph elaboration

Elaboration comes in two modes with different machinery.

#### 2.3.1 Tree-to-tree: syntax-directed

Lowering between two different trees (`elab Surface -> Core`) stays a
deterministic, syntax-directed, single-pass translation: one rule per source
node kind, exhaustiveness checked, recursion explicit via `elab(...)`.
Nothing to optimize here — it is a definition of meaning, not a search.

#### 2.3.2 Same-tree optimization: equality saturation

Same-tree rule groups (`elab Core -> Core`) run on an **e-graph**: rules are
non-destructive rewrites applied to saturation, so rule *order carries no
meaning* — the same principle the project format applies to files. This is
where the legacy LTO work (inlining resolvable performs, CPS elimination,
DCE) is reborn: as declared equalities instead of hand-sequenced passes.
Candidate engines: `egg` / `egglog`.

#### 2.3.3 Cost and extraction

After saturation, extraction picks the best representative per e-class using
a cost model. Where the cost model is declared — annotations on tree
constructors, per-rule weights, or a Rust-side extern — is an open question
(section 8).

### 2.4 Pluggable type system

#### 2.4.1 The reasoning engine

Type checking runs on a generic **reasoning engine**: an inference-rule
interpreter that builds proper derivation trees. Premises, modes
(infer ⇒ / check ⇐), unification, instantiation/generalization, and fresh
metavariables are engine services. The engine knows nothing about Lumo.

#### 2.4.2 Type systems as plugins

A *type system* is a plugin on that engine: a Rust-side capability bundle
(type formers, kind rules, row operations) plus the DSL rule set that drives
it. Lumo v1 plugs in "Fω + spine-local bidirectional inference + capability
rows" — the committed design. Because the engine is generic, a different
language (or a future Lumo) can plug a different system without touching the
engine. The plugin boundary (how much lives in the Rust trait vs the DSL
rules) is an open question (section 8).

#### 2.4.3 Derivations as artifacts

Because the engine produces real derivations, failed premises map to the
DSL's diagnostic templates with full context, successful ones power
explanations ("why does this expression have this type") and LSP hovers, and
every derivation is a salsa query — incremental re-typechecking falls out.

## 3. Project format

### 3.1 File kinds

A definition is a multi-file Langue project. The suffix declares the role:

```
<name>.langue         project manifest: language name + options (no kind suffix)
<name>.syn.langue     lexical structure + CST grammar rules
<name>.tree.langue    tree declarations (post-desugar trees)
<name>.elab.langue    elaboration: rewrite rules between/within trees
<name>.scope.langue   binding: scopes, declarations, references
<name>.type.langue    kinds, type formers, bidirectional typing judgments
```

The Lumo definition (first customer) slices by feature:

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

### 3.2 Cat: one global namespace

**Decided: no import system.** `langc` collects every kind-suffixed file
under the project root and concatenates them into one global namespace.
Neither file order nor item order carries meaning, and no forward
declarations are needed — letrec-style visibility across all files.

### 3.3 stdlib

The Langue stdlib (helper combinators such as `foldr`/`map`, common token
classes, builtin value shapes such as `List`) participates in the same
namespace as project files, exactly as if its files were part of the cat.

### 3.4 DCE instead of reachability

Unused items — stdlib or project — are dead-code-eliminated when the
definition is loaded. Nothing has to be "imported to count", and there is no
unreachable-file error. An optional lint can surface unused *project* items.

### 3.5 Collisions and additive merge

Two same-named items in the global namespace are an error. The designed
exception is additive merging: multiple files contributing rules to the same
judgment or constructors to the same tree (`fn.type.langue` and
`cap.type.langue` both feed `infer`/`check`), with exhaustiveness and overlap
checked globally after the merge.

## 4. File-kind designs

### 4.1 syn: tokens

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

### 4.2 syn: grammar

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

### 4.3 tree

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

### 4.4 elab

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

Same-tree groups run as e-graph equality saturation (section 2.3.2); their
rules are equalities, not directed passes.

### 4.5 scope

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

### 4.6 type

The most ambitious kind, and deliberately the most constrained. The reasoning
engine and the plugged type system (section 2.4) provide the primitives; the
DSL declares the syntax-directed rules on top:

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

Engine/plugin services (invocable, not definable): unification (`~>`),
instantiation/generalization, kind checking, row operations for capabilities,
fresh metavariables. The DSL owns which rule fires for which node, what the
premises are, and what errors each failed premise reports — **decided:
diagnostic message templates live in the DSL** (each premise may carry an
`else error "..." at node` clause; the engine fills type/name holes and maps
`at` to spans). If a rule cannot be expressed, it is written as `extern rule`
in Rust against a stable API — the definition file still names it, so coverage
is visible.

### 4.7 extern hatches

Every kind can declare `extern` items implemented in Rust and registered by
name. The rule: the `.langue` files must mention every extern, so
`grep extern` over the definition shows exactly where the declarative story
has holes.

## 5. Execution model (proposed)

Hybrid, per kind:

### 5.1 Generated Rust

For the hot, shape-defining parts: syn (token DFA, syntax kinds, AST
accessors, parser) and tree (enums, printers). Same workflow as legacy
langue: edit, run `langc gen`, commit generated code.

### 5.2 Interpreted rule tables

For elab/scope/type: `langc` compiles the rules to a compact checked IR that
generic Rust engines execute — the e-graph engine for same-tree elab, the
scope-graph engine, the reasoning engine. Rationale: changing a typing rule
should not require recompiling generated Rust, and these rules are dense in
semantics but not performance-critical enough to need codegen in v1. If
profiling disagrees later, codegen them then.

### 5.3 langc

One Rust crate (`crates/langc`) with a library API so the compiler embeds the
loaded definition, plus a CLI for generation and checking (`langc gen`,
`langc check` — exhaustiveness, unknown labels, extern coverage, collisions).
Both sit on the salsa layer (section 2.1).

## 6. Bootstrapping note

Langue 2 files need their own parser. `langc` hand-writes exactly one parser —
for the `.langue` format itself — and everything downstream is generated. If
we later want langue-in-langue, the format is simple enough to self-describe,
but that is explicitly not a v1 goal (the legacy Lumo-hosted langue rewrite
stays archived as prior art).

## 7. Milestones

- **M0 — langc core**: `.langue` parser, project cat/merge/DCE model,
  `langc check`, salsa query runtime skeleton. Port Lumo's `.syn.langue`
  files; regenerate what legacy langue generated (syntax kinds, AST, lossless
  tree) and now also the surface parser.
- **M1 — tree + elab (lowering)**: declare Core trees, write
  `Surface -> Core` syntax-directed rules; rewrite engine replaces
  hand-written HIR lowering. CBPV split decided here.
- **M2 — scope**: scope facts + resolution engine.
- **M3 — type**: reasoning engine + Fω/caps plugin + judgment DSL; port the
  capability typing rules.
- **M4 — e-graph optimization**: same-tree elab groups on equality
  saturation; port the legacy LTO fixtures as the contract.
- **M5 — reconnect**: backends (start from legacy `ts.rs`/`rs.rs` knowledge,
  written fresh against Core), remaining test fixtures ported from
  `legacy/crates/compiler/tests/fixtures/`.

Each milestone keeps `langc check` + golden-file tests green; fixtures are the
contract carried over from legacy.

## 8. Decisions and open questions

Decided (2026-07-09 ~ 10):

1. **Full-language-definition scope**; name stays **"Langue" (v2)**; tooling
   in **Rust** (`crates/langc`); legacy archived under `legacy/`.
2. **Parser is generated** from the `.syn.langue` files, with precedence
   annotations and extern recovery hooks. Fall back to hand-written only if
   recovery quality proves insufficient at M0 exit.
3. **Arbitrary trees**: Langue fixes no pipeline shape; definitions declare
   any number of trees and elaborate between them. Lumo's CBPV split is a
   `.tree.langue` choice deferred to M1.
4. **Diagnostics live in the DSL** as message templates on rule premises.
5. **Multi-file project format, no imports**: kind-suffixed files
   concatenated project-wide together with the Langue stdlib into one global
   namespace — no `use`, no forward declarations, unused items DCE-ed.
   Same-kind files merge additively, checked globally; the suffix-less
   `<name>.langue` is a manifest (language name + options).
6. **Salsa-like architecture from day one**, at both the definition layer and
   the compiled-language layer.
7. **E-graph equality saturation** for same-tree elaboration (optimization);
   tree-to-tree elaboration stays syntax-directed.
8. **Pluggable type system on a generic reasoning engine**; Lumo v1 plugs
   Fω + spine-local bidirectional inference + capability rows.

Still open:

9. **Cost model declaration** for e-graph extraction — constructor
   annotations, per-rule weights, or extern?
10. **Type-plugin boundary** — how much of a type system lives in the Rust
    plugin trait vs the DSL rule set?
11. **E-graph engine choice** — `egg`, `egglog`, or purpose-built (egglog's
    datalog side might also serve the scope engine).
12. **Hybrid execution boundary** (section 5) — confirm generated vs
    interpreted split against the pillar engines.
