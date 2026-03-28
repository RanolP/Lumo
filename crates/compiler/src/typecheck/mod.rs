use std::collections::{HashMap, HashSet};

use crate::lexer::Span;
use crate::lir::{self, Expr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    pub node_id: u64,
    pub span: Option<Span>,
    pub message: String,
}

impl TypeError {
    fn new(node_id: u64, message: String) -> Self {
        Self {
            node_id,
            span: None,
            message,
        }
    }

    fn with_span(node_id: u64, span: Span, message: String) -> Self {
        Self {
            node_id,
            span: Some(span),
            message,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Named(String),
    Thunk(Box<CompType>),
    Func {
        params: Vec<ValueType>,
        ret: Box<ValueType>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompType {
    Produce(Box<ValueType>),
    Fn {
        params: Vec<ValueType>,
        ret: Box<CompType>,
        effect: Option<String>,
    },
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
        effect_defs: HashMap::new(),
    };
    tc.check_file(file);
    (tc.bindings, tc.errors)
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
        ValueType::Thunk(inner) => format!("thunk {}", render_c_type(inner)),
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
        CompType::Produce(inner) => format!("produce {}", render_v_type(inner)),
        CompType::Fn {
            params,
            ret,
            effect,
        } => {
            let ps = params
                .iter()
                .map(render_v_type)
                .collect::<Vec<_>>()
                .join(", ");
            if let Some(e) = effect {
                if !e.is_empty() {
                    format!("({ps}) -> {} / {e}", render_c_type(ret))
                } else {
                    format!("({ps}) -> {}", render_c_type(ret))
                }
            } else {
                format!("({ps}) -> {}", render_c_type(ret))
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
    effect_defs: HashMap<String, EffectDef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DataDef {
    generics: Vec<String>,
    variants: HashMap<String, Vec<ValueType>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EffectDef {
    operations: HashMap<String, CompType>,
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
                    for (payload_index, t) in v.payload_types.iter().enumerate() {
                        match parse_v_type(t) {
                            Some(ty) => payload.push(ty),
                            None => self.errors.push(TypeError::new(
                                payload_node_id(v, payload_index),
                                format!(
                                    "variant payload type must be a value type, got {}",
                                    normalize_type_text(t)
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
            if let lir::Item::Effect(e) = item {
                let mut operations = HashMap::new();
                for op in &e.operations {
                    let params = op
                        .params
                        .iter()
                        .map(|p| match parse_v_type(&p.ty_repr) {
                            Some(ty) => ty,
                            None => {
                                self.errors.push(TypeError::new(
                                    lir::source_id("param", p.source_span).0,
                                    format!(
                                        "operation parameter `{}` must be a value type",
                                        p.name
                                    ),
                                ));
                                ValueType::Named("__invalid".to_owned())
                            }
                        })
                        .collect::<Vec<_>>();
                    let ret = match op.return_type_repr.as_ref() {
                        Some(r) => match parse_c_type(r) {
                            Some(ct) => ct,
                            None => {
                                self.errors.push(TypeError::new(
                                    op.id.0,
                                    format!(
                                        "operation return type must be a computation type (e.g. `produce {}`), got `{}`",
                                        normalize_type_text(r),
                                        normalize_type_text(r),
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
                            effect: None,
                        }
                    };
                    operations.insert(op.name.clone(), op_ty);
                }
                self.effect_defs
                    .insert(e.name.clone(), EffectDef { operations });
            }
        }

        for item in &file.items {
            match item {
                lir::Item::Fn(f) => self.predeclare_fn(f),
                lir::Item::ExternFn(f) => self.predeclare_extern_fn(f),
                _ => {}
            }
        }

        for item in &file.items {
            match item {
                lir::Item::Fn(f) => self.check_fn(f),
                lir::Item::ExternFn(f) => self.check_extern_fn(f),
                _ => {}
            }
        }
    }

    fn predeclare_fn(&mut self, f: &lir::FnDecl) {
        let params = f
            .params
            .iter()
            .map(|p| {
                parse_v_type(&p.ty_repr).unwrap_or_else(|| ValueType::Named("__invalid".to_owned()))
            })
            .collect::<Vec<_>>();
        let ret = if let Some(ret) = &f.return_type_repr {
            parse_c_type(ret)
                .unwrap_or_else(|| CompType::Produce(Box::new(ValueType::Named("Unit".to_owned()))))
        } else {
            CompType::Produce(Box::new(ValueType::Named("Unit".to_owned())))
        };
        let effect = f
            .effect_repr
            .as_ref()
            .map(|e| normalize_effect_text(&normalize_type_text(e)));
        self.fn_defs.insert(
            f.name.clone(),
            CompType::Fn {
                params,
                ret: Box::new(ret),
                effect,
            },
        );
    }

    fn predeclare_extern_fn(&mut self, f: &lir::ExternFnDecl) {
        let params = f
            .params
            .iter()
            .map(|p| {
                parse_v_type(&p.ty_repr).unwrap_or_else(|| ValueType::Named("__invalid".to_owned()))
            })
            .collect::<Vec<_>>();
        let ret = if let Some(ret) = &f.return_type_repr {
            parse_c_type(ret)
                .unwrap_or_else(|| CompType::Produce(Box::new(ValueType::Named("Unit".to_owned()))))
        } else {
            CompType::Produce(Box::new(ValueType::Named("Unit".to_owned())))
        };
        let effect = f
            .effect_repr
            .as_ref()
            .map(|e| normalize_effect_text(&normalize_type_text(e)));
        self.fn_defs.insert(
            f.name.clone(),
            CompType::Fn {
                params,
                ret: Box::new(ret),
                effect,
            },
        );
    }

    fn check_fn(&mut self, f: &lir::FnDecl) {
        let mut env = HashMap::new();
        let mut param_types = Vec::new();
        for p in &f.params {
            let Some(ty) = parse_v_type(&p.ty_repr) else {
                self.errors.push(TypeError::new(
                    param_node_id(p),
                    format!("function parameter `{}` must be a value type", p.name),
                ));
                continue;
            };
            env.insert(p.name.clone(), ty.clone());
            param_types.push(ty);
        }

        let effect = f
            .effect_repr
            .as_ref()
            .map(|e| normalize_effect_text(&normalize_type_text(e)));
        let expected = if let Some(ret) = &f.return_type_repr {
            let Some(expected) = parse_c_type(ret) else {
                self.errors.push(TypeError::new(
                    type_sig_node_id("fn-return", f.return_type_span, f.id),
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
            effect: effect.clone(),
        };
        self.fn_defs.insert(f.name.clone(), fn_ty.clone());

        let Some(body) = unwrap_fn_body(f) else {
            self.errors.push(TypeError::new(
                f.id.0,
                "malformed LIR function value: expected thunk/lambda spine".to_owned(),
            ));
            return;
        };
        self.check_c_expr(body, &expected, &env);
        self.bindings.push(CheckedBinding {
            name: f.name.clone(),
            ty: fn_ty,
        });
    }

    fn check_extern_fn(&mut self, f: &lir::ExternFnDecl) {
        let mut param_types = Vec::new();
        for p in &f.params {
            let Some(ty) = parse_v_type(&p.ty_repr) else {
                self.errors.push(TypeError::new(
                    param_node_id(p),
                    format!("function parameter `{}` must be a value type", p.name),
                ));
                continue;
            };
            param_types.push(ty);
        }
        let effect = f
            .effect_repr
            .as_ref()
            .map(|e| normalize_effect_text(&normalize_type_text(e)));
        let ret = if let Some(ret) = &f.return_type_repr {
            let Some(expected) = parse_c_type(ret) else {
                self.errors.push(TypeError::new(
                    type_sig_node_id("extern-fn-return", f.return_type_span, f.id),
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
                effect,
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
        // Check bundle expression against effect type
        if let (
            Expr::Bundle {
                entries, id, ..
            },
            ValueType::Named(effect_name),
        ) = (expr, expected)
        {
            if let Some(def) = self.effect_defs.get(effect_name).cloned() {
                self.check_bundle_against_effect(entries, &def, effect_name, id.0, env, None);
                return;
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

    fn check_bundle_against_effect(
        &mut self,
        entries: &[lir::LirBundleEntry],
        def: &EffectDef,
        effect_name: &str,
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
                        "bundle for effect `{effect_name}` is missing operation `{op_name}`"
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
                                effect: None,
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
                        "bundle entry `{}` is not an operation of effect `{effect_name}`",
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
        let Some(owner_by_variant) = self.variant_owner.get(&member) else {
            self.errors.push(TypeError::new(
                node_id,
                format!("unknown constructor `{owner}.{member}`"),
            ));
            return BundleExprInferResult::Error;
        };
        if owner_by_variant != &owner {
            self.errors.push(TypeError::new(
                node_id,
                format!("constructor `{owner}.{member}` does not belong to `{owner}`"),
            ));
            return BundleExprInferResult::Error;
        }
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
                effect: None,
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
        if !unresolved.is_empty() {
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
                .filter_map(|generic| subst.get(generic))
                .map(render_v_type)
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
                effect: None,
            }
        };

        BundleExprInferResult::Typed(resolved)
    }

    fn check_c_expr(&mut self, expr: &Expr, expected: &CompType, env: &HashMap<String, ValueType>) {
        match self.infer_bundle_expr_as_comp(expr, env, Some(expected)) {
            BundleExprInferResult::Typed(_) | BundleExprInferResult::Error => return,
            BundleExprInferResult::NotBundleExpr => {}
        }

        if let CompType::Produce(inner) = expected {
            if let Expr::Produce { expr, .. } = expr {
                self.check_v_expr(expr, inner, env);
                return;
            }
            if is_syntactic_value_expr(expr) {
                self.check_v_expr(expr, inner, env);
                return;
            }
        }

        match expr {
            Expr::LetIn {
                name, value, body, ..
            } => {
                let value_ty = match value.as_ref() {
                    Expr::Produce { .. }
                    | Expr::Force { .. }
                    | Expr::Apply { .. }
                    | Expr::LetIn { .. }
                    | Expr::Match { .. }
                    | Expr::Ctor { .. }
                    | Expr::Roll { .. }
                    | Expr::Perform { .. }
                    | Expr::Handle { .. }
                    | Expr::Member { .. }
                    | Expr::Ann { .. }
                    | Expr::Error { .. } => self.infer_c_expr(value, env),
                    _ => self
                        .infer_v_expr(value, env)
                        .map(|v| CompType::Produce(Box::new(v))),
                };
                let Some(value_ty) = value_ty else {
                    return;
                };
                let CompType::Produce(inner) = value_ty else {
                    self.errors.push(TypeError::new(
                        expr_node_id(value),
                        format!(
                            "let expects produce computation, got {}",
                            render_c_type(&value_ty)
                        ),
                    ));
                    return;
                };
                let mut child = env.clone();
                child.insert(name.clone(), *inner);
                self.check_c_expr(body, expected, &child);
            }
            Expr::Match {
                scrutinee,
                arms,
                id,
                ..
            } => {
                let scrutinee_ty = self.infer_v_expr(scrutinee, env);
                let scrutinee_data = scrutinee_ty
                    .as_ref()
                    .and_then(nominal_head_name)
                    .and_then(|name| self.data_defs.get(&name).cloned());
                if let Some(ref ty) = scrutinee_ty {
                    if scrutinee_data.is_some() {
                        self.check_match_exhaustive(arms, ty, id.0);
                    }
                }
                for arm in arms {
                    let mut parsed_pattern = parse_match_pattern(&arm.pattern);
                    if parsed_pattern.is_none() {
                        self.errors.push(TypeError::new(
                            arm_node_id(arm),
                            invalid_pattern_message(&arm.pattern),
                        ));
                    }
                    if let (Some(data_def), Some(Pattern::Bind(name))) =
                        (scrutinee_data.as_ref(), parsed_pattern.as_ref())
                    {
                        if data_def.variants.contains_key(name) {
                            self.errors.push(TypeError::new(
                                arm_node_id(arm),
                                format!(
                                    "variant pattern `{name}` must be written `.{name}`"
                                ),
                            ));
                            parsed_pattern = None;
                        }
                    }
                    let mut arm_env = env.clone();
                    if let (Some(ty), Some(pattern)) =
                        (scrutinee_ty.as_ref(), parsed_pattern.as_ref())
                    {
                        self.bind_pattern(pattern, ty, &mut arm_env, arm_node_id(arm));
                    } else if let Some(ty) = &scrutinee_ty {
                        for binding in parsed_pattern
                            .as_ref()
                            .map(pattern_bindings)
                            .unwrap_or_default()
                        {
                            arm_env.insert(binding, ty.clone());
                        }
                    }
                    self.check_c_expr(&arm.body, expected, &arm_env);
                }
            }
            Expr::Handle {
                effect,
                handler,
                body,
                id,
                ..
            } => {
                if self.effect_defs.get(effect).is_none() {
                    self.errors.push(TypeError::new(
                        id.0,
                        format!("unknown effect `{effect}`"),
                    ));
                    let _ = self.infer_c_expr(handler, env);
                    self.check_c_expr(body, expected, env);
                    return;
                }
                let handler_value_type = self.effect_handler_type(effect);
                // Check body first to determine handle result type for resume
                let mut body_env = env.clone();
                if let Some(ref ty) = handler_value_type {
                    body_env.insert(format!("__effect_{effect}"), ty.clone());
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
                        if let Some(def) = self.effect_defs.get(effect).cloned() {
                            self.check_bundle_against_effect(
                                entries,
                                &def,
                                effect,
                                bundle_id.0,
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
                } else {
                    self.errors.push(TypeError::new(
                        id.0,
                        format!("unknown variable `{name}`"),
                    ));
                    None
                }
            }
            Expr::String { .. } => Some(ValueType::Named("String".to_owned())),
            Expr::Thunk { expr, .. } => {
                let inner = self.infer_c_expr(expr, env)?;
                Some(ValueType::Thunk(Box::new(inner)))
            }
            Expr::Unroll { expr, .. } => self.infer_v_expr(expr, env),
            Expr::Ann {
                expr, ty_repr, id, ..
            } => {
                if let Some(v_ty) = parse_v_type(ty_repr) {
                    self.check_v_expr(expr, &v_ty, env);
                    Some(v_ty)
                } else {
                    self.errors.push(TypeError::new(
                        id.0,
                        format!(
                            "annotation type `{}` is not a valid value type",
                            normalize_type_text(ty_repr)
                        ),
                    ));
                    None
                }
            }
            Expr::Bundle { id, .. } => {
                self.errors.push(TypeError::new(
                    id.0,
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
                self.errors.push(TypeError::new(
                    expr_node_id(expr),
                    format!(
                        "expected value expression, got computation `{}`; if you need its result, bind it first with `let`",
                        render_expr_head(expr)
                    ),
                ));
                None
            }
            Expr::Error { .. } => None,
            _ => {
                self.errors.push(TypeError::new(
                    expr_node_id(expr),
                    "expected value expression".to_owned(),
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
                        id.0,
                        format!("`{name}` is a value, not a computation"),
                    ));
                    None
                } else if let Some(ty) = self.fn_defs.get(name) {
                    Some(ty.clone())
                } else {
                    self.errors.push(TypeError::new(
                        id.0,
                        format!("unknown computation `{name}`"),
                    ));
                    None
                }
            }
            Expr::String { .. } => {
                self.errors.push(TypeError::new(
                    expr_node_id(expr),
                    "expected computation expression".to_owned(),
                ));
                None
            }
            Expr::Ann {
                expr, ty_repr, id, ..
            } => {
                if let Some(c_ty) = parse_c_type(ty_repr) {
                    self.check_c_expr(expr, &c_ty, env);
                    Some(c_ty)
                } else if let Some(v_ty) = parse_v_type(ty_repr) {
                    self.check_v_expr(expr, &v_ty, env);
                    Some(CompType::Produce(Box::new(v_ty)))
                } else {
                    self.errors.push(TypeError::new(
                        id.0,
                        format!(
                            "annotation type `{}` is not a valid type",
                            normalize_type_text(ty_repr)
                        ),
                    ));
                    None
                }
            }
            Expr::Produce { expr, .. } => {
                let inner = self.infer_v_expr(expr, env)?;
                Some(CompType::Produce(Box::new(inner)))
            }
            Expr::Force { expr, .. } => {
                if let Expr::Ident { name, .. } = expr.as_ref() {
                    if let Some(ty) = self.fn_defs.get(name) {
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
                let CompType::Fn { params, ret, .. } = callee_ty else {
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
                for (arg, param_ty) in args.iter().zip(params.iter()) {
                    self.check_v_expr(arg, param_ty, env);
                }
                Some(*ret)
            }
            Expr::LetIn {
                name, value, body, ..
            } => {
                let value_ty = match value.as_ref() {
                    Expr::Produce { .. }
                    | Expr::Force { .. }
                    | Expr::Apply { .. }
                    | Expr::LetIn { .. }
                    | Expr::Match { .. }
                    | Expr::Ctor { .. }
                    | Expr::Roll { .. }
                    | Expr::Perform { .. }
                    | Expr::Handle { .. }
                    | Expr::Member { .. }
                    | Expr::Ann { .. }
                    | Expr::Error { .. } => self.infer_c_expr(value, env)?,
                    Expr::Ident { .. }
                    | Expr::String { .. }
                    | Expr::Thunk { .. }
                    | Expr::Lambda { .. }
                    | Expr::Unroll { .. }
                    | Expr::Bundle { .. } => {
                        CompType::Produce(Box::new(self.infer_v_expr(value, env)?))
                    }
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
                let mut child = env.clone();
                child.insert(name.clone(), *inner);
                self.infer_c_expr(body, &child)
            }
            Expr::Match {
                scrutinee,
                arms,
                id,
                ..
            } => {
                let scrutinee_ty = self.infer_v_expr(scrutinee, env);
                let scrutinee_data = scrutinee_ty
                    .as_ref()
                    .and_then(nominal_head_name)
                    .and_then(|name| self.data_defs.get(&name).cloned());
                if let Some(ref ty) = scrutinee_ty {
                    if scrutinee_data.is_some() {
                        self.check_match_exhaustive(arms, ty, id.0);
                    }
                }
                let mut body_ty = None;
                for arm in arms {
                    let mut parsed_pattern = parse_match_pattern(&arm.pattern);
                    if parsed_pattern.is_none() {
                        self.errors.push(TypeError::new(
                            arm_node_id(arm),
                            invalid_pattern_message(&arm.pattern),
                        ));
                    }
                    if let (Some(data_def), Some(Pattern::Bind(name))) =
                        (scrutinee_data.as_ref(), parsed_pattern.as_ref())
                    {
                        if data_def.variants.contains_key(name) {
                            self.errors.push(TypeError::new(
                                arm_node_id(arm),
                                format!(
                                    "variant pattern `{name}` must be written `.{name}`"
                                ),
                            ));
                            parsed_pattern = None;
                        }
                    }
                    let mut arm_env = env.clone();
                    if let (Some(ty), Some(pattern)) =
                        (scrutinee_ty.as_ref(), parsed_pattern.as_ref())
                    {
                        self.bind_pattern(pattern, ty, &mut arm_env, arm_node_id(arm));
                    } else if let Some(ty) = &scrutinee_ty {
                        for binding in parsed_pattern
                            .as_ref()
                            .map(pattern_bindings)
                            .unwrap_or_default()
                        {
                            arm_env.insert(binding, ty.clone());
                        }
                    }
                    let arm_ty = self.infer_c_expr(&arm.body, &arm_env)?;
                    if let Some(expected) = &body_ty {
                        if *expected != arm_ty {
                            self.errors.push(TypeError::new(
                                id.0,
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
            Expr::Perform { effect, id, .. } => {
                if self.effect_defs.get(effect).is_none() {
                    self.errors.push(TypeError::new(
                        id.0,
                        format!("unknown effect `{effect}`"),
                    ));
                    return None;
                }
                let effect_var = format!("__effect_{effect}");
                if let Some(handler_ty) = env.get(&effect_var) {
                    Some(CompType::Produce(Box::new(handler_ty.clone())))
                } else {
                    self.errors.push(TypeError::new(
                        id.0,
                        format!(
                            "effect `{effect}` is not handled in this context; \
                             wrap in `handle {effect} with <handler> in ...`"
                        ),
                    ));
                    None
                }
            }
            Expr::Handle {
                effect,
                handler,
                body,
                id,
                ..
            } => {
                if self.effect_defs.get(effect).is_none() {
                    self.errors.push(TypeError::new(
                        id.0,
                        format!("unknown effect `{effect}`"),
                    ));
                    let _ = self.infer_c_expr(handler, env);
                    return self.infer_c_expr(body, env);
                }
                let handler_value_type = self.effect_handler_type(effect);
                // Infer body first to determine handle result type (needed for resume typing)
                let mut body_env = env.clone();
                if let Some(ref ty) = handler_value_type {
                    body_env.insert(format!("__effect_{effect}"), ty.clone());
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
                        if let Some(def) = self.effect_defs.get(effect).cloned() {
                            self.check_bundle_against_effect(
                                entries,
                                &def,
                                effect,
                                bundle_id.0,
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
                                id.0,
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
                    if let Some(def) = self.effect_defs.get(name).cloned() {
                        if let Some(op_ty) = def.operations.get(field) {
                            return Some(op_ty.clone());
                        }
                        self.errors.push(TypeError::new(
                            id.0,
                            format!("effect `{name}` has no operation `{field}`"),
                        ));
                        return None;
                    }
                }

                self.errors.push(TypeError::new(
                    id.0,
                    format!(
                        "member access `.{field}` on type `{}`",
                        render_v_type(&obj_ty)
                    ),
                ));
                None
            }
            Expr::Error { .. } => None,
            Expr::Thunk { .. }
            | Expr::Lambda { .. }
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
        node_id: u64,
    ) {
        match pattern {
            Pattern::Wildcard => {}
            Pattern::Bind(name) => {
                env.insert(name.clone(), expected.clone());
            }
            Pattern::Ctor { name, args } => {
                let Some(data_name) = nominal_head_name(expected) else {
                    self.errors.push(TypeError::new(
                        node_id,
                        format!(
                            "constructor pattern `{name}` used on non-data type {}",
                            render_v_type(expected)
                        ),
                    ));
                    return;
                };
                let Some(data_def) = self.data_defs.get(&data_name).cloned() else {
                    self.errors.push(TypeError::new(
                        node_id,
                        format!("unknown data type `{data_name}` in match scrutinee"),
                    ));
                    return;
                };
                if let Some(payload_types) = data_def.variants.get(name) {
                    if payload_types.len() != args.len() {
                        self.errors.push(TypeError::new(
                            node_id,
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
                                &head, &args_text, &data_def, node_id,
                            )
                            .unwrap_or_default()
                        }
                        _ => HashMap::new(),
                    };
                    for (arg, payload_ty) in args.iter().zip(payload_types.iter()) {
                        let expected_payload = subst_v_type(payload_ty, &subst);
                        self.bind_pattern(arg, &expected_payload, env, node_id);
                    }
                } else {
                    self.errors.push(TypeError::new(
                        node_id,
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
        node_id: u64,
    ) -> Option<HashMap<String, ValueType>> {
        if data_def.generics.is_empty() {
            return Some(HashMap::new());
        }
        if expected_args_text.len() != data_def.generics.len() {
            self.errors.push(TypeError::new(
                node_id,
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
                self.errors.push(TypeError::new(
                    node_id,
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
                    effect: lhs_effect,
                },
                CompType::Fn {
                    params: rhs_params,
                    ret: rhs_ret,
                    effect: rhs_effect,
                },
            ) => {
                if lhs_params.len() != rhs_params.len() || lhs_effect != rhs_effect {
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

    /// Compute the value type that a handler for effect `name` must have.
    /// Always returns `Named(effect_name)` — effects are bundles regardless of op count.
    fn effect_handler_type(&self, name: &str) -> Option<ValueType> {
        self.effect_defs.get(name)?;
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
            let Some(pattern) = parse_match_pattern(&arm.pattern) else {
                continue;
            };
            if !self.pattern_compatible_with_type(&pattern, scrutinee_ty) {
                continue;
            }
            if !self.is_useful_pattern(&matrix, scrutinee_ty, &pattern) {
                self.errors.push(TypeError::new(
                    arm_node_id(arm),
                    "unreachable match arm: pattern already covered".to_owned(),
                ));
                continue;
            }
            matrix.push(vec![pattern]);
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
    None
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
            effect,
        } => CompType::Fn {
            params: params.iter().map(|p| subst_v_type(p, subst)).collect(),
            ret: Box::new(subst_c_type(ret, subst)),
            effect: effect.clone(),
        },
    }
}

fn normalize_type_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_effect_text(text: &str) -> String {
    text.replace("{ }", "{}")
}

fn parse_match_pattern(pattern: &str) -> Option<Pattern> {
    let mut parser = PatternParser::new(pattern);
    let pat = parser.parse_pattern();
    if parser.failed || parser.peek().is_some() {
        return None;
    }
    Some(pat)
}

fn invalid_pattern_message(pattern: &str) -> String {
    let text = pattern.trim();
    if text.contains('(') && !text.starts_with('.') {
        "constructor pattern must start with `.`".to_owned()
    } else {
        "invalid match pattern".to_owned()
    }
}

fn pattern_bindings(pat: &Pattern) -> Vec<String> {
    let mut out = Vec::new();
    collect_pattern_bindings(pat, &mut out);
    out
}

fn collect_pattern_bindings(pat: &Pattern, out: &mut Vec<String>) {
    match pat {
        Pattern::Wildcard => {}
        Pattern::Bind(name) => out.push(name.clone()),
        Pattern::Ctor { args, .. } => {
            for arg in args {
                collect_pattern_bindings(arg, out);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Pattern {
    Wildcard,
    Bind(String),
    Ctor { name: String, args: Vec<Pattern> },
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
                .map(render_pattern)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PatternToken {
    Ident(String),
    Underscore,
    Dot,
    LParen,
    RParen,
    Comma,
}

struct PatternParser {
    tokens: Vec<PatternToken>,
    index: usize,
    failed: bool,
}

impl PatternParser {
    fn new(text: &str) -> Self {
        Self {
            tokens: lex_pattern(text),
            index: 0,
            failed: false,
        }
    }

    fn parse_pattern(&mut self) -> Pattern {
        let Some(token) = self.bump() else {
            self.failed = true;
            return Pattern::Wildcard;
        };
        match token {
            PatternToken::Underscore => Pattern::Wildcard,
            PatternToken::Dot => {
                let Some(PatternToken::Ident(name)) = self.bump() else {
                    self.failed = true;
                    return Pattern::Wildcard;
                };
                self.parse_constructor_pattern(name)
            }
            PatternToken::Ident(head) => {
                if head == "let" || head == "mut" {
                    let Some(PatternToken::Ident(name)) = self.bump() else {
                        self.failed = true;
                        return Pattern::Wildcard;
                    };
                    if is_binding_name(&name) {
                        Pattern::Bind(name)
                    } else {
                        self.failed = true;
                        Pattern::Wildcard
                    }
                } else if is_binding_name(&head) {
                    Pattern::Bind(head)
                } else {
                    self.failed = true;
                    Pattern::Wildcard
                }
            }
            _ => {
                self.failed = true;
                Pattern::Wildcard
            }
        }
    }

    fn peek(&self) -> Option<&PatternToken> {
        self.tokens.get(self.index)
    }

    fn parse_constructor_pattern(&mut self, name: String) -> Pattern {
        if self.peek() == Some(&PatternToken::LParen) {
            self.bump(); // (
            let mut args = Vec::new();
            if self.peek() != Some(&PatternToken::RParen) {
                loop {
                    let arg = self.parse_pattern();
                    args.push(arg);
                    if self.peek() == Some(&PatternToken::Comma) {
                        self.bump();
                        continue;
                    }
                    break;
                }
            }
            if self.peek() == Some(&PatternToken::RParen) {
                self.bump();
                Pattern::Ctor { name, args }
            } else {
                self.failed = true;
                Pattern::Wildcard
            }
        } else {
            Pattern::Ctor {
                name,
                args: Vec::new(),
            }
        }
    }

    fn bump(&mut self) -> Option<PatternToken> {
        let out = self.tokens.get(self.index).cloned();
        if out.is_some() {
            self.index += 1;
        }
        out
    }
}

fn lex_pattern(text: &str) -> Vec<PatternToken> {
    let mut out = Vec::new();
    let mut i = 0;
    let bytes = text.as_bytes();
    while i < bytes.len() {
        let ch = text[i..].chars().next().unwrap_or('\0');
        if ch.is_whitespace() {
            i += ch.len_utf8();
            continue;
        }
        match ch {
            '_' => {
                out.push(PatternToken::Underscore);
                i += 1;
            }
            '.' => {
                out.push(PatternToken::Dot);
                i += 1;
            }
            '(' => {
                out.push(PatternToken::LParen);
                i += 1;
            }
            ')' => {
                out.push(PatternToken::RParen);
                i += 1;
            }
            ',' => {
                out.push(PatternToken::Comma);
                i += 1;
            }
            _ => {
                if ch == '_' || ch.is_alphabetic() {
                    let start = i;
                    i += ch.len_utf8();
                    while i < bytes.len() {
                        let c = text[i..].chars().next().unwrap_or('\0');
                        if c == '_' || c.is_alphanumeric() {
                            i += c.len_utf8();
                        } else {
                            break;
                        }
                    }
                    out.push(PatternToken::Ident(text[start..i].to_owned()));
                } else {
                    i += ch.len_utf8();
                }
            }
        }
    }
    out
}

fn is_binding_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_alphabetic()) && chars.all(|c| c == '_' || c.is_alphanumeric())
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

fn render_expr_head(expr: &Expr) -> String {
    match expr {
        Expr::Ident { name, .. } => name.clone(),
        Expr::Force { expr, .. } => format!("force {}", render_expr_head(expr)),
        Expr::Apply { callee, .. } => format!("{}(...)", render_expr_head(callee)),
        Expr::Ctor { name, .. } => name.clone(),
        Expr::Roll { expr, .. } => render_expr_head(expr),
        Expr::Perform { effect, .. } => format!("perform {effect}"),
        Expr::Handle { effect, .. } => format!("handle {effect}"),
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
                id.0,
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
                    id.0,
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
    match expr {
        Expr::Ident { id, .. } => id.0,
        Expr::String { id, .. } => id.0,
        Expr::Produce { id, .. } => id.0,
        Expr::Thunk { id, .. } => id.0,
        Expr::Force { id, .. } => id.0,
        Expr::Lambda { id, .. } => id.0,
        Expr::Apply { id, .. } => id.0,
        Expr::Unroll { id, .. } => id.0,
        Expr::LetIn { id, .. } => id.0,
        Expr::Match { id, .. } => id.0,
        Expr::Ctor { id, .. } => id.0,
        Expr::Roll { id, .. } => id.0,
        Expr::Perform { id, .. } => id.0,
        Expr::Handle { id, .. } => id.0,
        Expr::Ann { id, .. } => id.0,
        Expr::Error { id, .. } => id.0,
        Expr::Bundle { id, .. } => id.0,
        Expr::Member { id, .. } => id.0,
    }
}

fn param_node_id(param: &lir::ParamDecl) -> u64 {
    lir::source_id("param", param.source_span).0
}

fn payload_node_id(variant: &lir::VariantDecl, index: usize) -> u64 {
    variant
        .payload_spans
        .get(index)
        .copied()
        .map(|span| lir::source_id("variant-payload", span).0)
        .unwrap_or(variant.id.0)
}

fn type_sig_node_id(
    tag: &str,
    span: Option<Span>,
    fallback: lir::ContentHash,
) -> u64 {
    span.map(|span| lir::source_id(tag, span).0)
        .unwrap_or(fallback.0)
}

fn arm_node_id(arm: &lir::MatchArm) -> u64 {
    lir::source_id("match-arm", arm.source_span).0
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
