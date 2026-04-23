use std::collections::{HashMap, HashSet};

use crate::lexer::Span;
use crate::lir::{self, Expr};
use crate::types::{CapEntry, CapRef, Pattern, TypeExpr, cap_ref_is_open};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    pub node_id: u64,
    pub span: Option<Span>,
    pub message: String,
    pub fn_name: String,
}

impl TypeError {
    fn new(node_id: u64, message: String) -> Self {
        Self {
            node_id,
            span: None,
            message,
            fn_name: String::new(),
        }
    }

    fn with_span(node_id: u64, span: Span, message: String) -> Self {
        Self {
            node_id,
            span: Some(span),
            message,
            fn_name: String::new(),
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub enum ValueType {
    Named(String),
    Thunk(Box<CompType>),
    Func {
        params: Vec<ValueType>,
        ret: Box<ValueType>,
    },
}

/// `Self` is treated as a wildcard that matches any type.
impl PartialEq for ValueType {
    fn eq(&self, other: &Self) -> bool {
        v_types_match(self, other)
    }
}

#[derive(Debug, Clone, Eq)]
pub enum CompType {
    Produce(Box<ValueType>),
    Fn {
        params: Vec<ValueType>,
        ret: Box<CompType>,
        cap: Vec<CapEntry>,
    },
}

/// `Self` is treated as a wildcard that matches any type.
impl PartialEq for CompType {
    fn eq(&self, other: &Self) -> bool {
        c_types_match(self, other)
    }
}

/// Compare two value types, treating `Self` as a wildcard that matches anything.
fn v_types_match(a: &ValueType, b: &ValueType) -> bool {
    match (a, b) {
        // `Self` and `_` (inferred placeholder) are wildcards that match any type
        (ValueType::Named(n), _) | (_, ValueType::Named(n)) if n == "Self" || n == "_" => true,
        (ValueType::Named(a), ValueType::Named(b)) => a == b,
        (ValueType::Thunk(a), ValueType::Thunk(b)) => c_types_match(a, b),
        (
            ValueType::Func { params: pa, ret: ra },
            ValueType::Func { params: pb, ret: rb },
        ) => pa.len() == pb.len() && pa.iter().zip(pb.iter()).all(|(a, b)| v_types_match(a, b)) && v_types_match(ra, rb),
        _ => false,
    }
}

/// Compare two computation types, treating `Self` as a wildcard.
fn c_types_match(a: &CompType, b: &CompType) -> bool {
    match (a, b) {
        (CompType::Produce(a), CompType::Produce(b)) => v_types_match(a, b),
        (
            CompType::Fn { params: pa, ret: ra, .. },
            CompType::Fn { params: pb, ret: rb, .. },
        ) => pa.len() == pb.len() && pa.iter().zip(pb.iter()).all(|(a, b)| v_types_match(a, b)) && c_types_match(ra, rb),
        _ => false,
    }
}

/// Substitute `Self` in a value type with a concrete type.
fn subst_self_v(ty: &ValueType, concrete: &ValueType) -> ValueType {
    match ty {
        ValueType::Named(n) if n == "Self" => concrete.clone(),
        ValueType::Named(_) => ty.clone(),
        ValueType::Thunk(inner) => ValueType::Thunk(Box::new(subst_self_c(inner, concrete))),
        ValueType::Func { params, ret } => ValueType::Func {
            params: params.iter().map(|p| subst_self_v(p, concrete)).collect(),
            ret: Box::new(subst_self_v(ret, concrete)),
        },
    }
}

fn v_type_references_self(ty: &ValueType) -> bool {
    match ty {
        ValueType::Named(n) => n == "Self",
        ValueType::Thunk(inner) => c_type_references_self(inner),
        ValueType::Func { params, ret } => {
            params.iter().any(v_type_references_self) || v_type_references_self(ret)
        }
    }
}

fn c_type_references_self(ty: &CompType) -> bool {
    match ty {
        CompType::Produce(inner) => v_type_references_self(inner),
        CompType::Fn { params, ret, .. } => {
            params.iter().any(v_type_references_self) || c_type_references_self(ret)
        }
    }
}

/// Substitute `Self` in a computation type with a concrete type.
fn subst_self_c(ty: &CompType, concrete: &ValueType) -> CompType {
    match ty {
        CompType::Produce(inner) => CompType::Produce(Box::new(subst_self_v(inner, concrete))),
        CompType::Fn { params, ret, cap } => CompType::Fn {
            params: params.iter().map(|p| subst_self_v(p, concrete)).collect(),
            ret: Box::new(subst_self_c(ret, concrete)),
            cap: cap.clone(),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedBinding {
    pub name: String,
    pub ty: CompType,
}

pub fn typecheck_file(file: &lir::File) -> Vec<TypeError> {
    typecheck_and_bindings(file).1
}

pub fn typecheck_and_bindings(file: &lir::File) -> (Vec<CheckedBinding>, Vec<TypeError>) {
    let mut tc = TypeChecker {
        errors: Vec::new(),
        bindings: Vec::new(),
        data_defs: HashMap::new(),
        variant_owner: HashMap::new(),
        fn_defs: HashMap::new(),
        cap_defs: HashMap::new(),
        current_fn: String::new(),
        cap_for_types: HashMap::new(),
        perform_for_types: HashMap::new(),
        impl_consts: HashMap::new(),
        value_type_methods: HashMap::new(),
    };
    tc.check_file(file);
    (tc.bindings, tc.errors)
}

/// Run type checking and return inferred caps for each function,
/// plus per-Perform-site type_args resolutions.
pub fn infer_caps_for_file(
    file: &lir::File,
) -> (HashMap<String, Vec<CapEntry>>, HashMap<u64, Vec<String>>) {
    let mut tc = TypeChecker {
        errors: Vec::new(),
        bindings: Vec::new(),
        data_defs: HashMap::new(),
        variant_owner: HashMap::new(),
        fn_defs: HashMap::new(),
        cap_defs: HashMap::new(),
        current_fn: String::new(),
        cap_for_types: HashMap::new(),
        perform_for_types: HashMap::new(),
        impl_consts: HashMap::new(),
        value_type_methods: HashMap::new(),
    };
    tc.check_file(file);
    let mut result = HashMap::new();
    for (name, ty) in &tc.fn_defs {
        if let CompType::Fn { cap, .. } = ty {
            if !cap.is_empty() {
                result.insert(name.clone(), cap.clone());
            }
        }
    }
    (result, tc.perform_for_types)
}

/// Patch LIR `FnDecl.cap` fields with inferred caps from the type checker.
pub fn apply_inferred_caps(file: &mut lir::File, inferred: &HashMap<String, Vec<CapEntry>>) {
    for item in &mut file.items {
        if let lir::Item::Fn(f) = item {
            if let Some(caps) = inferred.get(&f.name) {
                // Only patch functions that have no annotation or open (Infer) annotation.
                let is_patchable = f.cap.as_ref().map_or(true, |c| cap_ref_is_open(c));
                if is_patchable {
                    let new_cap: Vec<CapEntry> = std::iter::once(CapEntry::Infer)
                        .chain(caps.iter().filter(|e| matches!(e, CapEntry::Cap(_))).cloned())
                        .collect();
                    f.cap = Some(new_cap);
                }
            }
        }
    }
}

pub fn render_type(ty: &CompType) -> String {
    render_c_type(ty)
}

/// Extract the innermost return value type from a computation type.
/// E.g. `Fn { params, ret: Produce(V) }` → `Some(V)`, `Produce(V)` → `Some(V)`.
fn comp_type_return_value(ct: &CompType) -> Option<&ValueType> {
    match ct {
        CompType::Produce(v) => Some(v),
        CompType::Fn { ret, .. } => comp_type_return_value(ret),
    }
}

fn render_v_type(ty: &ValueType) -> String {
    match ty {
        ValueType::Named(n) => n.clone(),
        ValueType::Thunk(inner) => match inner.as_ref() {
            CompType::Fn { params, ret, cap } => {
                let ps = params.iter().map(render_v_type).collect::<Vec<_>>().join(", ");
                let cap_str = if cap.is_empty() {
                    String::new()
                } else {
                    format!(" / {{{}}}", cap.iter().map(|e| e.display()).collect::<Vec<_>>().join(", "))
                };
                format!("fn({ps}): {}{cap_str}", render_c_type(ret))
            }
            _ => format!("thunk {}", render_c_type(inner)),
        },
        ValueType::Func { params, ret } => {
            let ps = params
                .iter()
                .map(render_v_type)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({ps}) -> {}", render_v_type(ret))
        }
    }
}

fn render_c_type(ty: &CompType) -> String {
    match ty {
        CompType::Produce(inner) => render_v_type(inner),
        CompType::Fn {
            params,
            ret,
            cap,
        } => {
            let ps = params
                .iter()
                .map(render_v_type)
                .collect::<Vec<_>>()
                .join(", ");
            if cap.is_empty() {
                format!("({ps}) -> {}", render_c_type(ret))
            } else {
                format!("({ps}) -> {} / {{{}}}", render_c_type(ret), cap.iter().map(|e| e.display()).collect::<Vec<_>>().join(", "))
            }
        }
    }
}

struct TypeChecker {
    errors: Vec<TypeError>,
    bindings: Vec<CheckedBinding>,
    data_defs: HashMap<String, DataDef>,
    variant_owner: HashMap<String, String>,
    fn_defs: HashMap<String, CompType>,
    cap_defs: HashMap<String, CapDef>,
    current_fn: String,
    /// Per-function: resolved (cap_name, type_args) from Self-using cap operations.
    /// Populated during type checking, used to enrich inferred caps.
    cap_for_types: HashMap<String, Vec<(String, Vec<String>)>>,
    /// Per-Perform-site: resolved type_args from Self-using cap operations.
    /// Maps ExprId (u64) → resolved type args (e.g. vec!["Number"]).
    perform_for_types: HashMap<u64, Vec<String>>,
    /// Resolved impl const names → cap name they implement.
    /// e.g. "StrOps" → "StrOps", "__impl_Number_Add" → "Add"
    /// Used to type-check Ident nodes that reference auto-resolved cap impls.
    impl_consts: HashMap<String, String>,
    /// Methods available on value types via inherent or typeclass impls.
    /// Maps type_name → { method_name → method CompType (including self param) }.
    value_type_methods: HashMap<String, HashMap<String, CompType>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DataDef {
    generics: Vec<String>,
    variants: HashMap<String, Vec<ValueType>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapDef {
    operations: HashMap<String, CompType>,
    uses_self: bool,
}

enum BundleExprInferResult {
    NotBundleExpr,
    Typed(CompType),
    Error,
}

impl TypeChecker {
    fn check_file(&mut self, file: &lir::File) {
        for item in &file.items {
            if let lir::Item::Data(d) = item {
                for v in &d.variants {
                    self.variant_owner.insert(v.name.clone(), d.name.clone());
                }
                let mut variants = HashMap::new();
                for v in &d.variants {
                    let mut payload = Vec::new();
                    for spanned_ty in &v.payload {
                        match v_type_from_type_expr(&spanned_ty.value) {
                            Some(ty) => payload.push(ty),
                            None => self.errors.push(TypeError::with_span(
                                0,
                                spanned_ty.span,
                                format!(
                                    "variant payload type must be a value type, got {}",
                                    spanned_ty.value.display()
                                ),
                            )),
                        }
                    }
                    variants.insert(v.name.clone(), payload);
                }
                self.data_defs.insert(
                    d.name.clone(),
                    DataDef {
                        generics: d.generics.clone(),
                        variants,
                    },
                );
            }
            if let lir::Item::Cap(e) = item {
                let mut operations = HashMap::new();
                for op in &e.operations {
                    let params = op
                        .params
                        .iter()
                        .map(|p| match v_type_from_type_expr(&p.ty.value) {
                            Some(ty) => ty,
                            None => {
                                self.errors.push(TypeError::with_span(
                                    0,
                                    p.span,
                                    format!(
                                        "operation parameter `{}` must be a value type",
                                        p.name
                                    ),
                                ));
                                ValueType::Named("__invalid".to_owned())
                            }
                        })
                        .collect::<Vec<_>>();
                    let ret = match op.return_type.as_ref() {
                        Some(r) => match c_type_from_type_expr(&r.value) {
                            Some(ct) => ct,
                            None => {
                                self.errors.push(TypeError::with_span(
                                    0,
                                    r.span,
                                    format!(
                                        "operation return type must be a computation type (e.g. `produce {}`), got `{}`",
                                        r.value.display(),
                                        r.value.display(),
                                    ),
                                ));
                                CompType::Produce(Box::new(ValueType::Named("__invalid".to_owned())))
                            }
                        },
                        None => CompType::Produce(Box::new(ValueType::Named("Unit".to_owned()))),
                    };
                    let op_ty = if params.is_empty() {
                        ret
                    } else {
                        CompType::Fn {
                            params,
                            ret: Box::new(ret),
                            cap: vec![],
                        }
                    };
                    operations.insert(op.name.clone(), op_ty);
                }
                let uses_self = operations.values().any(|op_ty| c_type_references_self(op_ty));
                self.cap_defs
                    .insert(e.name.clone(), CapDef { operations, uses_self });
            }
        }

        for item in &file.items {
            match item {
                lir::Item::Fn(f) => self.predeclare_fn(f),
                lir::Item::ExternFn(f) => self.predeclare_extern_fn(f),
                _ => {}
            }
        }

        // Register resolved impl consts so Ident references to them are valid.
        // Platform impl: `impl StrOps { ... }` → const "StrOps" has cap type "StrOps"
        // Typeclass impl: `impl Number: Add { ... }` → const "__impl_Number_Add" has cap type "Add"
        // Non-cap inherent impl: `impl String { ... }` → const "String" for value methods
        let cap_name_set: HashSet<String> = self.cap_defs.keys().cloned().collect();
        for item in &file.items {
            if let lir::Item::Impl(impl_decl) = item {
                let target = impl_decl.target_type.value.display();
                if impl_decl.capability.is_none() && cap_name_set.contains(&target) {
                    // Platform: inherent impl whose target = cap name
                    self.impl_consts.insert(target.clone(), target.clone());
                    self.register_impl_methods(impl_decl, &target);
                } else if impl_decl.capability.is_none() {
                    // Non-cap inherent impl: register const and methods
                    self.impl_consts.insert(target.clone(), target.clone());
                    self.register_impl_methods(impl_decl, &target);
                } else if let Some(cap_ty) = &impl_decl.capability {
                    let cap = cap_ty.value.display();
                    if cap_name_set.contains(&cap) {
                        let const_name = if let Some(name) = &impl_decl.name {
                            name.clone()
                        } else {
                            format!("__impl_{target}_{cap}")
                        };
                        self.impl_consts.insert(const_name, cap);
                        self.register_impl_methods(impl_decl, &target);
                    }
                }
            }
        }

        // Infer caps for functions without explicit annotations
        self.infer_caps(file);

        for item in &file.items {
            match item {
                lir::Item::Fn(f) => self.check_fn(f),
                lir::Item::ExternFn(f) => self.check_extern_fn(f),
                _ => {}
            }
        }

        // Post-pass: propagate type_args info across fn_defs.
        // Build a global cap_name → type_args map from all resolutions.
        let mut global_for_types: HashMap<String, Vec<String>> = HashMap::new();
        for resolutions in self.cap_for_types.values() {
            for (cap_name, type_args) in resolutions {
                global_for_types
                    .entry(cap_name.clone())
                    .or_insert_with(|| type_args.clone());
            }
        }
        // Also harvest type_args from fn_defs that already have them (from explicit annotations)
        for ty in self.fn_defs.values() {
            if let CompType::Fn { cap, .. } = ty {
                for entry in cap {
                    if let CapEntry::Cap(ty) = entry {
                        if ty.cap_for_type().is_some() {
                            let args: Vec<String> = ty.cap_type_args().iter().map(|t| t.display()).collect();
                            global_for_types
                                .entry(ty.cap_name().to_owned())
                                .or_insert(args);
                        }
                    }
                }
            }
        }
        // Enrich all fn_defs caps with type_args (always runs — default to [cap_name])
        {
            let fn_names: Vec<String> = self.fn_defs.keys().cloned().collect();
            for fn_name in fn_names {
                if let Some(CompType::Fn { params, ret, cap }) = self.fn_defs.remove(&fn_name) {
                    let enriched: Vec<CapEntry> = cap
                        .into_iter()
                        .map(|entry| enrich_cap_entry(entry, &global_for_types))
                        .collect();
                    self.fn_defs.insert(
                        fn_name,
                        CompType::Fn {
                            params,
                            ret,
                            cap: enriched,
                        },
                    );
                }
            }
            // Also update bindings
            for binding in &mut self.bindings {
                if let CompType::Fn { params, ret, cap } = &binding.ty {
                    let enriched: Vec<CapEntry> = cap
                        .iter()
                        .map(|entry| enrich_cap_entry(entry.clone(), &global_for_types))
                        .collect();
                    binding.ty = CompType::Fn {
                        params: params.clone(),
                        ret: ret.clone(),
                        cap: enriched,
                    };
                }
            }
        }
    }

    fn predeclare_fn_common(
        &mut self,
        name: &str,
        params: &[lir::Param],
        return_type: Option<&TypeExpr>,
        cap: Option<&CapRef>,
    ) {
        let param_types = params
            .iter()
            .map(|p| {
                v_type_from_type_expr(&p.ty.value).unwrap_or_else(|| ValueType::Named("__invalid".to_owned()))
            })
            .collect::<Vec<_>>();
        let ret = match return_type {
            Some(te) => c_type_from_type_expr(te).unwrap_or_else(|| {
                CompType::Produce(Box::new(ValueType::Named("Unit".to_owned())))
            }),
            None => CompType::Produce(Box::new(ValueType::Named("Unit".to_owned()))),
        };
        let caps: Vec<CapEntry> = cap
            .map(|c| c.iter().filter(|e| matches!(e, CapEntry::Cap(_))).cloned().collect())
            .unwrap_or_default();
        self.fn_defs.insert(
            name.to_owned(),
            CompType::Fn {
                params: param_types,
                ret: Box::new(ret),
                cap: caps,
            },
        );
    }

    fn predeclare_fn(&mut self, f: &lir::FnDecl) {
        self.predeclare_fn_common(
            &f.name,
            &f.params,
            f.return_type.as_ref().map(|t| &t.value),
            f.cap.as_ref(),
        );
    }

    /// Register impl methods in `value_type_methods` for value method dispatch.
    fn register_impl_methods(&mut self, impl_decl: &lir::ImplDecl, target: &str) {
        let methods = self.value_type_methods.entry(target.to_owned()).or_default();
        for m in &impl_decl.methods {
            let param_types = m
                .params
                .iter()
                .map(|p| {
                    v_type_from_type_expr(&p.ty.value)
                        .unwrap_or_else(|| ValueType::Named("__invalid".to_owned()))
                })
                .collect::<Vec<_>>();
            let ret = match &m.return_type {
                Some(te) => c_type_from_type_expr(&te.value).unwrap_or_else(|| {
                    CompType::Produce(Box::new(ValueType::Named("Unit".to_owned())))
                }),
                None => CompType::Produce(Box::new(ValueType::Named("Unit".to_owned()))),
            };
            methods.insert(
                m.name.clone(),
                CompType::Fn {
                    params: param_types,
                    ret: Box::new(ret),
                    cap: vec![],
                },
            );
        }
    }

    fn predeclare_extern_fn(&mut self, f: &lir::ExternFnDecl) {
        self.predeclare_fn_common(
            &f.name,
            &f.params,
            f.return_type.as_ref().map(|t| &t.value),
            f.cap.as_ref(),
        );
    }

    fn check_fn(&mut self, f: &lir::FnDecl) {
        self.current_fn = f.name.clone();
        let mut env = HashMap::new();
        let mut param_types = Vec::new();
        for p in &f.params {
            let Some(ty) = v_type_from_type_expr(&p.ty.value) else {
                self.errors.push(TypeError::with_span(
                    0,
                    p.span,
                    format!("function parameter `{}` must be a value type", p.name),
                ));
                continue;
            };
            env.insert(p.name.clone(), ty.clone());
            param_types.push(ty);
        }

        // Get caps: use inferred caps from fn_defs (set by infer_caps pass)
        let caps: Vec<CapEntry> = self
            .fn_defs
            .get(&f.name)
            .and_then(|ty| {
                if let CompType::Fn { cap, .. } = ty {
                    Some(cap.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let expected = if let Some(ret) = &f.return_type {
            let Some(expected) = c_type_from_type_expr(&ret.value) else {
                self.errors.push(TypeError::with_span(
                    0,
                    ret.span,
                    "function return type must be a computation type".to_owned(),
                ));
                return;
            };
            expected
        } else {
            CompType::Produce(Box::new(ValueType::Named("Unit".to_owned())))
        };

        let fn_ty = CompType::Fn {
            params: param_types.clone(),
            ret: Box::new(expected.clone()),
            cap: caps.clone(),
        };
        self.fn_defs.insert(f.name.clone(), fn_ty.clone());

        let Some(body) = unwrap_fn_body(f) else {
            self.errors.push(TypeError::with_span(
                0,
                f.span,
                "malformed LIR function value: expected thunk/lambda spine".to_owned(),
            ));
            return;
        };
        // Validate explicit (closed) cap entries
        let is_open = f.cap.as_ref().map_or(true, |c| cap_ref_is_open(c));
        if !is_open {
            let concrete_caps: Vec<TypeExpr> = caps.iter()
                .filter_map(|e| if let CapEntry::Cap(ty) = e { Some(ty.clone()) } else { None })
                .collect();
            self.validate_cap_entries(&concrete_caps, f.span);
        }
        // Inject caps into env
        for entry in &caps {
            if let CapEntry::Cap(ty) = entry {
                let name = ty.cap_name();
                if let Some(handler_ty) = self.cap_handler_type(name) {
                    env.insert(format!("__cap_{name}"), handler_ty);
                }
            }
        }
        let err_before = self.errors.len();
        self.check_c_expr(body, &expected, &env);
        for e in &mut self.errors[err_before..] {
            e.fn_name = f.name.clone();
        }
        // Enrich inferred caps with for_type from Self resolutions
        let fn_ty = self.enrich_caps_with_for_types(&f.name, fn_ty);
        self.fn_defs.insert(f.name.clone(), fn_ty.clone());
        self.bindings.push(CheckedBinding {
            name: f.name.clone(),
            ty: fn_ty,
        });
    }

    fn check_extern_fn(&mut self, f: &lir::ExternFnDecl) {
        let mut param_types = Vec::new();
        for p in &f.params {
            let Some(ty) = v_type_from_type_expr(&p.ty.value) else {
                self.errors.push(TypeError::with_span(
                    0,
                    p.span,
                    format!("function parameter `{}` must be a value type", p.name),
                ));
                continue;
            };
            param_types.push(ty);
        }
        let caps: Vec<CapEntry> = f.cap.as_ref()
            .map(|c| c.iter().filter(|e| matches!(e, CapEntry::Cap(_))).cloned().collect())
            .unwrap_or_default();
        let concrete_caps: Vec<TypeExpr> = caps.iter()
            .filter_map(|e| if let CapEntry::Cap(ty) = e { Some(ty.clone()) } else { None })
            .collect();
        self.validate_cap_entries(&concrete_caps, f.span);
        let ret = if let Some(ret) = &f.return_type {
            let Some(expected) = c_type_from_type_expr(&ret.value) else {
                self.errors.push(TypeError::with_span(
                    0,
                    ret.span,
                    "function return type must be a computation type".to_owned(),
                ));
                return;
            };
            expected
        } else {
            CompType::Produce(Box::new(ValueType::Named("Unit".to_owned())))
        };
        self.bindings.push(CheckedBinding {
            name: f.name.clone(),
            ty: CompType::Fn {
                params: param_types,
                ret: Box::new(ret),
                cap: caps,
            },
        });
    }

    fn check_v_expr(
        &mut self,
        expr: &Expr,
        expected: &ValueType,
        env: &HashMap<String, ValueType>,
    ) {
        if let (Expr::Thunk { expr, .. }, ValueType::Thunk(inner)) = (expr, expected) {
            self.check_c_expr(expr, inner, env);
            return;
        }
        // Check bundle expression against cap type
        if let (
            Expr::Bundle {
                entries, id, ..
            },
            ValueType::Named(cap_name),
        ) = (expr, expected)
        {
            if let Some(def) = self.cap_defs.get(cap_name).cloned() {
                self.check_bundle_against_cap(entries, &def, cap_name, id.0 as u64, env, None);
                return;
            }
        }
        // For constructor expressions, pass expected type to resolve generics
        if matches!(expr, Expr::Ctor { .. } | Expr::Roll { .. }) {
            let expected_ct = CompType::Produce(Box::new(expected.clone()));
            match self.infer_bundle_expr_as_comp(expr, env, Some(&expected_ct)) {
                BundleExprInferResult::Typed(ct) => {
                    if let CompType::Produce(actual) = ct {
                        if &*actual != expected {
                            self.errors.push(TypeError::new(
                                expr_node_id(expr),
                                format!(
                                    "type mismatch: expected {}, got {}",
                                    render_v_type(expected),
                                    render_v_type(&actual)
                                ),
                            ));
                        }
                    }
                    return;
                }
                BundleExprInferResult::Error => return,
                BundleExprInferResult::NotBundleExpr => {}
            }
        }
        if let Some(actual) = self.infer_v_expr(expr, env) {
            if &actual != expected {
                self.errors.push(TypeError::new(
                    expr_node_id(expr),
                    format!(
                        "type mismatch: expected {}, got {}",
                        render_v_type(expected),
                        render_v_type(&actual)
                    ),
                ));
            }
        }
    }

    fn check_bundle_against_cap(
        &mut self,
        entries: &[lir::BundleEntry],
        def: &CapDef,
        cap_name: &str,
        node_id: u64,
        env: &HashMap<String, ValueType>,
        handle_result_type: Option<&CompType>,
    ) {
        // Check for missing operations
        for op_name in def.operations.keys() {
            if !entries.iter().any(|e| e.name == *op_name) {
                self.errors.push(TypeError::new(
                    node_id,
                    format!(
                        "bundle for cap `{cap_name}` is missing operation `{op_name}`"
                    ),
                ));
            }
        }
        // Check each entry
        for entry in entries {
            if let Some(op_ty) = def.operations.get(&entry.name) {
                let mut entry_env = env.clone();
                let uses_resume = lir::expr_references_name(&entry.body, "resume");

                // If entry uses resume and we have a handle result type,
                // add resume to the environment
                if uses_resume {
                    if let Some(handle_result) = handle_result_type {
                        if let Some(op_ret_val) = comp_type_return_value(op_ty) {
                            let resume_ty = ValueType::Thunk(Box::new(CompType::Fn {
                                params: vec![op_ret_val.clone()],
                                ret: Box::new(handle_result.clone()),
                                cap: vec![],
                            }));
                            entry_env.insert("resume".to_owned(), resume_ty);
                        }
                    }
                }

                match op_ty {
                    CompType::Fn { params, ret, .. } => {
                        if entry.params.len() != params.len() {
                            self.errors.push(TypeError::new(
                                node_id,
                                format!(
                                    "bundle entry `{}` expects {} params, got {}",
                                    entry.name,
                                    params.len(),
                                    entry.params.len()
                                ),
                            ));
                            continue;
                        }
                        for (p, expected_ty) in entry.params.iter().zip(params.iter()) {
                            entry_env.insert(p.name.clone(), expected_ty.clone());
                        }
                        // If entry uses resume, check body against handle result type;
                        // otherwise check against op return type (tail-resumptive)
                        let check_against = if uses_resume {
                            handle_result_type.unwrap_or(ret.as_ref())
                        } else {
                            ret.as_ref()
                        };
                        self.check_c_expr(&entry.body, check_against, &entry_env);
                    }
                    CompType::Produce(_) => {
                        if !entry.params.is_empty() {
                            self.errors.push(TypeError::new(
                                node_id,
                                format!(
                                    "bundle entry `{}` should take no params (op type is `{}`)",
                                    entry.name,
                                    render_c_type(op_ty)
                                ),
                            ));
                            continue;
                        }
                        let check_against = if uses_resume {
                            handle_result_type.unwrap_or(op_ty)
                        } else {
                            op_ty
                        };
                        self.check_c_expr(&entry.body, check_against, &entry_env);
                    }
                }
            } else {
                self.errors.push(TypeError::new(
                    node_id,
                    format!(
                        "bundle entry `{}` is not an operation of cap `{cap_name}`",
                        entry.name
                    ),
                ));
            }
        }
    }

    fn infer_bundle_expr_as_comp(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, ValueType>,
        expected: Option<&CompType>,
    ) -> BundleExprInferResult {
        let Some((owner, member, args, node_id, called)) = decompose_bundle_expr(expr) else {
            return BundleExprInferResult::NotBundleExpr;
        };
        let Some(def) = self.data_defs.get(&owner).cloned() else {
            self.errors.push(TypeError::new(
                node_id,
                format!("unknown data bundle `{owner}`"),
            ));
            return BundleExprInferResult::Error;
        };
        let Some(payload_types) = def.variants.get(&member).cloned() else {
            self.errors.push(TypeError::new(
                node_id,
                format!("unknown constructor `{owner}.{member}`"),
            ));
            return BundleExprInferResult::Error;
        };

        if called && payload_types.is_empty() {
            let field_ty = if def.generics.is_empty() {
                CompType::Produce(Box::new(ValueType::Named(owner.clone())))
            } else {
                CompType::Produce(Box::new(ValueType::Named(format!(
                    "{owner}[{}]",
                    def.generics.join(", ")
                ))))
            };
            self.errors.push(TypeError::new(
                node_id,
                format!(
                    "`{owner}.{member}` is a `{}`, which is not a function",
                    render_c_type(&field_ty)
                ),
            ));
            return BundleExprInferResult::Error;
        }
        if called && payload_types.len() != args.len() {
            self.errors.push(TypeError::new(
                node_id,
                format!(
                    "constructor `{owner}.{member}` expects {} args, got {}",
                    payload_types.len(),
                    args.len()
                ),
            ));
            return BundleExprInferResult::Error;
        }

        let generic_set = def.generics.iter().cloned().collect::<HashSet<_>>();
        let mut subst = HashMap::new();

        if called {
            for (arg, payload_ty) in args.iter().zip(payload_types.iter()) {
                let Some(actual_ty) = self.infer_v_expr(arg, env) else {
                    return BundleExprInferResult::Error;
                };
                if !self.unify_ctor_payload_type(
                    payload_ty,
                    &actual_ty,
                    &generic_set,
                    &mut subst,
                    expr_node_id(arg),
                ) {
                    return BundleExprInferResult::Error;
                }
            }
        }

        let result_template = if def.generics.is_empty() {
            ValueType::Named(owner.clone())
        } else {
            ValueType::Named(format!("{owner}[{}]", def.generics.join(", ")))
        };
        let expr_template = if called || payload_types.is_empty() {
            CompType::Produce(Box::new(result_template.clone()))
        } else {
            CompType::Fn {
                params: payload_types.clone(),
                ret: Box::new(CompType::Produce(Box::new(result_template.clone()))),
                cap: vec![],
            }
        };

        if let Some(expected_ty) = expected {
            if !self.unify_ctor_payload_comp_type(
                &expr_template,
                expected_ty,
                &generic_set,
                &mut subst,
                node_id,
            ) {
                return BundleExprInferResult::Error;
            }
        }

        let mut unresolved = Vec::new();
        for generic in &def.generics {
            if !subst.contains_key(generic) {
                unresolved.push(generic.clone());
            }
        }
        // For zero-payload constructors (e.g. List.nil), unresolved generics
        // are allowed — the type parameter can only be inferred from context.
        // Use the generic name as-is for the result type.
        if !unresolved.is_empty() && !payload_types.is_empty() {
            self.errors.push(TypeError::new(
                node_id,
                format!(
                    "cannot infer type arguments for bundle field `{owner}.{member}`: unresolved {}",
                    unresolved.join(", ")
                ),
            ));
            return BundleExprInferResult::Error;
        }

        let result_ty = if def.generics.is_empty() {
            ValueType::Named(owner.clone())
        } else {
            let resolved_args = def
                .generics
                .iter()
                .map(|generic| {
                    subst
                        .get(generic)
                        .map(render_v_type)
                        .unwrap_or_else(|| generic.clone())
                })
                .collect::<Vec<_>>();
            ValueType::Named(format!("{owner}[{}]", resolved_args.join(", ")))
        };
        let resolved = if called || payload_types.is_empty() {
            CompType::Produce(Box::new(result_ty))
        } else {
            CompType::Fn {
                params: payload_types
                    .iter()
                    .map(|payload| subst_v_type(payload, &subst))
                    .collect(),
                ret: Box::new(CompType::Produce(Box::new(result_ty))),
                cap: vec![],
            }
        };

        BundleExprInferResult::Typed(resolved)
    }

    /// Shared match-arm preparation: infer scrutinee, check exhaustiveness, validate patterns,
    /// validate variant-as-binding, and build per-arm environments.
    fn prepare_match_arms<'a>(
        &mut self,
        scrutinee: &Expr,
        arms: &'a [lir::MatchArm],
        env: &HashMap<String, ValueType>,
        id: u64,
    ) -> Vec<(&'a Expr, HashMap<String, ValueType>)> {
        let scrutinee_ty = self.infer_v_expr(scrutinee, env);
        let scrutinee_data = scrutinee_ty
            .as_ref()
            .and_then(nominal_head_name)
            .and_then(|name| self.data_defs.get(&name).cloned());
        if let Some(ref ty) = scrutinee_ty {
            if scrutinee_data.is_some() {
                self.check_match_exhaustive(arms, ty, id);
            }
        }
        arms.iter()
            .map(|arm| {
                let pat = &arm.pattern;
                let mut skip_bind = false;
                if let (Some(data_def), Pattern::Bind(name)) =
                    (scrutinee_data.as_ref(), pat)
                {
                    if data_def.variants.contains_key(name) {
                        self.errors.push(TypeError::with_span(
                            0,
                            arm.span,
                            format!("variant pattern `{name}` must be written `.{name}`"),
                        ));
                        skip_bind = true;
                    }
                }
                let mut arm_env = env.clone();
                if !skip_bind {
                    if let Some(ty) = scrutinee_ty.as_ref() {
                        self.bind_pattern(pat, ty, &mut arm_env, arm.span);
                    }
                } else if let Some(ty) = &scrutinee_ty {
                    for binding in pat.bindings() {
                        arm_env.insert(binding, ty.clone());
                    }
                }
                (&arm.body as &'a Expr, arm_env)
            })
            .collect()
    }

    /// Infer the value type bound by a `let` — classifies the value as computation or value,
    /// infers its type, and unwraps `Produce`.
    fn infer_let_value_type(
        &mut self,
        value: &Expr,
        env: &HashMap<String, ValueType>,
    ) -> Option<ValueType> {
        let value_ty = if is_syntactic_computation_expr(value) {
            self.infer_c_expr(value, env)?
        } else {
            CompType::Produce(Box::new(self.infer_v_expr(value, env)?))
        };
        let CompType::Produce(inner) = value_ty else {
            self.errors.push(TypeError::new(
                expr_node_id(value),
                format!(
                    "let expects produce computation, got {}",
                    render_c_type(&value_ty)
                ),
            ));
            return None;
        };
        Some(*inner)
    }

    fn check_c_expr(&mut self, expr: &Expr, expected: &CompType, env: &HashMap<String, ValueType>) {
        match self.infer_bundle_expr_as_comp(expr, env, Some(expected)) {
            BundleExprInferResult::Typed(_) | BundleExprInferResult::Error => return,
            BundleExprInferResult::NotBundleExpr => {}
        }

        if let CompType::Produce(inner) = expected {
            if let Expr::Produce { expr, .. } = expr {
                if is_syntactic_computation_expr(expr) {
                    // Implicit sequencing: `produce comp` checks comp against `produce T`
                    self.check_c_expr(expr, expected, env);
                } else {
                    self.check_v_expr(expr, inner, env);
                }
                return;
            }
            if is_syntactic_value_expr(expr) {
                self.check_v_expr(expr, inner, env);
                return;
            }
        }

        match expr {
            Expr::Let {
                name, value, body, ..
            } => {
                let Some(inner) = self.infer_let_value_type(value, env) else {
                    return;
                };
                let mut child = env.clone();
                child.insert(name.clone(), inner);
                self.check_c_expr(body, expected, &child);
            }
            Expr::Match {
                scrutinee,
                arms,
                id,
                ..
            } => {
                let prepared = self.prepare_match_arms(scrutinee, arms, env, id.0 as u64);
                for (body, arm_env) in prepared {
                    self.check_c_expr(body, expected, &arm_env);
                }
            }
            Expr::Handle {
                cap,
                handler,
                body,
                id,
                ..
            } => {
                let base = cap.as_str();
                if self.cap_defs.get(base).is_none() {
                    self.errors.push(TypeError::new(
                        id.0 as u64,
                        format!("unknown cap `{base}`"),
                    ));
                    let _ = self.infer_c_expr(handler, env);
                    self.check_c_expr(body, expected, env);
                    return;
                }
                let handler_value_type = self.cap_handler_type(base);
                // Check body first to determine handle result type for resume
                let mut body_env = env.clone();
                if let Some(ref ty) = handler_value_type {
                    body_env.insert(format!("__cap_{cap}"), ty.clone());
                }
                self.check_c_expr(body, expected, &body_env);
                // Check handler with handle result type for resume
                if let Some(ref expected_handler_ty) = handler_value_type {
                    if let Expr::Bundle {
                        entries,
                        id: bundle_id,
                        ..
                    } = handler.as_ref()
                    {
                        if let Some(def) = self.cap_defs.get(base).cloned() {
                            self.check_bundle_against_cap(
                                entries,
                                &def,
                                base,
                                bundle_id.0 as u64,
                                env,
                                Some(expected),
                            );
                        }
                    } else {
                        self.check_v_expr(handler, expected_handler_ty, env);
                    }
                } else {
                    let _ = self.infer_c_expr(handler, env);
                }
            }
            Expr::Lambda { .. } => {
                if let Some(actual) = self.infer_c_expr(expr, env) {
                    if !c_types_match(&actual, expected) {
                        self.errors.push(TypeError::new(
                            expr_node_id(expr),
                            format!(
                                "type mismatch: expected {}, got {}",
                                render_c_type(expected),
                                render_c_type(&actual)
                            ),
                        ));
                    }
                }
            }
            _ => {
                if let Some(actual) = self.infer_c_expr(expr, env) {
                    if &actual != expected {
                        self.errors.push(TypeError::new(
                            expr_node_id(expr),
                            format!(
                                "type mismatch: expected {}, got {}",
                                render_c_type(expected),
                                render_c_type(&actual)
                            ),
                        ));
                    }
                }
            }
        }
    }

    fn infer_v_expr(&mut self, expr: &Expr, env: &HashMap<String, ValueType>) -> Option<ValueType> {
        match expr {
            Expr::Ident { name, id, .. } => {
                if let Some(ty) = env.get(name) {
                    Some(ty.clone())
                } else if let Some(cap_name) = self.impl_consts.get(name) {
                    // Resolved impl const — type as its cap name for member access
                    Some(ValueType::Named(cap_name.clone()))
                } else {
                    self.errors.push(TypeError::new(
                        id.0 as u64,
                        format!("unknown variable `{name}`"),
                    ));
                    None
                }
            }
            Expr::String { .. } => Some(ValueType::Named("String".to_owned())),
            Expr::Number { .. } => Some(ValueType::Named("Number".to_owned())),
            Expr::Thunk { expr, .. } => {
                let inner = self.infer_c_expr(expr, env)?;
                Some(ValueType::Thunk(Box::new(inner)))
            }
            Expr::Unroll { expr, .. } => self.infer_v_expr(expr, env),
            Expr::Ann {
                expr, ty, id, ..
            } => {
                if let Some(v_ty) = v_type_from_type_expr(ty) {
                    self.check_v_expr(expr, &v_ty, env);
                    Some(v_ty)
                } else {
                    self.errors.push(TypeError::new(
                        id.0 as u64,
                        format!(
                            "annotation type `{}` is not a valid value type",
                            ty.display()
                        ),
                    ));
                    None
                }
            }
            Expr::Bundle { id, .. } => {
                self.errors.push(TypeError::new(
                    id.0 as u64,
                    "cannot infer type of bundle expression; provide type context".to_owned(),
                ));
                None
            }
            Expr::Ctor { .. }
            | Expr::Roll { .. }
            | Expr::Apply { .. }
            | Expr::Force { .. }
            | Expr::Perform { .. }
            | Expr::Handle { .. }
            | Expr::Member { .. } => {
                self.implicit_sequence_as_value(expr, env)
            }
            Expr::Error { .. } => None,
            _ => {
                if is_syntactic_computation_expr(expr) {
                    self.implicit_sequence_as_value(expr, env)
                } else {
                    self.errors.push(TypeError::new(
                        expr_node_id(expr),
                        "expected value expression".to_owned(),
                    ));
                    None
                }
            }
        }
    }

    /// Implicit ANF sequencing: computation in value position.
    /// `f(x)` as a value ≡ `let tmp = f(x) in tmp`.
    /// Zero-arg functions `f()` are auto-applied.
    fn implicit_sequence_as_value(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, ValueType>,
    ) -> Option<ValueType> {
        let ct = self.infer_c_expr(expr, env)?;
        self.extract_produced_value(&ct, expr)
    }

    fn extract_produced_value(&mut self, ct: &CompType, expr: &Expr) -> Option<ValueType> {
        match ct {
            CompType::Produce(vt) => Some(*vt.clone()),
            // Auto-apply zero-arg functions: `f` with type `() -> produce T` gives `T`
            CompType::Fn {
                params, ret, ..
            } if params.is_empty() => self.extract_produced_value(ret, expr),
            other => {
                self.errors.push(TypeError::new(
                    expr_node_id(expr),
                    format!(
                        "expected value expression, got computation `{}` of type `{}`",
                        render_expr_head(expr),
                        render_c_type(other)
                    ),
                ));
                None
            }
        }
    }

    fn infer_c_expr(&mut self, expr: &Expr, env: &HashMap<String, ValueType>) -> Option<CompType> {
        match expr {
            Expr::Ident { name, id, .. } => {
                if env.contains_key(name) {
                    self.errors.push(TypeError::new(
                        id.0 as u64,
                        format!("`{name}` is a value, not a computation"),
                    ));
                    None
                } else if let Some(ty) = self.fn_defs.get(name) {
                    Some(ty.clone())
                } else {
                    self.errors.push(TypeError::new(
                        id.0 as u64,
                        format!("unknown computation `{name}`"),
                    ));
                    None
                }
            }
            Expr::String { .. } | Expr::Number { .. } => {
                self.errors.push(TypeError::new(
                    expr_node_id(expr),
                    "expected computation expression".to_owned(),
                ));
                None
            }
            Expr::Ann {
                expr, ty, id, ..
            } => {
                if let Some(c_ty) = c_type_from_type_expr(ty) {
                    self.check_c_expr(expr, &c_ty, env);
                    Some(c_ty)
                } else if let Some(v_ty) = v_type_from_type_expr(ty) {
                    self.check_v_expr(expr, &v_ty, env);
                    Some(CompType::Produce(Box::new(v_ty)))
                } else {
                    self.errors.push(TypeError::new(
                        id.0 as u64,
                        format!(
                            "annotation type `{}` is not a valid type",
                            ty.display()
                        ),
                    ));
                    None
                }
            }
            Expr::Produce { expr, .. } => {
                // If inner is a syntactic value, infer as value
                if !is_syntactic_computation_expr(expr) {
                    let inner = self.infer_v_expr(expr, env)?;
                    return Some(CompType::Produce(Box::new(inner)));
                }
                // Implicit sequencing: `produce comp` ≡ `let x = comp in produce x`
                let vt = self.implicit_sequence_as_value(expr, env)?;
                Some(CompType::Produce(Box::new(vt)))
            }
            Expr::Force { expr, id, .. } => {
                if let Expr::Ident { name, .. } = expr.as_ref() {
                    if let Some(ty) = self.fn_defs.get(name).cloned() {
                        // Zero-arg functions: return the result type directly
                        if let CompType::Fn { params, ret, cap: callee_caps } = &ty {
                            if params.is_empty() {
                                // Check that required caps are available at call site
                                for cap_entry in callee_caps {
                                    if let CapEntry::Cap(cap_ty) = cap_entry {
                                        let cap_name = cap_ty.cap_name();
                                        let cap_var = format!("__cap_{cap_name}");
                                        if !env.contains_key(&cap_var) {
                                            self.errors.push(TypeError::new(
                                                id.0 as u64,
                                                format!(
                                                    "function `{name}` requires cap `{cap_name}` which is not available; \
                                                     provide it with `handle {cap_name} with <handler> in ...`",
                                                ),
                                            ));
                                        }
                                    }
                                }
                                return Some(*ret.clone());
                            }
                        }
                        return Some(ty.clone());
                    }
                }
                let inner = self.infer_v_expr(expr, env)?;
                if let ValueType::Thunk(thunked) = inner {
                    Some(*thunked)
                } else {
                    self.errors.push(TypeError::new(
                        expr_node_id(expr),
                        format!("cannot force non-thunk type {}", render_v_type(&inner)),
                    ));
                    None
                }
            }
            Expr::Ctor { .. } | Expr::Roll { .. } => {
                match self.infer_bundle_expr_as_comp(expr, env, None) {
                    BundleExprInferResult::Typed(ty) => Some(ty),
                    BundleExprInferResult::Error => None,
                    BundleExprInferResult::NotBundleExpr => {
                        self.errors.push(TypeError::new(
                            expr_node_id(expr),
                            "expected computation expression".to_owned(),
                        ));
                        None
                    }
                }
            }
            Expr::Apply { .. } => {
                let Some((callee, args, node_id)) = decompose_apply_chain(expr) else {
                    self.errors.push(TypeError::new(
                        expr_node_id(expr),
                        "expected computation expression".to_owned(),
                    ));
                    return None;
                };
                let callee_ty = self.infer_c_expr(callee, env)?;
                let CompType::Fn { params, ret, cap: callee_caps } = callee_ty else {
                    self.errors.push(TypeError::new(
                        node_id,
                        format!(
                            "`{}` is a `{}`, which is not a function",
                            render_expr_head(callee),
                            render_c_type(&callee_ty)
                        ),
                    ));
                    return None;
                };
                if params.len() != args.len() {
                    self.errors.push(TypeError::new(
                        node_id,
                        format!(
                            "function expects {} args, got {}",
                            params.len(),
                            args.len()
                        ),
                    ));
                    return None;
                }
                // Infer concrete Self type from arguments matched against Self-typed params
                let mut self_concrete: Option<ValueType> = None;
                for (arg, param_ty) in args.iter().zip(params.iter()) {
                    if matches!(param_ty, ValueType::Named(n) if n == "Self") {
                        if self_concrete.is_none() {
                            self_concrete = self.infer_v_expr(arg, env);
                        }
                    }
                    self.check_v_expr(arg, param_ty, env);
                }
                // Record Self resolution for cap member calls
                if let Some(ref concrete) = self_concrete {
                    if let Some(cap_name) = extract_cap_from_callee(callee) {
                        let resolved_type = render_v_type(concrete);
                        self.cap_for_types
                            .entry(self.current_fn.clone())
                            .or_default()
                            .push((cap_name.to_owned(), vec![resolved_type.clone()]));
                        // Also record per-Perform-site type_args
                        if let Some(perform_id) = extract_perform_id_from_callee(callee) {
                            self.perform_for_types.insert(perform_id, vec![resolved_type]);
                        }
                    }
                }
                // Check that required caps are available at call site
                for cap_entry in &callee_caps {
                    if let CapEntry::Cap(cap_ty) = cap_entry {
                        let cap_name = cap_ty.cap_name();
                        let cap_var = format!("__cap_{cap_name}");
                        if !env.contains_key(&cap_var) {
                            self.errors.push(TypeError::new(
                                node_id,
                                format!(
                                    "function `{}` requires cap `{cap_name}` which is not available; \
                                     provide it with `handle {cap_name} with <handler> in ...`",
                                    render_expr_head(callee),
                                ),
                            ));
                        }
                    }
                }
                // Substitute Self in return type if we resolved it
                let resolved_ret = if let Some(ref concrete) = self_concrete {
                    subst_self_c(&ret, concrete)
                } else {
                    *ret
                };
                Some(resolved_ret)
            }
            Expr::Let {
                name, value, body, ..
            } => {
                let inner = self.infer_let_value_type(value, env)?;
                let mut child = env.clone();
                child.insert(name.clone(), inner);
                self.infer_c_expr(body, &child)
            }
            Expr::Match {
                scrutinee,
                arms,
                id,
                ..
            } => {
                let prepared = self.prepare_match_arms(scrutinee, arms, env, id.0 as u64);
                let mut body_ty = None;
                for (body, arm_env) in prepared {
                    let arm_ty = self.infer_c_expr(body, &arm_env)?;
                    if let Some(expected) = &body_ty {
                        if *expected != arm_ty {
                            self.errors.push(TypeError::new(
                                id.0 as u64,
                                format!(
                                    "match arm type mismatch: expected {}, got {}",
                                    render_c_type(expected),
                                    render_c_type(&arm_ty)
                                ),
                            ));
                            return None;
                        }
                    } else {
                        body_ty = Some(arm_ty);
                    }
                }
                body_ty
            }
            Expr::Perform { cap, id, .. } => {
                let base = cap.as_str();
                if self.cap_defs.get(base).is_none() {
                    self.errors.push(TypeError::new(
                        id.0 as u64,
                        format!("unknown cap `{base}`"),
                    ));
                    return None;
                }
                let cap_var = format!("__cap_{cap}");
                if let Some(handler_ty) = env.get(&cap_var) {
                    Some(CompType::Produce(Box::new(handler_ty.clone())))
                } else {
                    self.errors.push(TypeError::new(
                        id.0 as u64,
                        format!(
                            "cap `{cap}` is not handled in this context; \
                             wrap in `handle {cap} with <handler> in ...`"
                        ),
                    ));
                    None
                }
            }
            Expr::Handle {
                cap,
                handler,
                body,
                id,
                ..
            } => {
                let base = cap.as_str();
                if self.cap_defs.get(base).is_none() {
                    self.errors.push(TypeError::new(
                        id.0 as u64,
                        format!("unknown cap `{base}`"),
                    ));
                    let _ = self.infer_c_expr(handler, env);
                    return self.infer_c_expr(body, env);
                }
                let handler_value_type = self.cap_handler_type(base);
                // Infer body first to determine handle result type (needed for resume typing)
                let mut body_env = env.clone();
                if let Some(ref ty) = handler_value_type {
                    body_env.insert(format!("__cap_{cap}"), ty.clone());
                }
                let body_comp_type = self.infer_c_expr(body, &body_env);
                // Check handler — if it's a bundle literal, pass handle_result_type for resume
                if let Some(ref expected_handler_ty) = handler_value_type {
                    if let Expr::Bundle {
                        entries,
                        id: bundle_id,
                        ..
                    } = handler.as_ref()
                    {
                        if let Some(def) = self.cap_defs.get(base).cloned() {
                            self.check_bundle_against_cap(
                                entries,
                                &def,
                                base,
                                bundle_id.0 as u64,
                                env,
                                body_comp_type.as_ref(),
                            );
                        }
                    } else {
                        self.check_v_expr(handler, expected_handler_ty, env);
                    }
                } else {
                    let _ = self.infer_c_expr(handler, env);
                }
                body_comp_type
            }
            Expr::Member {
                object, field, id, ..
            } => {
                // Member access is a computation-level operation.
                // Object can be a value (e.g. h.op) or computation (e.g. (perform E).op).
                let obj_ty = if is_syntactic_value_expr(object) {
                    self.infer_v_expr(object, env)?
                } else {
                    let comp_ty = self.infer_c_expr(object, env)?;
                    match comp_ty {
                        CompType::Produce(inner) => *inner,
                        other => {
                            self.errors.push(TypeError::new(
                                id.0 as u64,
                                format!(
                                    "member access `.{field}` on computation `{}`",
                                    render_c_type(&other)
                                ),
                            ));
                            return None;
                        }
                    }
                };

                if let ValueType::Named(ref name) = obj_ty {
                    if let Some(def) = self.cap_defs.get(name).cloned() {
                        if let Some(op_ty) = def.operations.get(field) {
                            return Some(op_ty.clone());
                        }
                        // Fall through to value_type_methods check
                    }

                    // Check value type methods (from inherent + typeclass impls)
                    if let Some(methods) = self.value_type_methods.get(name) {
                        if let Some(method_ty) = methods.get(field) {
                            return Some(method_ty.clone());
                        }
                    }

                    // Cap with no matching operation
                    if self.cap_defs.contains_key(name) {
                        self.errors.push(TypeError::new(
                            id.0 as u64,
                            format!("cap `{name}` has no operation `{field}`"),
                        ));
                        return None;
                    }
                }

                self.errors.push(TypeError::new(
                    id.0 as u64,
                    format!(
                        "member access `.{field}` on type `{}`",
                        render_v_type(&obj_ty)
                    ),
                ));
                None
            }
            Expr::Error { .. } => None,
            Expr::Lambda { param, body, .. } => {
                let mut child = env.clone();
                child.insert(param.clone(), ValueType::Named("_".to_owned()));
                self.infer_c_expr(body, &child).map(|ret| CompType::Fn {
                    params: vec![ValueType::Named("_".to_owned())],
                    ret: Box::new(ret),
                    cap: vec![],
                })
            }
            Expr::Thunk { .. }
            | Expr::Unroll { .. }
            | Expr::Bundle { .. } => {
                self.errors.push(TypeError::new(
                    expr_node_id(expr),
                    "expected computation expression".to_owned(),
                ));
                None
            }
        }
    }

    fn bind_pattern(
        &mut self,
        pattern: &Pattern,
        expected: &ValueType,
        env: &mut HashMap<String, ValueType>,
        span: Span,
    ) {
        match pattern {
            Pattern::Wildcard => {}
            Pattern::Bind(name) => {
                env.insert(name.clone(), expected.clone());
            }
            Pattern::Ctor { name, args } => {
                let Some(data_name) = nominal_head_name(expected) else {
                    self.errors.push(TypeError::with_span(
                        0,
                        span,
                        format!(
                            "constructor pattern `{name}` used on non-data type {}",
                            render_v_type(expected)
                        ),
                    ));
                    return;
                };
                let Some(data_def) = self.data_defs.get(&data_name).cloned() else {
                    self.errors.push(TypeError::with_span(
                        0,
                        span,
                        format!("unknown data type `{data_name}` in match scrutinee"),
                    ));
                    return;
                };
                if let Some(payload_types) = data_def.variants.get(name) {
                    if payload_types.len() != args.len() {
                        self.errors.push(TypeError::with_span(
                            0,
                            span,
                            format!(
                                "pattern `{name}` expects {} args, got {}",
                                payload_types.len(),
                                args.len()
                            ),
                        ));
                        return;
                    }

                    let payload_types = payload_types.clone();
                    let subst = match expected {
                        ValueType::Named(named) => {
                            let (head, args_text) = split_nominal_type_args(named);
                            self.generic_subst_from_named_type(
                                &head, &args_text, &data_def, span,
                            )
                            .unwrap_or_default()
                        }
                        _ => HashMap::new(),
                    };
                    for (arg, payload_ty) in args.iter().zip(payload_types.iter()) {
                        let expected_payload = subst_v_type(payload_ty, &subst);
                        self.bind_pattern(arg, &expected_payload, env, span);
                    }
                } else {
                    self.errors.push(TypeError::with_span(
                        0,
                        span,
                        format!("unknown variant `{name}` in match pattern"),
                    ));
                }
            }
        }
    }

    fn generic_subst_from_named_type(
        &mut self,
        expected_head: &str,
        expected_args_text: &[String],
        data_def: &DataDef,
        span: Span,
    ) -> Option<HashMap<String, ValueType>> {
        if data_def.generics.is_empty() {
            return Some(HashMap::new());
        }
        if expected_args_text.len() != data_def.generics.len() {
            self.errors.push(TypeError::with_span(
                0,
                span,
                format!(
                    "type argument count mismatch for `{expected_head}`: expected {}, got {}",
                    data_def.generics.len(),
                    expected_args_text.len()
                ),
            ));
            return None;
        }
        let mut subst = HashMap::with_capacity(data_def.generics.len());
        for (generic, arg_text) in data_def.generics.iter().zip(expected_args_text.iter()) {
            let Some(arg_ty) = parse_v_type(arg_text) else {
                self.errors.push(TypeError::with_span(
                    0,
                    span,
                    format!("invalid type argument `{arg_text}` for `{expected_head}`"),
                ));
                return None;
            };
            subst.insert(generic.clone(), arg_ty);
        }
        Some(subst)
    }

    fn unify_ctor_payload_type(
        &mut self,
        template: &ValueType,
        actual: &ValueType,
        generics: &HashSet<String>,
        subst: &mut HashMap<String, ValueType>,
        node_id: u64,
    ) -> bool {
        match template {
            ValueType::Named(name) => {
                if generics.contains(name) {
                    if let Some(bound) = subst.get(name) {
                        if bound != actual {
                            self.errors.push(TypeError::new(
                                node_id,
                                format!(
                                    "type mismatch: expected {}, got {}",
                                    render_v_type(bound),
                                    render_v_type(actual)
                                ),
                            ));
                            return false;
                        }
                        return true;
                    }
                    subst.insert(name.clone(), actual.clone());
                    return true;
                }
                let ValueType::Named(actual_name) = actual else {
                    self.errors.push(TypeError::new(
                        node_id,
                        format!(
                            "type mismatch: expected {}, got {}",
                            render_v_type(template),
                            render_v_type(actual)
                        ),
                    ));
                    return false;
                };
                let (template_head, template_args_text) = split_nominal_type_args(name);
                let (actual_head, actual_args_text) = split_nominal_type_args(actual_name);
                if template_head != actual_head
                    || template_args_text.len() != actual_args_text.len()
                {
                    self.errors.push(TypeError::new(
                        node_id,
                        format!(
                            "type mismatch: expected {}, got {}",
                            render_v_type(template),
                            render_v_type(actual)
                        ),
                    ));
                    return false;
                }
                for (lhs, rhs) in template_args_text.iter().zip(actual_args_text.iter()) {
                    let Some(lhs_ty) = parse_v_type(lhs) else {
                        self.errors.push(TypeError::new(
                            node_id,
                            format!("invalid type `{lhs}`"),
                        ));
                        return false;
                    };
                    let Some(rhs_ty) = parse_v_type(rhs) else {
                        self.errors.push(TypeError::new(
                            node_id,
                            format!("invalid type `{rhs}`"),
                        ));
                        return false;
                    };
                    if !self.unify_ctor_payload_type(&lhs_ty, &rhs_ty, generics, subst, node_id) {
                        return false;
                    }
                }
                true
            }
            ValueType::Thunk(template_inner) => {
                let ValueType::Thunk(actual_inner) = actual else {
                    self.errors.push(TypeError::new(
                        node_id,
                        format!(
                            "type mismatch: expected {}, got {}",
                            render_v_type(template),
                            render_v_type(actual)
                        ),
                    ));
                    return false;
                };
                self.unify_ctor_payload_comp_type(
                    template_inner,
                    actual_inner,
                    generics,
                    subst,
                    node_id,
                )
            }
            ValueType::Func {
                params: template_params,
                ret: template_ret,
            } => {
                let ValueType::Func {
                    params: actual_params,
                    ret: actual_ret,
                } = actual
                else {
                    self.errors.push(TypeError::new(
                        node_id,
                        format!(
                            "type mismatch: expected {}, got {}",
                            render_v_type(template),
                            render_v_type(actual)
                        ),
                    ));
                    return false;
                };
                if template_params.len() != actual_params.len() {
                    self.errors.push(TypeError::new(
                        node_id,
                        format!(
                            "type mismatch: expected {}, got {}",
                            render_v_type(template),
                            render_v_type(actual)
                        ),
                    ));
                    return false;
                }
                for (lhs, rhs) in template_params.iter().zip(actual_params.iter()) {
                    if !self.unify_ctor_payload_type(lhs, rhs, generics, subst, node_id) {
                        return false;
                    }
                }
                self.unify_ctor_payload_type(template_ret, actual_ret, generics, subst, node_id)
            }
        }
    }

    fn unify_ctor_payload_comp_type(
        &mut self,
        template: &CompType,
        actual: &CompType,
        generics: &HashSet<String>,
        subst: &mut HashMap<String, ValueType>,
        node_id: u64,
    ) -> bool {
        match (template, actual) {
            (CompType::Produce(lhs), CompType::Produce(rhs)) => {
                self.unify_ctor_payload_type(lhs, rhs, generics, subst, node_id)
            }
            (
                CompType::Fn {
                    params: lhs_params,
                    ret: lhs_ret,
                    cap: lhs_cap,
                },
                CompType::Fn {
                    params: rhs_params,
                    ret: rhs_ret,
                    cap: rhs_cap,
                },
            ) => {
                if lhs_params.len() != rhs_params.len() || lhs_cap != rhs_cap {
                    self.errors.push(TypeError::new(
                        node_id,
                        format!(
                            "type mismatch: expected {}, got {}",
                            render_c_type(template),
                            render_c_type(actual)
                        ),
                    ));
                    return false;
                }
                for (lhs, rhs) in lhs_params.iter().zip(rhs_params.iter()) {
                    if !self.unify_ctor_payload_type(lhs, rhs, generics, subst, node_id) {
                        return false;
                    }
                }
                self.unify_ctor_payload_comp_type(lhs_ret, rhs_ret, generics, subst, node_id)
            }
            _ => {
                self.errors.push(TypeError::new(
                    node_id,
                    format!(
                        "type mismatch: expected {}, got {}",
                        render_c_type(template),
                        render_c_type(actual)
                    ),
                ));
                false
            }
        }
    }

    /// Validate cap annotation entries: caps with Self must have type_args, caps without Self must not.
    fn validate_cap_entries(&mut self, entries: &[TypeExpr], span: Span) {
        for entry in entries {
            let name = entry.cap_name();
            if let Some(def) = self.cap_defs.get(name) {
                if def.uses_self && entry.cap_for_type().is_none() {
                    self.errors.push(TypeError::with_span(
                        0,
                        span,
                        format!(
                            "cap `{name}` uses `Self` and requires a type: `{name} for <Type>`",
                        ),
                    ));
                }
                // Non-Self caps may still have type_args (consistently using self as first generic)
            }
        }
    }

    /// Compute the value type that a handler for cap `name` must have.
    /// Always returns `Named(cap_name)` — caps are bundles regardless of op count.
    fn cap_handler_type(&self, name: &str) -> Option<ValueType> {
        self.cap_defs.get(name)?;
        Some(ValueType::Named(name.to_owned()))
    }

    fn check_match_exhaustive(
        &mut self,
        arms: &[lir::MatchArm],
        scrutinee_ty: &ValueType,
        node_id: u64,
    ) {
        let mut matrix = Vec::new();

        for arm in arms {
            let pattern = &arm.pattern;
            if !self.pattern_compatible_with_type(pattern, scrutinee_ty) {
                continue;
            }
            if !self.is_useful_pattern(&matrix, scrutinee_ty, pattern) {
                self.errors.push(TypeError::with_span(
                    0,
                    arm.span,
                    "unreachable match arm: pattern already covered".to_owned(),
                ));
                continue;
            }
            matrix.push(vec![pattern.clone()]);
        }

        let uncovered = if matrix.is_empty() {
            self.root_uncovered_patterns(scrutinee_ty)
        } else {
            self.uncovered_patterns(&matrix, std::slice::from_ref(scrutinee_ty))
        };
        if uncovered.is_empty() {
            return;
        }

        let mut missing = uncovered
            .into_iter()
            .filter_map(|mut row| {
                if row.len() != 1 {
                    return None;
                }
                Some(render_pattern(&row.remove(0)))
            })
            .collect::<Vec<_>>();
        missing.sort();
        missing.dedup();

        self.errors.push(TypeError::new(
            node_id,
            format!(
                "non-exhaustive match: missing patterns {}",
                missing.join(", ")
            ),
        ));
    }

    fn is_useful_pattern(
        &self,
        matrix: &[Vec<Pattern>],
        scrutinee_ty: &ValueType,
        pattern: &Pattern,
    ) -> bool {
        self.is_useful_matrix(
            matrix,
            &[pattern.clone()],
            std::slice::from_ref(scrutinee_ty),
        )
    }

    fn is_useful_matrix(
        &self,
        matrix: &[Vec<Pattern>],
        vector: &[Pattern],
        tys: &[ValueType],
    ) -> bool {
        if tys.is_empty() {
            return matrix.is_empty();
        }
        if matrix.is_empty() {
            return true;
        }
        if self.matrix_has_irrefutable_row(matrix) {
            return false;
        }

        let first_ty = &tys[0];
        if let Some(constructors) = self.constructors_for_type(first_ty) {
            match &vector[0] {
                Pattern::Ctor { name, args } => {
                    let Some(ctor) = constructors.iter().find(|ctor| ctor.name == *name) else {
                        return false;
                    };
                    let matrix =
                        self.specialize_matrix_ctor(matrix, &ctor.name, ctor.payload_types.len());
                    let mut next_vector = args.clone();
                    next_vector.extend_from_slice(&vector[1..]);
                    let mut next_tys = ctor.payload_types.clone();
                    next_tys.extend_from_slice(&tys[1..]);
                    self.is_useful_matrix(&matrix, &next_vector, &next_tys)
                }
                Pattern::Wildcard | Pattern::Bind(_) => constructors.into_iter().any(|ctor| {
                    let matrix =
                        self.specialize_matrix_ctor(matrix, &ctor.name, ctor.payload_types.len());
                    let mut next_vector = vec![Pattern::Wildcard; ctor.payload_types.len()];
                    next_vector.extend_from_slice(&vector[1..]);
                    let mut next_tys = ctor.payload_types;
                    next_tys.extend_from_slice(&tys[1..]);
                    self.is_useful_matrix(&matrix, &next_vector, &next_tys)
                }),
            }
        } else {
            let matrix = self.default_matrix(matrix);
            self.is_useful_matrix(&matrix, &vector[1..], &tys[1..])
        }
    }

    fn uncovered_patterns(&self, matrix: &[Vec<Pattern>], tys: &[ValueType]) -> Vec<Vec<Pattern>> {
        if tys.is_empty() {
            return if matrix.is_empty() {
                vec![Vec::new()]
            } else {
                Vec::new()
            };
        }
        if self.matrix_has_irrefutable_row(matrix) {
            return Vec::new();
        }
        if matrix.is_empty() {
            return vec![vec![Pattern::Wildcard; tys.len()]];
        }

        let first_ty = &tys[0];
        if let Some(constructors) = self.constructors_for_type(first_ty) {
            let mut out = Vec::new();
            for ctor in constructors {
                let matrix =
                    self.specialize_matrix_ctor(matrix, &ctor.name, ctor.payload_types.len());
                let mut next_tys = ctor.payload_types.clone();
                next_tys.extend_from_slice(&tys[1..]);
                for row in self.uncovered_patterns(&matrix, &next_tys) {
                    let (args, rest) = row.split_at(ctor.payload_types.len());
                    let mut rebuilt = vec![Pattern::Ctor {
                        name: ctor.name.clone(),
                        args: args.to_vec(),
                    }];
                    rebuilt.extend_from_slice(rest);
                    out.push(rebuilt);
                }
            }
            out
        } else {
            let matrix = self.default_matrix(matrix);
            self.uncovered_patterns(&matrix, &tys[1..])
                .into_iter()
                .map(|row| {
                    let mut rebuilt = Vec::with_capacity(row.len() + 1);
                    rebuilt.push(Pattern::Wildcard);
                    rebuilt.extend(row);
                    rebuilt
                })
                .collect()
        }
    }

    fn root_uncovered_patterns(&self, scrutinee_ty: &ValueType) -> Vec<Vec<Pattern>> {
        let Some(constructors) = self.constructors_for_type(scrutinee_ty) else {
            return vec![vec![Pattern::Wildcard]];
        };
        constructors
            .into_iter()
            .map(|ctor| {
                vec![Pattern::Ctor {
                    name: ctor.name,
                    args: vec![Pattern::Wildcard; ctor.payload_types.len()],
                }]
            })
            .collect()
    }

    fn constructors_for_type(&self, ty: &ValueType) -> Option<Vec<ConstructorSig>> {
        let ValueType::Named(name) = ty else {
            return None;
        };
        let (head, args_text) = split_nominal_type_args(name);
        let data_def = self.data_defs.get(&head)?;

        let subst = if data_def.generics.is_empty() || args_text.len() != data_def.generics.len() {
            HashMap::new()
        } else {
            let mut subst = HashMap::with_capacity(data_def.generics.len());
            for (generic, arg_text) in data_def.generics.iter().zip(args_text.iter()) {
                let Some(arg_ty) = parse_v_type(arg_text) else {
                    return None;
                };
                subst.insert(generic.clone(), arg_ty);
            }
            subst
        };

        let mut constructors = data_def
            .variants
            .iter()
            .map(|(name, payload_types)| ConstructorSig {
                name: name.clone(),
                payload_types: payload_types
                    .iter()
                    .map(|payload| subst_v_type(payload, &subst))
                    .collect(),
            })
            .collect::<Vec<_>>();
        constructors.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
        Some(constructors)
    }

    fn pattern_compatible_with_type(&self, pattern: &Pattern, ty: &ValueType) -> bool {
        match pattern {
            Pattern::Wildcard => true,
            Pattern::Bind(name) => !self.type_has_variant_named(ty, name),
            Pattern::Ctor { name, args } => {
                let Some(constructors) = self.constructors_for_type(ty) else {
                    return false;
                };
                let Some(ctor) = constructors.into_iter().find(|ctor| ctor.name == *name) else {
                    return false;
                };
                if ctor.payload_types.len() != args.len() {
                    return false;
                }
                args.iter()
                    .zip(ctor.payload_types.iter())
                    .all(|(arg, ty)| self.pattern_compatible_with_type(arg, ty))
            }
        }
    }

    fn type_has_variant_named(&self, ty: &ValueType, name: &str) -> bool {
        self.constructors_for_type(ty)
            .map(|constructors| constructors.into_iter().any(|ctor| ctor.name == name))
            .unwrap_or(false)
    }

    fn specialize_matrix_ctor(
        &self,
        matrix: &[Vec<Pattern>],
        ctor_name: &str,
        arity: usize,
    ) -> Vec<Vec<Pattern>> {
        matrix
            .iter()
            .filter_map(|row| self.specialize_row_ctor(row, ctor_name, arity))
            .collect()
    }

    fn specialize_row_ctor(
        &self,
        row: &[Pattern],
        ctor_name: &str,
        arity: usize,
    ) -> Option<Vec<Pattern>> {
        let (head, tail) = row.split_first()?;
        match head {
            Pattern::Ctor { name, args } if name == ctor_name && args.len() == arity => {
                let mut out = args.clone();
                out.extend_from_slice(tail);
                Some(out)
            }
            Pattern::Wildcard | Pattern::Bind(_) => {
                let mut out = vec![Pattern::Wildcard; arity];
                out.extend_from_slice(tail);
                Some(out)
            }
            _ => None,
        }
    }

    fn default_matrix(&self, matrix: &[Vec<Pattern>]) -> Vec<Vec<Pattern>> {
        matrix
            .iter()
            .filter_map(|row| {
                let (head, tail) = row.split_first()?;
                match head {
                    Pattern::Wildcard | Pattern::Bind(_) => Some(tail.to_vec()),
                    Pattern::Ctor { .. } => None,
                }
            })
            .collect()
    }

    fn matrix_has_irrefutable_row(&self, matrix: &[Vec<Pattern>]) -> bool {
        matrix.iter().any(|row| {
            row.iter()
                .all(|pattern| matches!(pattern, Pattern::Wildcard | Pattern::Bind(_)))
        })
    }

    // -----------------------------------------------------------------------
    /// Enrich inferred cap entries with type_args based on Self resolutions
    /// collected during type checking.
    fn enrich_caps_with_for_types(&self, fn_name: &str, ty: CompType) -> CompType {
        let CompType::Fn { params, ret, cap } = ty else {
            return ty;
        };
        let resolutions = match self.cap_for_types.get(fn_name) {
            Some(r) => r,
            None => return CompType::Fn { params, ret, cap },
        };
        let enriched: Vec<CapEntry> = cap
            .into_iter()
            .map(|entry| {
                if let CapEntry::Cap(ty) = &entry {
                    if ty.cap_for_type().is_some() {
                        return entry; // already has type_args
                    }
                    let cap_name = ty.cap_name();
                    // Find the resolved type_args from this function's resolutions
                    if let Some((_, type_args)) = resolutions.iter().find(|(n, _)| n == cap_name) {
                        CapEntry::Cap(TypeExpr::Cap {
                            name: cap_name.to_owned(),
                            type_args: type_args.iter().map(|t| TypeExpr::Named(t.clone())).collect(),
                        })
                    } else {
                        // Default: self = cap_name
                        CapEntry::Cap(TypeExpr::Cap {
                            name: cap_name.to_owned(),
                            type_args: vec![TypeExpr::Named(cap_name.to_owned())],
                        })
                    }
                } else {
                    entry // Spread and Infer pass through unchanged
                }
            })
            .collect();
        CompType::Fn {
            params,
            ret,
            cap: enriched,
        }
    }

    // Cap inference
    // -----------------------------------------------------------------------

    /// Run cap inference for all functions that need it (no annotation or `Infer`).
    /// Updates `self.fn_defs` with inferred caps via fixed-point iteration.
    fn infer_caps(&mut self, file: &lir::File) {
        let cap_names: HashSet<String> = self
            .cap_defs
            .keys()
            .cloned()
            .collect();

        // Build fn_caps map: fn_name → current cap TypeExpr list (concrete caps only, for inference)
        let mut fn_caps: HashMap<String, Vec<TypeExpr>> = HashMap::new();
        for (name, ty) in &self.fn_defs {
            if let CompType::Fn { cap, .. } = ty {
                let cap_exprs: Vec<TypeExpr> = cap.iter()
                    .filter_map(|e| if let CapEntry::Cap(ty) = e { Some(ty.clone()) } else { None })
                    .collect();
                fn_caps.insert(name.clone(), cap_exprs);
            }
        }

        // Identify functions that need inference
        let needs_inference: Vec<(String, bool)> = file
            .items
            .iter()
            .filter_map(|item| {
                if let lir::Item::Fn(f) = item {
                    match &f.cap {
                        None => Some((f.name.clone(), true)),
                        Some(c) if cap_ref_is_open(c) => Some((f.name.clone(), true)),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect();

        if needs_inference.is_empty() {
            return;
        }

        // Fixed-point iteration
        for _ in 0..100 {
            let mut changed = false;
            for item in &file.items {
                if let lir::Item::Fn(f) = item {
                    let should_infer = needs_inference.iter().any(|(n, _)| n == &f.name);
                    if !should_infer {
                        continue;
                    }
                    let Some(body) = unwrap_fn_body(f) else {
                        continue;
                    };
                    let handled = HashSet::new();
                    let mut inferred = collect_caps_from_expr(body, &handled, &fn_caps, &cap_names);

                    // For open cap set, include any explicitly-listed minimum caps
                    if let Some(cap_entries) = &f.cap {
                        if cap_ref_is_open(cap_entries) {
                            for entry in cap_entries {
                                if let CapEntry::Cap(ty) = entry {
                                    let name = ty.cap_name();
                                    if !inferred.iter().any(|c: &TypeExpr| c.cap_name() == name) {
                                        inferred.push(ty.clone());
                                    }
                                }
                            }
                        }
                    }

                    let old = fn_caps.get(&f.name).cloned().unwrap_or_default();
                    if !caps_equal(&inferred, &old) {
                        fn_caps.insert(f.name.clone(), inferred);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        // Write inferred caps back to fn_defs (convert TypeExpr → CapEntry::Cap)
        for (name, caps) in &fn_caps {
            if let Some(ty) = self.fn_defs.get_mut(name) {
                if let CompType::Fn { cap, .. } = ty {
                    *cap = caps.iter().map(|ty| CapEntry::Cap(ty.clone())).collect();
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Cap inference helpers (free functions)
// ---------------------------------------------------------------------------

/// Collect caps required by an expression, excluding caps in `handled`.
fn collect_caps_from_expr(
    expr: &Expr,
    handled: &HashSet<String>,
    fn_caps: &HashMap<String, Vec<TypeExpr>>,
    cap_defs: &HashSet<String>,
) -> Vec<TypeExpr> {
    let mut caps = Vec::new();
    collect_caps_inner(expr, handled, fn_caps, cap_defs, &mut caps);
    caps
}

fn collect_caps_inner(
    expr: &Expr,
    handled: &HashSet<String>,
    fn_caps: &HashMap<String, Vec<TypeExpr>>,
    cap_defs: &HashSet<String>,
    out: &mut Vec<TypeExpr>,
) {
    // Decompose perform call: Apply*(Force(Member(Perform(cap), op)), args)
    if let Some((cap, type_args, args)) = decompose_perform_call_for_inference(expr) {
        if !handled.contains(cap) && cap_defs.contains(cap) {
            // Use the Perform's type_args (set by `patch_perform_type_args`)
            // so we distinguish e.g. `Add[Number]` from `Add[String]` when
            // inferring a caller's cap set.
            let type_args_expr: Vec<TypeExpr> = type_args
                .iter()
                .map(|t| TypeExpr::Named(t.clone()))
                .collect();
            add_cap(out, TypeExpr::Cap {
                name: cap.to_owned(),
                type_args: type_args_expr,
            });
        }
        for arg in args {
            collect_caps_inner(arg, handled, fn_caps, cap_defs, out);
        }
        return;
    }

    // Decompose function call: Apply*(Force(Ident(name)), args)
    if let Some((fn_name, args)) = decompose_fn_call_for_inference(expr) {
        if let Some(callee_caps) = fn_caps.get(fn_name) {
            for c in callee_caps {
                if !handled.contains(c.cap_name()) {
                    add_cap(out, c.clone());
                }
            }
        }
        for arg in args {
            collect_caps_inner(arg, handled, fn_caps, cap_defs, out);
        }
        return;
    }

    match expr {
        Expr::Perform { cap, .. } => {
            // Bare perform (not as part of a member/apply chain)
            if !handled.contains(cap.as_str()) && cap_defs.contains(cap.as_str()) {
                add_cap(out, TypeExpr::Cap {
                    name: cap.clone(),
                    type_args: vec![],
                });
            }
        }
        Expr::Handle {
            cap,
            handler,
            body,
            ..
        } => {
            // Handler is evaluated in the outer cap scope
            collect_caps_inner(handler, handled, fn_caps, cap_defs, out);
            // Body has the cap handled
            let mut inner_handled = handled.clone();
            inner_handled.insert(cap.clone());
            collect_caps_inner(body, &inner_handled, fn_caps, cap_defs, out);
        }
        Expr::Apply { callee, arg, .. } => {
            collect_caps_inner(callee, handled, fn_caps, cap_defs, out);
            collect_caps_inner(arg, handled, fn_caps, cap_defs, out);
        }
        Expr::Force { expr, .. } => {
            // Force(Ident(name)) is a zero-arg function call — pull in transitive caps
            if let Expr::Ident { name, .. } = expr.as_ref() {
                if let Some(callee_caps) = fn_caps.get(name.as_str()) {
                    for c in callee_caps {
                        if !handled.contains(c.cap_name()) {
                            add_cap(out, c.clone());
                        }
                    }
                    return;
                }
            }
            collect_caps_inner(expr, handled, fn_caps, cap_defs, out);
        }
        Expr::Let { value, body, .. } => {
            collect_caps_inner(value, handled, fn_caps, cap_defs, out);
            collect_caps_inner(body, handled, fn_caps, cap_defs, out);
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            collect_caps_inner(scrutinee, handled, fn_caps, cap_defs, out);
            for arm in arms {
                collect_caps_inner(&arm.body, handled, fn_caps, cap_defs, out);
            }
        }
        Expr::Produce { expr, .. } => {
            collect_caps_inner(expr, handled, fn_caps, cap_defs, out);
        }
        Expr::Member { object, .. } => {
            collect_caps_inner(object, handled, fn_caps, cap_defs, out);
        }
        Expr::Thunk { expr, .. } => {
            collect_caps_inner(expr, handled, fn_caps, cap_defs, out);
        }
        Expr::Lambda { body, .. } => {
            collect_caps_inner(body, handled, fn_caps, cap_defs, out);
        }
        Expr::Ctor { args, .. } => {
            for arg in args {
                collect_caps_inner(arg, handled, fn_caps, cap_defs, out);
            }
        }
        Expr::Roll { expr, .. } | Expr::Unroll { expr, .. } | Expr::Ann { expr, .. } => {
            collect_caps_inner(expr, handled, fn_caps, cap_defs, out);
        }
        Expr::Bundle { entries, .. } => {
            for entry in entries {
                collect_caps_inner(&entry.body, handled, fn_caps, cap_defs, out);
            }
        }
        // Leaves: no caps
        Expr::Ident { .. } | Expr::String { .. } | Expr::Number { .. } | Expr::Error { .. } => {}
    }
}

/// Decompose: Apply*(Force(Member(Perform(cap), op)), args) → Some((cap, args))
fn decompose_perform_call_for_inference(expr: &Expr) -> Option<(&str, &[String], Vec<&Expr>)> {
    let (root, args) = unwrap_apply_chain_ref(expr);
    let root = if let Expr::Force { expr, .. } = root {
        expr.as_ref()
    } else {
        root
    };
    if let Expr::Member { object, .. } = root {
        if let Expr::Perform { cap, type_args, .. } = object.as_ref() {
            return Some((cap.as_str(), type_args.as_slice(), args));
        }
    }
    None
}

/// Decompose: Apply*(Force(Ident(name)), args) → Some((name, args))
fn decompose_fn_call_for_inference(expr: &Expr) -> Option<(&str, Vec<&Expr>)> {
    let (root, args) = unwrap_apply_chain_ref(expr);
    if let Expr::Force { expr, .. } = root {
        if let Expr::Ident { name, .. } = expr.as_ref() {
            return Some((name.as_str(), args));
        }
    }
    None
}

/// Unwrap a chain of Apply nodes: Apply(Apply(root, a1), a2) → (root, [a1, a2])
fn unwrap_apply_chain_ref(expr: &Expr) -> (&Expr, Vec<&Expr>) {
    let mut args = Vec::new();
    let mut cursor = expr;
    while let Expr::Apply { callee, arg, .. } = cursor {
        args.push(arg.as_ref());
        cursor = callee.as_ref();
    }
    args.reverse();
    (cursor, args)
}

/// Enrich a CapEntry::Cap with type_args from a global resolution map.
/// Spread and Infer entries pass through unchanged.
fn enrich_cap_entry(entry: CapEntry, global_for_types: &HashMap<String, Vec<String>>) -> CapEntry {
    if let CapEntry::Cap(ty) = &entry {
        if ty.cap_for_type().is_some() {
            return entry; // already has type_args
        }
        let cap_name = ty.cap_name();
        if let Some(type_args) = global_for_types.get(cap_name) {
            return CapEntry::Cap(TypeExpr::Cap {
                name: cap_name.to_owned(),
                type_args: type_args.iter().map(|t| TypeExpr::Named(t.clone())).collect(),
            });
        } else {
            return CapEntry::Cap(TypeExpr::Cap {
                name: cap_name.to_owned(),
                type_args: vec![TypeExpr::Named(cap_name.to_owned())],
            });
        }
    }
    entry
}

/// Add a cap entry to the list, deduplicating by cap name + type_args.
fn add_cap(caps: &mut Vec<TypeExpr>, cap: TypeExpr) {
    let name = cap.cap_name();
    let args: Vec<_> = cap.cap_type_args().iter().map(|t| t.display()).collect();
    if !caps.iter().any(|c| {
        c.cap_name() == name
            && c.cap_type_args().iter().map(|t| t.display()).collect::<Vec<_>>() == args
    }) {
        caps.push(cap);
    }
}

/// Compare two cap lists (order-insensitive, by cap name + type_args).
fn caps_equal(a: &[TypeExpr], b: &[TypeExpr]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().all(|ac| {
        b.iter().any(|bc| {
            bc.cap_name() == ac.cap_name()
                && bc.cap_type_args().iter().map(|t| t.display()).collect::<Vec<_>>()
                    == ac.cap_type_args().iter().map(|t| t.display()).collect::<Vec<_>>()
        })
    })
}

/// Convert a `TypeExpr` to a `ValueType`.
/// TypeExpr only represents named/parameterized types, so we always succeed.
fn v_type_from_type_expr(te: &TypeExpr) -> Option<ValueType> {
    match te {
        TypeExpr::Named(n) => Some(ValueType::Named(n.clone())),
        TypeExpr::App { head, args } => {
            let args_str = args
                .iter()
                .map(|a| a.display())
                .collect::<Vec<_>>()
                .join(", ");
            Some(ValueType::Named(format!("{head}[{args_str}]")))
        }
        TypeExpr::Thunk(inner) => {
            let ct = c_type_from_type_expr(inner)?;
            Some(ValueType::Thunk(Box::new(ct)))
        }
        TypeExpr::Produce(_) => None, // `produce T` is not a value type
        TypeExpr::Cap { name, .. } => Some(ValueType::Named(name.clone())),
        TypeExpr::Fn { params, ret, cap } => {
            let param_types: Vec<ValueType> = params.iter().filter_map(v_type_from_type_expr).collect();
            let ret_ct = c_type_from_type_expr(ret)?;
            Some(ValueType::Thunk(Box::new(CompType::Fn {
                params: param_types,
                ret: Box::new(ret_ct),
                cap: cap.clone(),
            })))
        }
    }
}

/// Convert a `TypeExpr` to a `CompType`.
fn c_type_from_type_expr(te: &TypeExpr) -> Option<CompType> {
    match te {
        TypeExpr::Produce(inner) => {
            let vt = v_type_from_type_expr(inner)?;
            Some(CompType::Produce(Box::new(vt)))
        }
        // Bare value type is shorthand for `produce T`
        _ => v_type_from_type_expr(te).map(|vt| CompType::Produce(Box::new(vt))),
    }
}

fn parse_v_type(repr: &str) -> Option<ValueType> {
    let text = repr.trim();
    if let Some(rest) = text.strip_prefix("produce") {
        let _ = rest;
        return None;
    }
    if let Some(rest) = text.strip_prefix("thunk") {
        return Some(ValueType::Thunk(Box::new(parse_c_type(rest)?)));
    }
    Some(ValueType::Named(canonicalize_named_type_text(text)))
}

fn parse_c_type(repr: &str) -> Option<CompType> {
    let text = repr.trim();
    if let Some(rest) = text.strip_prefix("produce") {
        return Some(CompType::Produce(Box::new(parse_v_type(rest)?)));
    }
    // Bare value type is shorthand for `produce T`
    parse_v_type(text).map(|vt| CompType::Produce(Box::new(vt)))
}

fn canonicalize_named_type_text(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn split_nominal_type_args(text: &str) -> (String, Vec<String>) {
    let text = text.trim();
    let Some(start) = text.find('[') else {
        return (text.to_owned(), Vec::new());
    };
    if !text.ends_with(']') {
        return (text.to_owned(), Vec::new());
    }
    let head = text[..start].to_owned();
    let inner = &text[start + 1..text.len() - 1];
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut begin = 0usize;
    for (idx, ch) in inner.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                out.push(inner[begin..idx].trim().to_owned());
                begin = idx + 1;
            }
            _ => {}
        }
    }
    let tail = inner[begin..].trim();
    if !tail.is_empty() {
        out.push(tail.to_owned());
    }
    (head, out)
}

fn subst_v_type(ty: &ValueType, subst: &HashMap<String, ValueType>) -> ValueType {
    match ty {
        ValueType::Named(name) => {
            if let Some(mapped) = subst.get(name) {
                return mapped.clone();
            }
            let (head, args) = split_nominal_type_args(name);
            if args.is_empty() {
                ValueType::Named(name.clone())
            } else {
                let args = args
                    .iter()
                    .filter_map(|arg| parse_v_type(arg))
                    .map(|arg| subst_v_type(&arg, subst))
                    .map(|arg| render_v_type(&arg))
                    .collect::<Vec<_>>();
                ValueType::Named(format!("{head}[{}]", args.join(", ")))
            }
        }
        ValueType::Thunk(inner) => ValueType::Thunk(Box::new(subst_c_type(inner, subst))),
        ValueType::Func { params, ret } => ValueType::Func {
            params: params.iter().map(|p| subst_v_type(p, subst)).collect(),
            ret: Box::new(subst_v_type(ret, subst)),
        },
    }
}

fn subst_c_type(ty: &CompType, subst: &HashMap<String, ValueType>) -> CompType {
    match ty {
        CompType::Produce(inner) => CompType::Produce(Box::new(subst_v_type(inner, subst))),
        CompType::Fn {
            params,
            ret,
            cap,
        } => CompType::Fn {
            params: params.iter().map(|p| subst_v_type(p, subst)).collect(),
            ret: Box::new(subst_c_type(ret, subst)),
            cap: cap.clone(),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConstructorSig {
    name: String,
    payload_types: Vec<ValueType>,
}

fn render_pattern(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Wildcard | Pattern::Bind(_) => "_".to_owned(),
        Pattern::Ctor { name, args } if args.is_empty() => format!(".{name}"),
        Pattern::Ctor { name, args } => format!(
            ".{name}({})",
            args.iter()
                .map(|p| render_pattern(p))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn nominal_head_name(ty: &ValueType) -> Option<String> {
    let ValueType::Named(n) = ty else {
        return None;
    };
    Some(split_nominal_type_args(n).0)
}

fn is_syntactic_value_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Ident { .. }
            | Expr::String { .. }
            | Expr::Thunk { .. }
            | Expr::Unroll { .. }
            | Expr::Bundle { .. }
    )
}

fn is_syntactic_computation_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Produce { .. }
            | Expr::Force { .. }
            | Expr::Apply { .. }
            | Expr::Let { .. }
            | Expr::Match { .. }
            | Expr::Ctor { .. }
            | Expr::Roll { .. }
            | Expr::Perform { .. }
            | Expr::Handle { .. }
            | Expr::Member { .. }
            | Expr::Ann { .. }
            | Expr::Error { .. }
    )
}

fn render_expr_head(expr: &Expr) -> String {
    match expr {
        Expr::Ident { name, .. } => name.clone(),
        Expr::Force { expr, .. } => format!("force {}", render_expr_head(expr)),
        Expr::Apply { callee, .. } => format!("{}(...)", render_expr_head(callee)),
        Expr::Ctor { name, .. } => name.clone(),
        Expr::Roll { expr, .. } => render_expr_head(expr),
        Expr::Perform { cap, .. } => format!("perform {cap}"),
        Expr::Handle { cap, .. } => format!("handle {cap}"),
        Expr::Bundle { .. } => "bundle { ... }".to_owned(),
        Expr::Member { object, field, .. } => format!("{}.{field}", render_expr_head(object)),
        _ => "<expr>".to_owned(),
    }
}

fn decompose_bundle_expr(expr: &Expr) -> Option<(String, String, &[Expr], u64, bool)> {
    match expr {
        Expr::Ctor {
            name,
            args,
            id,
            called,
            ..
        } => {
            let (owner, member) = name.split_once('.')?;
            Some((
                owner.to_owned(),
                member.to_owned(),
                args.as_slice(),
                id.0 as u64,
                *called,
            ))
        }
        Expr::Roll { expr, id, .. } => match expr.as_ref() {
            Expr::Ctor {
                name, args, called, ..
            } => {
                let (owner, member) = name.split_once('.')?;
                Some((
                    owner.to_owned(),
                    member.to_owned(),
                    args.as_slice(),
                    id.0 as u64,
                    *called,
                ))
            }
            _ => None,
        },
        _ => None,
    }
}

fn decompose_apply_chain(expr: &Expr) -> Option<(&Expr, Vec<&Expr>, u64)> {
    let Expr::Apply { callee, arg, .. } = expr else {
        return None;
    };
    let mut args = vec![arg.as_ref()];
    let mut cursor = callee.as_ref();
    while let Expr::Apply {
        callee: inner_callee,
        arg: inner_arg,
        ..
    } = cursor
    {
        args.push(inner_arg.as_ref());
        cursor = inner_callee.as_ref();
    }
    args.reverse();
    Some((cursor, args, expr_node_id(expr)))
}

fn expr_node_id(expr: &Expr) -> u64 {
    expr.id().0 as u64
}

/// Extract the ExprId of the Perform node inside a callee expression.
/// Matches `Force(Member(Perform { id, .. }, op))` or `Member(Perform { id, .. }, op)`.
fn extract_perform_id_from_callee(callee: &Expr) -> Option<u64> {
    if let Expr::Force { expr, .. } = callee {
        if let Expr::Member { object, .. } = expr.as_ref() {
            if let Expr::Perform { id, .. } = object.as_ref() {
                return Some(id.0 as u64);
            }
        }
    }
    if let Expr::Member { object, .. } = callee {
        if let Expr::Perform { id, .. } = object.as_ref() {
            return Some(id.0 as u64);
        }
    }
    None
}

/// Extract cap name from a callee expression matching the pattern
/// `Force(Member(Perform(cap), op))` — i.e. a cap member call.
fn extract_cap_from_callee(callee: &Expr) -> Option<&str> {
    if let Expr::Force { expr, .. } = callee {
        if let Expr::Member { object, .. } = expr.as_ref() {
            if let Expr::Perform { cap, .. } = object.as_ref() {
                return Some(cap.as_str());
            }
        }
    }
    // Also handle non-forced member: Member(Perform(cap), op)
    if let Expr::Member { object, .. } = callee {
        if let Expr::Perform { cap, .. } = object.as_ref() {
            return Some(cap.as_str());
        }
    }
    None
}

fn unwrap_fn_body<'a>(func: &'a lir::FnDecl) -> Option<&'a Expr> {
    let Expr::Thunk { expr, .. } = &func.value else {
        return None;
    };
    let mut cursor = expr.as_ref();
    for param in &func.params {
        let Expr::Lambda {
            param: actual,
            body,
            ..
        } = cursor
        else {
            return None;
        };
        if actual != &param.name {
            return None;
        }
        cursor = body.as_ref();
    }
    Some(cursor)
}
