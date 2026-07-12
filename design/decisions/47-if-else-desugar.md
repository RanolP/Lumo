# if/else desugars to a Bool match

Settled 2026-07-12. `if c { a } else { b }` elaborates to
`case unroll [c] { .true => [a] .false => [b] }` — exactly the match
the programmer could write, so the judge and the JS backend see
nothing new. `else if` chains fall out of `ElseClause`'s recursion.

- **`Bool` is not built in.** The `.true`/`.false` tags resolve
  against whatever `data Bool { .true, .false }` is in scope (the
  stdlib's, or a local one). No decl in scope ⟶ the usual unknown-tag
  judge bail. This keeps the elab type-agnostic — it never inspects
  the condition's type — at the cost of hard-coding the two tag
  *names* as the if/else protocol.
- **`if` without `else` is an elab error.** An expression-oriented
  else-less `if` needs a Unit value for the false branch, and MIR has
  no unit literal yet; defer rather than invent one here.
- Implementation: one extern rule (`if_else`) in the Lumo→MIR elab —
  the optional else and the `ElseClause` enum sit easier in an extern
  than in derived-rule patterns.
