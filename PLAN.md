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

## M1 — MIR + elaboration (landed 2026-07-11)

Goal: `elab Lumo -> MIR` runs end to end as generated Rust.

- [x] `MIR.syn.langue`: strict CBPV, two syntactic sorts (decision 36);
      TypeV/TypeC sub-language live via `(v : T)` annotations (D-15);
      `:parse(MIR)` fixtures; manifest stage `elab Lumo to MIR` live.
- [x] Elab surface locked as decision 35 (derived form: syn-label
      fields, omitted-is-wildcard, `$x`, `[$x*]`, `$x to Lang`,
      `$e[$b := $a]`); lexer/parser/AST for `from`/`between`/`extern
      rule`/`extern pass` items.
- [x] elab codegen (D-21): merged blocks by (from,to)/language (D-05,
      D-13, D-14) → generated Rust per pair (dispatch by root kind,
      accessor-scheme matchers, builder-rendered constructions reparsed
      with the target parser); strict-subtree `to` recursion (D-28);
      conflict detection (same root, not literal/ctor-disjoint → error,
      D-13); checks for nodes/fields/metavars/list shapes/required
      fields. Sort coercion + extern rules/passes = decision 38
      (auto-let `__tN` for comp-in-value, `ret` for value-in-comp;
      externs are trait methods without defaults — missing Rust impls
      fail the build, D-01). Extern rules grew beyond the planned
      `member_classify`: `module`, `fn_curry`, `block`, `match_arm`,
      `use_decl` host the fold/optional-field lowerings the derived rule
      form cannot express.
- [x] Recursion lowering (D-12), M1 slice: `extern pass scc_fix` wraps
      self-recursive defs in `fix`; mutually recursive groups are left
      for M2's typechecker to reject (tuple/projection encoding deferred).
- [x] Scope simulation (D-30), M1 slice: `use foo.bar;` →
      `def bar = thunk { let m = force require("foo") in sel m.bar }`,
      hoisted first by `extern pass use_require`; handlers lexically
      scoped via `handle … with … in`. Γ-context name resolution waits
      for M2's judgment machinery. Known M1 semantics note: a chained
      call `f(x)(y)` elaborates through an auto-let of the inner call
      (legacy folded applies without rebinding) — revisit at M4 parity.
- [x] `between A { lhs === rhs }` → egglog program text (D-14, D-19):
      grammar as `datatype*` with `:cost 1` (D-31), `subst` declared
      from lhs binding sorts (D-24), one rewrite per relation; golden
      fixture `tests/fixtures/egglog/MIR.egg`. Execution (saturation +
      extraction) parked for M3 = decision 37.
- [x] `:elab(A -> B)` corpus attribute with canonicalize-then-compare
      (D-32); 15 fixtures covering the legacy lowering behaviors
      (curried thunk spines, force-then-apply, roll'd ctors, case
      unroll, cap ops via `sel (perform C)`, blocks, handle/bundle,
      use/require, fix).

## M2 — type

Goal: `MIR.type.langue` judgments typecheck Lumo programs — on MIR
directly, no LIR (decision 39). Scope slice = D-39, Fω depth = D-40,
capability rows = D-41; design artifact `design/m2-type-brainstorm.html`.

- [x] Step 1 (2026-07-11) — MIR type sub-language grows rows + forall
      (D-39/40/41): `FTypeC` takes an optional `CapRow` (rows serialize
      on F only), `ForallTypeC` with arity-0 binders (no kind grammar),
      CapRow/CapSet/CapSig/CapRest mirroring the Lumo surface
      `/ { ..c, E1 }`; new `:parse(MIR)` fixtures; egglog golden
      re-blessed.
- [x] Step 2 (2026-07-11) — elab preserves type information: fully
      annotated FnDecl signatures become def-level `(thunk { … } :
      U(…))` annotations (types curried to match the term; nullary =
      bare F; CapAnnotation → CapRow on the innermost F; GenericParams
      → forall, cap params included); `(e : T)` survives as ParenV via
      the new `paren_annot` extern rule; `scc_fix` keeps annotations
      around `fix`. Partial signatures, assoc types, bounded generics,
      and latent-effect arrows bail to inference (D-39/41).
- [x] Step 3 (2026-07-11) — relational engine: `langue-rt::judge`
      (λProlog reference, D-23). First-order terms + unification with
      occurs check; contexts as named multimaps, reads newest-first
      (shadowing), scoped `with Γ+{k: v}` extension around calls
      (D-16/D-23); exactly-one-rule-succeeds via snapshot trials —
      zero = soft bail (D-26), two+ = hard error; strict decrease
      checked at runtime per judgment on the first argument's term
      size (D-28) — non-decreasing rules bail instead of diverging;
      derivation trees with args resolved once the whole proof
      succeeds (assignment propagates up, D-17). 7 unit tests.
- [x] Step 4 (2026-07-11) — judgment codegen (D-17, D-21): `.type.langue`
      is the third item kind — `context Γ = [Ident: TypeV]` decls,
      `infer_C MIR -> TypeC with Γ` declarations (first sort = subject
      language; arrows are separators), `head := body` rules with goals
      (unify, `Γ.$k` reads, calls with `with Γ+{k: v}`; a call as an
      expression pads trailing args and evaluates to its last one).
      Full slice: lexer (unicode names, `->`/`<-`, `.`), parser,
      merge (decl collisions strict, rules additive), `check/judge.rs`,
      `codegen/judge.rs` emitting `<lang>/judgments.rs` (rule table +
      canonical tree→term encoder + `solve`). Term contract: node =
      `Struct(Name, fields in node_fields order)`, tokens = `Atom(text)`,
      absent = `#none`, lists = `#list`; heads wildcard omitted fields,
      bodies default them. Seed `MIR.type.langue` (infer_V/infer_C over
      Num/Str/Var/Thunk/Paren/Ret/Let) + 3 end-to-end smoke tests.
