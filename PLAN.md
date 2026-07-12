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

- [x] Step 1 (2026-07-12) — decision 42 written; this breakdown.
- [x] Step 2 (2026-07-12) — egglog 2.0 dependency in `langue-rt`
      (default-features off, pure Rust); `langue-rt::optimize`:
      `Optimizer` (load program, define root, run, extract to an owned
      `EggTerm`, union, read a tactic table's pending calls) + the D-42
      loop. First real execution of the M1-compiled format; egglog 2.0
      rejects merge-less `function`s, so `codegen/between.rs` emits
      `subst` as a `:cost 1000` constructor (extraction steering);
      `tests/fixtures/egglog/MIR.egg` re-blessed.
- [x] Step 3 (2026-07-12) — `:optimize(L)` corpus attribute in `langc`
      (D-32): `ElabReport`-shaped driver fn, canonicalize-then-compare
      like `:elab`, `ERROR` expected matches any error,
      `LANGC_UPDATE=1` blessing.
- [x] Step 4 (2026-07-12) — new `between MIR` rules: `ParenC`
      transparency and handle/perform resolution (the local core of
      legacy LTO's capability monomorphization). The checker now allows
      nonlinear patterns in between relations (egglog equality
      constraints); from-rules still reject them.
- [x] Step 5 (2026-07-12) — handwritten MIR optimize driver
      (`lumo-syntax::optimize_driver`, mirrors `judge_driver`): encode
      rides the generated `judgments::term_of` (Mk structs,
      vec-of/vec-empty, D-42 optional-field rules, bare-paren
      transparency); host-side naive subst; handwritten MIR printer +
      reparse for canonical output; wired into the corpus test; smoke
      fixtures (U-beta, F-beta, paren, no-op).
- [x] Step 6 (2026-07-12) — full fixture suite
      (`tests/fixtures/optimize/mir.test`, 14 cases): nested beta
      chains, duplicating subst both ways (the D-31 caveat stands —
      egglog 2.0 extract is tree-cost — and lands as a free size-based
      inlining heuristic: big values keep the let), handle/perform
      cases on legacy `01_trivial_leaf`/`03_fixed_point_chain` intent
      (incl. through force-thunk chains, plus the negative case),
      multi-def files, unencodable-input `ERROR`.

Deferred from M3: everything interprocedural from legacy LTO
(cross-def resolution maps, inline/clone heuristics, DCE, `resume`
stripping — needs def-level context beyond a single-term e-graph;
revisit at/after M4); binder-aware subst (shadow-stopping needs binder
markers or an alpha-uniqueness invariant from elab, D-42); a generated
optimize driver (the encode/decode walk is handwritten like the M2
judge driver); DAG-aware extraction (only if genuine sharing gets
mis-ranked, D-31/D-42).

## M4 — JS emission

Goal: end-to-end Lumo → JS compilation. Scope and the CBPV→JS mapping =
decision 43 (direct style, no CPS; runtime as free identifiers;
"legacy golden parity" corrected: legacy has no full-text JS goldens —
parity means porting emission *behaviors* as new-format fixtures).

- [x] Step 1 (2026-07-12) — decision 43 written; this breakdown.
- [x] Step 2 (2026-07-12) — `JS.syn.langue`: const decls; praat
      expression grammar where `(…)` is the n-ary ParenExpr (doubles
      as an arrow parameter list) and `=>` is a right-associative
      infix operator — no JS cover grammar in LL(1); call/member/index
      postfix rows, `===`, ternary as a postfix row with a then/alt
      payload, object/array literals, named function expressions.
      12 `:parse(JS)` fixtures. Manifest pipe grew `| elab MIR to JS`.
- [x] Step 3 (2026-07-12) — `elab MIR to JS`: derived rules for leaves
      and transparents (ret/roll/unroll/parens/annotations erase),
      `sel` as a member construction; six extern rules host singleton
      param lists, arg/arm folds, and ident→"string" quoting
      (thunk_lambda, force_apply, let_fix, case_arms, ctor_bundle,
      caps). Grammar fix from blessing: object props are a sep list —
      optional commas canonicalize away, which real JS rejects. The
      generated JS builder cannot auto-parenthesize (the n-ary
      ParenExpr is not a single-required-field paren atom), so externs
      pre-wrap risky operands. 10 `:elab(MIR -> JS)` fixtures; outputs
      run under node.
- [x] Step 4 (2026-07-12) — end-to-end `:elab(Lumo -> JS)` through
      `compile_driver` (chains the pipe's elab stages) in the corpus
      lookup; 8 legacy-behavior fixtures (identity gate, curry spines,
      auto-let calls, data+match, cap ops through handle/bundle,
      use→require, fix recursion, blocks); functional check under node
      with the minimal D-43 runtime.
- [x] Step 5 (2026-07-12) — **M4 COMPLETE**; deferred below.

Deferred from M4: `resume`/CPS (with D-39's resume); readability
post-passes (IIFE flattening, const collapsing, arity-based
uncurrying — the contract is correctness, not prettiness, D-43);
exports/`main` entry wrapper; TypeScript type emission; shipping the
runtime prelude (`__lumo_perform`/`__lumo_handle`/`__lumo_match_error`
/`require` stay host-provided free identifiers); the M2 elab-gap
backlog (if/else, impl dispatch, nested patterns, mutual recursion)
which needs Lumo→MIR work before it reaches JS; `:optimize` between
the pipe's elab stages (compile_driver goes straight through today).

## Post-M4 — feature backlog toward the stdlib port

The stdlib port (legacy `packages/`) is blocked on `impl` blocks — every
Tier-2/3 backend file opens with one. Rule: land the blocking feature
first, port after.

- [x] Impl slice 1 (2026-07-12) — bare cap impls, decision 44.
      `impl Cap { fn op(…) = e; … }` ⟶
      `def __impl_Cap = (bundle {…} : Cap)`; the ParenV annotation
      rides the existing bundle-vs-cap judgment, MIR→JS is untouched.
      `Cap.op(args)` with a bare impl in scope resolves statically to
      `sel __impl_Cap . op` — no perform, empty-row callers typecheck
      (legacy `resolve_default_cap_impls` parity). `handle` does not
      override static resolution. Non-bare forms and assoc-type
      bindings raise a loud elab error. Fixtures: 3 `:elab(Lumo ->
      MIR)`, 4 `:infer(Lumo)` (`type/impl.test`), 1 end-to-end
      `:elab(Lumo -> JS)` run under node.

Deferred impl forms (subsequent slices): `impl T: Cap` / inherent /
named / generic (need type-directed dispatch), value method dispatch
(`x.method()`), operators, `resume`, if/else, multi-file. Stdlib
porting resumes once its blocking features land.

- [x] Stdlib port slice 1 (2026-07-12) — decision 45. The JS-target
      subset of legacy `packages/` lives under root `packages/` as ONE
      compilation unit (`packages/stdlib.manifest` order; `use` and
      `lumo.toml` dropped — no build system yet): libcore
      prelude/Ordering/NumOps/StrOps + bare impls, libstd
      IO/FS/Process/List + bare impls, hello main. Host bindings live
      in `packages/runtime/js/prelude.js` (extern-mapping attributes
      are a deferred backend feature); `resume` dropped per D-44;
      operator uses rewrote to NumOps calls. Gate:
      `crates/lumo-syntax/tests/stdlib.rs` (parse + infer + compile);
      smoke: `scripts/stdlib_smoke.sh` runs the unit under node
      (strings, numbers, list fns, FS round-trip, process args — all
      print correctly). Port-driven backend fixes: `_` case binders
      uniquify at JS emission; empty application collapses over
      `force` (judge-consistent; D-30 hosts now export plain values).
      Known judge gap hit: nested matches bail on the D-28 `arm_bind`
      guard (NumOps.cmp uses a helper fn; backlog below).

Deferred from the stdlib port (D-45): Self-typed operator caps
(`Add`…`Not`, PartialEq/PartialOrd) + all typeclass impls; inherent
impls / UFCS (`impl String`, `List.map`); multi-file `use`
resolution and an lbs successor; `src#rs` backends. (The legacy
langue package is NOT a port target — langc is the toolchain.)

- [x] Nested matches (2026-07-12) — decision 46: the D-28 runtime
      guard is now amortized — a non-descending same-judgment
      re-entry is allowed when a strict same-judgment descent is
      active strictly between the frames (nested matches re-enter
      `arm_bind`/`binds` through `check_C`'s descent); a 100k-frame
      depth cap backstops rule sets that construct growing terms.
      Engine-only change in `langue-rt` (+2 unit tests); two
      `:infer` fixtures (binder-less and binder'd nesting);
      `NumOps.cmp` back to the legacy nested shape.

- [x] if/else (2026-07-12) — decision 47: desugars in one extern elab
      rule to `case unroll c { .true => a .false => b }`; `Bool` is
      whatever `data Bool` is in scope, not a builtin; else-if chains
      recurse through ElseClause; else-less ifs are an elab error (no
      Unit value in MIR yet). Fixtures: 2 `:elab(Lumo -> MIR)`, 2
      `:infer` (agreement + branch-mismatch ERROR), 1 `:elab(Lumo ->
      JS)` run under node. Accessor caveat: the generated
      `else_clause()` matches the then-block first (M0 offset scheme)
      — the extern takes the second ElseClause-shaped child.

- [x] Typeclass impls + operators (2026-07-12) — decision 48.
      `impl T: Cap` (ground targets) elaborates to
      `def __impl_{Cap}_{T} = (bundle {…} : {Cap}_{T})`; the judge
      driver seeds `{Cap}_{T}` as an ordinary ground instance cap with
      `Self := T` in every op type — zero judge changes (Σ keys by
      bare cap name). Operators desugar in elab: arith + `==` to
      instance-cap selections dispatched on syntactic operand types
      (literals, annotated params, ctor owners, fn/cap-op returns,
      parens, operator recursion — unresolvable operands are a loud
      elab error); `!=`/`!`/`&&`/`||` and comparisons desugar
      structurally over the Bool/Ordering tag protocol (comparisons
      via `PartialOrd.cmp`); `**` errors (no legacy cap). Stdlib port
      grew: ops.lumo (7 operator caps), PartialEq/PartialOrd,
      number_impls/string_impls (typeclass halves of legacy
      number/string.lumo — split files because the judge needs
      `__impl_NumOps` before impls that delegate to it), `impl Bool:
      Not`; hello main exercises `+ * % / -` (unary too), `< <= &&`,
      string `+`/`==` under node. Deferred: generic targets
      (`impl List[A]: …`), `Self`-typed direct cap calls,
      let-annotation scope, operators inside impl-method bodies;
      instance-cap mangling is replaced by structured cap type-args
      when generic impls land.

- [x] Inherent impls + UFCS (2026-07-12) — decision 49. Same impl
      syntax, disambiguated by the head name: `impl String {…}` where
      `String` is not a cap is inherent. No cap decl exists, so the
      judge driver derives the `{T}_impl` instance cap from the impl's
      own method signatures (self ⟶ T, annotated params/returns,
      untyped/generic methods stay out of Σ and bail loudly at the
      bundle check); elab emits `def __impl_{T} = (bundle {…} :
      {T}_impl)`. `x.m(args)` with `syn_type(x) = T` dispatches to
      `sel __impl_T . m (x, args…)`; method returns join the D-48
      syntactic table so chains resolve
      (`("hi" + "!").len().to_string()`). Unknown-type objects fall
      through to the old plain-sel path. Stdlib: `impl String {…}` +
      `impl Number { to_string, to_char }` ported into
      string_impls.lumo; smoke exercises chained UFCS under node.
      Deferred: generic impls (`List.map` stays unported), typeclass
      methods via dot, self-less static methods.

- [x] Generic inherent impls (2026-07-12) — decision 50. Every `Σ.Op`
      value is now `Sig(binders, type)` (ground = `Sig([], t)`): the
      impl's bundle checks with binders held rigid (skolems — a `map`
      body that fixes `R` errors), and every `sel` use site
      instantiates them fresh via the existing `inst` (the data-ctor
      pattern). Two judgment-rule edits (`clauses_check`,
      `infer_C SelC`); `impl[T] List[T] { fn map[R] … }` seeds
      `List_impl` from its own signatures with impl+method binders.
      Two elab gaps closed: match-arm binders join the syntactic
      scope with field heads from the (unambiguous) variant tag, and
      the dispatch table stores type-constructor heads
      (`xs: List[Number]` → `List`), so `t.map(mapper)` inside map's
      own body dispatches; Γ pre-seeds every impl def since a method
      body may reference its own def. `List.map` ported — the legacy
      stdlib JS-target port is now COMPLETE; smoke runs a mapped list
      under node. Deferred: generic typeclass impls and structured
      cap type-args (need Σ keys that unify on args without
      rigidifying skolems — the `{Cap}_{T}` naming survives until
      then), bounded binders.

- [x] Capability passing (2026-07-12) — decision 51, per direction:
      Effekt's capability-passing style, tail-resumptive fragment, NO
      continuations. Rows become leading capability params
      (`fn hi(x): S / {Console}` ⟶ `fn(__cap_Console) => fn(x) => …`,
      typed `U((Console) -> (S) -> …)` — rows leave types); `handle E
      with h in b` is a lexical binding `let __tN = ret (h : E) in b`
      (supersedes D-44's non-override: innermost lexical provider
      wins — handle > row param > default impl > loud error); calls
      to row'd fns thread capability values as leading args; lambdas
      capture capabilities by closure, so `..c` rests are vestigial.
      The elab never emits PerformC/HandleC (both stay for MIR-level
      programs and the `perform` escape hatch); zero judge changes;
      `__lumo_perform`/`__lumo_handle` removed from the stdlib
      runtime prelude. Fixtures rewritten (perform/handle/row cases)
      + new threading/override cases; smoke's 12th line runs a
      lexical handle whose handler delegates to the default impl
      under node. Sharp edges documented in D-51: first-class use of
      row'd fns is not re-arranged; row'd fns want full signatures;
      non-tail-resumptive handlers (exceptions/generators) deferred
      to a future delimited-control slice.

## Cross-cutting rules

- The definition is the source of truth; Rust is engines + generated
  output, never the definition (D-01).
- All three kinds are code-generated — edit, `langc gen`, commit; no
  interpreted rule tables (D-21).
- Documents stay snapshots; new decisions get a new numbered file in
  `design/decisions/` (D-10).
- stdlib and built-in tactics start minimal and grow only on proven need
  (D-24, D-29).
