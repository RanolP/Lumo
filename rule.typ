#let judge = (
  c: $attach(tack.r, tr: C)$,
  v: $attach(tack.r, tr: V)$,
)

#let check = text(fill: blue)[$arrow.l.double$]
#let synth = text(fill: red)[$arrow.r.double$]

#let rule(premises, conclusion, name) = {
  $
    #premises / #conclusion italic(#name)
  $
}

= Lumo

== Types

As we follows the Call-by-Push-Value, we have different type for value and computations.
We'll call the value type as $A$ and the computation type as $underline(B)$.

$
             A ::= & Sigma_(i in I) A_i         && quad italic("Sum") \
                 | & Pi_(i in I) A_i            && quad italic("Prod") \
                 | & j Pi_(i in I) A_i          && quad italic("Variant") \
                 | & "thunk" underline(B)       && quad italic("Thunk") \
                 | & mu X.A                     && quad italic("Recursive") \
                 | & X                          && quad italic("Variable") \
                   \
  underline(B) ::= & "produce" A" "epsilon      && quad italic("Produce") \
                 | & Pi_(i in I) underline(B)_i && quad italic("With") \
                 | & A -> underline(B)          && quad italic("Arrow") \
$

== Before the semantics

*Definition 1.* The context $Gamma$ is a key-value record containing a mapping from a variable name to a type.
In other words, `let Γ: HashMap<string, RefinedTypeV>` or ${ x_0 |-> A_0, dots, x_n |-> A_n }$.

The calculus has four judgements (two for value and computation, two for infer and check).

- $Gamma judge.v x synth A$ : The inferred type of value $x$ is $A$
- $Gamma judge.v x check A$ : The value $x$ must be type-checked against $A$
- $Gamma judge.c M synth underline(B)$ : The inferred type of computation $M$ is $underline(B)$
- $Gamma judge.c M check underline(B)$ : The computation $M$ must be type-checked against $underline(B)$

#pagebreak()

== Terms and Big-step semantics

#columns[
  #rule[
    $Gamma judge.v x check A$
  ][
    $Gamma judge.v (x : A) synth A$
  ][Annotate]

  #rule[
    $Gamma judge.v x check (mu X. A)[X |-> mu X.A]$
  ][
    $Gamma judge.v"roll" x check mu X.A$
  ][Roll]

  #rule[
  ][
    TODO
  ][Unroll]

  #rule[
    $Gamma judge.v v check A_(i^prime)$
  ][
    $Gamma judge.v "inj"_(i^prime) v check Sigma_(i in {i^prime, I}) (dots, i^prime |-> A_(i^prime), dots)$
  ][Injection]

  #rule[
  ][
    TODO
  ][Variable]

  #rule[
    $Gamma judge.c M check underline(B)$
  ][
    $Gamma judge.v "thunk" M check "thunk" underline(B)$
  ][Thunk]

  #rule[
  ][
    TODO
  ][$"TyAbs"_V$]

  #rule[
  ][
    TODO
  ][$"TyAbs"_C$]


  #rule[
    $dots.c quad Gamma judge.v x_i check A_i quad dots.c$
  ][
    $Gamma judge.v "record" { dots, i = x_i, dots } check Pi_(i in I) A_i$
  ][Record]

  #colbreak()

  #rule[
    $dots.c quad Gamma judge.v x_i check A_i quad dots.c$
  ][
    $Gamma judge.v j" "{ dots, i = x_i, dots} check j Pi_(i in I) A_i$
  ][Variant]


  #rule[
    $Gamma judge.v x check A$
  ][
    $Gamma judge.c "return" x check "produce" A$
  ][Return]

  #rule[
  ][
    TODO
  ][Force]

  #rule[
  ][
    TODO
  ][Apply]

  #rule[
    $Gamma, x : A judge.c M check underline(B)$
  ][
    $Gamma judge.c lambda x. M check A -> underline(B)$
  ][Lambda]

  #rule[
  ][
    TODO
  ][Sequence]


  #rule[
  ][
    TODO
  ][$"TyApp"_V$]

  #rule[
  ][
    TODO
  ][$"TyApp"_C$]

  #rule[
  ][
    TODO
  ][Projection]

  #rule[
  ][
    TODO
  ][Match]
]

== Bibliography

- Call by Push value
- Bidirectional Typing

== Appendix

=== Quick lesson on Inference Rules

Inference rules have following form.

#rule[
  Premises
][
  Conclusion
][RuleName]

Let's become familiar with inference rules by reading examples below:

#rule[
  $a = 1 quad b = 2$
][
  $a + b = 3$
][Add]

#rule[
  $f : A -> underline(B) quad x : A$
][
  $f(x) : underline(B)$
][FunApp#super[Function Application]]
