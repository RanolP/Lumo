use crate::{
    BundleEntry, CapDecl, DataDecl, ExternFnDecl, ExternTypeDecl, Expr, File, FnDecl, ImplDecl,
    ImplMethodDecl, Item, MatchArm,
};
use lumo_span::Span;
use lumo_types::{CapRef, Pattern, TypeExpr};
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckError {
    pub span: Span,
    pub message: String,
}

pub fn check_file(file: &File) -> Vec<CheckError> {
    let mut ctx = CheckCtx::new();
    ctx.collect_declarations(file);
    ctx.check_items(file);
    ctx.errors
}

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct DataInfo {
    generics: usize,
    variants: HashMap<String, usize>, // variant name → payload arity
    has_as_raw: bool,                 // at least one variant carries `#[as__raw]`
}

struct FnInfo {
    arity: usize,
}

struct CapInfo {
    operations: HashMap<String, usize>, // op name → param count
}

struct CheckCtx {
    errors: Vec<CheckError>,
    types: HashSet<String>,   // known type names (extern type + data)
    fns: HashMap<String, FnInfo>, // known fn names → info
    data: HashMap<String, DataInfo>,
    caps: HashMap<String, CapInfo>,
    // All variant names across all data types: variant_name → data_name
    variant_owner: HashMap<String, String>,
}

impl CheckCtx {
    fn new() -> Self {
        let mut types = HashSet::new();
        // Built-in types always available
        types.insert("Unit".to_string());
        Self {
            errors: Vec::new(),
            types,
            fns: HashMap::new(),
            data: HashMap::new(),
            caps: HashMap::new(),
            variant_owner: HashMap::new(),
        }
    }

    fn error(&mut self, span: Span, message: String) {
        self.errors.push(CheckError { span, message });
    }

    // -------------------------------------------------------------------
    // Phase 1: Collect all top-level declarations
    // -------------------------------------------------------------------

    fn collect_declarations(&mut self, file: &File) {
        for item in &file.items {
            match item {
                Item::ExternType(ext) => self.collect_extern_type(ext),
                Item::ExternFn(ext) => self.collect_extern_fn(ext),
                Item::Data(data) => self.collect_data(data),
                Item::Cap(cap) => self.collect_cap(cap),
                Item::Fn(func) => self.collect_fn(func),
                Item::Use(_) | Item::Impl(_) => {} // handled in check phase
            }
        }
    }

    fn collect_extern_type(&mut self, ext: &ExternTypeDecl) {
        if !self.types.insert(ext.name.clone()) {
            self.error(ext.span, format!("duplicate type `{}`", ext.name));
        }
    }

    fn collect_extern_fn(&mut self, ext: &ExternFnDecl) {
        if self.fns.contains_key(&ext.name) {
            self.error(ext.span, format!("duplicate function `{}`", ext.name));
        } else {
            self.fns.insert(
                ext.name.clone(),
                FnInfo {
                    arity: ext.params.len(),
                },
            );
        }
    }

    fn collect_data(&mut self, data: &DataDecl) {
        if !self.types.insert(data.name.clone()) {
            // Allow a duplicate `data X` decl when the new one (or the
            // previously seen one) carries `#[as__raw]` on any variant.
            // This supports platform-specific overrides where `src#js/`
            // redefines a common `data X` with raw-literal variants.
            let new_has_as_raw = data.variants.iter().any(|v| v.as_raw.is_some());
            let prev_has_as_raw = self
                .data
                .get(&data.name)
                .map_or(false, |info| info.has_as_raw);
            if !new_has_as_raw && !prev_has_as_raw {
                self.error(data.span, format!("duplicate type `{}`", data.name));
                return;
            }
            // If the new decl has `as_raw`, it replaces the previous one in
            // the analysis tables (the HIR merge pass also prefers it).
            if new_has_as_raw {
                if let Some(prev) = self.data.get(&data.name).cloned() {
                    for vname in prev.variants.keys() {
                        self.variant_owner.remove(vname);
                    }
                }
                self.data.remove(&data.name);
            } else {
                // Previous decl has as_raw, keep it; skip the new one.
                return;
            }
        }
        let mut variants = HashMap::new();
        for v in &data.variants {
            if variants.contains_key(&v.name) {
                self.error(v.span, format!("duplicate variant `.{}`", v.name));
            } else {
                variants.insert(v.name.clone(), v.payload.len());
                self.variant_owner
                    .insert(v.name.clone(), data.name.clone());
            }
        }
        let has_as_raw = data.variants.iter().any(|v| v.as_raw.is_some());
        self.data.insert(
            data.name.clone(),
            DataInfo {
                generics: data.generics.len(),
                variants,
                has_as_raw,
            },
        );
    }

