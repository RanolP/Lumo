# Associated Types for Cap Declarations — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `type Item` to `cap` declarations, `type Item = T` to `impl` blocks, and `I.Item` projection in type position, resolved via a fulfillment-based obligation queue.

**Architecture:** Eight sequential tasks: (1) `Keyword::Type` in lexer, (2) `TypeExpr::Proj` + postfix parse, (3) grammar + struct additions across LST/HIR/LIR, (4) HIR parser for assoc items, (5) typechecker registration, (6) obligation system, (7) call-site type variable substitution, (8) test fixtures + Iterator cap. Each task compiles and tests clean before the next starts.

**Tech Stack:** Rust, Lumo compiler (crates/lexer, crates/lst, crates/hir, crates/lir, crates/types, crates/compiler), `cargo test --workspace`.

---

## File Map

| File | Change |
|------|--------|
| `crates/lexer/src/lib.rs` | Add `Keyword::Type`, add `"type"` mapping |
| `crates/hir/src/parse.rs` | Fix `eat_ident("type")` → `eat_kw`; add projection postfix; parse assoc items in cap/impl |
| `crates/types/src/lib.rs` | Add `TypeExpr::Proj` variant |
| `crates/compiler/src/backend/ts.rs` | Handle `Proj` in all `TypeExpr` match arms |
| `crates/compiler/lumo.langue` | Add `CapItem`, `AssocTypeDecl`, `ImplItem`, `AssocTypeBinding` |
| `crates/lst/src/parser.rs` | Add `assoc_types` to `CapDecl` and `ImplDecl` |
| `crates/hir/src/lib.rs` | Add `assoc_types` to `CapDecl`/`ImplDecl`; update `lower_cap`/`lower_impl` |
| `crates/lir/src/lib.rs` | Add `assoc_types` to `CapDecl`/`ImplDecl`; update lowering |
| `crates/compiler/src/typecheck/mod.rs` | Add `assoc_types` to `CapDef`; add `assoc_type_bindings`, `Obligation` queue, drain, substitution |
| `crates/compiler/tests/fixtures/type/assoc_types.txt` | New type-check fixtures |
| `packages/libcore/src/iterator.lumo` | New `Iterator` cap |
| `packages/libstd/src/list.lumo` | Add `impl List[T]: Iterator` |

---

## Task 1 — Add `Keyword::Type` to lexer

**Files:**
- Modify: `crates/lexer/src/lib.rs`
- Modify: `crates/hir/src/parse.rs`

- [ ] **Step 1: Add `Type` to the `Keyword` enum**

In `crates/lexer/src/lib.rs`, find `pub enum Keyword {` (line ~19) and add:

```rust
Type,
```

alongside the other variants.

- [ ] **Step 2: Add the `"type"` → `Keyword::Type` mapping**

In `crates/lexer/src/lib.rs`, find the `match` that maps string slices to `LosslessTokenKind::Keyword(...)` (around line 276) and add:

```rust
"type" => LosslessTokenKind::Keyword(Keyword::Type),
```

- [ ] **Step 3: Fix `eat_ident("type")` → `eat_kw(Keyword::Type)` in HIR parser**

In `crates/hir/src/parse.rs` line 267, change:

```rust
if self.eat_ident("type") {
```

to:

```rust
if self.eat_kw(Keyword::Type) {
```

- [ ] **Step 4: Handle `Keyword::Type` in exhaustive matches**

Run:

```bash
source ~/.cargo/env && cargo check --workspace 2>&1 | grep "non-exhaustive\|Keyword::Type\|error\[" | head -30
```

For each non-exhaustive match on `Keyword` reported, add:

```rust
Keyword::Type => { /* handle as appropriate — typically as an identifier or unreachable */ }
```

In `crates/compiler/src/backend/ts.rs` and any highlight/token_text functions, add `Keyword::Type => "type".to_owned()` or equivalent.

- [ ] **Step 5: Verify tests pass**

```bash
source ~/.cargo/env && cargo test --workspace 2>&1 | grep -E "FAILED|error\[" | head -10
```

Expected: no failures.

- [ ] **Step 6: Commit**

```bash
git add crates/lexer/src/lib.rs crates/hir/src/parse.rs
# also any other files touched by exhaustive match fixes
git commit -m "feat(lexer): add Keyword::Type for 'type' keyword"
```

---

