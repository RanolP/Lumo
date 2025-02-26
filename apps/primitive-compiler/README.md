# The Lumo Compiler

## Hyper-TODO

Short-term goals

- [ ] Parse Basic Syntax
  - [ ] Function Definition
  - [ ] Struct Definition
  - [ ] Enum Definition
  - [ ] Match Expression
- [ ] Type Check
  - [ ] Implement Hindley-Milner Typing
  - [ ] Generate Typed AST
- [ ] Module Synthesis
- [ ] Lowering to Lumo IR

Mid-term goals

- [ ] Introduce `mut name` semantics
- [ ] Introduce `trait` system
- [ ] Introduce `effect` system

Long-term goals

- [ ] Introduce `record` and implement row-polymorphism in type checker
- [ ] Introduce Refinement Types

---

## Ideas

`#` in directory denotes module namspace.

e.g. for module `pulsegraph`

```
module.toml
#core
- #graphql
  - response.lumo
  - request.lumo
```

You can `use pulsegraph.core.graphql.Request`