    fn collect_cap(&mut self, cap: &CapDecl) {
        if self.caps.contains_key(&cap.name) {
            self.error(cap.span, format!("duplicate capability `{}`", cap.name));
            return;
        }
        let mut operations = HashMap::new();
        for op in &cap.operations {
            if operations.contains_key(&op.name) {
                self.error(op.span, format!("duplicate operation `{}`", op.name));
            } else {
                operations.insert(op.name.clone(), op.params.len());
            }
        }
        self.caps.insert(cap.name.clone(), CapInfo { operations });
    }

    fn collect_fn(&mut self, func: &FnDecl) {
        if self.fns.contains_key(&func.name) {
            self.error(func.span, format!("duplicate function `{}`", func.name));
        } else {
            self.fns.insert(
                func.name.clone(),
                FnInfo {
                    arity: func.params.len(),
                },
            );
        }
    }

    // -------------------------------------------------------------------
    // Phase 2: Check all items
    // -------------------------------------------------------------------

    fn check_items(&mut self, file: &File) {
        for item in &file.items {
            match item {
                Item::ExternType(_) => {} // already collected
                Item::ExternFn(ext) => self.check_extern_fn(ext),
                Item::Data(data) => self.check_data(data),
                Item::Cap(cap) => self.check_cap(cap),
                Item::Fn(func) => self.check_fn(func),
                Item::Use(_) => {}
                Item::Impl(impl_decl) => self.check_impl(impl_decl),
            }
        }
    }

    fn check_extern_fn(&mut self, ext: &ExternFnDecl) {
        for param in &ext.params {
            self.check_type_expr(&param.ty.value, param.ty.span);
        }
        if let Some(ret) = &ext.return_type {
            self.check_type_expr(&ret.value, ret.span);
        }
        self.check_cap_ref(&ext.cap, ext.span);
    }

    fn check_data(&mut self, data: &DataDecl) {
        // Check variant payload types reference known types
        let generics: HashSet<&str> = data.generics.iter().map(|s| s.as_str()).collect();
        for v in &data.variants {
            for ty in &v.payload {
                self.check_type_expr_with_generics(&ty.value, ty.span, &generics);
            }
        }
    }

    fn check_cap(&mut self, cap: &CapDecl) {
        for op in &cap.operations {
            for param in &op.params {
                self.check_type_expr(&param.ty.value, param.ty.span);
            }
            if let Some(ret) = &op.return_type {
                self.check_type_expr(&ret.value, ret.span);
            }
        }
    }

    fn check_fn(&mut self, func: &FnDecl) {
        let generics: HashSet<&str> = func.generics.iter().map(|g| g.name()).collect();
        for param in &func.params {
            self.check_type_expr_with_generics(&param.ty.value, param.ty.span, &generics);
        }
        if let Some(ret) = &func.return_type {
            self.check_type_expr_with_generics(&ret.value, ret.span, &generics);
        }
        self.check_cap_ref(&func.cap, func.span);

        let mut locals: HashSet<String> = func.params.iter().map(|p| p.name.clone()).collect();
        self.check_expr(&func.body, &mut locals);
    }

    fn check_impl(&mut self, impl_decl: &ImplDecl) {
        let generics: HashSet<&str> = impl_decl.generics.iter().map(|g| g.name()).collect();
        self.check_type_expr_with_generics(
            &impl_decl.target_type.value,
            impl_decl.target_type.span,
            &generics,
        );
        if let Some(cap) = &impl_decl.capability {
            self.check_type_expr_with_generics(&cap.value, cap.span, &generics);
        }

        let mut method_names: HashSet<String> = HashSet::new();
        for method in &impl_decl.methods {
            if !method_names.insert(method.name.clone()) {
                self.error(method.span, format!("duplicate method `{}`", method.name));
            }
            self.check_impl_method(method, &generics);
        }
    }