## Task 2 — Add `TypeExpr::Proj` and projection parsing

**Files:**
- Modify: `crates/types/src/lib.rs`
- Modify: `crates/hir/src/parse.rs`
- Modify: `crates/compiler/src/backend/ts.rs`
- Modify: `crates/compiler/src/typecheck/mod.rs`

- [ ] **Step 1: Write a failing parse test**

In `crates/hir/src/parse.rs`, in the `#[cfg(test)]` block at the bottom, add:

```rust
#[test]
fn parse_type_projection() {
    let file = parse("fn f(x: I.Item): I.Item { x }").unwrap();
    // Should parse without error — exact AST shape verified in later tasks
    assert_eq!(file.items.len(), 1);
}
```

- [ ] **Step 2: Run to confirm it fails**

```bash
source ~/.cargo/env && cargo test -p lumo_hir parse_type_projection 2>&1 | tail -10
```

Expected: compile error — `TypeExpr::Proj` doesn't exist yet.

- [ ] **Step 3: Add `Proj` variant to `TypeExpr`**

In `crates/types/src/lib.rs`, in `pub enum TypeExpr {`, add after the `Fn` variant:

```rust
/// Associated type projection in type position: `I.Item`
Proj { base: Box<TypeExpr>, assoc: String },
```

- [ ] **Step 4: Update `display()` in `TypeExpr`**

In the `display()` match, add:

```rust
TypeExpr::Proj { base, assoc } => format!("{}.{assoc}", base.display()),
```

- [ ] **Step 5: Update `head_name()` in `TypeExpr`**

Add:

```rust
TypeExpr::Proj { .. } => "?proj",
```

- [ ] **Step 6: Update `references_name()` in `TypeExpr`**

Add:

```rust
TypeExpr::Proj { base, .. } => base.references_name(target),
```

- [ ] **Step 7: Stub `Proj` in `v_type_from_type_expr` (typecheck)**

In `crates/compiler/src/typecheck/mod.rs`, in `v_type_from_type_expr`, add before the final `_ => None`:

```rust
TypeExpr::Proj { base, assoc } => {
    // Full resolution in Task 6 — for now return None to avoid crashes
    let _ = (base, assoc);
    None
}
```

- [ ] **Step 8: Stub `Proj` in `ts.rs` TypeExpr matches**

Run:

```bash
source ~/.cargo/env && cargo check --workspace 2>&1 | grep "non-exhaustive\|Proj\|error\[" | head -30
```

For each non-exhaustive match in `crates/compiler/src/backend/ts.rs`, add:

```rust
TypeExpr::Proj { base, assoc } => lower_type_expr_to_ts_type(base), // resolve to base for now
```

(and in `type_refs_self`: `TypeExpr::Proj { base, .. } => type_refs_self(base)`)
(and in string-returning type functions: `TypeExpr::Proj { base, assoc } => format!("{}.{assoc}", /* base name */)`)

- [ ] **Step 9: Add projection postfix parsing in `parse_type_expr`**

In `crates/hir/src/parse.rs`, in `parse_type_expr`, replace the final two `Some(Spanned { ... })` returns with a block that first builds the base, then applies projection postfixes:

```rust
let mut ty = if self.eat_sym(Symbol::LBracket) {
    let mut args = Vec::new();
    while self.peek() != Some(&TokenKind::Symbol(Symbol::RBracket)) && !self.at_end() {
        if let Some(arg) = self.parse_type_expr() {
            args.push(arg.value);
        }
        self.eat_sym(Symbol::Comma);
    }
    let end = self.expect_sym(Symbol::RBracket).ok()?;
    Spanned {
        span: Span::new(span.start, end.end),
        value: TypeExpr::App { head: name, args },
    }
} else {
    Spanned { span, value: TypeExpr::Named(name) }
};

// Postfix projection: `.UppercaseIdent` in type context
loop {
    if self.peek() != Some(&TokenKind::Symbol(Symbol::Dot)) {
        break;
    }
    // Peek two tokens ahead: must be an ident starting with uppercase
    let is_proj = matches!(
        self.tokens.get(self.pos + 1),
        Some(Token { kind: TokenKind::Ident(n), .. }) if n.chars().next().map_or(false, |c| c.is_uppercase())
    );
    if !is_proj {
        break;
    }
    self.advance(); // consume '.'
    let (assoc, assoc_span) = self.expect_ident().ok()?;
    ty = Spanned {
        span: Span::new(ty.span.start, assoc_span.end),
        value: TypeExpr::Proj { base: Box::new(ty.value), assoc },
    };
}

Some(ty)
```

