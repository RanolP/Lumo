# Langue 2 implementation plan

Derived from the 32 locked decisions in `design/decisions/`; the design
snapshot is `design/langue.md`. Each item cites the decisions it
implements (D-numbers). Every milestone keeps `langc check` + the fixture
suite green.

## M0 — langc core: project model + syn codegen

Goal: `langc gen` turns `Lumo.syn.langue` into a working parser and
pretty-printer.

- [x] Cargo workspace at repo root; `crates/langc` = library + CLI
      (`langc gen`, `langc check`) (D-01).
- [x] Hand-written parser for the `.langue` format itself — the only
      hand-written parser in the system (bootstrapping note; D-01).
- [x] Project model: glob kind-suffixed files + stdlib into one global
      namespace (cat, D-05); strict name-collision errors, stdlib
      included (D-22); additive merge for same judgment / same language /
      same from-to and between blocks (D-05, D-13, D-14); DCE of unused
      items (D-05); manifest parsing — the pipe that glues syn/elab/type
      files, the entry point (D-27, concrete form in D-33).
- [x] salsa query layer from day one: `parse(file)` →
      `merged_definition(project)` → `generated_files(project)` (D-06;
      salsa 0.27, thin `db.rs` façade).
- [x] syn codegen (D-21): token DFA with longest-match, literal-beats-
      regex, dotted names as highlight scopes (D-09); SyntaxKind; typed
      AST accessors; tree with per-language losslessness (D-18); parser
      from shapes + `praat` blocks with extern recovery hooks (D-02,
      D-03; postfix rows may carry node payloads, D-34); pretty-printer
      (text round-trip, D-03).
- [x] Fixture harness (D-32): corpus format at
      `tests/fixtures/{syn,elab,type}/**/*.test`; `:parse(L)` with
      automatic parse → print → re-parse round-trip; `:fails(L)`;
      `LANGC_UPDATE=1` bless mode.
- [x] Write `Lumo.tokens.syn.langue` / `Lumo.item.syn.langue` /
      `Lumo.expr.syn.langue`; seed fixtures from tree-sitter style and
      `legacy/crates/compiler/tests/fixtures/` (D-32).
- [x] Legacy syntax migration (2026-07-11): the full legacy surface
      grammar (items, attributes, extern, data/cap/impl/use, match,
      if/else, handle, thunk/force, perform, bundle, lambdas, blocks +
      let statements, patterns, types) and every case from the 8 legacy
      syntax fixture files, under `tests/fixtures/syn/legacy/`.
      LL(1) deviations from legacy: annotation folded into ParenExpr;
      `impl Name = Target: Cap` left-factored (ImplAssign/ImplCap);
      ProjTypeExpr folded into NamedTypeExpr; LetExpr atom dropped
      (blocks own `let`); praat AssignExpr row (`x = e; body`) not
      migrated — no fixture uses it; add with M4 parity if needed.
      Verification loop: `scripts/verify.sh`.
- [x] Legacy source migration, parse level (2026-07-11): two standing
      gates in `crates/lumo-syntax/tests/legacy_sources.rs` — all 34
      `legacy/packages/**/*.lumo` files parse cleanly, print losslessly,
      and survive the canonical round-trip; all 139 case sources from
      `legacy/crates/compiler/tests/fixtures/{syntax,type,lto}` parse
      cleanly. Grammar additions this uncovered: generic params on impl
      methods/cap operations, cap annotations on impl methods,
      row-polymorphic cap sets (`/ { ..c, E1 }` — CapEntry/CapRest),
      `cap c` generic params, ident patterns with fields. Excluded:
      `legacy/apps/lumoc/main.lumo` (a ∑/μ/∀ sketch the legacy compiler
      never parsed). The type/lto fixture *expectations* migrate with
      M2/M3; JS golden parity with M4.
- [x] `langc check`: exhaustiveness (LL(1) `|`-arm overlap), unknown
      refs/labels, regex validity, praat sanity, kind-name collisions,
      extern coverage (every extern must be named in the definition,
      D-01/extern rule), collisions.

