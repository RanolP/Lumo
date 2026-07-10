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
e-graph equalities, and chapter order project layout → syntax →
elaboration → type.

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

- Pluggable type system on a generic reasoning engine (section 5.4); Lumo
  v1 plugs Fω + spine-local bidirectional inference + capability rows.
- Diagnostic message templates live in the DSL, attached to rule premises.

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
equivalent if salsa's model fights the reasoning engine.

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
so rule order carries no meaning. Candidate engines: `egg` / `egglog`.
Same-language rules are declared as `between A` relation blocks
(section 3.2); the cost-model declaration site is open (section 10).

### 5.4 Pluggable type system

#### 5.4.1 The reasoning engine

Type checking runs on a generic **reasoning engine**: an inference-rule
interpreter that builds proper derivation trees. Premises, modes
(infer ⇒ / check ⇐), unification, instantiation/generalization, and fresh
metavariables are engine services. The engine knows nothing about Lumo.

#### 5.4.2 Type systems as plugins

A *type system* is a plugin on that engine: a Rust-side capability bundle
(type formers, kind rules, row operations) plus the DSL rule set that drives
it. Lumo v1 plugs in "Fω + spine-local bidirectional inference + capability
rows". Because the engine is generic, a different language (or a future
Lumo) can plug a different system without touching the engine. The plugin
boundary (Rust trait vs DSL rules) is open (section 10).

#### 5.4.3 Derivations as artifacts

Because the engine produces real derivations, failed premises map to the
DSL's diagnostic templates with full context, successful ones power
explanations ("why does this expression have this type") and LSP hovers, and
every derivation is a salsa query — incremental re-typechecking falls out.

## 6. Extern hatches

Every kind can declare `extern` items implemented in Rust and registered by
name. The rule: the `.langue` files must mention every extern, so
`grep extern` over the definition shows exactly where the declarative story
has holes.

## 7. Execution model (proposed)

Hybrid, per kind:

### 7.1 Generated Rust

For the hot, shape-defining parts: syn — per declared language, the token
DFA, syntax kinds, AST accessors, parser, and pretty-printer. Workflow:
edit, run `langc gen`, commit generated code.

### 7.2 Interpreted rule tables

For elab/type: `langc` compiles the rules to a compact checked IR that
generic Rust engines execute — the e-graph engine for same-language elab,
the reasoning engine. Rationale: changing a typing rule should not require
recompiling generated Rust; these rules are dense in semantics but not
performance-critical enough to need codegen in v1. If profiling disagrees
later, codegen them then.

### 7.3 langc

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
- **M2 — type**: reasoning engine + Fω/caps plugin + judgment DSL; port the
  capability typing rules.
- **M3 — e-graph optimization**: same-language elab groups on equality
  saturation; optimization golden fixtures are the contract.
- **M4 — JS**: write `JS.syn.langue` and `elab MIR/LIR -> JS` — emission is
  pretty-printing the JS tree; remaining golden fixtures brought over
  (source material in `legacy/crates/compiler/tests/fixtures/`).

Each milestone keeps `langc check` + golden-file tests green.

## 10. Decisions and open questions

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
10. **Chapter order**: project layout → syntax → elaboration → type;
    architecture pillars after them; documents are snapshots (no changelog,
    no legacy exposition).
11. **Scope is not a first-party concept**: no `.scope.langue` kind, no
    scope engine — elaboration simulates scope.
12. **Recursion via `fix` only**: elaboration lowers each mutually-recursive
    SCC through a core `fix` primitive — no `letrec` core form.
13. **Elab rule form**: `from A to B { pattern ==> construction }` blocks;
    `<subtree> to <Lang>` inside a construction is recursive elaboration;
    same from/to blocks merge across files. Only strictly decreasing
    recursion allowed; conflicting rules disallowed.
14. **Same-language relations = `between A` blocks**: `lhs === rhs`
    equalities run as e-graph equality saturation; `$x` metavariables;
    `subst` tactic built-in (`$e[$b := $a]`).
15. **The type AST is a syn sub-language**: types have presentation, so the
    type AST is defined in `*.syn.langue`; `TypeV`/`TypeC` are supported
    naturally with a syntax-integrated AST.
16. **Contexts**: `context Γ = [Ident: TypeV]` — a named multimap
    (theoretically a set of tuples).
17. **Definable judgments, λProlog style**: `infer_C LIR -> TypeC with Γ`,
    `check_C LIR <- TypeC with Γ` — the arrow is separator notation only;
    both sides are parameters and the type is effectively inout (relational:
    assignment propagates bottom-up); `with` attaches contexts (one or
    many); rules are `head := body`.

Still open (mirrored in the artifact's 회신 대기 box):

15. **Cost model declaration** for e-graph extraction — proposal: grammar
    annotations by default, extern as escape hatch.
16. **Type-plugin boundary** — how much of a type system lives in the Rust
    plugin trait vs the DSL rule set?
17. **E-graph engine choice** — proposal: one egglog spike, then decide.
18. **Hybrid execution boundary** (section 7) — confirm generated vs
    interpreted split against the pillar engines.
19. **Losslessness scope** — proposal: only Lumo (the surface language) is
    lossless; other languages round-trip via canonical pretty-print (e-graph
    nodes carry no trivia).
20. **Strictly-decreasing measure** — interpreted as: a recursive `to` call
    may only take a strict subtree of the matched pattern. Confirm. (Also
    interpreted the dictated `*.elab.lumo` header as `*.elab.langue`.)
