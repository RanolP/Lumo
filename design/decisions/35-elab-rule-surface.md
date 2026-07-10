# Elab rule surface: the derived form

Settled 2026-07-11 (M1 kickoff). The locked examples (D-13, D-14) fix the
shape; the elided `...` parts are filled in as follows:

```
from Lumo to MIR {
  FnDecl { name: $n, param_list: ParamList { params: [$p*] }, body: $b }
    ==> Lambda { params: [$p* to MIR], body: $b to MIR }
}

between MIR {
  Apply { fn: Lambda { param: $b, body: $e }, arg: $a } === $e[$b := $a]
}
```

- **Pattern** = `Node { field: Pattern, … }` | `$var` | `'literal'` |
  `[$x*]` (list capture under a labeled sep/rep field). Field names are
  the syn labels; omitted fields match anything.
- **Construction** = `Node { field: Construction, … }` | `$var` |
  `$var to Lang` (recursive elaboration, strict-subtree checked, D-28) |
  `[$x* to Lang]` (elementwise recursion over a list capture) |
  `$e[$b := $a]` (built-in subst, D-24) | `'literal'`.
- Node names may qualify as `Lumo::FnDecl` when ambiguous (D-13).
- `from A to B` blocks with the same (from, to) pair merge across files
  (D-05); `between L` groups merge per language (D-14).
- Two rules whose patterns can fire on the same input are a compile-time
  error — no ordering, no priority (D-13).
