# Associated Types for Cap Declarations — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `type Item` to `cap` declarations, `type Item = T` to `impl` blocks, and `I.Item` projection in type position, resolved via a fulfillment-based obligation queue.

**Architecture:** Seven sequential tasks — Task 1 (`Keyword::Type`) is already done. Remaining: (2) `TypeExpr::Proj` + postfix parse, (3) grammar + struct additions across HIR/LIR + update `from_cst.rs`, (4) update `from_hir_cst.rs` for assoc items in HIR roundtrip, (5) typechecker registration, (6) obligation system, (7) call-site type variable substitution, (8) test fixtures + Iterator cap. Each task compiles and tests clean before the next starts.

**Tech Stack:** Rust, Lumo compiler (crates/lexer, crates/lst, crates/hir, crates/lir, crates/types, crates/compiler), `cargo test --workspace`.

---

## File Map

| File | Change |
|------|--------|
| ~~`crates/lexer/src/lib.rs`~~ | ✅ Done — `Keyword::Type` already exists |
| `crates/types/src/lib.rs` | Add `TypeExpr::Proj` variant |
| `crates/compiler/src/backend/ts.rs` | Handle `Proj` in all `TypeExpr` match arms |
| `crates/compiler/lumo.langue` | Add `CapItem`, `AssocTypeDecl`, `ImplItem`, `AssocTypeBinding` |
| `crates/hir/src/from_cst.rs` | Handle `CapItem` / `ImplItem` grammar nodes, populate `assoc_types` |
| `crates/hir/src/lib.rs` | Add `assoc_types` to `CapDecl`/`ImplDecl` structs |
| `crates/hir/src/from_hir_cst.rs` | Parse assoc type items in HIR roundtrip (replaces old `parse.rs` task) |
| `crates/lir/src/lib.rs` | Add `assoc_types` to `CapDecl`/`ImplDecl`; update lowering |
| `crates/compiler/src/typecheck/mod.rs` | Add `assoc_types` to `CapDef`; add `assoc_type_bindings`, `Obligation` queue, drain, substitution |
| `crates/compiler/tests/fixtures/type/assoc_types.txt` | New type-check fixtures |
| `packages/libcore/src/iterator.lumo` | New `Iterator` cap |
| `packages/libstd/src/list.lumo` | Add `impl List[T]: Iterator` |

---

## ~~Task 1 — Add `Keyword::Type` to lexer~~ ✅ DONE

`Keyword::Type` already exists in `crates/lexer/src/lib.rs` and is handled in all exhaustive matches. Skip this task entirely.

---

## Task 2 — Add `TypeExpr::Proj` and projection parsing

**Files:**
- Modify: `crates/types/src/lib.rs`
- Modify: `crates/hir/src/from_cst.rs`
- Modify: `crates/compiler/src/backend/ts.rs`
- Modify: `crates/compiler/src/typecheck/mod.rs`

- [ ] **Step 1: Write a failing parse test**

In `crates/hir/src/from_cst.rs`, add a `#[cfg(test)]` block:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_type_projection() {
        let lossless = lumo_lst::lossless::parse("fn f(x: I.Item): I.Item { x }");
        let file = lower_file(&lossless);
        assert_eq!(file.items.len(), 1);
        assert!(file.errors.is_empty(), "parse errors: {:?}", file.errors);
    }
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

- [ ] **Step 9: Add projection postfix parsing in `from_cst.rs`**

In `crates/hir/src/from_cst.rs`, in the type expression lowering function (wherever `TypeExpr::Named`/`TypeExpr::App` are produced from CST `TypeExpr` nodes), add a postfix projection check after building the base type. In the lossless CST, `I.Item` will parse as a `MemberExpr`-style node or as `TypeExpr` with a dot child — check what the generated `ast.rs` produces for the `lumo.langue` grammar. If `.Item` is not yet in the grammar, add it to `TypeExpr` in `lumo.langue` first (as `TypeExpr = ... | ProjTypeExpr` and `ProjTypeExpr = base:TypeExpr '.' assoc:Ident`). Then lower it:

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
git add crates/types/src/lib.rs crates/hir/src/from_cst.rs crates/compiler/src/typecheck/mod.rs crates/compiler/src/backend/ts.rs
git commit -m "feat(types): add TypeExpr::Proj for I.Item associated type projection"
```

---

## Task 3 — Grammar + HIR/LIR structs for `assoc_types`

**Files:**
- Modify: `crates/compiler/lumo.langue`
- Modify: `crates/hir/src/lib.rs`
- Modify: `crates/hir/src/from_cst.rs`
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

- [ ] **Step 3: Add `assoc_types` to HIR `CapDecl` and `ImplDecl`**

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

Fix all `CapDecl { .. }` and `ImplDecl { .. }` construction sites to include `assoc_types: vec![]`.

- [ ] **Step 4: Update `from_cst.rs` to populate `assoc_types` from grammar nodes**

In `crates/hir/src/from_cst.rs`, update `lower_cap_decl` and `lower_impl_decl` (or whatever the CST walking functions are named) to iterate `CapItem` children for `AssocTypeDecl` nodes and `ImplItem` children for `AssocTypeBinding` nodes:

```rust
// In lower_cap_decl:
let mut assoc_types = Vec::new();
let mut operations = Vec::new();
for item in cap_node.items() {
    match item {
        CapItem::AssocTypeDecl(a) => {
            if let Some(name) = a.name() { assoc_types.push(name.text().to_owned()); }
        }
        CapItem::OperationDecl(op) => {
            operations.push(lower_operation_decl(&op));
        }
    }
}

