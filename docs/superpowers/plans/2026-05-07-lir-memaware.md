# LIR / LIR-Memaware Split Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the LIR into a pure-functional `lir` crate and a memory-aware `lir-memaware` crate with explicit `Dup`/`Drop`/`IsUnique` nodes, wiring the elaboration pass into the compiler pipeline.

**Architecture:** `crates/lir` stays untouched. A new `crates/lir-memaware` crate defines `Expr { Pure(lir::Expr), Dup, Drop, IsUnique }` and re-declares only the types that contain `Expr` (`FnDecl`, `ImplMethodDecl`, `ImplDecl`, `Item`, `File`). The elaboration pass (`crates/compiler/src/elaborate.rs`) converts `lir::File → lir_memaware::File`, initially wrapping all function bodies in `Pure(...)`. Backends are updated to accept `lir_memaware::File` via a thin `lower_memaware_expr` dispatch layer; `Dup`/`Drop` are no-ops today.

**Tech Stack:** Rust, Cargo workspaces, existing `crates/lir`, `crates/compiler`, `crates/lbs`.

---

### Task 1: Create `crates/lir-memaware`

**Files:**
- Create: `crates/lir-memaware/Cargo.toml`
- Create: `crates/lir-memaware/src/lib.rs`
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Create crate directory and Cargo.toml**

```toml
# crates/lir-memaware/Cargo.toml
[package]
name = "lumo-lir-memaware"
version = "0.1.0"
edition = "2021"

[lib]
name = "lumo_lir_memaware"
path = "src/lib.rs"

[dependencies]
lumo-span  = { path = "../span" }
lumo-types = { path = "../types" }
lumo-hir   = { path = "../hir" }
lumo-lir   = { path = "../lir" }
```

- [ ] **Step 2: Write `crates/lir-memaware/src/lib.rs`**

```rust
use lumo_lir as lir;
use lumo_span::Span;
use lumo_types::{CapRef, ContentHash, ExprId, Spanned, TypeExpr};
pub use lumo_hir::GenericParam;

// Re-export all lir types that do NOT contain Expr.
pub use lir::{
    AsRawValue, BundleEntry, CapDecl, DataDecl, ExternFnDecl, ExternTypeDecl,
    MatchArm, OperationDecl, Param, UseDecl, VariantDecl,
};

// ---------------------------------------------------------------------------
// Memory-aware expression type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// A pure sub-tree from the functional LIR — no RC operations inside.
    Pure(lir::Expr),

    /// Increment the refcount before this use (all non-consuming uses of a
    /// binding that is referenced N ≥ 2 times).
    Dup { id: ExprId, expr: Box<Expr> },

    /// Release `name` before evaluating `body` (binding used 0 times).
    Drop { id: ExprId, name: String, body: Box<Expr> },

    /// FBIP branch: unique ownership → `unique_branch`, shared → `shared_branch`.
    /// Not inserted by the elaboration pass; reserved for a future FBIP pass.
    IsUnique {
        id: ExprId,
        expr: Box<Expr>,
        unique_branch: Box<Expr>,
        shared_branch: Box<Expr>,
    },
}

// ---------------------------------------------------------------------------
// Types re-declared to swap in lir_memaware::Expr for lir::Expr
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub cap: Option<CapRef>,
    pub value: Expr,
    pub inline: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplMethodDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplDecl {
    pub name: Option<String>,
    pub generics: Vec<GenericParam>,
    pub target_type: Spanned<TypeExpr>,
    pub capability: Option<Spanned<TypeExpr>>,
    pub assoc_types: Vec<(String, Spanned<TypeExpr>)>,
    pub methods: Vec<ImplMethodDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    ExternType(lir::ExternTypeDecl),
    ExternFn(lir::ExternFnDecl),
    Data(lir::DataDecl),
    Cap(lir::CapDecl),
    Fn(FnDecl),
    Use(lir::UseDecl),
    Impl(ImplDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub items: Vec<Item>,
    pub content_hash: ContentHash,
    pub spans: Vec<Span>,
}
```

- [ ] **Step 3: Add crate to workspace**

In the root `Cargo.toml`, add `"crates/lir-memaware"` to the `members` array:

