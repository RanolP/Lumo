// Global Settings
#show smallcaps: set text(font: "New Computer Modern")
#set heading(numbering: "1.")
#set par(first-line-indent: 1em)

#let definition-counter = counter("definition")
#let definition(title: "", content) = {
  definition-counter.step()
  [
    *Definition #context definition-counter.display("1.")* #if title != "" { [(_#(title)_)] } #content
  ]
}

#let judge = (c: $attach(tack.r, tr: C)$, v: $attach(tack.r, tr: V)$)

#let check = text(fill: blue)[$arrow.l.double$]
#let synth = text(fill: red)[$arrow.r.double$]
#let emits = $arrow.squiggly.r$
#let bv = $"bv"$
#let kw = s => box(text(
  font: "New Computer Modern",
  weight: "bold",
  fill: color.rgb("#7B1FA2"),
)[#s])
#let kwThunk = kw("thunk")
#let kwLet = kw("let")
#let kwLambda = kw("lambda")
#let kwForce = kw("force")
#let kwMatch = kw("match")
#let kwMut = kw("mut")
#let kwFn = kw("fn")
#let kwPerform = kw("perform")
#let kwHandle = kw("handle")
#let kwEffect = kw("effect")
#let kwForall = kw("forall")
#let kwProduce = kw("produce")
#let kwData = kw("data")
#let kwBundle = kw("bundle")
#let kwOf = kw("of")
#let kwRoll = kw("roll")
#let kwUnroll = kw("unroll")
#let LetSlots = "LetSlots"
#let MutSlots = "MutSlots"
#let EffectTag = "EffectTag"
#let binds = "binds"
#let assigncount = "assign-count"

#let rule(premises, conclusion, name) = {
  $
    #premises / #conclusion [#smallcaps(name)]
  $
}

#let reduce-rule(from, to, name) = {
  $
    #grid(columns: 2, align: center + horizon, column-gutter: 4pt)[#from \
      #text(fill: color.rgb("#008404"))[$-->$] #to ][#box(inset: (bottom: 4pt))[[#smallcaps(name)]]]
  $
}

#let elaboration-rule(from, to, name) = {
  $
    #grid(
      columns: 3,
      rows: 2,
      align: center + horizon,
      row-gutter: 16pt,
      column-gutter: 4pt,
      grid.cell(colspan: 2)[
        #from
      ],
      grid.cell(rowspan: 2)[#box(inset: (bottom: 4pt))[[#smallcaps(name)]]],
    )[
      #box(inset: (right: 4pt))[#text(fill: color.rgb("#1565C0"), size: 1.5em)[$~>$]]
    ][#to]
  $
}

#let version = sys.inputs.at("version", default: "0.1.0-DRAFT")
#outline(
  title: [Formalization of Lumo IR v#version (#datetime.today().display("[year]-[month]-[day]"))],
)

= Introduction

In this document, we write a Lumo IR, built upon:

- _Algebraic Effects_ research done by *Effekt* @brachthaeuser2020effektjfp.
- _Linear Resource Calculus_($lambda^1$) proposed by Perceus @reinking2021perceus.
- _Mutable Value Semantics_ formalized by *Hylo/Val* @racordon2022implementation @racordon2021native.

= Reading Typing Rules

We'll use several typing judgments with the following contexts:

- $Delta$: Variable and its Assigned Types, Borrowed. Must own before use.
- $Gamma$: Variable and its Assigned Types, Owned. Use values exactly once.
- $E$: Named Capabilities available in this context, implemented in a Bundle.

Judgment forms:

- $Delta | Gamma; E judge.v x synth A$ : The inferred type of value $x$ is $A$
- $Delta | Gamma; E judge.v x check A$ : The value $x$ must be type-checked against $A$
- $Delta | Gamma; E judge.c M synth underline(B)$ : The inferred type of computation $M$ is $underline(B)$
- $Delta | Gamma; E judge.c M check underline(B)$ : The computation $M$ must be type-checked against $underline(B)$

