# Associated Types for Cap Declarations

## Goal

Allow `cap` declarations to carry named abstract types (`type Item`) and `impl` blocks to bind them to concrete types (`type Item = T`). Enable projection syntax `I.Item` in type position, resolved via a fulfillment-based obligation queue modeled after Rust's trait solver.

## Motivation

Needed for `Iterator` cap (`fn next(self): Option[Item]`) and any cap whose operations return or accept a type that varies per implementation.

## Syntax

### Cap declaration

```lumo
cap Iterator {
    type Item
    fn next(self): Option[Item]
}
```

### Impl binding

```lumo
impl List[T]: Iterator {
    type Item = T
    fn next(self): Option[T] = ...
}
```

### Type projection

```lumo
fn first[I: Iterator](it: I): Option[I.Item] = it.next()
```

`I.Item` in **type position** — `.` after a type followed by an uppercase-starting ident.  
Distinguished from value-level `.field` access by parser context (`parse_type_expr` vs `parse_expr`).

---

## Data Structures

### `crates/types/src/lib.rs`

Add to `TypeExpr`:

```rust
/// Associated type projection: `I.Item`, `List[T].Item`
Proj { base: Box<TypeExpr>, assoc: String },
```

- `display()`: `"{base}.{assoc}"`
- `head_name()`: `None`
- `references_name(target)`: delegate to `base.references_name(target)`

### `crates/hir/src/lib.rs` and `crates/lir/src/lib.rs`

```rust
pub struct CapDecl {
    pub name: String,
    pub assoc_types: Vec<String>,          // NEW: ["Item"]
    pub operations: Vec<OperationDecl>,
    pub span: Span,
}

pub struct ImplDecl {
    pub name: Option<String>,
    pub generics: Vec<GenericParam>,
    pub target_type: Spanned<TypeExpr>,
    pub capability: Option<Spanned<TypeExpr>>,
    pub assoc_types: Vec<(String, TypeExpr)>, // NEW: [("Item", Named("T"))]
    pub methods: Vec<ImplMethodDecl>,
    pub span: Span,
}
```

### `crates/compiler/src/typecheck/mod.rs`

```rust
struct CapDef {
    operations: HashMap<String, CompType>,
    uses_self: bool,
    assoc_types: Vec<String>,              // NEW
}

// NEW — associated type binding registry
// key: (cap_name, target_base_name) → (impl_generic_param_names, {assoc_name → TypeExpr})
assoc_type_bindings: HashMap<(String, String), (Vec<String>, HashMap<String, TypeExpr>)>,

// NEW — obligation queue
obligations: Vec<Obligation>,

// NEW — resolved placeholder vars
assoc_subst: HashMap<String, ValueType>,

// NEW — counter for fresh var names
assoc_var_counter: usize,
```

```rust
enum Obligation {
    Normalize { base: TypeExpr, assoc: String, var: String },
}
```

---

## Grammar Changes (`crates/compiler/lumo.langue`)

```
// Before:
CapDecl = 'cap' name:Ident '{' operations:OperationDecl* '}'

// After:
CapDecl = 'cap' name:Ident '{' items:CapItem* '}'
CapItem =
  | AssocTypeDecl
  | OperationDecl
AssocTypeDecl = 'type' name:Ident

// Before:
ImplDecl = 'impl' generic_params:GenericParams? name:Ident? target:TypeExpr (':' cap:TypeExpr)? '{' methods:ImplMethod* '}'

// After:
ImplDecl = 'impl' generic_params:GenericParams? name:Ident? target:TypeExpr (':' cap:TypeExpr)? '{' items:ImplItem* '}'
ImplItem =
  | AssocTypeBinding
  | ImplMethod
AssocTypeBinding = 'type' name:Ident '=' ty:TypeExpr
```

`TypeExpr` projection is parsed as a postfix in `parse_type_expr` — no grammar rule needed (same approach as how value-level `.field` is handled in `parse_expr`).