    fn check_impl_method(&mut self, method: &ImplMethodDecl, generics: &HashSet<&str>) {
        for param in &method.params {
            self.check_type_expr_with_generics(&param.ty.value, param.ty.span, generics);
        }
        if let Some(ret) = &method.return_type {
            self.check_type_expr_with_generics(&ret.value, ret.span, generics);
        }
        let mut locals: HashSet<String> = method.params.iter().map(|p| p.name.clone()).collect();
        self.check_expr(&method.body, &mut locals);
    }

    // -------------------------------------------------------------------
    // Type expression checking
    // -------------------------------------------------------------------

    fn check_type_expr(&mut self, ty: &TypeExpr, span: Span) {
        self.check_type_expr_with_generics(ty, span, &HashSet::new());
    }

    fn check_type_expr_with_generics(
        &mut self,
        ty: &TypeExpr,
        span: Span,
        generics: &HashSet<&str>,
    ) {
        match ty {
            TypeExpr::Named(name) => {
                if !self.types.contains(name) && !generics.contains(name.as_str()) {
                    self.error(span, format!("unknown type `{name}`"));
                } else if let Some(data) = self.data.get(name) {
                    if data.generics > 0 {
                        self.error(
                            span,
                            format!(
                                "type `{name}` expects {} generic argument(s), got 0",
                                data.generics
                            ),
                        );
                    }
                }
            }
            TypeExpr::App { head, args } => {
                if !self.types.contains(head) && !generics.contains(head.as_str()) {
                    self.error(span, format!("unknown type `{head}`"));
                } else if let Some(data) = self.data.get(head) {
                    if args.len() != data.generics {
                        self.error(
                            span,
                            format!(
                                "type `{head}` expects {} generic argument(s), got {}",
                                data.generics,
                                args.len()
                            ),
                        );
                    }
                }
                for arg in args {
                    self.check_type_expr_with_generics(arg, span, generics);
                }
            }
            TypeExpr::Produce(inner) | TypeExpr::Thunk(inner) => {
                self.check_type_expr_with_generics(inner, span, generics);
            }
            TypeExpr::Cap { name, type_args } => {
                if !self.caps.contains_key(name) {
                    self.error(span, format!("unknown capability `{name}`"));
                }
                for arg in type_args {
                    self.check_type_expr_with_generics(arg, span, generics);
                }
            }
            TypeExpr::Fn { params, ret, .. } => {
                for p in params {
                    self.check_type_expr_with_generics(p, span, generics);
                }
                self.check_type_expr_with_generics(ret, span, generics);
            }
        }
    }

    fn check_cap_ref(&mut self, cap: &Option<CapRef>, span: Span) {
        use lumo_types::CapEntry;
        if let Some(entries) = cap {
            for entry in entries {
                if let CapEntry::Cap(ty) = entry {
                    let name = ty.cap_name();
                    if !self.caps.contains_key(name) {
                        self.error(span, format!("unknown capability `{name}`"));
                    }
                }
            }
        }
    }

    // -------------------------------------------------------------------
    // Expression checking
    // -------------------------------------------------------------------