Pattern binder notation:

- $bv(p)$: the set (or ordered list) of variables bound by pattern $p$.
- $bv("_") = emptyset$.
- $LetSlots(p)$: variables introduced as #kwLet slots by pattern $p$.
- $MutSlots(p)$: variables introduced as #kwMut slots by pattern $p$.

Pattern syntax:

$
  p ::= "_"
  | i(p_0, dots.c, p_n)
  | #kwLet x
  | #kwMut x
$

= Tags

Tags are second-class names, used for places such as variant constructors and function names inside bundles. They cannot be used alone.
For effects, we assume a tag-extraction operation $EffectTag(e)$ for every effect $e$ (including instantiated effects after generic monomorphization).

= Types

Lumo IR uses System F with several extensions: higher-order types ($omega$), recursive types ($mu$), and a Call-by-Push-Value style separation between values ($A$) and computations ($underline(B)$).

$
             K ::= & "*" | K -> K \
             A ::= & X
                     | i #kwOf [A_0, A_1, dots.c, A_n]
                     | A + A
                     | #kwThunk underline(B)
                     | #kwMut A \
                 | & #kwForall "[" X : K "]" "." A
                     | A "[" A "]"
                     | mu X "." A \
  underline(B) ::= & #kwProduce A
                     | A -> underline(B)
                     | Pi_((i in I)) (i times underline(B_i))
$

== Type Formation

We separate value types (`"Type"`) from computation types (`"CType"`).

$
  (A : "Type") / (#kwProduce A : "CType")
$

$
  (A : "Type" quad underline(B) : "CType")
  / ((A -> underline(B)) : "CType")
$

$
  (forall i in I. (underline(B_i) : "CType"))
  / (Pi_((i in I))(i times underline(B_i)) : "CType")
$

#kwBundle values are named in $E$ and treated as computation-level entities.

= On Bidirectional Typing

We use Pfenning Recipe for our type system with following rules included.

== Variable Synthesis

Variables synthesize their type directly from the owned context.

#rule[
  $x : A in Gamma$
][
  $Delta | Gamma; E
  judge.v
  x synth A$
][Var-Synth]

== Annotation Synthesis

A type annotation upgrades checking into synthesis.

#rule[
  $Delta | Gamma; E
  judge.v
  v check A$
][
  $Delta | Gamma; E
  judge.v
  (v : A) synth A$
][Ann-Synth]

== Value Synth-to-Check

Any synthesized value can be consumed in checking mode.

#rule[
  $Delta | Gamma; E
  judge.v
  v synth A$
][
  $Delta | Gamma; E
  judge.v
  v check A$
][V-Synth-Check]

== Computation Synth-to-Check

Any synthesized computation can be consumed in checking mode.

#rule[
  $Delta | Gamma; E
  judge.c
  M synth underline(B)$
][
  $Delta | Gamma; E
  judge.c
  M check underline(B)$
][C-Synth-Check]

== Forall Elimination

Type application instantiates a polymorphic value and synthesizes the instantiated type.

#rule[
  $Delta | Gamma; E
  judge.v
  u synth kwForall[X: K]. A \
  T : K$
][
  $Delta | Gamma; E
  judge.v
  u[T] synth A[X := T]$
][Forall-Elim-Synth]


= Data

Lumo IR uses a single concept for constructing values, called #kwData, which can cover the following real-world practices:

- `enum`: finite #kwData variants
- `struct`: single #kwData variant
- `primitives`: modeled as infinite-variant #kwData. \
  for example: $#kwData "nat" ::= { x | x in NN }$

== Definition of Data

$
  #kwData T "=" Sigma_((i in I)) i #kwOf [A_0, A_1, dots.c, A_n]
$