```toml
[workspace]
members = [
    "crates/span",
    "crates/types",
    "crates/lexer",
    "crates/lst",
    "crates/hir",
    "crates/lir",
    "crates/lir-memaware",
    "crates/compiler",
    "crates/lbs",
    "crates/lsp",
    "crates/playground-wasm",
    "crates/simple-ts-ast",
]
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo build -p lumo-lir-memaware
```

Expected: compiles with no errors.

- [ ] **Step 5: Commit**

```bash
git add crates/lir-memaware/ Cargo.toml
git commit -m "feat(lir-memaware): new crate — Pure/Dup/Drop/IsUnique Expr + wrapper types"
```

---

### Task 2: Elaboration pass (trivial — all Pure)

**Files:**
- Create: `crates/compiler/src/elaborate.rs`
- Modify: `crates/compiler/Cargo.toml`
- Modify: `crates/compiler/src/lib.rs`
- Create: `crates/compiler/tests/elaborate.rs`

- [ ] **Step 1: Add dependency to compiler**

In `crates/compiler/Cargo.toml`:

```toml
[dependencies]
# ... existing deps ...
lumo-lir-memaware = { path = "../lir-memaware" }
```

- [ ] **Step 2: Re-export from `lib.rs`**

In `crates/compiler/src/lib.rs`, add:

```rust
pub use lumo_lir_memaware as lir_memaware;
pub mod elaborate;
```

- [ ] **Step 3: Write the failing test**

Create `crates/compiler/tests/elaborate.rs`:

```rust
use lumo_compiler::{elaborate, hir, lir, lir_memaware};

fn parse_and_lower(src: &str) -> lir::File {
    let lossless = lumo_compiler::lst::lossless::parse(src);
    let hir = hir::lower_lossless(&lossless);
    lir::lower(&hir)
}

#[test]
fn elaborate_trivial_fn() {
    let lir_file = parse_and_lower("fn id(x: Number): Number / {} { x }");
    let mem_file = elaborate::elaborate(&lir_file);

    // There should be exactly one Fn item.
    let fn_decl = mem_file.items.iter().find_map(|i| {
        if let lir_memaware::Item::Fn(f) = i { Some(f) } else { None }
    }).expect("no Fn item");

    assert_eq!(fn_decl.name, "id");

    // The body should be Pure wrapping the original lir expr.
    assert!(
        matches!(&fn_decl.value, lir_memaware::Expr::Pure(_)),
        "expected Pure(…), got {:?}", fn_decl.value
    );
}

#[test]
fn elaborate_impl_method() {
    let src = r#"
        data List[A] { .nil, .cons(A, List[A]) }
        impl[T] List[T] { fn len(self: List[T]): Number / {} { 0 } }
    "#;
    let lir_file = parse_and_lower(src);
    let mem_file = elaborate::elaborate(&lir_file);

    let impl_decl = mem_file.items.iter().find_map(|i| {
        if let lir_memaware::Item::Impl(d) = i { Some(d) } else { None }
    }).expect("no Impl item");

    let method = impl_decl.methods.first().expect("no methods");
    assert!(
        matches!(&method.value, lir_memaware::Expr::Pure(_)),
        "expected Pure(…), got {:?}", method.value
    );
}
```

- [ ] **Step 4: Run test to verify it fails**

```bash
cargo test --test elaborate 2>&1 | tail -10
```

Expected: compile error — `elaborate` module does not exist.

- [ ] **Step 5: Write `crates/compiler/src/elaborate.rs`**

