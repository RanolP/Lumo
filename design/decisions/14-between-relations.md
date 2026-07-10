# Same-language relations: `between A` blocks

```
between A {
  Apply { fn: Lambda { param: $f, body: $f }, arg: $e } === $e
}

between A {
  Apply { fn: Lambda { param: $b, body: $e }, arg: $a } === $e[$b := $a]
}
```

On the same language, rules define relations: `lhs === rhs` equalities,
run as e-graph equality saturation. `$x` binds a metavariable. The
`subst` tactic is built-in — `$e[$b := $a]`.
