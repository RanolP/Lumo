# Praat postfix rows may carry node payloads

Resolves the call-expr question flagged in the M0 plan (step 9 risk):
call expressions are a **postfix praat row whose tail may reference
rules**:

```
Expr = praat {
  simple = ...
  operators {
    @110 '(' CallArgs ')',
  }
}
CallArgs = args:sep(Expr, ',')?
```

A postfix row is `@lbp` followed by tokens/node refs with no further
operand; it must lead with tokens so the Pratt loop can dispatch. Node
payloads are only allowed in postfix tails — prefix/infix/mixfix rows
stay token-and-operand only.