```rust
use crate::{lir, lir_memaware};

/// Convert a pure-functional `lir::File` into a `lir_memaware::File`.
///
/// This trivial pass wraps every function/method body in `Pure(…)`.
/// Dup/Drop insertion is added in a subsequent pass once usage analysis is in place.
pub fn elaborate(file: &lir::File) -> lir_memaware::File {
    lir_memaware::File {
        content_hash: file.content_hash.clone(),
        spans: file.spans.clone(),
        items: file.items.iter().map(elaborate_item).collect(),
    }
}

fn elaborate_item(item: &lir::Item) -> lir_memaware::Item {
    match item {
        lir::Item::Fn(f)         => lir_memaware::Item::Fn(elaborate_fn(f)),
        lir::Item::Impl(i)       => lir_memaware::Item::Impl(elaborate_impl(i)),
        lir::Item::ExternType(e) => lir_memaware::Item::ExternType(e.clone()),
        lir::Item::ExternFn(e)   => lir_memaware::Item::ExternFn(e.clone()),
        lir::Item::Data(d)       => lir_memaware::Item::Data(d.clone()),
        lir::Item::Cap(c)        => lir_memaware::Item::Cap(c.clone()),
        lir::Item::Use(u)        => lir_memaware::Item::Use(u.clone()),
    }
}

fn elaborate_fn(f: &lir::FnDecl) -> lir_memaware::FnDecl {
    lir_memaware::FnDecl {
        name:        f.name.clone(),
        generics:    f.generics.clone(),
        params:      f.params.clone(),
        return_type: f.return_type.clone(),
        cap:         f.cap.clone(),
        inline:      f.inline,
        span:        f.span,
        value:       lir_memaware::Expr::Pure(f.value.clone()),
    }
}

fn elaborate_impl(i: &lir::ImplDecl) -> lir_memaware::ImplDecl {
    lir_memaware::ImplDecl {
        name:         i.name.clone(),
        generics:     i.generics.clone(),
        target_type:  i.target_type.clone(),
        capability:   i.capability.clone(),
        assoc_types:  i.assoc_types.clone(),
        span:         i.span,
        methods:      i.methods.iter().map(elaborate_method).collect(),
    }
}

fn elaborate_method(m: &lir::ImplMethodDecl) -> lir_memaware::ImplMethodDecl {
    lir_memaware::ImplMethodDecl {
        name:        m.name.clone(),
        params:      m.params.clone(),
        return_type: m.return_type.clone(),
        span:        m.span,
        value:       lir_memaware::Expr::Pure(m.value.clone()),
    }
}
```

- [ ] **Step 6: Run test to verify it passes**

```bash
cargo test --test elaborate
```

Expected:
```
test elaborate_trivial_fn ... ok
test elaborate_impl_method ... ok
```

- [ ] **Step 7: Commit**

```bash
git add crates/compiler/Cargo.toml crates/compiler/src/elaborate.rs \
        crates/compiler/src/lib.rs crates/compiler/tests/elaborate.rs
git commit -m "feat(compiler): trivial elaboration pass lir → lir_memaware (all Pure)"
```

---

### Task 3: Update backends to accept `lir_memaware::File`

**Files:**
- Modify: `crates/compiler/src/backend/mod.rs`
- Modify: `crates/compiler/src/backend/ts.rs`
- Modify: `crates/compiler/src/backend/rs.rs`
- Modify: `crates/lbs/src/main.rs`

The strategy: add a one-line `lower_memaware_expr` / `emit_memaware_expr` dispatch in each backend that handles the four `lir_memaware::Expr` variants and delegates `Pure(e)` to the existing `lower_expr(e)` / `emit_expr(e)`. Only `lower_fn_decl`, `lower_impl_const`, and their `rs.rs` equivalents change signatures; all inner logic stays the same.

- [ ] **Step 1: Update `backend/mod.rs`**

Change the `Backend` trait and `emit` helper to take `lir_memaware::File`:

```rust
// crates/compiler/src/backend/mod.rs
use crate::{lir, lir_memaware};
// ... existing imports ...

pub trait Backend: Send + Sync {
    fn supports(&self, target: CodegenTarget) -> bool;
    fn emit(&self, file: &lir_memaware::File, target: CodegenTarget) -> Result<String, BackendError>;
}

// ... Emitter struct unchanged ...

impl Emitter {
    pub fn emit(&self, file: &lir_memaware::File, target: CodegenTarget) -> Result<String, BackendError> {
        let backend = self
            .backends
            .iter()
            .find(|b| b.supports(target))
            .ok_or_else(|| BackendError::new(format!("no backend for {target:?}")))?;
        backend.emit(file, target)
    }
}

pub fn emit(file: &lir_memaware::File, target: CodegenTarget) -> Result<String, BackendError> {
    Emitter::with_defaults().emit(file, target)
}
```

- [ ] **Step 2: Add `lower_memaware_expr` to `ts.rs`**

At the top of the `impl LoweringContext` block (near where `lower_expr` is defined), add:

```rust
/// Dispatch a memory-aware expression. Pure(e) delegates to the existing
/// lower_expr; Dup/Drop/IsUnique are no-ops in the JS backend today.
fn lower_memaware_expr(
    &self,
    expr: &lir_memaware::Expr,
    env: &[&str],
    handled: &std::collections::HashSet<String>,
) -> tsast::Expr {
    match expr {
        lir_memaware::Expr::Pure(e) => self.lower_expr(e, env, handled),
        lir_memaware::Expr::Dup { expr, .. } => self.lower_memaware_expr(expr, env, handled),
        lir_memaware::Expr::Drop { body, .. } => self.lower_memaware_expr(body, env, handled),
        lir_memaware::Expr::IsUnique { shared_branch, .. } => {
            self.lower_memaware_expr(shared_branch, env, handled)
        }
    }
}
```

- [ ] **Step 3: Update `ts.rs` signatures for `lower_file`, `lower_fn_decl`, `lower_impl_const`**

Change every function that takes `&lir::File`, `&lir::FnDecl`, or `&lir::ImplDecl` to take the corresponding `lir_memaware` type. Inside these functions, replace calls to `self.lower_expr(&func.value, ...)` with `self.lower_memaware_expr(&func.value, ...)`.

The signature changes (search for these exact lines and update):

```rust
// was: fn lower_file(&self, file: &lir::File) -> Result<tsast::Program, BackendError>
fn lower_file(&self, file: &lir_memaware::File) -> Result<tsast::Program, BackendError>

// was: fn lower_fn_decl(func: &lir::FnDecl, ...) -> ...
fn lower_fn_decl(func: &lir_memaware::FnDecl, ...) -> ...

// was: fn lower_impl_const(impl_decl: &lir::ImplDecl, ...) -> ...
fn lower_impl_const(impl_decl: &lir_memaware::ImplDecl, ...) -> ...
```

Also update the four helper functions that take `&lir::File` for scanning purposes (they iterate items to build lookup maps). Change them to iterate `lir_memaware::Item` and match accordingly — `lir_memaware::Item::Fn(f)` still gives a `lir_memaware::FnDecl` whose `name`, `params`, `cap` fields are identical to before.

For the item-scanning helpers (`collect_direct_callable_arities`, `collect_impl_method_arities`, `collect_as_raw_variants`, `collect_default_impls`, `collect_fn_caps_map`), update their parameter from `&lir::File` to `&lir_memaware::File` and change `lir::Item::Fn(f)` → `lir_memaware::Item::Fn(f)`, `lir::Item::Impl(i)` → `lir_memaware::Item::Impl(i)`, etc. All other variants (`ExternFn`, `Data`, `Cap`, `Use`) are still the same lir types so those arms are unchanged.

Update the `Backend::emit` impl for `TypeScriptBackend`:

```rust
impl Backend for TypeScriptBackend {
    fn supports(&self, target: CodegenTarget) -> bool { ... } // unchanged

    fn emit(&self, file: &lir_memaware::File, target: CodegenTarget) -> Result<String, BackendError> {
        // unchanged body — calls self.lower_file(file)
    }
}
```

- [ ] **Step 4: Add `emit_memaware_expr` to `rs.rs`**

Near the existing `emit_expr` function, add:

```rust
fn emit_memaware_expr(expr: &lir_memaware::Expr, ctx: &LoweringContext) -> String {
    match expr {
        lir_memaware::Expr::Pure(e) => emit_expr(e, ctx),
        lir_memaware::Expr::Dup { expr, .. } => emit_memaware_expr(expr, ctx),
        lir_memaware::Expr::Drop { body, .. } => emit_memaware_expr(body, ctx),
        lir_memaware::Expr::IsUnique { shared_branch, .. } => {
            emit_memaware_expr(shared_branch, ctx)
        }
    }
}
```

- [ ] **Step 5: Update `rs.rs` signatures**

Same approach as ts.rs. Change `emit_file`, `emit_fn_decl`, `emit_impl_decl`, `emit_main_fn`, and the `LoweringContext::from_file` to take `lir_memaware` types. Replace `emit_expr(&func.value, ctx)` with `emit_memaware_expr(&func.value, ctx)` in those functions.