The #box(fill: color.rgb("d9d9d9"), outset: (top: 4pt, bottom: 4pt), inset: (left: 4pt, right: 4pt))[$#kwOf [dots]$] part can be omitted; if omitted, the tag is treated as nullary.

Examples:

$
  #kwData "Nat" "=" "Zero" + "Succ" #kwOf ["Nat"]
$

$
  #kwData "Bool" "=" "False" + "True"
$

$
  #kwData "NatList" "=" "Nil" + "Cons" #kwOf ["Nat", "NatList"]
$

== Rules of Data

=== Introduction

Before constructing `data`, we must "own" the parameters — each $p_i$ must be available in $Gamma_i$ — and consume contexts sequentially so the constructed data owns its values.

#rule[
  $Delta | Gamma_(i+1), dots.c, Gamma_n ; E | Gamma_i
  judge.v
  p_i check A_i
  quad
  (0 <= i <= n)$
][
  $Delta | Gamma_0, Gamma_1, dots.c, Gamma_n; E
  judge.v
  i(p_0, p_1, dots.c, p_n) check i #kwOf [A_0, A_1, dots.c, A_n]$
][Data-Intro]

We can get `data T` by `roll`-ing the expression above with type assertion `(e : T)`.

=== Elimination

Next we eliminate unrolled data (a sum of variants).
#kwMatch requires an owned variable `x`, and each branch is checked under freshly owned bindings introduced by the match pattern.
For the bindings, we can use #kwLet $x$ and for mut data, #kwMut $x$.

#rule[
  $forall i in I. MutSlots(p_i) = emptyset \
  Delta | Gamma, bv(p_i); E
  judge.c
  e_i check underline(B)$
][
  $Delta | Gamma, x; E
  judge.c
  #kwMatch x { p_i |-> e_i } check underline(B)$
][Data-Elim]

#rule[
  $x : #kwMut A in Gamma \
  Delta | Gamma, bv(p_i); E
  judge.c
  e_i check underline(B)$
][
  $Delta | Gamma, x; E
  judge.c
  #kwMatch x { p_i |-> e_i } check underline(B)$
][Data-Elim-Mut]

=== $beta$-reduction and $eta$-expansion

When a pattern matches, only bound variables are substituted.
Wildcard `_` drops the matched value and contributes no binding.

$
  binds("_", v) = [] \
  binds(#kwLet x, v) = [x := v] \
  binds(#kwMut x, v) = [x := v] \
  binds(i(p_0, dots.c, p_n), i(v_0, dots.c, v_n))
  = binds(p_0, v_0), dots.c, binds(p_n, v_n)
$

#reduce-rule[
  $#kwMatch i(v_0, v_1, dots.c, v_n) { i(p_0, p_1, dots.c, p_n) |-> e_i }_(i in I)$
][
  $e_i[binds(i(p_0, p_1, dots.c, p_n), i(v_0, v_1, dots.c, v_n))]$
][Data-Beta]

Reconstructing the same data through #kwMatch yields the original value.

#reduce-rule[
  $#kwMatch v { i(#kwLet x_0, #kwLet x_1, dots.c, #kwLet x_n) |-> i(x_0, x_1, dots.c, x_n) }_(i in I)$
][
  $v$
][Data-Eta]

= Recursive Types

Recursive types are introduced by #kwRoll and eliminated by #kwUnroll. It is crucial, to represent recursive #kwData types.

== Introduction / Elimination

#rule[
  $Delta | Gamma; E
  judge.v
  v check A[X := mu X "." A]$
][
  $Delta | Gamma; E
  judge.v
  #kwRoll v check mu X "." A$
][Mu-Intro]

#rule[
  $Delta | Gamma; E
  judge.v
  v synth "mu" X "." A$
][
  $Delta | Gamma; E
  judge.v
  #kwUnroll v synth A[X := "mu" X "." A]$
][Mu-Elim]

== $beta$-reduction

