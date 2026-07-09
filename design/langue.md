# Langue 2 — a full language-definition DSL

Status: current snapshot of the design (as of 2026-07-10); no changelog —
this document only states what is decided and what is open.
One artifact defines all of Lumo — tokens, grammar, every pipeline language,
elaboration, binding, typing. Rust code is engines that execute the
definition plus explicitly-declared escape hatches, never the definition
itself.

Decided so far: full-language-definition scope, tooling in Rust, parsers
generated from grammars, diagnostic templates in the DSL, name "Langue"
(v2), multi-file project with no imports (cat + stdlib + DCE), the four
architecture pillars (section 6), every pipeline stage is an individual
language declared by its `.syn.langue` file name, tokens are named
literals/regexes only, and chapter order project layout → syntax → scope →
elaboration → type.

## 1. Project layout

### 1.1 File kinds

A definition is a multi-file Langue project. The suffix declares the role:

```
<name>.langue         project manifest: language name + options (no kind suffix)
<Name>.syn.langue     a language: lexical structure + grammar = shape AND display syntax
<name>.scope.langue   binding: scopes, declarations, references
<name>.elab.langue    elaboration: rewrite rules between/within languages
<name>.type.langue    kinds, type formers, bidirectional typing judgments
```

The Lumo definition slices by feature:

```
lumo/
  lumo.langue
  Lumo.tokens.syn.langue
  Lumo.item.syn.langue
  Lumo.expr.syn.langue
  MIR.syn.langue
  JS.syn.langue
  scope/lexical.scope.langue
  elab/item.elab.langue
  elab/expr.elab.langue
  types/fn.type.langue
  types/cap.type.langue
  types/data.type.langue
```

### 1.2 Cat: one global namespace

**Decided: no import system.** `langc` collects every kind-suffixed file
under the project root and concatenates them into one global namespace.
Neither file order nor item order carries meaning, and no forward
declarations are needed — letrec-style visibility across all files.

### 1.3 stdlib

The Langue stdlib (helper combinators such as `foldr`/`map`, common token
classes, builtin value shapes such as `List`) participates in the same
namespace as project files, exactly as if its files were part of the cat.

### 1.4 DCE instead of reachability

Unused items — stdlib or project — are dead-code-eliminated when the
definition is loaded. Nothing has to be "imported to count", and there is no
unreachable-file error. An optional lint can surface unused *project* items.

### 1.5 Collisions and additive merge

Two same-named items in the global namespace are an error. The designed
exception is additive merging: multiple files contributing rules to the same
judgment or grammar rules to the same language (`fn.type.langue` and
`cap.type.langue` both feed `infer`/`check`; `Lumo.item.syn.langue` and
`Lumo.expr.syn.langue` both feed language `Lumo`), with exhaustiveness and
overlap checked globally after the merge.

## 2. Syntax (`*.syn.langue`)

### 2.1 A `.syn.langue` file declares a language

**`*.syn.langue` declares a language.** A definition declares as many as it
wants — Lumo, MIR, LIR, JS — and a language consists of **tokens** and
**grammar**. The file name is the declaration: `MIR.syn.langue` existing
puts `MIR` into the global namespace, referenceable as a language. A
language split across files uses the first name segment
(`Lumo.expr.syn.langue`).

Per language, parser and pretty-printer are derived — every stage
round-trips as text, and emitting JavaScript is pretty-printing a JS tree.

### 2.2 Tokens

A token is a name bound to a string literal or a regex — nothing else. The
name is the token's display identity (debug dumps, syntax kinds,
diagnostics). Longest match wins; on ties a literal beats a regex. Dotted
names double as highlight scopes.

```
token keyword.fn = 'fn'            // 'fn' displays as keyword.fn
token ident      = /[a-zA-Z_][a-zA-Z0-9_]*/
token lit.number = /[0-9]+(\.[0-9]+)?/
trivia comment.line = /\/\/[^\n]*/
```

In grammar rules, literal tokens are written as their literal (`'fn'`) and
regex tokens by name (`name:ident`).

### 2.3 Grammar

Rules are shape and display syntax at once; labels become accessors. Lists:
`sep(Param, ',')`. Expressions: `praat` blocks — `simple` lists the atoms;
in `operators`, `@` is a placeholder for an expression operand and the
number is the binding power between tokens. Placement draws the operator
shape (prefix, infix, postfix, mixfix).