Note: `self.tokens` and `self.pos` are the internal parser fields — verify their exact names in the parser struct before implementing.

- [ ] **Step 10: Run tests**

```bash
source ~/.cargo/env && cargo test --workspace 2>&1 | grep -E "FAILED|error\[" | head -10
```

Expected: no failures.

- [ ] **Step 11: Commit**

```bash
git add crates/types/src/lib.rs crates/hir/src/parse.rs crates/compiler/src/typecheck/mod.rs crates/compiler/src/backend/ts.rs
git commit -m "feat(types): add TypeExpr::Proj for I.Item associated type projection"
```

---

## Task 3 — Grammar + LST/HIR/LIR structs for `assoc_types`

**Files:**
- Modify: `crates/compiler/lumo.langue`
- Modify: `crates/lst/src/parser.rs`
- Modify: `crates/hir/src/lib.rs`
- Modify: `crates/lir/src/lib.rs`

- [ ] **Step 1: Update `crates/compiler/lumo.langue`**

Replace:

```
CapDecl = 'cap' name:Ident '{' operations:OperationDecl* '}'
```

with:

```
CapDecl = 'cap' name:Ident '{' items:CapItem* '}'
CapItem =
  | AssocTypeDecl
  | OperationDecl
AssocTypeDecl = 'type' name:Ident
```

Replace:

```
ImplDecl = 'impl' generic_params:GenericParams? name:Ident? target:TypeExpr (':' cap:TypeExpr)? '{' methods:ImplMethod* '}'
```

with:

```
ImplDecl = 'impl' generic_params:GenericParams? name:Ident? target:TypeExpr (':' cap:TypeExpr)? '{' items:ImplItem* '}'
ImplItem =
  | AssocTypeBinding
  | ImplMethod
AssocTypeBinding = 'type' name:Ident '=' ty:TypeExpr
```

- [ ] **Step 2: Regenerate**

```bash
bash scripts/gen_langue.sh compiler
```

Expected: `crates/lst/src/syntax_kind.rs`, `ast.rs`, `lossless.rs` updated.

- [ ] **Step 3: Add `assoc_types` to LST `CapDecl`**

In `crates/lst/src/parser.rs`, change `CapDecl`:

```rust
pub struct CapDecl {
    pub name: String,
    pub assoc_types: Vec<String>,   // NEW
    pub operations: Vec<OperationDecl>,
    pub span: Span,
}
```

- [ ] **Step 4: Add `assoc_types` to LST `ImplDecl`**

```rust
pub struct ImplDecl {
    pub name: Option<String>,
    pub generics: Vec<GenericParam>,
    pub target_type: TypeSig,
    pub capability: Option<TypeSig>,
    pub assoc_types: Vec<(String, TypeSig)>,   // NEW
    pub methods: Vec<ImplMethod>,
    pub span: Span,
}
```

- [ ] **Step 5: Add `assoc_types` to HIR `CapDecl` and `ImplDecl`**

In `crates/hir/src/lib.rs`:

```rust
pub struct CapDecl {
    pub name: String,
    pub assoc_types: Vec<String>,                      // NEW
    pub operations: Vec<OperationDecl>,
    pub span: Span,
}

pub struct ImplDecl {
    pub name: Option<String>,
    pub generics: Vec<GenericParam>,
    pub target_type: Spanned<TypeExpr>,
    pub capability: Option<Spanned<TypeExpr>>,
    pub assoc_types: Vec<(String, TypeExpr)>,          // NEW
    pub methods: Vec<ImplMethodDecl>,
    pub span: Span,
}
```

- [ ] **Step 6: Update `lower_cap` in `crates/hir/src/lib.rs`**

```rust
fn lower_cap(cap: &lst::CapDecl) -> CapDecl {
    CapDecl {
        name: cap.name.clone(),
        assoc_types: cap.assoc_types.clone(),           // NEW
        operations: cap.operations.iter().map(lower_operation).collect(),
        span: cap.span,
    }
}
```

- [ ] **Step 7: Update `lower_impl` in `crates/hir/src/lib.rs`**