#reduce-rule[
  $#kwUnroll (#kwRoll v)$
][
  $v$
][Mu-Beta]

= Function

Functions are computations transforming input data into output data while having capabilities to utilize.

== User-level Syntax

The user only sees the surface-level syntax:

$
  #kwFn f[X_0: K_0, dots.c, X_n: K_n](p_0: A_0, dots.c, p_m: A_m): underline(B) "/" rho := M
$

where
$
  p ::= "_"
  | i(p_0, dots.c, p_n)
  | #kwLet x
  | #kwMut x
$

Also, we have the sibling call syntax:

$
  "Call" ::= u(overline(e))
$

$
  u ::= f | f[overline(A)]
$

Here each argument position in $overline(e)$ may be filled by either an ordinary expression argument or an explicit slot argument ($#kwLet x$ or $#kwMut x$). We keep the metavariable $e$ for both forms in the call rules below.

TODO: Slot semantics and metatheory for let/mut parameters are deferred.

=== Elaboration of User-level Syntax

The user-level syntax mixes several concerns, so we elaborate it step by step.

==== Let Elaboration

In fact, named #kwFn is just an alias for #kwLet. We can define #kwFn without giving a name.

#elaboration-rule[
  $&#kwFn f[X_0: K_0, dots.c, X_n: K_n](\
    &quad p_0 : A_0, dots.c, p_m : A_m\
    &): underline(B) "/" rho := M$
][
  $&kwLet f = \
  & quad kwProduce ( \
    & quad quad kwFn[X_0: K_0, dots.c, X_n: K_n](\
      &quad quad quad p_0 : A_0, dots.c, p_m : A_m\
      &quad quad ): underline(B) "/" rho := M) \
  & quad ) "in" f$
][Fn-Let-Elab]

==== Type Parameters Elaboration

Next we'll elaborate type parameters using #kwForall. Sequentially transform $X_i : K_i$ into $kwForall[X_i: K_i]$ syntax.

#elaboration-rule[
  $
    & kwFn[X_0: K_0, dots.c, X_n: K_n]( \
    & quad p_0 : A_0, dots.c, p_m : A_m \
    & ): underline(B) := M
  $
][
  $& kwForall[X_0 : K_0]. \
  & dots.v \
  & kwForall[X_n : K_n]. \
  & quad kwFn(
    \
    & quad quad p_0 : A_0, dots.c, p_m : A_m\
    & quad
  ): underline(B) := M$
][Fn-Type-Param-Elab]

==== Spine Elaboration

Now let's elaborate the rest -- spine of function -- at once.

#elaboration-rule[
  $kwFn ( p_0 : A_0, dots.c, p_m : A_m): underline(B) := M$
][
  $& kwThunk (\
    & quad kwLambda (p_0 : A_0). \
    &quad dots.v \
    &quad kwLambda (p_m : A_m). \
    &quad quad (M : underline(B)) \
    &)$
][Fn-Elab-Mono]

==== Call Elaboration

#elaboration-rule[
  $u(overline(e))$
][
  $(#kwForce u)(e_0) dots.c (e_n)$
][Call-Elab]

Note that you can pass $#kwLet x$ or $#kwMut x$ slots.

= CBPV Instructions

== #kwBundle (Computation-Level $Pi$)

$Pi_((i in I))(i times underline(B_i))$ is a finite computation-level record keyed by tags.

=== Bundle Introduction

Check each field and assemble them into one bundle.

#rule[
  $Delta | Gamma_(i+1), dots.c, Gamma_n ; E | Gamma_i
  judge.c
  M_i check underline(B_i)
  quad
  (0 <= i <= n)$
][
  $Delta | Gamma_0, Gamma_1, dots.c, Gamma_n; E
  judge.c
  kwBundle { i_0 |-> M_0, i_1 |-> M_1, dots.c, i_n |-> M_n }
  check Pi_((i in I))(i times underline(B_i))$
][Pi-Intro]