// In lower_impl_decl:
let mut assoc_types = Vec::new();
let mut methods = Vec::new();
for item in impl_node.items() {
    match item {
        ImplItem::AssocTypeBinding(b) => {
            if let (Some(name), Some(ty)) = (b.name(), b.ty()) {
                assoc_types.push((name.text().to_owned(), lower_type_expr(&ty)));
            }
        }
        ImplItem::ImplMethod(m) => methods.push(lower_impl_method(&m)),
    }
}
```

Adjust accessor names to match the generated `ast.rs` exactly.

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
git add crates/compiler/lumo.langue crates/lst/src/ crates/hir/src/lib.rs crates/hir/src/from_cst.rs crates/lir/src/lib.rs
git commit -m "feat: add assoc_types fields to CapDecl/ImplDecl in HIR/LIR and wire from_cst.rs"
```

---

## Task 4 — Update `from_hir_cst.rs` for assoc type items in HIR roundtrip

The HIR roundtrip tests use `from_hir_cst.rs` to re-parse HIR print form. Since `hir::print` will now emit `type Item` / `type Item = T` lines in cap/impl blocks, the HIR grammar and walker need updating.

**Files:**
- Modify: `crates/hir/hir.langue`
- Modify: `crates/hir/src/from_hir_cst.rs`

- [ ] **Step 1: Add `CapItem` and `ImplItem` to `crates/hir/hir.langue`**

In `hir.langue`, update `CapDecl` and `ImplDecl` grammar rules to use item lists (matching what `hir::print` will emit after Task 3):

```
CapDecl = 'cap' name:Ident '{' items:CapItem* '}'
CapItem =
  | AssocTypeDecl
  | OperationDecl
AssocTypeDecl = 'type' name:Ident

ImplDecl = 'impl' generic_params:GenericParams? name:Ident? target:TypeExpr (':' cap:TypeExpr)? '{' items:ImplItem* '}'
ImplItem =
  | AssocTypeBinding
  | ImplMethodDecl
AssocTypeBinding = 'type' name:Ident '=' ty:TypeExpr
```

- [ ] **Step 2: Regenerate HIR lossless**

```bash
bash scripts/gen_langue.sh hir
```

Expected: `crates/hir/src/syntax_kind.rs`, `ast.rs`, `lossless.rs` updated.

- [ ] **Step 3: Update `from_hir_cst.rs` to handle new item nodes**

In `from_hir_cst.rs`, update the cap and impl walkers to iterate `CapItem` / `ImplItem` children (same pattern as Task 3's `from_cst.rs` update).

- [ ] **Step 4: Update `hir::print` to emit assoc type lines**

In `crates/hir/src/print.rs`, in the cap/impl printers, emit `type Name` for each assoc type in `cap.assoc_types` and `type Name = TypeExpr` for each binding in `impl.assoc_types`.

- [ ] **Step 5: Run HIR roundtrip tests**

```bash
source ~/.cargo/env && cargo test -p lumo-compiler hir_roundtrip 2>&1 | tail -20
```

Expected: all pass. Fix `from_hir_cst.rs` or `print.rs` if any fail.

- [ ] **Step 6: Run full suite**

```bash
source ~/.cargo/env && cargo test --workspace 2>&1 | grep -E "FAILED|error\[" | head -10
```

Expected: no failures.

- [ ] **Step 7: Commit**

```bash
git add crates/hir/hir.langue crates/hir/src/
git commit -m "feat(hir): add assoc type items to HIR grammar, from_hir_cst, and print"
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
