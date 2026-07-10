# Salsa-like architecture from day one

Every derived artifact is a memoized query over inputs, at both layers:
the definition layer (`parse(file)` → `merged_definition(project)` →
`rule_tables(kind)` → generated code) and the compiled-language layer
(`cst(file)`, `mir(file)`, `infer(node)`). The cat/merge/DCE model
produces one immutable definition value per revision — exactly what
memoization wants. Candidate runtime: the `salsa` crate, or a
purpose-built equivalent if salsa's model fights the relational engine.