=== Bundle Elimination

Projecting a field from a synthesized bundle yields that field type.

#rule[
  $Delta | Gamma; E
  judge.c
  M synth Pi_((i in I))(i times underline(B_i)) \
  j in I$
][
  $Delta | Gamma; E
  judge.c
  M.j synth underline(B_j)$
][Pi-Elim]

=== Beta Reduction

Projecting from a literal bundle returns the corresponding field.

#reduce-rule[
  $(kwBundle { i_0 |-> M_0, i_1 |-> M_1, dots.c, i_n |-> M_n }).j$
][
  $M_j$
][Pi-Beta]

== Lambda

The function type $A -> underline(B)$ is made out of #kwLambda.

=== Introduction

#rule[
  $Delta | Gamma, x : A; E
  judge.c
  M check underline(B)$
][
  $Delta | Gamma; E
  judge.c
  #kwLambda x. M check A -> underline(B)$
][Lambda-Intro]

Also you can use slot-bindings here:

TODO: Formal proof obligations for slot-bindings (#kwLet/#kwMut parameters) will be specified later.

#rule[
  $Delta | Gamma, x : A; E
  judge.c
  M check underline(B) \
  assigncount(M, x) >= 1$
][
  $Delta | Gamma; E
  judge.c
  #kwLambda (#kwLet x). M check A -> underline(B)$
][Lambda-Let-Intro]

#rule[
  $Delta | Gamma, x : A; E
  judge.c
  M check underline(B)$
][
  $Delta | Gamma; E
  judge.c
  #kwLambda (#kwMut x). M check A -> underline(B)$
][Lambda-Mut-Intro]

=== Elimination

Application consumes the argument and runs the function computation.

#rule[
  $Delta | Gamma_1; E
  judge.c
  f synth A -> underline(B) \
  Delta | Gamma_2; E
  judge.v
  v check A$
][
  $Delta | Gamma_1, Gamma_2; E
  judge.c
  f(v) synth underline(B)$
][Lambda-Elim]

=== $beta$-reduction and $eta$-expansion

#reduce-rule[
  $(#kwLambda x. M)(v)$
][
  $M[x := v]$
][Lambda-Beta]

#reduce-rule[
  $#kwLambda x. f(x)$
][
  $f$
][Lambda-Eta]

== Thunk and Force

#kwThunk packages a computation as a value, and #kwForce restores the computation.

=== Type Formation

$
  #kwThunk underline(B) : "Type"
$

=== Typing Rules

#rule[
  $Delta | Gamma; E
  judge.c
  M check underline(B)$
][
  $Delta | Gamma; E
  judge.v
  #kwThunk M check #kwThunk underline(B)$
][Thunk-Intro]

#rule[
  $Delta | Gamma; E
  judge.v
  v synth #kwThunk underline(B)$
][
  $Delta | Gamma; E
  judge.c
  #kwForce v synth underline(B)$
][Force-Elim]

=== $beta$-reduction and $eta$-expansion

#reduce-rule[
  $#kwForce (#kwThunk M)$
][
  $M$
][Thunk-Beta]

#reduce-rule[
  $#kwThunk (#kwForce v)$
][
  $v$
][Thunk-Eta]

== Produce and Let

#kwProduce injects a value into the computation type $#kwProduce A$.

#rule[
  $Delta | Gamma; E
  judge.v
  v check A$
][
  $Delta | Gamma; E
  judge.c
  #kwProduce v check #kwProduce A$
][Produce-Intro]

#rule[
  $Delta | Gamma_1; E
  judge.c
  M synth #kwProduce A \
  Delta | Gamma_2, x : A; E
  judge.c
  N synth underline(B)$
][
  $Delta | Gamma_1, Gamma_2; E
  judge.c
  #kwLet x = M "in" N synth underline(B)$
][Produce-Elim]

=== $beta$-reduction