```rust
// was: fn emit_file(file: &lir::File) -> ...
fn emit_file(file: &lir_memaware::File) -> ...

// was: fn emit_fn_decl(func: &lir::FnDecl, ...) -> ...
fn emit_fn_decl(func: &lir_memaware::FnDecl, ...) -> ...

// was: fn emit_impl_decl(impl_decl: &lir::ImplDecl, ...) -> ...
fn emit_impl_decl(impl_decl: &lir_memaware::ImplDecl, ...) -> ...

// was: fn emit_main_fn(func: &lir::FnDecl, ...) -> ...
fn emit_main_fn(func: &lir_memaware::FnDecl, ...) -> ...

// was: fn from_file(file: &lir::File) -> Self
fn from_file(file: &lir_memaware::File) -> Self
```

Update `Backend::emit` impl for `RustBackend` similarly.

- [ ] **Step 6: Wire elaboration into `lbs/src/main.rs`**

In `crates/lbs/src/main.rs`, import the elaborate module and call it before `backend::emit`:

```rust
use lumo_compiler::{backend, elaborate, lir_memaware};
// ... in build_js and build_rust, change:

fn build_js(manifest: &manifest::Manifest, lir: &lir::File) {
    let mem = elaborate::elaborate(lir);
    let js = match backend::emit(&mem, CodegenTarget::JavaScript) {
        // ... rest unchanged
    };
    // ... rest unchanged
}

fn build_rust(manifest: &manifest::Manifest, lir: &lir::File) {
    let mem = elaborate::elaborate(lir);
    let rs_code = match backend::emit(&mem, CodegenTarget::Rust) {
        // ... rest unchanged
    };
    // ... rest unchanged
}
```

- [ ] **Step 7: Compile and fix any remaining type errors**

```bash
cargo build --workspace 2>&1 | grep "^error" | head -20
```

Fix any type errors from missed `lir::` → `lir_memaware::` references. Common pattern: any `lir::Item::Fn(f)` in a match on a `lir_memaware::File`'s items needs to become `lir_memaware::Item::Fn(f)`.

- [ ] **Step 8: Run full test suite**

```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED"
```

Expected: all test results `ok`, no `FAILED`.

- [ ] **Step 9: Commit**

```bash
git add crates/compiler/src/backend/ crates/lbs/src/main.rs
git commit -m "feat(backend): target lir_memaware::File; Dup/Drop/IsUnique are no-ops today"
```

---

### Task 4: Dup/Drop insertion in the elaboration pass

**Files:**
- Modify: `crates/compiler/src/elaborate.rs`
- Modify: `crates/compiler/tests/elaborate.rs`

- [ ] **Step 1: Write the failing tests**

Add to `crates/compiler/tests/elaborate.rs`:

