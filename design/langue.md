# Langue 2 — a full language-definition DSL

Status: current snapshot of the design (as of 2026-07-10); no changelog —
this document only states what is decided and what is open.
One artifact defines all of Lumo — tokens, grammar, every pipeline language,
elaboration, typing. Rust code is engines that execute the definition plus
explicitly-declared escape hatches, never the definition itself.

Decided so far: full-language-definition scope, tooling in Rust, parsers
generated from grammars, diagnostic templates in the DSL, name "Langue"
(v2), multi-file project with no imports (cat + stdlib + DCE), the four
architecture pillars (section 5), every pipeline stage is an individual
language declared by its `.syn.langue` file name, tokens are named
literals/regexes only, scope is not a first-party concept (elaboration
simulates it), elab rules are `from A to B` rewrite blocks (strictly
decreasing, no conflicts) plus same-language `between A` relations run as
e-graph equalities on egglog, a widely-adopted cost model (research
pending), all three kinds code-generated to Rust, and chapter order
project layout → syntax → elaboration → type.

## 1. Project layout

### 1.1 File kinds

A definition is a multi-file Langue project. The suffix declares the role:

```
<name>.langue         project manifest: language name + options (no kind suffix)
<Name>.syn.langue     a language: lexical structure + grammar = shape AND display syntax
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
declarations are needed — order-free visibility across all files.

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
Losslessness (trivia preservation) is defined language by language.

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

## 3. Elaboration (`*.elab.langue`)

### 3.1 `from A to B` blocks

Rules live in `from <Lang> to <Lang>` blocks: a source pattern `==>` a
target construction. Inside a construction, `<subtree> to <Lang>`
recursively elaborates that subtree; node names qualify as `Lumo::FnDecl`.
Blocks with the same from/to pair merge across definitions and files.

```
from Lumo to MIR {
  FnDecl { ... } ==> Lambda { ..., body: Lumo::FnDecl { ... } to MIR }
  ...
}

// merged across multiple definitions, multiple files
from Lumo to MIR { ... }
```

Two checked constraints:

- **Only strictly decreasing allowed** — a recursive `to` call takes a
  strictly smaller input than the matched pattern, so elaboration
  terminates.
- **Conflicting disallowed** — two rules that can fire on the same input
  are an error; there is no rule ordering or priority.

### 3.2 `between A` blocks

On the same language, rules define **relations**: `lhs === rhs` equalities,
run as e-graph equality saturation (section 5.3). `$x` binds a metavariable.
The `subst` tactic is built-in — `$e[$b := $a]`.

```
between A {
  Apply { fn: Lambda { param: $f, body: $f }, arg: $e } === $e
}

between A {
  Apply { fn: Lambda { param: $b, body: $e }, arg: $a } === $e[$b := $a]
}
```

### 3.3 Decided semantics

- Language-to-language elaboration is syntax-directed (`from`/`to`);
  same-language optimization is e-graph equality saturation (`between`).
- **Scope is not a first-party concept.** There is no `.scope.langue` kind
  and no separate scope engine — elaboration simulates scope.
- **Recursion is realized with `fix` only.** There is no `letrec` core form.
  Each mutually-recursive definition group (SCC) lowers through a `fix`
  primitive — `fix (λ(f, g). (body_f, body_g))`, projected back out —
  and acyclic definitions lower to plain `let`. (A `fix` primitive, not a
  literal Y combinator: Y is untypeable in Fω.)

## 4. Type (`*.type.langue`)

Being defined now, step by step from dictation.

### 4.1 The type AST is a sub-language in `*.syn.langue`

Types have presentation, so **the type AST itself is defined in
`*.syn.langue`** — it is a sub-language, with tokens and grammar like any
language. This supports `TypeV` and `TypeC` (CBPV value/computation types)
naturally, with a syntax-integrated AST.

### 4.2 Contexts

A **context** is a named multimap — theoretically a set of tuples:

```
context Γ = [Ident: TypeV]
```

### 4.3 Judgments: `infer` and `check` are definable

Judgments form a **λProlog-style relational language**. A declaration
separates the relation's two parameters with an arrow — `->` and `<-` both
just mean "parameter"; the arrow is mere separator notation, not a
direction. Because the language is relational, assignment happens at the
bottom of a derivation and propagates up, so the type parameter is in fact
**inout**. `with Γ` attaches a context — there can be many:

```
infer_C LIR -> TypeC with Γ
check_C LIR <- TypeC with Γ
```

Rules are `head := body`; `=` unifies, and a parenthesized judgment call is
a goal:

```
infer_V TypeAssignment { expr: $e, ty: $t } -> $inferred
  := $inferred = (check_V $e <- $t)