Run `scripts/gen_langue.sh compiler` after grammar change to regenerate `syntax_kind.rs`, `ast.rs`, `lossless.rs`.

---

## Resolution Algorithm

### Emission (`v_type_from_type_expr`)

```
match TypeExpr::Proj { base, assoc }:
    resolved = try_resolve_proj(base, assoc)
    if resolved:
        return resolved
    else:
        var = "?assoc_{counter++}"
        push Obligation::Normalize { base, assoc, var }
        return ValueType::Named(var)
```

### `try_resolve_proj(base, assoc)`

```
(base_name, base_args) = decompose(base)
    Named(n) → (n, [])
    App { head, args } → (head, args)
    _ → return None

for (cap_name, target_base), (generic_params, bindings) in assoc_type_bindings:
    if target_base == base_name:
        if assoc_ty = bindings.get(assoc):
            subst = zip(generic_params, base_args)
            return v_type_from_type_expr(substitute(assoc_ty, subst))

return None
```

### Obligation drain (after each function body)

```
obligations = take(self.obligations)
for Normalize { base, assoc, var } in obligations:
    if resolved = try_resolve_proj(base, assoc):
        assoc_subst[var] = resolved
    // if not resolved: abstract projection, valid under bound — no error
```

### `apply_assoc_subst(ty: ValueType) → ValueType`

Walk a `ValueType`, replacing `Named("?assoc_N")` with `assoc_subst["?assoc_N"]` where present. Applied to inferred/checked return types after drain.

### Type variable substitution in Apply (call sites)

When checking `Apply(fn_expr, args)` where `fn_expr` resolves to a method on a generic impl (e.g., `__impl_List.next`):

1. Match the self-param type `List[T]` against the actual arg type `List[Number]` → collect substitution `{T → Number}`
2. Apply substitution to the return type `Option[T]` → `Option[Number]`
3. This is done in `infer_c_expr` for the `Apply` case via `collect_type_subst` + `apply_type_subst` helpers

```rust
/// Match expected type against actual type, collecting Named("T") → concrete mappings.
/// Only substitutes single-uppercase type vars (is_type_var).
fn collect_type_subst(expected: &ValueType, actual: &ValueType, subst: &mut HashMap<String, ValueType>)

/// Apply a substitution map to a ValueType.
fn apply_type_subst(ty: ValueType, subst: &HashMap<String, ValueType>) -> ValueType
```

---

## Lexer

Add `type` as a new keyword — it is not currently in the `Keyword` enum.

```rust
// crates/lexer/src/lib.rs — add to Keyword enum:
Type,

// add to from_str match:
"type" => LosslessTokenKind::Keyword(Keyword::Type),
```

Also update all exhaustive `match` on `Keyword` elsewhere (parser, highlight, etc.) to add a `Keyword::Type` arm.

`extern type` is currently parsed via `eat_ident("type")` in `crates/hir/src/parse.rs` — change to `eat_kw(Keyword::Type)` after adding the keyword.

`.` is already `Symbol::Dot` — no change needed.

---

## Testing

### Fixture file: `crates/compiler/tests/fixtures/type/assoc_types.txt`

```
associated type in cap and impl
cap Iterator {
    type Item
    fn next(self): Option[Item]
}
data List[T] { .nil, .cons(T, List[T]) }
impl List[T]: Iterator {
    type Item = T
    fn next(self): Option[T] = Option.none
}
fn get_first(xs: List[Number]): Option[Number] = xs.next()
---
get_first : (List[Number]) -> Option[Number]
==========
projection in generic function signature
cap Iterator {
    type Item
    fn next(self): Option[Item]
}
fn first[I: Iterator](it: I): Option[I.Item] = it.next()
---
first : (I) -> Option[I.Item]
```

---

## Out of Scope (future)

- Associated type bounds: `type Item: Display`
- Higher-kinded associated types
- Multiple caps sharing an associated type name (disambiguation)
- `where` clauses