In the `ImplDecl { ... }` construction at the bottom of `lower_impl`, add:

```rust
assoc_types: impl_decl.assoc_types.iter()
    .filter_map(|(name, sig)| {
        lower_type_sig(sig).map(|ty| (name.clone(), ty.value))
    })
    .collect(),
```

- [ ] **Step 8: Add `assoc_types` to LIR `CapDecl` and `ImplDecl`**

In `crates/lir/src/lib.rs`, mirror the same field additions as HIR.

- [ ] **Step 9: Update LIR lowering for `CapDecl`**

In `crates/lir/src/lib.rs`, find the `hir::Item::Cap(cap) => Item::Cap(CapDecl { ... })` block (around line 334) and add:

```rust
assoc_types: cap.assoc_types.clone(),
```

- [ ] **Step 10: Update LIR `lower_impl`**

In `crates/lir/src/lib.rs`, in `lower_impl`, add:

```rust
assoc_types: impl_decl.assoc_types.clone(),
```

to the returned `ImplDecl`.

- [ ] **Step 11: Verify compilation**

```bash
source ~/.cargo/env && cargo check --workspace 2>&1 | grep "error\[" | head -20
```

Expected: no errors (struct literal mismatches will be caught here — fix any remaining field additions).

- [ ] **Step 12: Commit**

```bash
git add crates/compiler/lumo.langue crates/lst/src/ crates/hir/src/lib.rs crates/lir/src/lib.rs
git commit -m "feat: add assoc_types fields to CapDecl and ImplDecl across LST/HIR/LIR"
```

---

## Task 4 — HIR parser: parse assoc type items in cap and impl

**Files:**
- Modify: `crates/hir/src/parse.rs`

- [ ] **Step 1: Write failing test for cap with associated type**

In the `#[cfg(test)]` block:

```rust
#[test]
fn parse_cap_with_assoc_type() {
    let src = r#"cap Iterator {
    type Item
    fn next(self): Option[Item]
}"#;
    let file = parse(src).unwrap();
    let lumo_lst::Item::Cap(cap) = &file.items[0] else { panic!() };
    assert_eq!(cap.assoc_types, vec!["Item".to_owned()]);
    assert_eq!(cap.operations.len(), 1);
    assert_eq!(cap.operations[0].name, "next");
}
```

- [ ] **Step 2: Run to confirm failure**

```bash
source ~/.cargo/env && cargo test -p lumo_hir parse_cap_with_assoc_type 2>&1 | tail -10
```

Expected: test fails — `assoc_types` is empty.

- [ ] **Step 3: Update `parse_cap_decl`**

In `crates/hir/src/parse.rs`, replace the `parse_cap_decl` body to loop on both `Keyword::Type` and `Keyword::Fn`:

```rust
fn parse_cap_decl(&mut self) -> Option<CapDecl> {
    let start = self.expect_kw(Keyword::Cap).ok()?;
    let (name, _) = self.expect_ident().ok()?;
    self.expect_sym(Symbol::LBrace).ok()?;
    let mut assoc_types = Vec::new();
    let mut operations = Vec::new();
    loop {
        if self.eat_kw(Keyword::Type) {
            let (assoc_name, _) = self.expect_ident().ok()?;
            assoc_types.push(assoc_name);
        } else if self.peek() == Some(&TokenKind::Keyword(Keyword::Fn)) {
            if let Some(op) = self.parse_operation_decl() {
                operations.push(op);
            }
        } else {
            break;
        }
    }
    let end = self.expect_sym(Symbol::RBrace).ok()?;
    Some(CapDecl {
        name,
        assoc_types,
        operations,
        span: Span::new(start.start, end.end),
    })
}
```

- [ ] **Step 4: Run cap test**

```bash
source ~/.cargo/env && cargo test -p lumo_hir parse_cap_with_assoc_type 2>&1 | tail -5
```

Expected: PASS.

- [ ] **Step 5: Write failing test for impl with assoc type binding**

```rust
#[test]
fn parse_impl_with_assoc_type() {
    let src = r#"impl List[T]: Iterator {
    type Item = T
    fn next(self): Option[T] { Option.none }
}"#;
    let file = parse(src).unwrap();
    let lumo_lst::Item::Impl(impl_decl) = &file.items[0] else { panic!() };
    assert_eq!(impl_decl.assoc_types.len(), 1);
    assert_eq!(impl_decl.assoc_types[0].0, "Item");
    assert_eq!(impl_decl.assoc_types[0].1.repr, "T");
    assert_eq!(impl_decl.methods.len(), 1);
}
```

