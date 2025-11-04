
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

#let judge = (
  c: $attach(tack.r, tr: C)$,
  v: $attach(tack.r, tr: V)$,
)

#let check = text(fill: blue)[$arrow.l.double$]
#let synth = text(fill: red)[$arrow.r.double$]
#let emits = $arrow.squiggly.r$

#let rule(premises, conclusion, name) = {
  $
    #premises / #conclusion [#smallcaps(name)]
  $
}

#let version = sys.inputs.at("version", default: "0.0.0-DRAFT")
#outline(title: [Formalization of Lumo Language v#version])

= Introduction

Lumo is a brand-new programming language built upon modern cutting-edge theoretical foundations.

Its goal is:

- To provide a sophisticated way to express complex side-effectul codes in a safe and efficient manner.
- To keep no surprises for newcomers who are familiar with procedural programming languages such as C, TypeScript, Python, etc.

For achieving the goals, Lumo has three pillars:

- First, Lumo utilizes the essence of _Algebraic Effects_ research done by *Effekt*. \
  which gives us an alternative approach to Monadic side effect containerization \
  while keeping the traditional looks of throw/catch or async/await syntaxes.
- Second, Lumo compiles to a _Linear Resource Calculus_($lambda^1$) prposed on Perceus, by the *Koka*. \
  which makes us to enable the FBIP(Functional But In-Place) paradigm, which introduce efficient \
  memory management while avoiding complex reasoning on lifetime or ownership.
- Lastly, Lumo adapts the _Mutable Value Semantics_ formalized by *Hylo*. \
  which enables the users to write idiomatic codes on mutating the parameters, \
  powered by the standard mutable state effect `ref<τ>`.

Our goal is to compile the Lumo source code into the C or JavaScript code.

= The Source Language

Next we'll define the source language terms and its translation with denotational semantics using inference rules.

#definition(title: "Elaboration")[
  $Gamma tack.r e arrow.squiggly.r e^prime$ means in the context $Gamma$, the expression $e$ elaborates to $e^prime$.
  For example, $Gamma tack.r 1 + 2 emits 3$ means that the syntax 1 + 2 should be compiled into 3.
]

#definition(title: "Inference Rule")[ The inference rule
  #rule[
    Premises
  ][
    Conclusion
  ][
    Rule-Name
  ]

  means that when the premises are true, the conclusion must be true by the rule named as Rule-Name.
]

This document describes a intermediate representation of the Lumo programming language, which is a functional programming language built upon modern Algebraic Effects and efficient memory management powered by Linear Resource Calculus($lambda^1$) by Perceus.

== Types

As we follows the Call-by-Push-Value, we have different type for value and computations.
We'll call the value type as $A$ and the computation type as $underline(B)$ (underlined).

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

== Before the Core Language

We have combined 3 notions: Bidirectional Typing, Call-by-Push-Value, and Linear Resource Calculus($lambda^1$) from Perceus. So we propose a new judgement rule merged the notions for both typing and elaboration.

*Definition 1.* The context $Gamma$ is a key-value record containing a mapping from a variable name to a type.
In other words, `let Γ: HashMap<string, RefinedTypeV>` or ${ x_0 |-> A_0, dots, x_n |-> A_n }$.

*Definition 2.* The heap-context $Delta$ is a key-value record containing a mapping from a variable name to a tuple consist of the reference count and the value. In other words, `let Δ: HashMap<string, (Nat, Value)>` or ${ x_0 -> (NN^+_0, v_0), dots, x_n -> (NN^+_n, v_n) }$.

The calculus has four judgements (two for value and computation, two for infer and check).

- $Delta bar Gamma judge.v x synth A emits x^prime$ : The inferred type of value $x$ is $A$, emits $x^prime$
- $Delta bar Gamma judge.v x check A emits x^prime$ : The value $x$ must be type-checked against $A$, emits $x^prime$
- $Delta bar Gamma judge.c M synth underline(B) emits M^prime$ : The inferred type of computation $M$ is $underline(B)$, emits $M^prime$
- $Delta bar Gamma judge.c M check underline(B) emits M^prime$ : The computation $M$ must be type-checked against $underline(B)$, emits $M^prime$

#pagebreak()

== Syntax-Directed Typing Rules and Elaborations

#columns[
  #rule[
    $Delta bar Gamma judge.v x check A$
  ][
    $Delta bar Gamma judge.v (x : A) synth& A \
    emits& x$
  ][Annotate]

  #rule[
    $Delta bar Gamma judge.v x check (mu X. A)[X |-> mu X.A]$
  ][
    $Delta bar Gamma judge.v"roll" x check& mu X.A \
    emits& "roll" x$
  ][Roll]

  #rule[
  ][
    TODO
  ][Unroll]

  #rule[
    $Delta bar Gamma judge.v v check A_(i^prime)$
  ][
    $Delta bar Gamma judge.v "inj"_(i^prime) v
    check& Sigma_(i in {i^prime, I}) (dots, i^prime |-> A_(i^prime), dots) \
    emits& "inj"_(i^prime) v$
  ][Injection]

  #rule[
  ][
    $Delta bar {x |-> A} judge.v x synth& A \ emits& x$
  ][Variable]

  #rule[
    $Delta bar Gamma judge.c M check underline(B)$
  ][
    $Delta bar Gamma judge.v "thunk" M check& "thunk" underline(B) \
    emits& "thunk" M$
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
    $dots.c quad Delta bar Gamma judge.v x_i check A_i quad dots.c$
  ][
    $Delta bar Gamma judge.v "record" { dots, i = x_i, dots } check& Pi_(i in I) A_i \
    emits& "record" { dots, i = x_i, dots}$
  ][Record]

  #colbreak()

  #rule[
    $dots.c quad Delta bar Gamma judge.v x_i check A_i quad dots.c$
  ][
    $Delta bar Gamma judge.v j" "{ dots, i = x_i, dots}
    check& j Pi_(i in I) A_i \
    emits& j" "{ dots, i = x_i, dots}$
  ][Variant]


  #rule[
    $Delta bar Gamma judge.v x check A$
  ][
    $Delta bar Gamma judge.c "return" x check& "produce" A \
    emits& "return" x$
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
    $Delta bar Gamma, x : A judge.c M check underline(B)$
  ][
    $Delta bar Gamma judge.c lambda x. M check& A -> underline(B) \
    emits& lambda x. M$
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