```
FnDecl = 'fn' name:ident params:sep(Param, ',') …

Expr = praat {
  simple = Lit | Ident | ParenExpr
  operators {
    '+' | '-' | '!' @100,
    @89 '**' @90,
    @80 '*' | '/' @79,
    @70 '+' | '-' @69,
    @40 '?' @0 ':' @39,     // a ? b : c
  }
}
```

Per language, the generator emits syntax kinds, typed AST accessors, a
lossless tree, the parser (extern recovery hooks), and the pretty-printer.

## 3. Scope (`*.scope.langue`)

Scope-graph style fact declarations (Statix-inspired, much smaller):

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
and answers resolution queries. Output feeds both `type` (section 5) and the
LSP (go-to-def falls out for free).

## 4. Elaboration (`*.elab.langue`)

Not designed yet — taken up after scope (chapter 3) is settled. Decided at
the architecture level only: language-to-language elaboration is
syntax-directed; same-language optimization is e-graph equality saturation
(section 6.3).

## 5. Type (`*.type.langue`)

The most ambitious kind, and deliberately the most constrained. The reasoning
engine and the plugged type system (section 6.4) provide the primitives; the
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
in Rust against a stable API — the definition file still names it, so
coverage is visible.

## 6. Core architecture

Four pillars underneath chapters 1–5.

### 6.1 Salsa-like incremental queries

**Decided: day one.** Every derived artifact is a memoized query over
inputs, at both layers:

- **Definition layer (langc)**: `parse(file)` → `merged_definition(project)`
  → `rule_tables(kind)` → generated code. Editing one `.type.langue` file
  re-derives only the type tables.
- **Compiled-language layer**: the compiler that Langue produces is itself
  query-structured — `cst(file)`, `mir(file)`, `resolve(node)`,
  `infer(node)`.

Rule tables are pure values, which is exactly what memoization wants: the
cat/merge/DCE project model (section 1) produces one immutable definition
value per revision. Candidate runtime: the `salsa` crate, or a purpose-built
equivalent if salsa's model fights the reasoning engine.

### 6.2 Ungrammar-like notation

A `.langue` file describes the **shape of trees**, never a parsing
algorithm. Labels become accessors, alternatives become node kinds, and the
parser is *derived* from the shape (plus `praat` blocks) rather than
written. Every pipeline language is declared this way and therefore carries
its own display syntax (section 2.1). The philosophy extends further: elab
rules, scope facts, and typing rules are likewise shape-first,
algorithm-free declarations that engines interpret.

### 6.3 E-graph elaboration

Language-to-language elaboration is syntax-directed; same-language
optimization runs as e-graph equality saturation with cost-based extraction,
so rule order carries no meaning. Candidate engines: `egg` / `egglog`.
Details are filled in when chapter 4 is designed; the cost-model declaration
site is open (section 11).

### 6.4 Pluggable type system

#### 6.4.1 The reasoning engine

Type checking runs on a generic **reasoning engine**: an inference-rule
interpreter that builds proper derivation trees. Premises, modes
(infer ⇒ / check ⇐), unification, instantiation/generalization, and fresh
metavariables are engine services. The engine knows nothing about Lumo.

#### 6.4.2 Type systems as plugins

A *type system* is a plugin on that engine: a Rust-side capability bundle
(type formers, kind rules, row operations) plus the DSL rule set that drives
it. Lumo v1 plugs in "Fω + spine-local bidirectional inference + capability
rows". Because the engine is generic, a different language (or a future
Lumo) can plug a different system without touching the engine. The plugin
boundary (Rust trait vs DSL rules) is open (section 11).

#### 6.4.3 Derivations as artifacts

Because the engine produces real derivations, failed premises map to the
DSL's diagnostic templates with full context, successful ones power
explanations ("why does this expression have this type") and LSP hovers, and
every derivation is a salsa query — incremental re-typechecking falls out.

## 7. Extern hatches

Every kind can declare `extern` items implemented in Rust and registered by
name. The rule: the `.langue` files must mention every extern, so
`grep extern` over the definition shows exactly where the declarative story
has holes.

## 8. Execution model (proposed)

Hybrid, per kind:

### 8.1 Generated Rust

For the hot, shape-defining parts: syn — per declared language, the token
DFA, syntax kinds, AST accessors, parser, and pretty-printer. Workflow:
edit, run `langc gen`, commit generated code.

### 8.2 Interpreted rule tables