    fn check_expr(&mut self, expr: &Expr, locals: &mut HashSet<String>) {
        match expr {
            Expr::Ident { name, span } => {
                if !locals.contains(name) && !self.fns.contains_key(name) {
                    self.error(*span, format!("undefined variable `{name}`"));
                }
            }
            Expr::String { .. } | Expr::Number { .. } | Expr::Error { .. } => {}
            Expr::Call { callee, args, span } => {
                self.check_expr(callee, locals);
                // Arity check for direct calls: f(a, b)
                if let Expr::Ident { name, .. } = callee.as_ref() {
                    if let Some(info) = self.fns.get(name) {
                        if args.len() != info.arity {
                            self.error(
                                *span,
                                format!(
                                    "`{name}` expects {} argument(s), got {}",
                                    info.arity,
                                    args.len()
                                ),
                            );
                        }
                    }
                }
                for arg in args {
                    self.check_expr(arg, locals);
                }
            }
            Expr::Member { object, .. } => {
                self.check_expr(object, locals);
            }
            Expr::Produce { expr: inner, .. }
            | Expr::Thunk { expr: inner, .. }
            | Expr::Force { expr: inner, .. } => {
                self.check_expr(inner, locals);
            }
            Expr::Let {
                name, value, body, ..
            } => {
                self.check_expr(value, locals);
                let is_new = locals.insert(name.clone());
                self.check_expr(body, locals);
                if is_new {
                    locals.remove(name);
                }
            }
            Expr::Match {
                scrutinee, arms, ..
            } => {
                self.check_expr(scrutinee, locals);
                for arm in arms {
                    self.check_match_arm(arm, locals);
                }
            }
            Expr::Perform { cap, span } => {
                if !self.caps.contains_key(cap) {
                    self.error(*span, format!("unknown capability `{cap}`"));
                }
            }
            Expr::Handle {
                cap,
                handler,
                body,
                span,
                ..
            } => {
                if !self.caps.contains_key(cap) {
                    self.error(*span, format!("unknown capability `{cap}`"));
                }
                self.check_expr(handler, locals);
                self.check_expr(body, locals);
            }
            Expr::Bundle { entries, .. } => {
                for entry in entries {
                    self.check_bundle_entry(entry, locals);
                }
            }
            Expr::Ann { expr: inner, ty, .. } => {
                self.check_expr(inner, locals);
                self.check_type_expr(&ty.value, ty.span);
            }
            Expr::Lambda { params, body, .. } => {
                for (name, ty) in params {
                    if let Some(ty) = ty {
                        self.check_type_expr(&ty.value, ty.span);
                    }
                    locals.insert(name.clone());
                }
                self.check_expr(body, locals);
                for (name, _) in params {
                    locals.remove(name);
                }
            }
        }
    }

    fn check_match_arm(&mut self, arm: &MatchArm, locals: &mut HashSet<String>) {
        let mut arm_locals = locals.clone();
        self.check_pattern(&arm.pattern, arm.span, &mut arm_locals);
        self.check_expr(&arm.body, &mut arm_locals);
    }

    fn check_pattern(
        &mut self,
        pattern: &Pattern,
        span: Span,
        locals: &mut HashSet<String>,
    ) {
        match pattern {
            Pattern::Ctor { name, args } => {
                if let Some(owner) = self.variant_owner.get(name) {
                    if let Some(data) = self.data.get(owner) {
                        if let Some(&expected) = data.variants.get(name) {
                            if args.len() != expected {
                                self.error(
                                    span,
                                    format!(
                                        "pattern `.{name}` expects {} field(s), got {}",
                                        expected,
                                        args.len()
                                    ),
                                );
                            }
                        }
                    }
                } else {
                    self.error(span, format!("unknown constructor `.{name}`"));
                }
                for arg in args {
                    self.check_pattern(arg, span, locals);
                }
            }
            Pattern::Bind(name) => {
                locals.insert(name.clone());
            }
            Pattern::Wildcard => {}
        }
    }