- [x] Step 5 (2026-07-11) — minimal built-in tactics (D-24/25/41):
      engine `Term::Set` — hash-keyed entries (dedup by structural
      key, canonical order, rest absorption, `{|r}` collapse) with
      open-row unification (`{A|r} = {A,B}` binds `r={B}`; two open
      rows share a fresh tail; greedy entry matching, no backtracking
      across matchings); `Goal::Subst` (naively structural — capture
      is the rule writer's concern until binders demand better) and
      `Goal::Hash` (`#list` → set, idempotent). DSL: `$e[$b := $a]`
      terms and the reserved `(hash $list)` call. 6 engine tests +
      2 DSL-through-generated-code tests on real MIR cap rows.
- [x] Step 6 (2026-07-12) — the real judgments (~70 rules in
      `MIR.type.langue`): spine-local bidirectional (D-08) — lambdas
      and fix type in check mode only, off annotations; `annot`
      normalizes syntactic types to semantic (rows become engine sets,
      `..c` a rigid `RowVar` tail); rows thread as an *ambient*
      permission set on `infer_C`/`check_C` — performs need `subset`
      membership, `handle` extends the ambient, F-elimination sites
      (`let`, `apply`, `match_c`) pay row subsumption; foralls
      instantiate via `inst` (subst per binder as type var + row var)
      at application/coercion, and unwrap rigid at the annotated-thunk
      boundary (strict-decrease-safe); data ctors/match instantiate
      `Δ.tag = Variant(owner, binders, params)` against the scrutinee;
      bundles check clause-name sets (`hash`) and bodies against
      `Σ.Op`/`Σ.Ops`. Infra this forced: cons-list term encoding +
      `[]`/`[$h | $t]` patterns, `{…| rest}` set terms, raw functor
      terms, engine `Goal::Subset`, and a dotted-name split for
      `Σ.Op(...)` reads. 10 end-to-end tests (annotated checking,
      rows incl. discharge + rigid tails, data/match, forall
      instantiation, bundle completeness). Known edge: checking a
      thunk against an unconstrained type bails hard (three U-rules) —
      annotate the spine. Unannotated lambdas/fix bail (cap and type
      inference of bare defs deferred with D-39's cap_inference).
- [x] Step 7 (2026-07-12) — `:infer(Lumo)` corpus attribute runs the
      full pipeline (parse Lumo, elab, judge each def in order); the
      handwritten driver (`judge_driver.rs`, next to the extern impls)
      seeds Δ/Σ/Γ from the *Lumo* tree per the seed-shape contracts,
      prints types in the MIR type sub-language (`name : Type` lines;
      rows as ` / {entries, ..rest}`), and reports bails as `ERROR`
      (any-message match, D-26). 8 fixture files under
      `tests/fixtures/type/` migrate the in-scope legacy buckets
      (~40 cases): basics, annotation, data (+match+iso), recursion,
      extern, cap, cap_row, hof. Migration adaptations: curried
      CBPV-explicit types, `ERROR` without messages, hof row
      annotations spelled on both sides (exact coercion, no
      subsumption inside U). Bugs this shook out: FnTypeExpr's
      `params()` accessor includes the return (same kind class — split
      by position in the elab translation; ret is now always
      `F(value)`, a returned fn is an implicit thunk), and `ctor`'s
      subject is the args structure, not the shared tag. Deferred
      with machinery, as D-39: resume, bounds, assoc_types,
      cap_inference, exhaustiveness; also mutual recursion (D-12
      groups), nested patterns, `if/else` and impl dispatch (elab
      gaps, M4 parity).

## M3 — e-graph optimization

Goal: the between rule groups optimize MIR under golden fixtures.
Execution model = decision 42 (hybrid saturate/extract/reduce loop;
host-side subst; `subst` as a `:cost 1000` constructor).

- [ ] Step 1 — decision 42 written; this breakdown.
- [ ] Step 2 — egglog 2.0 dependency in `langue-rt`;
      `langue-rt::optimize` generic helpers (load program, define root,
      run, extract to an owned term, union, loop skeleton with a
      caller-supplied subst-reduction callback); smoke test executes the
      real M1-compiled `between MIR` program for the first time; fix
      `codegen/between.rs` where egglog 2.0 rejects the format (`subst`
      becomes a `:cost 1000` constructor) and re-bless
      `tests/fixtures/egglog/MIR.egg`.
- [ ] Step 3 — `:optimize(L)` corpus attribute in `langc` (D-32):
      `ElabReport`-shaped driver fn, canonicalize-then-compare like
      `:elab`, `LANGC_UPDATE=1` blessing.
- [ ] Step 4 — new `between MIR` rules: `ParenC` transparency and
      handle/perform resolution (the local core of legacy LTO's
      capability monomorphization); `langc gen` + re-bless.
- [ ] Step 5 — handwritten MIR optimize driver
      (`lumo-syntax::optimize_driver`, mirrors `judge_driver`): parse →
      encode (D-42 optional-field rules) → loop → decode → reparse
      canonical; wire into the corpus test; smoke fixtures under
      `tests/fixtures/optimize/` (U-beta, F-beta, paren, no-op).
- [ ] Step 6 — full fixture suite: nested beta chains, duplicating
      subst (the D-31 caveat watchpoint), handle/perform cases modeled
      on legacy LTO fixture intent, unencodable-input ERROR; record
      deferred items (interprocedural LTO, binder-aware subst).

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