For scope/elab/type: `langc` compiles the rules to a compact checked IR that
generic Rust engines execute — the scope-graph engine, the e-graph engine
for same-language elab, the reasoning engine. Rationale: changing a typing
rule should not require recompiling generated Rust; these rules are dense in
semantics but not performance-critical enough to need codegen in v1. If
profiling disagrees later, codegen them then.

### 8.3 langc

One Rust crate (`crates/langc`) with a library API so the compiler embeds
the loaded definition, plus a CLI (`langc gen`, `langc check` —
exhaustiveness, unknown labels, extern coverage, collisions). Both sit on
the salsa layer (section 6.1).

## 9. Bootstrapping note

Langue 2 files need their own parser. `langc` hand-writes exactly one
parser — for the `.langue` format itself — and everything downstream is
generated. Langue-in-langue self-description is explicitly not a v1 goal.

## 10. Milestones

- **M0 — langc core**: `.langue` parser, project cat/merge/DCE model,
  `langc check`, salsa query runtime skeleton. Write `Lumo.syn.langue`;
  generate SyntaxKind, AST, lossless tree, parser, and printer.
- **M1 — MIR + elab (lowering)**: write `MIR.syn.langue` and the
  `elab Lumo -> MIR` syntax-directed rules; the rewrite engine owns
  lowering. CBPV split decided here.
- **M2 — scope**: scope facts + resolution engine.
- **M3 — type**: reasoning engine + Fω/caps plugin + judgment DSL; port the
  capability typing rules.
- **M4 — e-graph optimization**: same-language elab groups on equality
  saturation; optimization golden fixtures are the contract.
- **M5 — JS**: write `JS.syn.langue` and `elab MIR/LIR -> JS` — emission is
  pretty-printing the JS tree; remaining golden fixtures brought over
  (source material in `legacy/crates/compiler/tests/fixtures/`).

Each milestone keeps `langc check` + golden-file tests green.

## 11. Decisions and open questions

Decided:

1. **Full-language-definition scope**; name stays **"Langue" (v2)**; tooling
   in **Rust** (`crates/langc`).
2. **Parsers are generated** from the `.syn.langue` files, with `praat`
   blocks and extern recovery hooks. Fall back to hand-written only if
   recovery quality proves insufficient at M0 exit.
3. **Every stage is an individual language declared by file name**:
   `MIR.syn.langue` puts `MIR` in the global namespace, referenceable as a
   language; no marker keywords, no surface/internal distinction, arbitrary
   chain length. Emission = pretty-printing the target language (JS).
   Lumo's CBPV split is a `.syn.langue` choice deferred to M1.
4. **Diagnostics live in the DSL** as message templates on rule premises.
5. **Multi-file project format, no imports**: kind-suffixed files
   concatenated project-wide together with the Langue stdlib into one global
   namespace — no `use`, no forward declarations, unused items DCE-ed.
   Same-kind files merge additively, checked globally; the suffix-less
   `<name>.langue` is a manifest (language name + options).
6. **Salsa-like architecture from day one**, at both the definition layer
   and the compiled-language layer.
7. **E-graph equality saturation** for same-language elaboration
   (optimization); language-to-language elaboration stays syntax-directed.
8. **Pluggable type system on a generic reasoning engine**; Lumo v1 plugs
   Fω + spine-local bidirectional inference + capability rows.
9. **Tokens are named literals/regexes only**: `token keyword.fn = 'fn'` —
   the name is the debug/display identity of the token; no special forms;
   longest match wins, literal beats regex on ties; dotted names double as
   highlight scopes.
10. **Chapter order**: project layout → syntax → scope → elaboration → type;
    architecture pillars after them; documents are snapshots (no changelog,
    no legacy exposition).

Still open (mirrored in the artifact's 회신 대기 box):

11. **Cost model declaration** for e-graph extraction — proposal: grammar
    annotations by default, extern as escape hatch.
12. **Type-plugin boundary** — how much of a type system lives in the Rust
    plugin trait vs the DSL rule set?
13. **E-graph engine choice** — proposal: one egglog spike that also tests
    the "datalog side doubles as the scope engine" hypothesis.
14. **Hybrid execution boundary** (section 8) — confirm generated vs
    interpreted split against the pillar engines.
15. **Losslessness scope** — proposal: only Lumo (the surface language) is
    lossless; other languages round-trip via canonical pretty-print (e-graph
    nodes carry no trivia).
16. **Milestone order** — chapter order is reading order; should the build
    order also move scope (M2) ahead of elab (M1)?