```rust
use lumo_compiler::lir_memaware::{Expr, Item};

fn has_dup(expr: &Expr) -> bool {
    match expr {
        Expr::Pure(_) => false,
        Expr::Dup { .. } => true,
        Expr::Drop { body, .. } => has_dup(body),
        Expr::IsUnique { unique_branch, shared_branch, .. } => {
            has_dup(unique_branch) || has_dup(shared_branch)
        }
    }
}

fn has_drop(expr: &Expr) -> bool {
    match expr {
        Expr::Pure(_) => false,
        Expr::Drop { .. } => true,
        Expr::Dup { expr, .. } => has_drop(expr),
        Expr::IsUnique { unique_branch, shared_branch, .. } => {
            has_drop(unique_branch) || has_drop(shared_branch)
        }
    }
}

#[test]
fn elaborate_no_dup_for_single_use() {
    // `x` is used exactly once — no Dup should appear.
    let lir_file = parse_and_lower("fn id(x: Number): Number / {} { x }");
    let mem = lumo_compiler::elaborate::elaborate(&lir_file);
    let fn_decl = mem.items.iter().find_map(|i| {
        if let Item::Fn(f) = i { Some(f) } else { None }
    }).unwrap();
    assert!(!has_dup(&fn_decl.value), "single-use binding must not be Dup'd");
}

#[test]
fn elaborate_dup_for_repeated_binding() {
    // Build a Let node where `x` appears twice in the body directly via LIR.
    use lumo_lir as lir;
    use lumo_types::{ExprId, ContentHash};

    // let x = 1 in (x, x)  — represented as nested Apply using x twice
    let body = lir::Expr::Apply {
        id: ExprId(3),
        callee: Box::new(lir::Expr::Apply {
            id: ExprId(4),
            callee: Box::new(lir::Expr::Ident { id: ExprId(5), name: "x".to_owned() }),
            arg:    Box::new(lir::Expr::Ident { id: ExprId(6), name: "x".to_owned() }),
        }),
        arg: Box::new(lir::Expr::Number { id: ExprId(7), value: "0".to_owned() }),
    };
    let let_expr = lir::Expr::Let {
        id: ExprId(0),
        name: "x".to_owned(),
        value: Box::new(lir::Expr::Number { id: ExprId(1), value: "1".to_owned() }),
        body: Box::new(body),
    };
    let file = lir::File {
        items: vec![lir::Item::Fn(lir::FnDecl {
            name: "f".to_owned(), generics: vec![], params: vec![],
            return_type: None, cap: None, inline: false,
            span: lumo_span::Span::default(), value: let_expr,
        })],
        content_hash: ContentHash::default(),
        spans: (0..8).map(|_| lumo_span::Span::default()).collect(),
    };
    let mem = lumo_compiler::elaborate::elaborate(&file);
    let fn_decl = mem.items.iter().find_map(|i| {
        if let Item::Fn(f) = i { Some(f) } else { None }
    }).unwrap();
    assert!(has_dup(&fn_decl.value), "binding used twice must produce a Dup node");
}

#[test]
fn elaborate_drop_for_unused_binding() {
    // `_unused` is bound but never referenced — should get a Drop node.
    // We test this by looking at any let-binding in the elaborated body.
    // Direct source test requires let-in syntax: `let _x = 1 in 2`
    // Use the LIR directly: build a Let node manually.
    use lumo_lir as lir;
    use lumo_compiler::lir_memaware;
    use lumo_types::{ExprId, ContentHash};

    let unused_let = lir::Expr::Let {
        id: ExprId(0),
        name: "_unused".to_owned(),
        value: Box::new(lir::Expr::Number { id: ExprId(1), value: "1".to_owned() }),
        body:  Box::new(lir::Expr::Number { id: ExprId(2), value: "2".to_owned() }),
    };
    // Wrap in a minimal FnDecl
    let fn_decl = lir::FnDecl {
        name: "f".to_owned(),
        generics: vec![],
        params: vec![],
        return_type: None,
        cap: None,
        inline: false,
        span: lumo_span::Span::default(),
        value: unused_let,
    };
    let file = lir::File {
        items: vec![lir::Item::Fn(fn_decl)],
        content_hash: ContentHash::default(),
        spans: vec![
            lumo_span::Span::default(),
            lumo_span::Span::default(),
            lumo_span::Span::default(),
        ],
    };
    let mem = lumo_compiler::elaborate::elaborate(&file);
    let mfn = mem.items.iter().find_map(|i| {
        if let lir_memaware::Item::Fn(f) = i { Some(f) } else { None }
    }).unwrap();
    assert!(has_drop(&mfn.value), "unused binding should produce a Drop node");
}
```

- [ ] **Step 2: Run to verify tests fail**

```bash
cargo test --test elaborate elaborate_drop_for_unused_binding 2>&1 | tail -5
```

Expected: `FAILED` — `has_drop` returns false because elaboration is still trivial.

- [ ] **Step 3: Add usage counting to `elaborate.rs`**

Add a helper that counts syntactic references to a name in a `lir::Expr` subtree:

```rust
fn count_uses(expr: &lir::Expr, name: &str) -> usize {
    match expr {
        lir::Expr::Ident { name: n, .. } => usize::from(n == name),
        lir::Expr::Let { name: n, value, body, .. } => {
            let in_value = count_uses(value, name);
            // If the let re-binds `name`, it shadows the outer binding.
            let in_body = if n == name { 0 } else { count_uses(body, name) };
            in_value + in_body
        }
        lir::Expr::Lambda { param, body, .. } => {
            if param == name { 0 } else { count_uses(body, name) }
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            count_uses(scrutinee, name)
                + arms.iter().map(|a| count_uses(&a.body, name)).sum::<usize>()
        }
        lir::Expr::Apply { callee, arg, .. } => count_uses(callee, name) + count_uses(arg, name),
        lir::Expr::Force { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Thunk { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Ann { expr, .. } => count_uses(expr, name),
        lir::Expr::Ctor { args, .. } => args.iter().map(|a| count_uses(a, name)).sum(),
        lir::Expr::Bundle { entries, .. } => {
            entries.iter().map(|e| {
                // Bundle entry params shadow the outer binding.
                if e.params.iter().any(|p| p.name == name) {
                    0
                } else {
                    count_uses(&e.body, name)
                }
            }).sum()
        }
        lir::Expr::Handle { handler, body, .. } => {
            count_uses(handler, name) + count_uses(body, name)
        }
        lir::Expr::Member { object, .. } => count_uses(object, name),
        lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Perform { .. }
        | lir::Expr::Error { .. } => 0,
    }
}
```