```

Suffixes like `_V`/`_C` carry no meaning to the engine — a judgment name is
just a name, like a function naming convention (`infer_ε`, `infer_wtf`,
`infer_YOU_WILL_BE_FIRED` are all valid).

The context is accessed as `Γ.$name`:

```
infer_V Ident { name: $name } -> $return := $return = Γ.$name
```

### 4.4 Decided at the architecture level

- **There is no "type plugin".** A type system is the judgments a
  definition declares — nothing else. Judgments run on a generic
  relational engine that builds real derivation trees (section 5.4); Lumo
  v1's judgments implement Fω + spine-local bidirectional inference +
  capability rows.
- Diagnostic message templates live in the DSL, attached to rule premises.

### 4.5 To close this chapter (open)

1. **Context extension** — reading is `Γ.$name`, but there is no writing
   yet. A λ rule must check its body with the parameter added: for
   `check_C Lambda { param: $x, body: $b } <- ...` the `$b` goal has to
   run under Γ extended with `[$x: $t1]`. Options: a local extension form
   on a goal (something like `Γ + [$x: $t1]`), or λProlog-style
   hypothetical assumptions (add the fact for the duration of the goal).
   Which notation?
2. **Rule overlap and search** — when two rules match the same head, is it
   an error (elab's "conflicting disallowed") or does the relational
   language backtrack, trying the next rule on failure? Backtracking makes
   diagnostics harder (which failure gets reported?) and needs its own
   termination story (elab's was "strictly decreasing").
3. **Diagnostic attachment point** — templates live in the DSL (decided),
   but a `head := body` body is a chain of goals. When `$return = Γ.$name`
   fails (unbound name), where does the "unbound variable {name}" template
   sit: per goal, per rule, or per judgment with a default?
4. **Built-in tactics for the type side** — elab got `subst` as a built-in
   tactic. Same question here for: fresh metavariables,
   ∀ instantiation/generalization, kind checking, type-level β. Which are
   built-in tactics, which are definable judgments? (A possible line:
   what must appear in the derivation tree is a judgment; purely
   mechanical operations are tactics.)
5. **Entry point** — a definition declares many judgments (`infer_V`,
   `infer_C`, `check_C`, …). When langc is asked to typecheck a Lumo
   file, which judgment is the root, and how is that declared? Also: the
   API that exposes derivation trees to the LSP (hover, "why this type").
6. **Capability rows** — Lumo function types carry `ret | ε`. Rows unify
   set-like (order-free, duplicate-free, row variables), unlike plain
   structural unification. How are rows written in the type sub-language,
   and does the engine get a built-in row-unification tactic?

## 5. Core architecture

Four pillars underneath chapters 1–4.

### 5.1 Salsa-like incremental queries

**Decided: day one.** Every derived artifact is a memoized query over
inputs, at both layers:

- **Definition layer (langc)**: `parse(file)` → `merged_definition(project)`
  → `rule_tables(kind)` → generated code. Editing one `.type.langue` file
  re-derives only the type tables.
- **Compiled-language layer**: the compiler that Langue produces is itself
  query-structured — `cst(file)`, `mir(file)`, `infer(node)`.

Rule tables are pure values, which is exactly what memoization wants: the
cat/merge/DCE project model (section 1) produces one immutable definition
value per revision. Candidate runtime: the `salsa` crate, or a purpose-built
equivalent if salsa's model fights the relational engine.

### 5.2 Ungrammar-like notation

A `.langue` file describes the **shape of trees**, never a parsing
algorithm. Labels become accessors, alternatives become node kinds, and the
parser is *derived* from the shape (plus `praat` blocks) rather than
written. Every pipeline language is declared this way and therefore carries
its own display syntax (section 2.1). The philosophy extends further: elab
rules and typing rules are likewise shape-first, algorithm-free declarations
that engines interpret.

### 5.3 E-graph elaboration

Language-to-language elaboration is syntax-directed; same-language
optimization runs as e-graph equality saturation with cost-based extraction,
so rule order carries no meaning. **Engine: egglog.** Same-language rules
are declared as `between A` relation blocks (section 3.2); the cost model
is a widely-adopted one, picked by a research pass (section 10.2).

### 5.4 Relational type engine

#### 5.4.1 The engine

Type checking runs on a generic **relational engine**: the executor of the
λProlog-style judgment language (section 4). It solves goals, unifies, and
builds proper derivation trees. The engine knows nothing about Lumo.
**There is no "type plugin"** — a type system is the judgments a definition
declares; Lumo v1's judgments implement Fω + spine-local bidirectional
inference + capability rows.

#### 5.4.2 Derivations as artifacts

Because the engine produces real derivations, failed premises map to the
DSL's diagnostic templates with full context, successful ones power
explanations ("why does this expression have this type") and LSP hovers, and
every derivation is a salsa query — incremental re-typechecking falls out.

## 6. Extern hatches

Every kind can declare `extern` items implemented in Rust and registered by
name. The rule: the `.langue` files must mention every extern, so
`grep extern` over the definition shows exactly where the declarative story
has holes.

## 7. Execution model

**Decided: all three kinds are code-generated.** `langc` generates Rust
for everything — there are no interpreted rule tables:

- **syn** — per declared language: token DFA, syntax kinds, AST accessors,
  parser, pretty-printer.
- **elab** — `from`/`to` rewrites and `between` relations, compiled to
  Rust running on egglog.
- **type** — judgment rules and diagnostic templates, compiled to Rust on
  the relational engine.

One workflow everywhere: edit, run `langc gen`, commit generated code.

### 7.1 langc

One Rust crate (`crates/langc`) with a library API so the compiler embeds
the loaded definition, plus a CLI (`langc gen`, `langc check` —
exhaustiveness, unknown labels, extern coverage, collisions). Both sit on
the salsa layer (section 5.1).

## 8. Bootstrapping note

Langue 2 files need their own parser. `langc` hand-writes exactly one
parser — for the `.langue` format itself — and everything downstream is
generated. Langue-in-langue self-description is explicitly not a v1 goal.

## 9. Milestones

- **M0 — langc core**: `.langue` parser, project cat/merge/DCE model,
  `langc check`, salsa query runtime skeleton. Write `Lumo.syn.langue`;
  generate SyntaxKind, AST, lossless tree, parser, and printer.
- **M1 — MIR + elab (lowering)**: write `MIR.syn.langue` and the
  `elab Lumo -> MIR` syntax-directed rules; the rewrite engine owns
  lowering. CBPV split decided here.
- **M2 — type**: relational engine + the Fω/caps judgments; port the
  capability typing rules.
- **M3 — e-graph optimization**: same-language elab groups on equality
  saturation; optimization golden fixtures are the contract.
- **M4 — JS**: write `JS.syn.langue` and `elab MIR/LIR -> JS` — emission is
  pretty-printing the JS tree; remaining golden fixtures brought over
  (source material in `legacy/crates/compiler/tests/fixtures/`).

Each milestone keeps `langc check` + golden-file tests green.

## 10. Decisions and open questions

Locked decisions live one per file in `design/decisions/`:

1. [Full-language-definition DSL](decisions/01-full-language-definition.md)
2. [Parsers are generated](decisions/02-generated-parsers.md)
3. [Every stage is a language declared by its file name](decisions/03-language-per-file.md)
4. [Diagnostics live in the DSL](decisions/04-diagnostics-in-dsl.md)
5. [No imports — cat + stdlib + DCE](decisions/05-no-imports.md)
6. [Salsa-like architecture from day one](decisions/06-salsa-day-one.md)
7. [E-graph for same-language elaboration](decisions/07-egraph-elaboration.md)
8. [No type plugin — a type system is its judgments](decisions/08-type-system-is-judgments.md)
9. [Tokens are named literals/regexes only](decisions/09-token-model.md)
10. [Chapter order and document policy](decisions/10-chapter-order.md)
11. [Scope is not a first-party concept](decisions/11-scope-not-first-party.md)
12. [Recursion via `fix` only](decisions/12-fix-only-recursion.md)
13. [Elab rule form: `from A to B` blocks](decisions/13-elab-from-to.md)
14. [Same-language relations: `between A` blocks](decisions/14-between-relations.md)
15. [The type AST is a syn sub-language](decisions/15-type-ast-sub-language.md)
16. [Contexts](decisions/16-contexts.md)
17. [Definable judgments — λProlog style](decisions/17-definable-judgments.md)
18. [Losslessness per language](decisions/18-losslessness-per-language.md)
19. [E-graph engine: egglog](decisions/19-egglog.md)
20. [Cost model: adopt a widely-used model](decisions/20-cost-model.md)
21. [All three kinds are code-generated](decisions/21-full-codegen.md)

Open questions, in detail (the six type-chapter questions live in
section 4.5; all are mirrored in the artifact):

### 10.1 Strictly-decreasing measure (confirm interpretation)

"only strictly decreasing allowed" was interpreted as: a recursive `to`
call inside a construction may only take a **strict subtree of the
matched pattern**, which guarantees elaboration terminates. Confirm, or
state the intended measure. (Also: the dictated `*.elab.lumo` header was
interpreted as a typo for `*.elab.langue`.)

### 10.2 Cost-model research (action item)

Decided to adopt a widely-used cost model; a research pass picks the
concrete one. Candidates to survey: egg / egglog per-constructor costs
with minimal-total-cost extraction (`:cost` annotations), and the
extraction-gym line of work (greedy vs DAG-aware/ILP extractors).