- [ ] **Step 6: Run to confirm failure**

```bash
source ~/.cargo/env && cargo test -p lumo_hir parse_impl_with_assoc_type 2>&1 | tail -10
```

Expected: fails — `assoc_types` is empty.

- [ ] **Step 7: Update `parse_impl_decl`**

In `crates/hir/src/parse.rs`, update `parse_impl_decl` to loop on both `Keyword::Type` and `Keyword::Fn`:

```rust
// Replace the methods-only loop:
let mut assoc_types = Vec::new();
let mut methods = Vec::new();
loop {
    if self.eat_kw(Keyword::Type) {
        let (assoc_name, _) = self.expect_ident().ok()?;
        self.expect_sym(Symbol::Eq).ok()?;
        let ty = self.parse_type_sig()?;
        assoc_types.push((assoc_name, ty));
    } else if self.peek() == Some(&TokenKind::Keyword(Keyword::Fn)) {
        if let Some(m) = self.parse_impl_method() {
            methods.push(m);
        }
    } else {
        break;
    }
}
```

And update the returned `ImplDecl` to include `assoc_types`.

Note: verify the exact method name for parsing type signatures in impl context — it may be `parse_type_sig` or similar. Look for how `capability: Option<TypeSig>` is parsed (uses the same helper).

- [ ] **Step 8: Run all HIR tests**

```bash
source ~/.cargo/env && cargo test -p lumo_hir 2>&1 | tail -10
```

Expected: all pass.

- [ ] **Step 9: Run full test suite**

```bash
source ~/.cargo/env && cargo test --workspace 2>&1 | grep -E "FAILED|error\[" | head -10
```

Expected: no failures.

- [ ] **Step 10: Commit**

```bash
git add crates/hir/src/parse.rs
git commit -m "feat(hir): parse assoc type declarations in cap and impl blocks"
```

---

## Task 5 — Typechecker: register associated types

**Files:**
- Modify: `crates/compiler/src/typecheck/mod.rs`

- [ ] **Step 1: Add `assoc_types` to `CapDef`**

In `crates/compiler/src/typecheck/mod.rs`, find `struct CapDef` and add:

```rust
struct CapDef {
    operations: HashMap<String, CompType>,
    uses_self: bool,
    assoc_types: Vec<String>,    // NEW
}
```

Update all `CapDef { ... }` constructor sites to include `assoc_types: vec![]` or the real values.

- [ ] **Step 2: Add `assoc_type_bindings` to `TypeChecker`**

In `struct TypeChecker`, add:

```rust
/// (cap_name, target_base_name) → (impl_generic_param_names, {assoc_name → TypeExpr})
assoc_type_bindings: HashMap<(String, String), (Vec<String>, HashMap<String, TypeExpr>)>,
```

Initialize it to `HashMap::new()` in both `TypeChecker::new()` / `TypeChecker::default()` sites.

- [ ] **Step 3: Populate `CapDef.assoc_types` during cap registration**

In `check_file`, find where `CapDecl` items are iterated and `cap_defs` is populated. Add `assoc_types: cap.assoc_types.clone()` to the `CapDef` construction.

- [ ] **Step 4: Populate `assoc_type_bindings` during impl registration**

In `check_file`, inside the impl registration loop, after the existing cap impl registration logic, add:

```rust
// Register associated type bindings
if let Some(cap_ty) = &impl_decl.capability {
    let cap_name = cap_ty.value.display();
    let target_base = target_base.clone(); // base name without generics
    let generic_param_names: Vec<String> = impl_decl.generics.iter()
        .filter_map(|g| match g {
            lir::GenericParam::Type(name, _) => Some(name.clone()),
            _ => None,
        })
        .collect();
    let mut bindings = HashMap::new();
    for (assoc_name, assoc_ty) in &impl_decl.assoc_types {
        bindings.insert(assoc_name.clone(), assoc_ty.clone());
    }
    if !bindings.is_empty() {
        self.assoc_type_bindings.insert(
            (cap_name, target_base),
            (generic_param_names, bindings),
        );
    }
}
```

- [ ] **Step 5: Verify compilation**