    fn check_bundle_entry(
        &mut self,
        entry: &BundleEntry,
        locals: &mut HashSet<String>,
    ) {
        let mut entry_locals = locals.clone();
        for param in &entry.params {
            self.check_type_expr(&param.ty.value, param.ty.span);
            entry_locals.insert(param.name.clone());
        }
        self.check_expr(&entry.body, &mut entry_locals);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn check(src: &str) -> Vec<CheckError> {
        let file = parse::parse(src).expect("parse failed");
        check_file(&file)
    }

    fn check_msgs(src: &str) -> Vec<String> {
        check(src).into_iter().map(|e| e.message).collect()
    }

    #[test]
    fn no_errors_on_valid_file() {
        let errors = check(
            "extern type Number
             data Bool { .true, .false }
             fn id(x: Number): Number := produce x",
        );
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    }

    #[test]
    fn undefined_variable() {
        let msgs = check_msgs("fn f() := produce y");
        assert!(msgs.iter().any(|m| m.contains("undefined variable `y`")));
    }

    #[test]
    fn let_binding_introduces_variable() {
        let errors = check("fn f() := let x = 42 in produce x");
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn param_is_in_scope() {
        let errors = check("extern type Number\nfn f(x: Number) := produce x");
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn unknown_type_in_param() {
        let msgs = check_msgs("fn f(x: Foo) := produce x");
        assert!(msgs.iter().any(|m| m.contains("unknown type `Foo`")));
    }

    #[test]
    fn unknown_type_in_return() {
        let msgs = check_msgs("fn f(): Bar := produce 1");
        assert!(msgs.iter().any(|m| m.contains("unknown type `Bar`")));
    }

    #[test]
    fn duplicate_fn() {
        let msgs = check_msgs(
            "fn f() := produce 1
             fn f() := produce 2",
        );
        assert!(msgs.iter().any(|m| m.contains("duplicate function `f`")));
    }

    #[test]
    fn duplicate_type() {
        let msgs = check_msgs(
            "data X { .a }
             data X { .b }",
        );
        assert!(msgs.iter().any(|m| m.contains("duplicate type `X`")));
    }

    #[test]
    fn duplicate_variant() {
        let msgs = check_msgs("data X { .a, .a }");
        assert!(msgs.iter().any(|m| m.contains("duplicate variant `.a`")));
    }

    #[test]
    fn duplicate_capability() {
        let msgs = check_msgs(
            "cap IO { fn log() }
             cap IO { fn read() }",
        );
        assert!(msgs
            .iter()
            .any(|m| m.contains("duplicate capability `IO`")));
    }

    #[test]
    fn unknown_capability_in_perform() {
        let msgs = check_msgs("fn f() := perform Unknown");
        assert!(msgs
            .iter()
            .any(|m| m.contains("unknown capability `Unknown`")));
    }

    #[test]
    fn unknown_capability_in_handle() {
        let msgs = check_msgs(
            "fn f() := handle Unknown with bundle { fn op() := produce 1 } in produce 2",
        );
        assert!(msgs
            .iter()
            .any(|m| m.contains("unknown capability `Unknown`")));
    }

    #[test]
    fn known_capability_no_error() {
        let errors = check(
            "cap IO { fn log() }
             fn f() := handle IO with bundle { fn log() := produce 1 } in produce 2",
        );
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn arity_mismatch_in_call() {
        let msgs = check_msgs(
            "fn add(a: Number, b: Number) := produce a
             fn f() := add(1)",
        );
        assert!(msgs
            .iter()
            .any(|m| m.contains("`add` expects 2 argument(s), got 1")));
    }

    #[test]
    fn generic_arity_mismatch() {
        let msgs = check_msgs(
            "data List[A] { .nil }
             fn f(x: List) := produce x",
        );
        assert!(msgs
            .iter()
            .any(|m| m.contains("expects 1 generic argument(s), got 0")));
    }

    #[test]
    fn generic_param_accepted_in_type() {
        let errors = check(
            "data Box[A] { .mk(A) }
             fn f[A](x: A): Box[A] := produce x",
        );
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn pattern_binding_in_scope() {
        let errors = check(
            "data Bool { .true, .false }
             fn f(b: Bool) := match b { .true => produce 1; .false => produce 0; }",
        );
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn pattern_with_bindings() {
        let errors = check(
            "extern type Number
             data Pair { .mk(Number, Number) }
             fn f(p: Pair) := match p { .mk(a, b) => produce a; }",
        );
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn unknown_constructor_in_pattern() {
        let msgs = check_msgs("fn f(x: Number) := match x { .nope => produce 1; }");
        assert!(msgs
            .iter()
            .any(|m| m.contains("unknown constructor `.nope`")));
    }

    #[test]
    fn pattern_arity_mismatch() {
        let msgs = check_msgs(
            "extern type Number
             data Pair { .mk(Number, Number) }
             fn f(p: Pair) := match p { .mk(a) => produce a; }",
        );
        assert!(msgs
            .iter()
            .any(|m| m.contains("`.mk` expects 2 field(s), got 1")));
    }

    #[test]
    fn impl_method_duplicate() {
        let msgs = check_msgs(
            "extern type Number
             impl Number { fn foo() := produce 1 fn foo() := produce 2 }",
        );
        assert!(msgs
            .iter()
            .any(|m| m.contains("duplicate method `foo`")));
    }

    #[test]
    fn fn_can_call_other_fn() {
        let errors = check(
            "fn g() := produce 1
             fn f() := g()",
        );
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn let_shadow_restored() {
        // After `let x = ... in body`, x should not leak outside
        let msgs = check_msgs("fn f() := let r = (let x = 1 in produce x) in produce x");
        assert!(
            msgs.iter().any(|m| m.contains("undefined variable `x`")),
            "x should not be in scope after let-in: {msgs:?}"
        );
    }
}
