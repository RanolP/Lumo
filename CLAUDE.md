# Lumo Project — Claude Instructions

## Syntax Changes: lumo.langue First

When adding or modifying any Lumo language syntax (new expression forms, statement types, type syntax, patterns, etc.):

1. **Update `crates/compiler/lumo.langue` first** — this is the source of truth for the CST grammar
2. **Run `bash scripts/gen_langue.sh compiler`** — regenerates `crates/lst/src/syntax_kind.rs`, `ast.rs`, `lossless.rs`
3. **Then update `crates/lst/src/parser.rs`** — hand-written recursive descent parser
4. **Then update HIR lowering** (`crates/hir/src/lib.rs`) if needed

Skipping step 1–2 leaves the generated files inconsistent with the actual language.

### Grammar correctness rules

- Comma-separated lists **must use wrapper nodes** to preserve accessors:
  ```
  FooList = '(' items:FooItems? ')'
  FooItems = Foo (',' Foo)* ','?
  ```
  Never use `Foo* ','?` (items without enforced separators) or inline `(Foo (',' Foo)*)?` (loses the label, no accessor generated).
- Optional type annotations use `(':' ty:TypeExpr)?`
- Reuse existing nodes where possible (`FnBody`, `ParamList`, `GenericParams`)