```bash
source ~/.cargo/env && cargo check --workspace 2>&1 | grep "error\[" | head -10
```

Expected: no errors.

- [ ] **Step 6: Run tests**

```bash
source ~/.cargo/env && cargo test --workspace 2>&1 | grep -E "FAILED" | head -10
```

Expected: no failures.

- [ ] **Step 7: Commit**

```bash
git add crates/compiler/src/typecheck/mod.rs
git commit -m "feat(typecheck): register associated types in CapDef and assoc_type_bindings"
```

---

## Task 6 — Obligation system: emit, drain, substitute

**Files:**
- Modify: `crates/compiler/src/typecheck/mod.rs`

- [ ] **Step 1: Add `Obligation` enum and new TypeChecker fields**

In `crates/compiler/src/typecheck/mod.rs`, add near the top (before `struct TypeChecker`):

```rust
#[derive(Clone, Debug)]
enum Obligation {
    Normalize { base: TypeExpr, assoc: String, var: String },
}
```

Add to `struct TypeChecker`:

```rust
obligations: Vec<Obligation>,
assoc_subst: HashMap<String, ValueType>,
assoc_var_counter: usize,
```

Initialize all three to empty/zero in every `TypeChecker` constructor.

- [ ] **Step 2: Add `substitute_type_expr` helper**

Add as a free function (not a method):

```rust
/// Substitute Named(k) → v for each (k, v) in `subst`.
fn substitute_type_expr(ty: &TypeExpr, subst: &HashMap<String, TypeExpr>) -> TypeExpr {
    match ty {
        TypeExpr::Named(n) => subst.get(n).cloned().unwrap_or_else(|| ty.clone()),
        TypeExpr::App { head, args } => TypeExpr::App {
            head: head.clone(),
            args: args.iter().map(|a| substitute_type_expr(a, subst)).collect(),
        },
        TypeExpr::Proj { base, assoc } => TypeExpr::Proj {
            base: Box::new(substitute_type_expr(base, subst)),
            assoc: assoc.clone(),
        },
        TypeExpr::Fn { params, ret, cap } => TypeExpr::Fn {
            params: params.iter().map(|p| substitute_type_expr(p, subst)).collect(),
            ret: Box::new(substitute_type_expr(ret, subst)),
            cap: cap.clone(),
        },
        other => other.clone(),
    }
}
```

- [ ] **Step 3: Add `try_resolve_proj` method on `TypeChecker`**

```rust
fn try_resolve_proj(&self, base: &TypeExpr, assoc: &str) -> Option<ValueType> {
    let (base_name, base_args) = match base {
        TypeExpr::Named(n) => (n.clone(), vec![]),
        TypeExpr::App { head, args } => (head.clone(), args.clone()),
        _ => return None,
    };
    for ((_, target_base), (generic_params, bindings)) in &self.assoc_type_bindings {
        if *target_base == base_name {
            if let Some(assoc_ty) = bindings.get(assoc) {
                let subst: HashMap<String, TypeExpr> = generic_params
                    .iter()
                    .zip(base_args.iter())
                    .map(|(name, arg)| (name.clone(), arg.clone()))
                    .collect();
                let resolved = substitute_type_expr(assoc_ty, &subst);
                return v_type_from_type_expr(&resolved);
            }
        }
    }
    None
}
```

- [ ] **Step 4: Add `apply_assoc_subst` method**

```rust
fn apply_assoc_subst(&self, ty: ValueType) -> ValueType {
    match ty {
        ValueType::Named(ref n) if n.starts_with("?assoc_") => {
            self.assoc_subst.get(n).cloned().unwrap_or(ty)
        }
        ValueType::Thunk(inner) => {
            ValueType::Thunk(Box::new(self.apply_assoc_subst(*inner)))
        }
        other => other,
    }
}
```

- [ ] **Step 5: Add `drain_obligations` method**

```rust
fn drain_obligations(&mut self) {
    let obligations = std::mem::take(&mut self.obligations);
    for ob in obligations {
        match ob {
            Obligation::Normalize { base, assoc, var } => {
                if let Some(resolved) = self.try_resolve_proj(&base, &assoc) {
                    self.assoc_subst.insert(var, resolved);
                }
                // Unresolved = abstract projection under a bound — valid, no error
            }
        }
    }
}
```

- [ ] **Step 6: Replace the stub in `v_type_from_type_expr` with real emission**