#reduce-rule[
  $#kwLet x = #kwProduce v "in" N$
][
  $N[x := v]$
][Produce-Beta]

= Algebraic Effects

Algebraic effects are expressed with $kwPerform_"op"$ and interpreted by $kwHandle_"op"$.

== User-level Syntax

Effects are computations passed naturally.

=== Effect Definition

You group compuations by the namespace, and can utilize same syntax as $kwFn$.

$
  & #kwEffect "Test" { \
  & quad #kwFn "argless"(): "Ret1" \
  & quad #kwFn "generic"[A: *](): "Ret1" \
  & }
$

So the effect declaration becomes a bundle of elaborated operation types:

#elaboration-rule[
  $&#kwEffect e { \
    &quad #kwFn "op"_0[X_(0,0): K_(0,0), dots.c, X_(0,n_0): K_(0,n_0)](p_(0,0) : A_(0,0), dots.c, p_(0,m_0) : A_(0,m_0)): B_0 \
    &quad dots.v \
    &quad #kwFn "op"_n[X_(n,0): K_(n,0), dots.c, X_(n,n_n): K_(n,n_n)](p_(n,0) : A_(n,0), dots.c, p_(n,m_n) : A_(n,m_n)): B_n \
    &}$
][
  $&e : Pi_((i in I))( \
    &quad "op"_i times ( \
      &quad quad kwForall[X_(i,0): K_(i,0)]. \
      &quad quad dots.v\
      &quad quad kwForall[X_(i,n_i): K_(i,n_i)]. ( \
        &quad quad quad A_(i,0) -> \
        &quad quad quad dots -> \
        &quad quad quad A_(i,m_i) -> \
        &quad quad quad quad underline(B_i) \
        & quad )) \
    &)$
][Effect-Def-Elab]

== Effect Operation Invocation

When we have the operation $op$ in our effect handler context, we can invoke $kwPerform_"op"$.

#rule[
  $op : underline(B) in E$
][
  $Delta | Gamma; E
  judge.c
  kwPerform_"op" synth underline(B)$
][Perform]

== Handlers

Of course you can pass the handler so the context takes care of it.

#rule[
  $Delta | Gamma_1; E
  judge.c
  M check underline(B) \
  Delta | Gamma_2; E, (op: underline(B))
  judge.c
  N check underline(C)$
][
  $Delta | Gamma_1, Gamma_2; E
  judge.c
  kwHandle_"op" "with" M "in" N check underline(C)$
][Handler-Intro]

== Operational Equations

#reduce-rule[
  $kwHandle_"op" "with" M "in" #kwProduce v$
][
  $#kwProduce v$
][Handle-Unit]

#reduce-rule[
  $kwHandle_"op" "with" M "in" kwPerform_"op"$
][
  $M$
][Handle-Perform-Beta]



= Meta-Theory Roadmap

Priority lemmas.

== Substitution

Goal: prove substitution while preserving linear/accounting side conditions.

Expected effect: enables modular proofs by replacing variables with well-typed terms safely.

== Preservation

Goal: prove that one-step reduction preserves typing and resource/effect invariants.

Expected effect: guarantees type/resource invariants are maintained by every reduction step.

== Progress

Goal: prove that well-typed closed programs are either terminal forms or can reduce.

Expected effect: rules out stuck well-typed closed programs (except designated terminal forms).

== NbE Soundness

Goal: prove soundness of NbE-based definitional equality.

Expected effect: provides a trustworthy basis for definitional equality checks.

== NbE (Normalization by Evaluation)

TODO: NbE formalization is deferred.

Expected benefits:

- Canonical decision procedure for definitional equality.
- Cleaner convertibility checks in type checking.
- Better foundation for future optimizations (normalization-driven simplification).

This document is the base spec for machine-checked proofs.

= References

#bibliography(
  "references.bib",
  style: "chicago-author-date",
  title: [Bibliography],
)