**Scope note:** `lir_memaware::Expr` has no `Let` variant — it can only wrap `lir::Expr` subtrees in `Pure(...)`. This means Dup/Drop can only be inserted at the *outermost* level of a function body, not inside compound sub-expressions. Fine-grained Dup/Drop (e.g. inside a `Let` body) requires adding compound variants to `lir_memaware::Expr`; that is a separate future task. This task implements the tractable subset: `Drop` at the function-body level for entirely-unused top-level `Let` bindings, and `Dup` when the direct body of a `Let` is a bare `Ident` used multiple times.

- [ ] **Step 4: Add `elaborate_expr` to `elaborate.rs`**

Replace the `Pure(f.value.clone())` in `elaborate_fn` / `elaborate_method` with a call to this function:

```rust
/// Elaborate the top-level of a function body.
/// Handles Drop for unused Let-bindings and Dup for bare repeated Idents at
/// the Let-body level. Deeper insertion requires compound variants (future task).
fn elaborate_expr(expr: &lir::Expr) -> lir_memaware::Expr {
    match expr {
        lir::Expr::Let { id, name, value, body } => {
            let uses = count_uses(body, name);
            if uses == 0 {
                // Unused binding: evaluate value (side-effect free in lir),
                // wrap continuation in Drop, continue elaborating body.
                lir_memaware::Expr::Drop {
                    id: *id,
                    name: name.clone(),
                    body: Box::new(elaborate_expr(body)),
                }
            } else if uses >= 2 {
                // Binding used N≥2 times: wrap a Pure Let whose body retains
                // the original structure; insert Dup around the whole value
                // so the owner keeps a copy before the Let consumes it.
                // (Deep per-use Dup requires a future compound-variant pass.)
                lir_memaware::Expr::Dup {
                    id: *id,
                    expr: Box::new(lir_memaware::Expr::Pure(expr.clone())),
                }
            } else {
                // Single use: pure.
                lir_memaware::Expr::Pure(expr.clone())
            }
        }
        // All other top-level forms stay Pure; deeper RC insertion is a future task.
        _ => lir_memaware::Expr::Pure(expr.clone()),
    }
}
```

Update `elaborate_fn` and `elaborate_method`:

```rust
fn elaborate_fn(f: &lir::FnDecl) -> lir_memaware::FnDecl {
    lir_memaware::FnDecl {
        name:        f.name.clone(),
        generics:    f.generics.clone(),
        params:      f.params.clone(),
        return_type: f.return_type.clone(),
        cap:         f.cap.clone(),
        inline:      f.inline,
        span:        f.span,
        value:       elaborate_expr(&f.value),
    }
}

fn elaborate_method(m: &lir::ImplMethodDecl) -> lir_memaware::ImplMethodDecl {
    lir_memaware::ImplMethodDecl {
        name:        m.name.clone(),
        params:      m.params.clone(),
        return_type: m.return_type.clone(),
        span:        m.span,
        value:       elaborate_expr(&m.value),
    }
}
```

- [ ] **Step 5: Run tests**

```bash
cargo test --test elaborate 2>&1 | grep -E "ok|FAILED"
```

Expected: all tests pass.

- [ ] **Step 6: Run full workspace tests**

```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED"
```

Expected: all `ok`, no `FAILED`.

- [ ] **Step 7: Commit**

```bash
git add crates/compiler/src/elaborate.rs crates/compiler/tests/elaborate.rs
git commit -m "feat(elaborate): Dup insertion for repeated bindings, Drop for unused bindings"
```