Replace the stub added in Task 2:

```rust
TypeExpr::Proj { base, assoc } => {
    // Try immediate resolution
    if let Some(resolved) = self.try_resolve_proj(base, assoc) {
        return Some(resolved);
    }
    // Emit obligation, return placeholder var
    let var = format!("?assoc_{}", self.assoc_var_counter);
    self.assoc_var_counter += 1;
    self.obligations.push(Obligation::Normalize {
        base: *base.clone(),
        assoc: assoc.clone(),
        var: var.clone(),
    });
    Some(ValueType::Named(var))
}
```

Note: `v_type_from_type_expr` is currently a free function (not a method). You will need to either make it a method or pass a resolver closure/reference. Look at how it is currently called and choose the least invasive approach — passing `&self.assoc_type_bindings` as a parameter is one option.

If converting to a method is too invasive, add a standalone `resolve_proj` free function that takes `&HashMap<(String, String), (Vec<String>, HashMap<String, TypeExpr>)>` and call it from the obligation site.

- [ ] **Step 7: Call `drain_obligations` after each function body**

In `check_file`, after `self.check_c_expr(...)` for each `FnDecl` and `ImplMethodDecl`, add:

```rust
self.drain_obligations();
```

- [ ] **Step 8: Write a fixture test for basic projection**

In `crates/compiler/tests/fixtures/type/assoc_types.txt`, create:

```
projection resolves for concrete type
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
```

- [ ] **Step 9: Run fixture**

```bash
source ~/.cargo/env && cargo test --test typecheck_fixtures 2>&1 | grep -E "assoc|FAILED|ok" | head -10
```

Expected: test passes or expected output needs adjustment — check actual output and align.

- [ ] **Step 10: Run full test suite**

```bash
source ~/.cargo/env && cargo test --workspace 2>&1 | grep "FAILED" | head -10
```

Expected: no failures.

- [ ] **Step 11: Commit**

```bash
git add crates/compiler/src/typecheck/mod.rs crates/compiler/tests/fixtures/type/assoc_types.txt
git commit -m "feat(typecheck): obligation system for associated type projection resolution"
```

---

## Task 7 — Call-site type variable substitution in Apply

**Files:**
- Modify: `crates/compiler/src/typecheck/mod.rs`

This task makes `xs.next()` return `Option[Number]` (not `Option[T]`) when `xs: List[Number]`.

- [ ] **Step 1: Add `collect_type_subst` helper**

```rust
/// Match `expected` ValueType against `actual`, collecting Named(n) → actual mappings
/// for single-uppercase type vars (is_type_var).
fn collect_type_subst(
    expected: &ValueType,
    actual: &ValueType,
    subst: &mut HashMap<String, ValueType>,
) {
    match (expected, actual) {
        (ValueType::Named(n), actual) if is_type_var(n) => {
            subst.entry(n.clone()).or_insert_with(|| actual.clone());
        }
        (ValueType::Named(a), ValueType::Named(b)) if a == b => {}
        // Recurse into App-like wrappers if needed
        _ => {}
    }
}
```

- [ ] **Step 2: Add `apply_type_subst_to_value` helper**

```rust
fn apply_type_subst_to_value(
    ty: ValueType,
    subst: &HashMap<String, ValueType>,
) -> ValueType {
    match ty {
        ValueType::Named(ref n) if subst.contains_key(n) => subst[n].clone(),
        ValueType::Thunk(inner) => {
            ValueType::Thunk(Box::new(apply_type_subst_to_value(*inner, subst)))
        }
        other => other,
    }
}

fn apply_type_subst_to_comp(
    ty: CompType,
    subst: &HashMap<String, ValueType>,
) -> CompType {
    match ty {
        CompType::Fn { params, ret, cap } => CompType::Fn {
            params: params.into_iter().map(|p| apply_type_subst_to_value(p, subst)).collect(),
            ret: Box::new(apply_type_subst_to_comp(*ret, subst)),
            cap,
        },
        CompType::Produce(inner) => {
            CompType::Produce(Box::new(apply_type_subst_to_value(*inner, subst)))
        }
        other => other,
    }
}
```

- [ ] **Step 3: Apply substitution in `infer_c_expr` Apply case**

In `infer_c_expr`, find the `Expr::Apply { func, args, .. }` arm. After resolving the function type to `CompType::Fn { params, ret, cap }`, add:

```rust
// Collect type variable substitutions from self-arg (first param) matching
let mut type_subst: HashMap<String, ValueType> = HashMap::new();
if let (Some(first_param), Some(first_arg)) = (params.first(), args.first()) {
    if let Some(arg_ty) = self.infer_v_expr(first_arg, env) {
        collect_type_subst(first_param, &arg_ty, &mut type_subst);
    }
}
// Apply substitution to return type
let ret = apply_type_subst_to_comp(*ret, &type_subst);
```

Note: verify the exact shape of the Apply arm in the current codebase before modifying — the substitution should be additive, not replacing the existing type checking.

- [ ] **Step 4: Add a fixture for concrete-type method return**

Append to `crates/compiler/tests/fixtures/type/assoc_types.txt`:

```
==========
next() return type resolved at concrete call site
cap Iterator {
    type Item
    fn next(self): Option[Item]
}
data List[T] { .nil, .cons(T, List[T]) }
impl List[T]: Iterator {
    type Item = T
    fn next(self): Option[T] = Option.none
}
fn use_next(xs: List[Number]): Option[Number] = xs.next()
---
use_next : (List[Number]) -> Option[Number]
```

- [ ] **Step 5: Run fixtures**

```bash
source ~/.cargo/env && cargo test --test typecheck_fixtures 2>&1 | grep -E "assoc|FAILED|ok" | head -10
```

Adjust expected output if needed.

- [ ] **Step 6: Run full suite**

```bash
source ~/.cargo/env && cargo test --workspace 2>&1 | grep "FAILED" | head -10
```

Expected: no failures.

- [ ] **Step 7: Commit**

```bash
git add crates/compiler/src/typecheck/mod.rs crates/compiler/tests/fixtures/type/assoc_types.txt
git commit -m "feat(typecheck): substitute type vars at Apply call sites for impl methods"
```

---

## Task 8 — Iterator cap + List impl + test fixtures

**Files:**
- Create: `packages/libcore/src/iterator.lumo`
- Modify: `packages/libstd/src/list.lumo`
- Modify: `crates/compiler/tests/fixtures/type/assoc_types.txt`

- [ ] **Step 1: Create `Iterator` cap in libcore**

Create `packages/libcore/src/iterator.lumo`:

```lumo
use libcore.prelude.{Option};

cap Iterator {
    type Item
    fn next(self): Option[Item]
}
```

- [ ] **Step 2: Add `impl List[T]: Iterator` to libstd**

In `packages/libstd/src/list.lumo`, append:

```lumo
use libcore.iterator.{Iterator};

impl List[T]: Iterator {
    type Item = T
    fn next(self): Option[T] =
        match self {
            .nil => Option.none,
            .cons(h, _) => Option.some(h)
        }
}
```

- [ ] **Step 3: Add abstract projection fixture**

Append to `crates/compiler/tests/fixtures/type/assoc_types.txt`:

```
==========
abstract projection in generic function
cap Iterator {
    type Item
    fn next(self): Option[Item]
}
fn first[I: Iterator](it: I): Option[I.Item] = it.next()
---
first : (I) -> Option[I.Item]
```

- [ ] **Step 4: Run all fixtures**

```bash
source ~/.cargo/env && cargo test --test typecheck_fixtures 2>&1 | grep -E "assoc|FAILED|ok"
```

Adjust expected output lines to match actual output if needed.

- [ ] **Step 5: Run full suite**

```bash
source ~/.cargo/env && cargo test --workspace 2>&1 | grep -E "FAILED|error\[" | head -20
```

Expected: no failures.

- [ ] **Step 6: Commit**

```bash
git add packages/libcore/src/iterator.lumo packages/libstd/src/list.lumo crates/compiler/tests/fixtures/type/assoc_types.txt
git commit -m "feat: add Iterator cap to libcore and impl List[T]: Iterator in libstd"
```

---

## Verification

```bash
source ~/.cargo/env

# All tests
cargo test --workspace

# Just assoc type fixtures
cargo test --test typecheck_fixtures 2>&1 | grep -E "assoc|FAILED|ok"

# Spot-check: Iterator cap parses
echo 'use libcore.iterator.{Iterator};
cap Test { type Item; fn next(self): Option[Item] }' | node packages/langue/dist/langue.js /dev/stdin /tmp/test_out 2>&1
```