Exit gate: parser recovery quality is acceptable — otherwise fall back to
hand-written per D-02. **Reviewed 2026-07-11: acceptable.** Repetitions
resync (garbage between decls yields an ERROR node and the next decl
parses; broken argument lists don't eat the following decl), `extern
recover` rules get default FOLLOW∪FIRST sync hooks, and losslessness
survives every error path. Known M0 limits: recovery inside a nested
construct can skip tokens up to (not past) its own follow set only, and
`sep()` interiors don't resync — revisit if real grammars hit it.

## M1 — MIR + elaboration

Goal: `elab Lumo -> MIR` runs end to end as generated Rust.

- [ ] Write `MIR.syn.langue`; decide the CBPV value/computation split
      here; the type AST (TypeV/TypeC) is part of syn (D-15).
- [ ] elab codegen (D-21): `from A to B { pattern ==> construction }`
      blocks → Rust; `<subtree> to <Lang>` recursion restricted to strict
      subtrees of the matched pattern (D-13, D-28); conflicting rules are
      compile-time errors (D-13); cross-file block merging (D-05).
- [ ] Recursion lowering: SCC detection, `fix` primitive for cyclic
      groups, plain `let` otherwise — no letrec core form (D-12).
- [ ] Scope simulation (D-11, D-30): name resolution through Γ contexts;
      `use` statements hoisted first in tree and lowered as
      `λrequire. let x = require('x') in ...`; capability handlers
      lexically scoped, Effekt-like — no dynamic scope.
- [ ] `between A { lhs === rhs }` → egglog programs (D-14, D-19);
      built-in `subst` tactic (`$e[$b := $a]`, D-14, D-24); per-
      constructor costs default 1 compiled to egglog `:cost`, built-in
      min-tree-cost `extract` (D-31).
- [ ] `:elab(A -> B)` fixtures with canonicalize-then-compare (D-32).

## M2 — type

Goal: `LIR.type.langue` judgments typecheck Lumo programs.

- [ ] Relational engine, λProlog as the implementation reference (D-23):
      goals, unification (`=`), contexts as multimaps
      (`context Γ = [Ident: TypeV]`, D-16), read `Γ.$name`, write
      `with Γ+{a: b}` (D-23); strictly decreasing recursion (D-23,
      D-28); exactly one rule may succeed per goal (D-23).
- [ ] Judgment codegen (D-17, D-21): `infer_C LIR -> TypeC with Γ`
      declarations (arrows are separators; type param is inout; names are
      just names), `head := body` rules → Rust on the engine.
- [ ] Minimal built-in tactics: `subst`, `hash`; capability rows as
      hash-keyed maps — a map is a set when the key is a hash, no row
      datatype (D-24, D-25).
- [ ] Diagnostics: failures bail with a generic message — nothing more
      for now (D-26).
- [ ] Write the Fω + spine-local bidirectional + capability-row judgments
      (D-08); port the legacy capability typing rules.
- [ ] `:infer(L)` fixtures (`name : Type` lines, types printed by the
      type sub-language's printer) and `:fails` fixtures (D-32).

## M3 — e-graph optimization

Goal: the between rule groups optimize MIR/LIR under golden fixtures.

- [ ] Same-language between groups run as saturation + extraction on
      egglog (D-07, D-14, D-19, D-31).
- [ ] `:optimize(L)` golden fixtures are the contract (D-32); port the
      legacy LTO fixtures' intent.
- [ ] Watch the known caveat: tree cost double-counts shared subterms —
      if subst-style rewrites get mis-ranked, evaluate DAG-aware
      extraction (D-31).

## M4 — JS emission

Goal: end-to-end Lumo → JS compilation; legacy golden parity.

- [ ] Write `JS.syn.langue`; emission is pretty-printing the JS tree
      (D-03).
- [ ] `elab MIR/LIR -> JS` rules.
- [ ] Bring over the remaining legacy golden fixtures
      (`legacy/crates/compiler/tests/fixtures/`, D-32).

## Cross-cutting rules

- The definition is the source of truth; Rust is engines + generated
  output, never the definition (D-01).
- All three kinds are code-generated — edit, `langc gen`, commit; no
  interpreted rule tables (D-21).
- Documents stay snapshots; new decisions get a new numbered file in
  `design/decisions/` (D-10).
- stdlib and built-in tactics start minimal and grow only on proven need
  (D-24, D-29).
