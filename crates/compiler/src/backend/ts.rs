use crate::{
    backend::{Backend, BackendError, BackendKind, CodegenTarget},
    lir,
    types::{Pattern, TypeExpr},
};
use simple_ts_ast as tsast;
use std::cell::Cell;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct TypeScriptBackend;

struct LoweringContext {
    direct_callable_arities: HashMap<String, usize>,
    /// Maps function name → cap runtime param names (e.g. `__cap_Add_Number`).
    fn_caps: HashMap<String, Vec<String>>,
    /// Maps (impl_const_name, method_name) → arity for uncurrying member calls.
    impl_method_arities: HashMap<(String, String), usize>,
    match_counter: Cell<usize>,
    k_counter: Cell<usize>,
}

impl LoweringContext {
    fn next_match_name(&self) -> String {
        let n = self.match_counter.get();
        self.match_counter.set(n + 1);
        format!("__match_{n}")
    }

    fn next_k_name(&self) -> String {
        let n = self.k_counter.get();
        self.k_counter.set(n + 1);
        format!("__k_{n}")
    }
}

impl TypeScriptBackend {
    pub fn new() -> Self {
        Self
    }

    fn lower_file(&self, file: &lir::File) -> Result<tsast::Program, BackendError> {
        let mut body = Vec::new();
        let mut extern_names = HashMap::new();
        let ctx = LoweringContext {
            direct_callable_arities: collect_direct_callable_arities(file),
            fn_caps: collect_fn_caps(file),
            impl_method_arities: collect_impl_method_arities(file),
            match_counter: Cell::new(0),
            k_counter: Cell::new(0),
        };

        // Deduplicate extern types: prefer annotated over bare
        let mut deduped_extern_types: HashMap<String, &lir::ExternTypeDecl> = HashMap::new();
        for item in &file.items {
            if let lir::Item::ExternType(ext) = item {
                deduped_extern_types
                    .entry(ext.name.clone())
                    .and_modify(|existing| {
                        if ext.extern_name.is_some() {
                            *existing = ext;
                        }
                    })
                    .or_insert(ext);
            }
        }

        // Emit deduplicated extern types
        for ext in deduped_extern_types.values() {
            body.push(tsast::Stmt::TypeAlias(tsast::TypeAlias {
                export: true,
                name: ext.name.clone(),
                type_params: Vec::new(),
                ty: ts_type_from_extern_name(ext),
            }));
        }

        for item in &file.items {
            match item {
                lir::Item::ExternType(_) => {
                    // Already emitted above (deduplicated)
                }
                lir::Item::ExternFn(func) => {
                    let params = func
                        .params
                        .iter()
                        .map(|param| {
                            tsast::Param::new(&param.name)
                                .with_type(lower_type_expr_to_ts_type(&param.ty.value))
                        })
                        .collect::<Vec<_>>();
                    let unit_ty = TypeExpr::Named("Unit".to_owned());
                    let return_ty = func
                        .return_type
                        .as_ref()
                        .map(|s| &s.value)
                        .unwrap_or(&unit_ty);
                    let extern_path = func
                        .extern_name
                        .clone()
                        .unwrap_or_else(|| format!("globalThis.{}", func.name));
                    extern_names.insert(func.name.clone(), extern_path.clone());
                    let body_expr = tsast::Expr::Call {
                        callee: Box::new(expr_from_extern_path(&extern_path)),
                        args: func
                            .params
                            .iter()
                            .map(|p| tsast::Expr::Ident(p.name.clone()))
                            .collect(),
                    };
                    body.push(tsast::Stmt::Function(tsast::FunctionDecl {
                        export: true,
                        name: func.name.clone(),
                        type_params: Vec::new(),
                        params,
                        return_type: Some(lower_type_expr_to_ts_type(return_ty)),
                        body: tsast::FunctionBody::Expr(Box::new(body_expr)),
                    }));
                }
                lir::Item::Data(data) => {
                    body.push(tsast::Stmt::TypeAlias(tsast::TypeAlias {
                        export: true,
                        name: data.name.clone(),
                        type_params: data.generics.clone(),
                        ty: lower_data_type(data),
                    }));
                    body.push(tsast::Stmt::Const(lower_data_bundle_const(data)));
                }
                lir::Item::Cap(_) | lir::Item::Use(_) => {
                    // Cap declarations and use items produce no TS output
                }
                lir::Item::Fn(func) => {
                    body.push(tsast::Stmt::Function(lower_fn_decl(func, &ctx)?));
                }
                lir::Item::Impl(impl_decl) => {
                    body.push(tsast::Stmt::Const(lower_impl_const(impl_decl, &ctx)?));
                }
            }
        }

        let mut program = tsast::Program::new(body);
        specialize_operator_extern_wrappers(&mut program, &extern_names);
        tsast::lower_expression_bodies(&mut program);
        tsast::flatten_iifes(&mut program);
        tsast::return_lifting(&mut program);
        tsast::return_lifting(&mut program);
        tsast::flatten_iifes(&mut program); // catch IIFEs exposed by return_lifting
        validate_program_has_no_any_or_unknown(&program)?;
        Ok(program)
    }
}

fn collect_direct_callable_arities(file: &lir::File) -> HashMap<String, usize> {
    let mut out = HashMap::new();
    for item in &file.items {
        match item {
            lir::Item::ExternFn(func) => {
                out.insert(func.name.clone(), func.params.len());
            }
            lir::Item::Fn(func) => {
                // Exclude effectful functions — they need CPS handling at call sites
                let is_effectful = func
                    .cap
                    .as_ref()
                    .map_or(false, |c| c.is_effectful());
                if !is_effectful {
                    out.insert(func.name.clone(), func.params.len());
                }
            }
            _ => {}
        }
    }
    out
}

fn collect_impl_method_arities(file: &lir::File) -> HashMap<(String, String), usize> {
    let mut out = HashMap::new();
    for item in &file.items {
        if let lir::Item::Impl(impl_decl) = item {
            let const_name = impl_const_name(impl_decl);
            for method in &impl_decl.methods {
                out.insert(
                    (const_name.clone(), method.name.clone()),
                    method.params.len(),
                );
            }
        }
    }
    out
}

fn collect_fn_caps(file: &lir::File) -> HashMap<String, Vec<String>> {
    let mut out = HashMap::new();
    for item in &file.items {
        match item {
            lir::Item::ExternFn(func) => {
                let caps = func
                    .cap
                    .as_ref()
                    .map(|c| c.cap_mangled_params())
                    .unwrap_or_default();
                out.insert(func.name.clone(), caps);
            }
            lir::Item::Fn(func) => {
                let caps = func
                    .cap
                    .as_ref()
                    .map(|c| c.cap_mangled_params())
                    .unwrap_or_default();
                out.insert(func.name.clone(), caps);
            }
            _ => {}
        }
    }
    out
}

/// Build the runtime JS variable name for a cap with optional type args.
/// e.g. ("Add", ["Number"]) → "__cap_Add_Number", ("IO", []) → "__cap_IO"
fn cap_runtime_name(cap: &str, type_args: &[String]) -> String {
    if type_args.is_empty() {
        format!("__cap_{cap}")
    } else {
        format!("__cap_{}_{}", cap, type_args.join("_"))
    }
}

fn lower_fn_decl(
    func: &lir::FnDecl,
    ctx: &LoweringContext,
) -> Result<tsast::FunctionDecl, BackendError> {
    let (lowered_params, lowered_body) = unwrap_fn_value(&func.value)?;
    if lowered_params.len() != func.params.len() {
        return Err(BackendError::EmitFailed(format!(
            "function `{}` lowered to {} lambda params but signature has {} params",
            func.name,
            lowered_params.len(),
            func.params.len()
        )));
    }

    let mut params: Vec<tsast::Param> = func
        .params
        .iter()
        .zip(lowered_params.iter())
        .map(|(param, lowered_name)| {
            tsast::Param::new(lowered_name).with_type(lower_type_expr_to_ts_type(&param.ty.value))
        })
        .collect();

    let caps: Vec<String> = func
        .cap
        .as_ref()
        .map(|c| c.cap_mangled_params())
        .unwrap_or_default();

    if !caps.is_empty() {
        // Effectful function: add __cap_E params and __k, CPS-compile body
        for cap in &caps {
            params.push(tsast::Param::new(cap));
        }
        params.push(tsast::Param::new("__k"));
        let cps_body = lower_cps_expr(
            lowered_body,
            tsast::Expr::Ident("__k".to_owned()),
            &caps,
            ctx,
        );
        Ok(tsast::FunctionDecl {
            export: true,
            name: func.name.clone(),
            type_params: func.generics.clone(),
            params,
            return_type: Some(tsast::TsType::Void),
            body: tsast::FunctionBody::Expr(Box::new(cps_body)),
        })
    } else {
        let unit_ty = TypeExpr::Named("Unit".to_owned());
        let return_ty = func
            .return_type
            .as_ref()
            .map(|s| &s.value)
            .unwrap_or(&unit_ty);
        Ok(tsast::FunctionDecl {
            export: true,
            name: func.name.clone(),
            type_params: func.generics.clone(),
            params,
            return_type: Some(lower_type_expr_to_ts_type(return_ty)),
            body: tsast::FunctionBody::Expr(Box::new(lower_expr(lowered_body, ctx))),
        })
    }
}

fn lower_impl_const(
    impl_decl: &lir::ImplDecl,
    ctx: &LoweringContext,
) -> Result<tsast::ConstDecl, BackendError> {
    let const_name = impl_const_name(impl_decl);

    let mut properties = Vec::new();
    for method in &impl_decl.methods {
        let (lowered_params, lowered_body) = unwrap_fn_value(&method.value)?;
        if lowered_params.len() != method.params.len() {
            return Err(BackendError::EmitFailed(format!(
                "impl method `{}` lowered to {} lambda params but signature has {} params",
                method.name,
                lowered_params.len(),
                method.params.len()
            )));
        }
        let params: Vec<tsast::Param> = method
            .params
            .iter()
            .zip(lowered_params.iter())
            .map(|(param, lowered_name)| {
                tsast::Param::new(lowered_name)
                    .with_type(lower_type_expr_to_ts_type(&param.ty.value))
            })
            .collect();
        let unit_ty = TypeExpr::Named("Unit".to_owned());
        let return_ty = method
            .return_type
            .as_ref()
            .map(|s| &s.value)
            .unwrap_or(&unit_ty);
        let body_expr = lower_expr(lowered_body, ctx);
        properties.push(tsast::ObjectProp {
            key: tsast::ObjectKey::Ident(method.name.clone()),
            value: tsast::Expr::Arrow {
                params,
                return_type: Some(lower_type_expr_to_ts_type(return_ty)),
                body: Box::new(tsast::FunctionBody::Expr(Box::new(body_expr))),
            },
        });
    }

    Ok(tsast::ConstDecl {
        export: true,
        name: const_name,
        type_ann: None,
        init: tsast::Expr::Object(properties),
    })
}

/// Compute the JS/TS const name for an impl block.
fn impl_const_name(impl_decl: &lir::ImplDecl) -> String {
    if let Some(name) = &impl_decl.name {
        // Named impl: use the given name
        name.clone()
    } else if let Some(cap) = &impl_decl.capability {
        // Unnamed cap impl: __impl_{Target}_{Cap}
        let target = impl_decl.target_type.value.display();
        let cap = cap.value.display();
        format!("__impl_{target}_{cap}")
    } else {
        // Inherent impl: name after target type
        impl_decl.target_type.value.display()
    }
}

fn unwrap_fn_value(value: &lir::Expr) -> Result<(Vec<String>, &lir::Expr), BackendError> {
    let lir::Expr::Thunk { expr, .. } = value else {
        return Err(BackendError::EmitFailed(
            "lowered function value must start with thunk".to_owned(),
        ));
    };

    let mut params = Vec::new();
    let mut cursor = expr.as_ref();
    while let lir::Expr::Lambda { param, body, .. } = cursor {
        params.push(param.clone());
        cursor = body.as_ref();
    }
    Ok((params, cursor))
}

impl Backend for TypeScriptBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::TypeScript
    }

    fn supports(&self, target: CodegenTarget) -> bool {
        matches!(
            target,
            CodegenTarget::TypeScript
                | CodegenTarget::TypeScriptDefinition
                | CodegenTarget::JavaScript
        )
    }

    fn emit(&self, file: &lir::File, target: CodegenTarget) -> Result<String, BackendError> {
        let program = self.lower_file(file)?;
        let target = match target {
            CodegenTarget::TypeScript => tsast::EmitTarget::TypeScript,
            CodegenTarget::TypeScriptDefinition => tsast::EmitTarget::TypeScriptDefinition,
            CodegenTarget::JavaScript => tsast::EmitTarget::JavaScript,
            _ => return Err(BackendError::UnsupportedTarget(target)),
        };

        let emitted = tsast::Emitter::default().emit_program(&program, target);
        Ok(format!("{}{}", runtime_prelude(target), emitted))
    }
}

fn runtime_prelude(target: tsast::EmitTarget) -> &'static str {
    match target {
        tsast::EmitTarget::TypeScript => {
            "const LUMO_TAG = Symbol.for(\"Lumo/tag\");
type __LumoRuntime = { [LUMO_TAG]: string; args?: __LumoRuntime[] } | string | number | boolean | null | undefined | (() => __LumoRuntime);
const __lumo_is = (value: __LumoRuntime, pattern: string): boolean =>
  !!value && typeof value === \"object\" && LUMO_TAG in value && (value as { [LUMO_TAG]: string })[LUMO_TAG] === pattern;
const __lumo_match_error = (value: __LumoRuntime): never => { throw new Error(\"non-exhaustive match: \" + JSON.stringify(value)); };
const __lumo_error = (): never => { throw new Error(\"lumo runtime error\"); };
const __thunk = (fn: Function): any => { (fn as any).__t = 1; return fn; };
const __trampoline = (v: any): any => { while (v && (v as any).__t) v = (v as () => any)(); return v; };

"
        }
        tsast::EmitTarget::JavaScript => {
            "import { readFileSync as __fs_readFileSync, writeFileSync as __fs_writeFileSync } from \"node:fs\";
const LUMO_TAG = Symbol.for(\"Lumo/tag\");
const __lumo_is = (value, pattern) =>
  !!value && typeof value === \"object\" && LUMO_TAG in value && value[LUMO_TAG] === pattern;
const __lumo_match_error = (value) => { throw new Error(\"non-exhaustive match: \" + JSON.stringify(value)); };
const __lumo_error = () => { throw new Error(\"lumo runtime error\"); };
const __thunk = (fn) => { fn.__t = 1; return fn; };
const __trampoline = (v) => { while (v && v.__t) v = v(); return v; };
const str = { len: (s) => s.length, char_at: (s, i) => s[i] ?? \"\", slice: (s, a, b) => s.slice(a, b), concat: (a, b) => a + b, eq: (a, b) => a === b ? { [LUMO_TAG]: \"true\" } : { [LUMO_TAG]: \"false\" }, starts_with: (s, p) => s.startsWith(p) ? { [LUMO_TAG]: \"true\" } : { [LUMO_TAG]: \"false\" }, contains: (s, p) => s.includes(p) ? { [LUMO_TAG]: \"true\" } : { [LUMO_TAG]: \"false\" }, index_of: (s, p) => s.indexOf(p), trim: (s) => s.trim(), char_code_at: (s, i) => s.charCodeAt(i), from_char_code: (c) => String.fromCharCode(c), replace_all: (s, f, t) => s.replaceAll(f, t) };
const num = { eq: (a, b) => a === b ? { [LUMO_TAG]: \"true\" } : { [LUMO_TAG]: \"false\" }, lt: (a, b) => a < b ? { [LUMO_TAG]: \"true\" } : { [LUMO_TAG]: \"false\" }, gt: (a, b) => a > b ? { [LUMO_TAG]: \"true\" } : { [LUMO_TAG]: \"false\" }, lte: (a, b) => a <= b ? { [LUMO_TAG]: \"true\" } : { [LUMO_TAG]: \"false\" }, gte: (a, b) => a >= b ? { [LUMO_TAG]: \"true\" } : { [LUMO_TAG]: \"false\" }, add: (a, b) => a + b, sub: (a, b) => a - b, mul: (a, b) => a * b, floor: (a) => Math.floor(a), to_string: (n) => String(n) };
const console = globalThis.console;
const fs = { read_file: (p) => __fs_readFileSync(p, \"utf8\"), write_file: (p, c) => __fs_writeFileSync(p, c, \"utf8\") };
const process = { ...globalThis.process, arg_at: (i) => globalThis.process.argv[i + 1] ?? \"\", args_count: () => globalThis.process.argv.length - 1, exit: (c) => globalThis.process.exit(c), panic: (m) => { console.error(m); globalThis.process.exit(1); } };

"
        }
        tsast::EmitTarget::TypeScriptDefinition => "",
    }
}

fn lower_type_expr_to_ts_type(ty: &TypeExpr) -> tsast::TsType {
    match ty {
        TypeExpr::Named(name) if name == "Unit" => tsast::TsType::Void,
        TypeExpr::Named(name) => tsast::TsType::TypeRef(name.clone()),
        TypeExpr::App { head, args } => tsast::TsType::TypeRef(format!(
            "{head}<{}>",
            args.iter()
                .map(type_expr_to_ts_text)
                .collect::<Vec<_>>()
                .join(", ")
        )),
        // `produce T` → just the inner type in TS
        TypeExpr::Produce(inner) => lower_type_expr_to_ts_type(inner),
        // `thunk T` → `() => T` in TS
        TypeExpr::Thunk(inner) => lower_type_expr_to_ts_type(inner),
        TypeExpr::Cap { name, .. } => tsast::TsType::TypeRef(name.clone()),
    }
}

fn lower_data_type(data: &lir::DataDecl) -> tsast::TsType {
    if data.variants.is_empty() {
        return tsast::TsType::Never;
    }
    let union = data
        .variants
        .iter()
        .map(|variant| {
            if variant.payload.is_empty() {
                format!("{{ [LUMO_TAG]: '{}' }}", variant.name)
            } else {
                let payloads = variant
                    .payload
                    .iter()
                    .map(|payload| type_expr_to_ts_text(&payload.value))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ [LUMO_TAG]: '{}', args: [{}] }}", variant.name, payloads)
            }
        })
        .collect::<Vec<_>>()
        .join(" | ");
    tsast::TsType::Raw(union)
}

fn ts_type_from_extern_name(ext: &lir::ExternTypeDecl) -> tsast::TsType {
    match ext.extern_name.as_deref().unwrap_or(&ext.name) {
        "string" => tsast::TsType::String,
        "number" => tsast::TsType::Number,
        "boolean" => tsast::TsType::Boolean,
        "void" => tsast::TsType::Void,
        "null" => tsast::TsType::Null,
        "undefined" => tsast::TsType::Undefined,
        "never" => tsast::TsType::Never,
        other => tsast::TsType::TypeRef(other.to_owned()),
    }
}

fn expr_from_extern_path(path: &str) -> tsast::Expr {
    let mut parts = path.split('.').filter(|s| !s.is_empty());
    let Some(head) = parts.next() else {
        return tsast::Expr::Ident("<invalid-extern>".to_owned());
    };
    let mut acc = if is_js_ident(head) {
        tsast::Expr::Ident(head.to_owned())
    } else {
        tsast::Expr::Index {
            object: Box::new(tsast::Expr::Ident("globalThis".to_owned())),
            index: Box::new(tsast::Expr::String(head.to_owned())),
        }
    };
    for part in parts {
        acc = if is_js_ident(part) {
            tsast::Expr::Member {
                object: Box::new(acc),
                property: part.to_owned(),
            }
        } else {
            tsast::Expr::Index {
                object: Box::new(acc),
                index: Box::new(tsast::Expr::String(part.to_owned())),
            }
        };
    }
    acc
}

fn is_js_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn specialize_operator_extern_wrappers(
    program: &mut tsast::Program,
    extern_names: &HashMap<String, String>,
) {
    for stmt in &mut program.body {
        let tsast::Stmt::Function(func) = stmt else {
            continue;
        };
        let Some(extern_name) = extern_names.get(&func.name) else {
            continue;
        };
        if let Some(expr) = specialize_operator_wrapper_expr(extern_name, &func.params) {
            func.body = tsast::FunctionBody::Expr(Box::new(expr));
            continue;
        }
        if let Some(expr) = specialize_stdlib_extern_body(extern_name, &func.params) {
            func.body = tsast::FunctionBody::Expr(Box::new(expr));
        }
    }
}

/// Inline implementations for stdlib extern functions that don't have
/// corresponding JS globals (e.g., `str.len` → `s.length`).
fn specialize_stdlib_extern_body(
    extern_name: &str,
    params: &[tsast::Param],
) -> Option<tsast::Expr> {
    let p = |i: usize| -> tsast::Expr { tsast::Expr::Ident(params[i].name.clone()) };

    let method_call = |obj: tsast::Expr, method: &str, args: Vec<tsast::Expr>| -> tsast::Expr {
        tsast::Expr::Call {
            callee: Box::new(tsast::Expr::Member {
                object: Box::new(obj),
                property: method.to_owned(),
            }),
            args,
        }
    };

    // Wrap a JS boolean expression in a Lumo Bool ADT:
    // `cond ? Bool["true"] : Bool["false"]`
    let bool_wrap = |cond: tsast::Expr| -> tsast::Expr {
        tsast::Expr::IfElse {
            cond: Box::new(cond),
            then_expr: Box::new(tsast::Expr::Index {
                object: Box::new(tsast::Expr::Ident("Bool".to_owned())),
                index: Box::new(tsast::Expr::String("true".to_owned())),
            }),
            else_expr: Box::new(tsast::Expr::Index {
                object: Box::new(tsast::Expr::Ident("Bool".to_owned())),
                index: Box::new(tsast::Expr::String("false".to_owned())),
            }),
        }
    };

    let cmp = |op: tsast::BinaryOp| -> tsast::Expr {
        bool_wrap(tsast::Expr::Binary {
            left: Box::new(p(0)),
            op,
            right: Box::new(p(1)),
        })
    };

    match extern_name {
        // String operations
        "str.len" => Some(tsast::Expr::Member {
            object: Box::new(p(0)),
            property: "length".to_owned(),
        }),
        "str.char_at" => Some(method_call(p(0), "charAt", vec![p(1)])),
        "str.slice" => Some(method_call(p(0), "slice", vec![p(1), p(2)])),
        "str.concat" => Some(tsast::Expr::Binary {
            left: Box::new(p(0)),
            op: tsast::BinaryOp::Add,
            right: Box::new(p(1)),
        }),
        "str.eq" => Some(cmp(tsast::BinaryOp::EqEqEq)),
        "str.starts_with" => Some(bool_wrap(method_call(p(0), "startsWith", vec![p(1)]))),
        "str.contains" => Some(bool_wrap(method_call(p(0), "includes", vec![p(1)]))),
        "str.index_of" => Some(method_call(p(0), "indexOf", vec![p(1)])),
        "str.trim" => Some(method_call(p(0), "trim", vec![])),
        "str.char_code_at" => Some(method_call(p(0), "charCodeAt", vec![p(1)])),
        "str.from_char_code" => Some(tsast::Expr::Call {
            callee: Box::new(tsast::Expr::Member {
                object: Box::new(tsast::Expr::Ident("String".to_owned())),
                property: "fromCharCode".to_owned(),
            }),
            args: vec![p(0)],
        }),
        "str.replace_all" => Some(method_call(p(0), "replaceAll", vec![p(1), p(2)])),
        "num.to_string" => Some(method_call(p(0), "toString", vec![])),

        // Number operations
        "num.eq" => Some(cmp(tsast::BinaryOp::EqEqEq)),
        "num.cmp" => {
            // a < b ? Ordering.less : a === b ? Ordering.equal : Ordering.greater
            let ordering_ctor = |variant: &str| -> tsast::Expr {
                tsast::Expr::Index {
                    object: Box::new(tsast::Expr::Ident("Ordering".to_owned())),
                    index: Box::new(tsast::Expr::String(variant.to_owned())),
                }
            };
            Some(tsast::Expr::IfElse {
                cond: Box::new(tsast::Expr::Binary {
                    left: Box::new(p(0)),
                    op: tsast::BinaryOp::Lt,
                    right: Box::new(p(1)),
                }),
                then_expr: Box::new(ordering_ctor("less")),
                else_expr: Box::new(tsast::Expr::IfElse {
                    cond: Box::new(tsast::Expr::Binary {
                        left: Box::new(p(0)),
                        op: tsast::BinaryOp::EqEqEq,
                        right: Box::new(p(1)),
                    }),
                    then_expr: Box::new(ordering_ctor("equal")),
                    else_expr: Box::new(ordering_ctor("greater")),
                }),
            })
        }
        "num.add" => Some(tsast::Expr::Binary {
            left: Box::new(p(0)),
            op: tsast::BinaryOp::Add,
            right: Box::new(p(1)),
        }),
        "num.sub" => Some(tsast::Expr::Binary {
            left: Box::new(p(0)),
            op: tsast::BinaryOp::Sub,
            right: Box::new(p(1)),
        }),
        "num.mul" => Some(tsast::Expr::Binary {
            left: Box::new(p(0)),
            op: tsast::BinaryOp::Mul,
            right: Box::new(p(1)),
        }),
        "num.div" => Some(tsast::Expr::Binary {
            left: Box::new(p(0)),
            op: tsast::BinaryOp::Div,
            right: Box::new(p(1)),
        }),
        "num.mod" => Some(tsast::Expr::Binary {
            left: Box::new(p(0)),
            op: tsast::BinaryOp::Mod,
            right: Box::new(p(1)),
        }),
        "num.neg" => Some(tsast::Expr::Unary {
            op: tsast::UnaryOp::Minus,
            expr: Box::new(p(0)),
        }),
        "num.floor" => Some(tsast::Expr::Call {
            callee: Box::new(tsast::Expr::Member {
                object: Box::new(tsast::Expr::Ident("Math".to_owned())),
                property: "floor".to_owned(),
            }),
            args: vec![p(0)],
        }),

        // Bool operations — Lumo Bool is an ADT: not(true) → false, not(false) → true
        "bool.not" => {
            // p(0)[LUMO_TAG] === "false" ? Bool["true"] : Bool["false"]
            Some(bool_wrap(tsast::Expr::Binary {
                left: Box::new(tsast::Expr::Index {
                    object: Box::new(p(0)),
                    index: Box::new(tsast::Expr::Ident("LUMO_TAG".to_owned())),
                }),
                op: tsast::BinaryOp::EqEqEq,
                right: Box::new(tsast::Expr::String("false".to_owned())),
            }))
        }

        _ => None,
    }
}

fn specialize_operator_wrapper_expr(
    extern_name: &str,
    params: &[tsast::Param],
) -> Option<tsast::Expr> {
    let operator = parse_extern_operator(extern_name)?;
    match operator {
        ExternOperator::Unary(op) if params.len() == 1 => Some(tsast::Expr::Unary {
            op,
            expr: Box::new(tsast::Expr::Ident(params[0].name.clone())),
        }),
        ExternOperator::Binary(op) if params.len() == 2 => Some(tsast::Expr::Binary {
            left: Box::new(tsast::Expr::Ident(params[0].name.clone())),
            op,
            right: Box::new(tsast::Expr::Ident(params[1].name.clone())),
        }),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExternOperator {
    Unary(tsast::UnaryOp),
    Binary(tsast::BinaryOp),
}

fn parse_extern_operator(extern_name: &str) -> Option<ExternOperator> {
    let segment = extern_name.rsplit('.').next().unwrap_or(extern_name);
    if segment.len() >= 3 && segment.starts_with('_') && segment.ends_with('_') {
        return map_binary_operator(&segment[1..segment.len() - 1]).map(ExternOperator::Binary);
    }
    if segment.len() >= 2 && segment.ends_with('_') {
        return map_unary_operator(&segment[..segment.len() - 1]).map(ExternOperator::Unary);
    }
    None
}

fn map_unary_operator(op: &str) -> Option<tsast::UnaryOp> {
    match op {
        "+" => Some(tsast::UnaryOp::Plus),
        "-" => Some(tsast::UnaryOp::Minus),
        "!" => Some(tsast::UnaryOp::Not),
        "~" => Some(tsast::UnaryOp::BitNot),
        _ => None,
    }
}

fn map_binary_operator(op: &str) -> Option<tsast::BinaryOp> {
    match op {
        "+" => Some(tsast::BinaryOp::Add),
        "-" => Some(tsast::BinaryOp::Sub),
        "*" => Some(tsast::BinaryOp::Mul),
        "/" => Some(tsast::BinaryOp::Div),
        "^" | "**" => Some(tsast::BinaryOp::Exp),
        "==" | "===" => Some(tsast::BinaryOp::EqEqEq),
        "!=" | "!==" => Some(tsast::BinaryOp::NotEqEq),
        "<" => Some(tsast::BinaryOp::Lt),
        "<=" => Some(tsast::BinaryOp::Lte),
        ">" => Some(tsast::BinaryOp::Gt),
        ">=" => Some(tsast::BinaryOp::Gte),
        "&&" => Some(tsast::BinaryOp::AndAnd),
        "||" => Some(tsast::BinaryOp::OrOr),
        _ => None,
    }
}

fn lower_data_bundle_const(data: &lir::DataDecl) -> tsast::ConstDecl {
    let init = tsast::Expr::Object(
        data.variants
            .iter()
            .map(|variant| tsast::ObjectProp {
                key: tsast::ObjectKey::String(variant.name.clone()),
                value: lower_variant_ctor_expr(data, variant),
            })
            .collect(),
    );
    tsast::ConstDecl {
        export: true,
        name: data.name.clone(),
        type_ann: Some(tsast::TsType::Raw(render_data_bundle_type(data))),
        init,
    }
}

fn lower_variant_ctor_expr(_data: &lir::DataDecl, variant: &lir::VariantDecl) -> tsast::Expr {
    if variant.payload.is_empty() {
        let fields = vec![tsast::ObjectProp {
            key: tsast::ObjectKey::Computed(Box::new(tsast::Expr::Ident("LUMO_TAG".to_owned()))),
            value: tsast::Expr::String(variant.name.clone()),
        }];
        return tsast::Expr::Object(fields);
    }

    let params = variant
        .payload
        .iter()
        .enumerate()
        .map(|(index, _)| tsast::Param::new(format!("arg{index}")))
        .collect::<Vec<_>>();
    let mut fields = vec![tsast::ObjectProp {
        key: tsast::ObjectKey::Computed(Box::new(tsast::Expr::Ident("LUMO_TAG".to_owned()))),
        value: tsast::Expr::String(variant.name.clone()),
    }];
    fields.push(tsast::ObjectProp {
        key: tsast::ObjectKey::Ident("args".to_owned()),
        value: tsast::Expr::Array(
            params
                .iter()
                .map(|param| tsast::Expr::Ident(param.name.clone()))
                .collect(),
        ),
    });
    tsast::Expr::Arrow {
        params,
        return_type: None,
        body: Box::new(tsast::FunctionBody::Expr(Box::new(tsast::Expr::Object(
            fields,
        )))),
    }
}

fn render_data_bundle_type(data: &lir::DataDecl) -> String {
    let generics_decl = if data.generics.is_empty() {
        String::new()
    } else {
        format!("<{}>", data.generics.join(", "))
    };
    let result_type = format!("{}{}", data.name, generics_decl);
    let members = data
        .variants
        .iter()
        .map(|variant| {
            let params = variant
                .payload
                .iter()
                .enumerate()
                .map(|(index, payload)| {
                    let payload = type_expr_to_ts_text(&payload.value);
                    format!("arg{index}: {payload}")
                })
                .collect::<Vec<_>>()
                .join(", ");
            if variant.payload.is_empty() {
                if data.generics.is_empty() {
                    format!("\"{}\": {result_type}", variant.name)
                } else {
                    let never_args = vec!["never"; data.generics.len()].join(", ");
                    format!("\"{}\": {}<{never_args}>", variant.name, data.name)
                }
            } else if data.generics.is_empty() {
                format!("\"{}\": ({params}) => {result_type}", variant.name)
            } else {
                format!(
                    "\"{}\": {}({params}) => {result_type}",
                    variant.name, generics_decl
                )
            }
        })
        .collect::<Vec<_>>()
        .join("; ");
    format!("{{ {members} }}")
}

fn type_expr_to_ts_text(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named(name) if name == "Unit" => "void".to_owned(),
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::App { head, args } => format!(
            "{head}<{}>",
            args.iter()
                .map(type_expr_to_ts_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeExpr::Produce(inner) => type_expr_to_ts_text(inner),
        TypeExpr::Thunk(inner) => type_expr_to_ts_text(inner),
        TypeExpr::Cap { name, .. } => name.clone(),
    }
}

fn validate_program_has_no_any_or_unknown(program: &tsast::Program) -> Result<(), BackendError> {
    for stmt in &program.body {
        validate_stmt_has_no_any_or_unknown(stmt)?;
    }
    Ok(())
}

fn validate_stmt_has_no_any_or_unknown(stmt: &tsast::Stmt) -> Result<(), BackendError> {
    match stmt {
        tsast::Stmt::Expr(expr) => validate_expr_has_no_any_or_unknown(expr),
        tsast::Stmt::Return(expr) => {
            if let Some(expr) = expr {
                validate_expr_has_no_any_or_unknown(expr)?;
            }
            Ok(())
        }
        tsast::Stmt::Const(decl) => {
            if let Some(ty) = &decl.type_ann {
                validate_ts_type_has_no_any_or_unknown(ty)?;
            }
            validate_expr_has_no_any_or_unknown(&decl.init)
        }
        tsast::Stmt::If {
            cond,
            then_branch,
            else_branch,
        } => {
            validate_expr_has_no_any_or_unknown(cond)?;
            for stmt in &then_branch.stmts {
                validate_stmt_has_no_any_or_unknown(stmt)?;
            }
            if let Some(else_branch) = else_branch {
                for stmt in &else_branch.stmts {
                    validate_stmt_has_no_any_or_unknown(stmt)?;
                }
            }
            Ok(())
        }
        tsast::Stmt::Block(block) => {
            for stmt in &block.stmts {
                validate_stmt_has_no_any_or_unknown(stmt)?;
            }
            Ok(())
        }
        tsast::Stmt::Function(decl) => {
            for param in &decl.params {
                if let Some(ty) = &param.type_ann {
                    validate_ts_type_has_no_any_or_unknown(ty)?;
                }
            }
            let Some(ret) = decl.return_type.as_ref() else {
                return Err(BackendError::EmitFailed(format!(
                    "function `{}` has no return type",
                    decl.name
                )));
            };
            validate_ts_type_has_no_any_or_unknown(ret)?;
            match &decl.body {
                tsast::FunctionBody::Expr(expr) => validate_expr_has_no_any_or_unknown(expr),
                tsast::FunctionBody::Block(block) => {
                    for stmt in &block.stmts {
                        validate_stmt_has_no_any_or_unknown(stmt)?;
                    }
                    Ok(())
                }
            }
        }
        tsast::Stmt::Let { init, type_ann, .. } => {
            if let Some(ty) = type_ann {
                validate_ts_type_has_no_any_or_unknown(ty)?;
            }
            if let Some(init) = init {
                validate_expr_has_no_any_or_unknown(init)?;
            }
            Ok(())
        }
        tsast::Stmt::Assign { value, .. } => validate_expr_has_no_any_or_unknown(value),
        tsast::Stmt::TypeAlias(alias) => validate_ts_type_has_no_any_or_unknown(&alias.ty),
        tsast::Stmt::Interface(interface) => {
            for member in &interface.members {
                validate_ts_type_has_no_any_or_unknown(&member.ty)?;
            }
            Ok(())
        }
    }
}

fn validate_expr_has_no_any_or_unknown(expr: &tsast::Expr) -> Result<(), BackendError> {
    match expr {
        tsast::Expr::Arrow {
            params,
            return_type,
            body,
        } => {
            for param in params {
                if let Some(ty) = &param.type_ann {
                    validate_ts_type_has_no_any_or_unknown(ty)?;
                }
            }
            if let Some(ret) = return_type {
                validate_ts_type_has_no_any_or_unknown(ret)?;
            }
            match body.as_ref() {
                tsast::FunctionBody::Expr(expr) => validate_expr_has_no_any_or_unknown(expr),
                tsast::FunctionBody::Block(block) => {
                    for stmt in &block.stmts {
                        validate_stmt_has_no_any_or_unknown(stmt)?;
                    }
                    Ok(())
                }
            }
        }
        tsast::Expr::Unary { expr, .. } => validate_expr_has_no_any_or_unknown(expr),
        tsast::Expr::Binary { left, right, .. } => {
            validate_expr_has_no_any_or_unknown(left)?;
            validate_expr_has_no_any_or_unknown(right)
        }
        tsast::Expr::Void(expr) => validate_expr_has_no_any_or_unknown(expr),
        tsast::Expr::Call { callee, args } => {
            validate_expr_has_no_any_or_unknown(callee)?;
            for arg in args {
                validate_expr_has_no_any_or_unknown(arg)?;
            }
            Ok(())
        }
        tsast::Expr::Member { object, .. } => validate_expr_has_no_any_or_unknown(object),
        tsast::Expr::Index { object, index } => {
            validate_expr_has_no_any_or_unknown(object)?;
            validate_expr_has_no_any_or_unknown(index)
        }
        tsast::Expr::Array(items) => {
            for item in items {
                validate_expr_has_no_any_or_unknown(item)?;
            }
            Ok(())
        }
        tsast::Expr::Object(props) => {
            for prop in props {
                if let tsast::ObjectKey::Computed(expr) = &prop.key {
                    validate_expr_has_no_any_or_unknown(expr)?;
                }
                validate_expr_has_no_any_or_unknown(&prop.value)?;
            }
            Ok(())
        }
        tsast::Expr::IfElse {
            cond,
            then_expr,
            else_expr,
        } => {
            validate_expr_has_no_any_or_unknown(cond)?;
            validate_expr_has_no_any_or_unknown(then_expr)?;
            validate_expr_has_no_any_or_unknown(else_expr)
        }
        tsast::Expr::Ident(_)
        | tsast::Expr::String(_)
        | tsast::Expr::Number(_)
        | tsast::Expr::Bool(_)
        | tsast::Expr::Null
        | tsast::Expr::Undefined => Ok(()),
    }
}

fn validate_ts_type_has_no_any_or_unknown(ty: &tsast::TsType) -> Result<(), BackendError> {
    match ty {
        tsast::TsType::Any => Err(BackendError::EmitFailed(
            "refusing to emit TypeScript `any`".to_owned(),
        )),
        tsast::TsType::Unknown => Err(BackendError::EmitFailed(
            "refusing to emit TypeScript `unknown`".to_owned(),
        )),
        tsast::TsType::Array(inner) => validate_ts_type_has_no_any_or_unknown(inner),
        tsast::TsType::Union(items) => {
            for item in items {
                validate_ts_type_has_no_any_or_unknown(item)?;
            }
            Ok(())
        }
        tsast::TsType::Func { params, ret } => {
            for param in params {
                if let Some(ty) = &param.type_ann {
                    validate_ts_type_has_no_any_or_unknown(ty)?;
                } else {
                    return Err(BackendError::EmitFailed(format!(
                        "function type parameter `{}` has no type",
                        param.name
                    )));
                }
            }
            validate_ts_type_has_no_any_or_unknown(ret)
        }
        tsast::TsType::Raw(_) => Ok(()),
        tsast::TsType::Never
        | tsast::TsType::Void
        | tsast::TsType::Boolean
        | tsast::TsType::Number
        | tsast::TsType::String
        | tsast::TsType::Null
        | tsast::TsType::Undefined
        | tsast::TsType::TypeRef(_) => Ok(()),
    }
}

/// Build an immediately-invoked function expression: ((name) => body)(arg)
fn iife(name: &str, body: tsast::Expr, arg: tsast::Expr) -> tsast::Expr {
    tsast::Expr::Call {
        callee: Box::new(tsast::Expr::Arrow {
            params: vec![tsast::Param::new(name)],
            return_type: None,
            body: Box::new(tsast::FunctionBody::Expr(Box::new(body))),
        }),
        args: vec![arg],
    }
}

fn lower_expr(expr: &lir::Expr, ctx: &LoweringContext) -> tsast::Expr {
    match expr {
        lir::Expr::Ident { name, .. } if name == "Unit" => {
            tsast::Expr::Void(Box::new(tsast::Expr::Number(0.0)))
        }
        lir::Expr::Ident { name, .. } => tsast::Expr::Ident(name.clone()),
        lir::Expr::String { value, .. } => tsast::Expr::String(value.clone()),
        lir::Expr::Number { value, .. } => {
            tsast::Expr::Number(value.parse::<f64>().unwrap_or(0.0))
        }
        lir::Expr::Produce { expr, .. } => lower_expr(expr, ctx),
        lir::Expr::Thunk { expr, .. } => tsast::Expr::Arrow {
            params: Vec::new(),
            return_type: None,
            body: Box::new(tsast::FunctionBody::Expr(Box::new(lower_expr(expr, ctx)))),
        },
        lir::Expr::Force { expr, .. } => lower_force_expr(expr, ctx),
        lir::Expr::Lambda { param, body, .. } => tsast::Expr::Arrow {
            params: vec![tsast::Param::new(param)],
            return_type: None,
            body: Box::new(tsast::FunctionBody::Expr(Box::new(lower_expr(body, ctx)))),
        },
        lir::Expr::Apply { callee, arg, .. } => lower_apply_expr(callee, arg, ctx),
        lir::Expr::Unroll { expr, .. } => lower_expr(expr, ctx),
        lir::Expr::Roll { expr, .. } => lower_expr(expr, ctx),
        lir::Expr::Let {
            name, value, body, ..
        } => iife(name, lower_expr(body, ctx), lower_expr(value, ctx)),
        lir::Expr::Match {
            scrutinee, arms, ..
        } => lower_match_expr(scrutinee, arms, ctx),
        lir::Expr::Ctor { name, args, .. } => {
            if let Some((owner, variant)) = name.split_once('.') {
                let ctor = tsast::Expr::Index {
                    object: Box::new(tsast::Expr::Ident(owner.to_owned())),
                    index: Box::new(tsast::Expr::String(variant.to_owned())),
                };
                if args.is_empty() {
                    ctor
                } else {
                    tsast::Expr::Call {
                        callee: Box::new(ctor),
                        args: args.iter().map(|arg| lower_expr(arg, ctx)).collect(),
                    }
                }
            } else {
                let callee = tsast::Expr::Ident(name.clone());
                if args.is_empty() {
                    callee
                } else {
                    tsast::Expr::Call {
                        callee: Box::new(callee),
                        args: args.iter().map(|arg| lower_expr(arg, ctx)).collect(),
                    }
                }
            }
        }
        lir::Expr::Perform { cap, type_args, .. } => {
            tsast::Expr::Ident(cap_runtime_name(cap, type_args))
        }
        lir::Expr::Handle {
            cap, type_args, handler, body, ..
        } => {
            // All handles use CPS (deep CPS: effectful functions need continuation threading)
            lower_cps_handle(cap, type_args, handler, body, ctx)
        }
        lir::Expr::Ann { expr, .. } => lower_expr(expr, ctx),
        lir::Expr::Error { .. } => runtime_call("__lumo_error", Vec::new()),
        lir::Expr::Bundle { .. } => {
            // All bundles are cap handlers — use CPS entries (with __k)
            lower_handler_with_resume(expr, ctx)
        }
        lir::Expr::Member {
            object, field, ..
        } => tsast::Expr::Member {
            object: Box::new(lower_expr(object, ctx)),
            property: field.clone(),
        },
    }
}

fn lower_force_expr(expr: &lir::Expr, ctx: &LoweringContext) -> tsast::Expr {
    if let lir::Expr::Ident { name, .. } = expr {
        if let Some(arity) = ctx.direct_callable_arities.get(name) {
            if *arity == 0 {
                return tsast::Expr::Call {
                    callee: Box::new(tsast::Expr::Ident(name.clone())),
                    args: Vec::new(),
                };
            }
            return tsast::Expr::Ident(name.clone());
        }
    }

    tsast::Expr::Call {
        callee: Box::new(lower_expr(expr, ctx)),
        args: Vec::new(),
    }
}

fn lower_apply_expr(callee: &lir::Expr, arg: &lir::Expr, ctx: &LoweringContext) -> tsast::Expr {
    if let Some((root_name, args)) = collect_direct_apply_chain(callee, arg, ctx) {
        return tsast::Expr::Call {
            callee: Box::new(tsast::Expr::Ident(root_name)),
            args: args.into_iter().map(|arg| lower_expr(arg, ctx)).collect(),
        };
    }

    // Try uncurrying member-based call chains: Apply*(Member(Ident(obj), method), args)
    if let Some((obj_name, method, args)) = collect_member_apply_chain(callee, arg, ctx) {
        return tsast::Expr::Call {
            callee: Box::new(tsast::Expr::Member {
                object: Box::new(tsast::Expr::Ident(obj_name)),
                property: method,
            }),
            args: args.into_iter().map(|arg| lower_expr(arg, ctx)).collect(),
        };
    }

    tsast::Expr::Call {
        callee: Box::new(lower_expr(callee, ctx)),
        args: vec![lower_expr(arg, ctx)],
    }
}

fn collect_direct_apply_chain<'a>(
    callee: &'a lir::Expr,
    arg: &'a lir::Expr,
    ctx: &LoweringContext,
) -> Option<(String, Vec<&'a lir::Expr>)> {
    let mut args = vec![arg];
    let mut cursor = callee;

    while let lir::Expr::Apply {
        callee: inner_callee,
        arg: inner_arg,
        ..
    } = cursor
    {
        args.push(inner_arg.as_ref());
        cursor = inner_callee.as_ref();
    }

    let lir::Expr::Force { expr, .. } = cursor else {
        return None;
    };
    let lir::Expr::Ident { name, .. } = expr.as_ref() else {
        return None;
    };
    if ctx.direct_callable_arities.get(name).copied().unwrap_or(0) == 0 {
        return None;
    }

    args.reverse();
    Some((name.clone(), args))
}

/// Uncurry member-based call chains: `Apply*(Member(Ident(obj), method), args)`
/// into `(obj_name, method, args)` when the impl method arity matches.
fn collect_member_apply_chain<'a>(
    callee: &'a lir::Expr,
    arg: &'a lir::Expr,
    ctx: &LoweringContext,
) -> Option<(String, String, Vec<&'a lir::Expr>)> {
    let mut args = vec![arg];
    let mut cursor = callee;

    while let lir::Expr::Apply {
        callee: inner_callee,
        arg: inner_arg,
        ..
    } = cursor
    {
        args.push(inner_arg.as_ref());
        cursor = inner_callee.as_ref();
    }

    // LIR member calls use Member directly (no Force wrapper)
    let lir::Expr::Member { object, field, .. } = cursor else {
        return None;
    };
    let lir::Expr::Ident { name: obj_name, .. } = object.as_ref() else {
        return None;
    };
    let key = (obj_name.clone(), field.clone());
    if !ctx.impl_method_arities.contains_key(&key) {
        return None;
    }

    args.reverse();
    Some((obj_name.clone(), field.clone(), args))
}

fn lower_match_expr(
    scrutinee: &lir::Expr,
    arms: &[lir::MatchArm],
    ctx: &LoweringContext,
) -> tsast::Expr {
    let lowered_scrutinee = lower_expr(scrutinee, ctx);
    let scrutinee_name = ctx.next_match_name();
    let scrutinee_expr = tsast::Expr::Ident(scrutinee_name.clone());
    let rows = arms
        .iter()
        .map(|arm| MatchRow {
            patterns: vec![pattern_to_match_pattern(&arm.pattern)],
            bindings: Vec::new(),
            body: arm.body.clone(),
        })
        .collect::<Vec<_>>();
    let decision = build_match_decision(vec![scrutinee_expr.clone()], rows);
    let lowered = lower_match_decision(&scrutinee_expr, decision, &|body, bindings| {
        wrap_bindings(lower_expr(body, ctx), bindings)
    });

    iife(&scrutinee_name, lowered, lowered_scrutinee)
}


/// Wrap an expression with IIFE bindings: ((name) => expr)(value) for each binding.
fn wrap_bindings(mut expr: tsast::Expr, bindings: Vec<(String, tsast::Expr)>) -> tsast::Expr {
    for (name, value) in bindings.into_iter().rev() {
        expr = iife(&name, expr, value);
    }
    expr
}

fn payload_access_expr(value: &tsast::Expr, index: usize) -> tsast::Expr {
    tsast::Expr::Index {
        object: Box::new(tsast::Expr::Member {
            object: Box::new(value.clone()),
            property: "args".to_owned(),
        }),
        index: Box::new(tsast::Expr::Number(index as f64)),
    }
}

fn is_irrefutable_pattern(pattern: &MatchPattern) -> bool {
    matches!(pattern, MatchPattern::Wildcard | MatchPattern::Bind(_))
}

#[derive(Debug, Clone)]
struct MatchRow {
    patterns: Vec<MatchPattern>,
    bindings: Vec<(String, tsast::Expr)>,
    body: lir::Expr,
}

#[derive(Debug, Clone)]
enum MatchDecision {
    Fail,
    Leaf {
        bindings: Vec<(String, tsast::Expr)>,
        body: lir::Expr,
    },
    Switch {
        occurrence: tsast::Expr,
        cases: Vec<MatchCase>,
        default: Box<MatchDecision>,
    },
}

#[derive(Debug, Clone)]
struct MatchCase {
    ctor_name: String,
    subtree: MatchDecision,
}

fn build_match_decision(occurrences: Vec<tsast::Expr>, rows: Vec<MatchRow>) -> MatchDecision {
    if rows.is_empty() {
        return MatchDecision::Fail;
    }
    if rows[0].patterns.iter().all(is_irrefutable_pattern) {
        let mut bindings = rows[0].bindings.clone();
        for (pattern, occurrence) in rows[0].patterns.iter().zip(occurrences.iter()) {
            collect_irrefutable_bindings(pattern, occurrence, &mut bindings);
        }
        return MatchDecision::Leaf {
            bindings,
            body: rows[0].body.clone(),
        };
    }

    let Some(column) = rows[0]
        .patterns
        .iter()
        .position(|pattern| !is_irrefutable_pattern(pattern))
    else {
        return MatchDecision::Leaf {
            bindings: rows[0].bindings.clone(),
            body: rows[0].body.clone(),
        };
    };

    let occurrence = occurrences[column].clone();
    let default_rows = default_specialize_rows(&rows, column, &occurrence);
    let default_occurrences = default_specialize_occurrences(&occurrences, column);
    let default = if default_rows.is_empty() {
        MatchDecision::Fail
    } else {
        build_match_decision(default_occurrences, default_rows)
    };

    let cases = collect_ctor_cases(&rows, column)
        .into_iter()
        .map(|(ctor_name, arity)| MatchCase {
            ctor_name: ctor_name.clone(),
            subtree: build_match_decision(
                specialize_occurrences_for_ctor(&occurrences, column, arity),
                specialize_rows_for_ctor(&rows, column, &ctor_name, arity, &occurrence),
            ),
        })
        .collect::<Vec<_>>();

    MatchDecision::Switch {
        occurrence,
        cases,
        default: Box::new(default),
    }
}

fn collect_irrefutable_bindings(
    pattern: &MatchPattern,
    occurrence: &tsast::Expr,
    bindings: &mut Vec<(String, tsast::Expr)>,
) {
    if let MatchPattern::Bind(name) = pattern {
        bindings.push((name.clone(), occurrence.clone()));
    }
}

fn collect_ctor_cases(rows: &[MatchRow], column: usize) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for row in rows {
        let Some(pattern) = row.patterns.get(column) else {
            continue;
        };
        let MatchPattern::Ctor { name, args } = pattern else {
            continue;
        };
        if out.iter().all(|(seen, _)| seen != name) {
            out.push((name.clone(), args.len()));
        }
    }
    out
}

fn specialize_rows_for_ctor(
    rows: &[MatchRow],
    column: usize,
    ctor_name: &str,
    arity: usize,
    occurrence: &tsast::Expr,
) -> Vec<MatchRow> {
    rows.iter()
        .filter_map(|row| specialize_row_for_ctor(row, column, ctor_name, arity, occurrence))
        .collect()
}

fn specialize_row_for_ctor(
    row: &MatchRow,
    column: usize,
    ctor_name: &str,
    arity: usize,
    occurrence: &tsast::Expr,
) -> Option<MatchRow> {
    let pattern = row.patterns.get(column)?;
    let mut patterns = row.patterns[..column].to_vec();
    let mut bindings = row.bindings.clone();
    match pattern {
        MatchPattern::Ctor { name, args } if name == ctor_name && args.len() == arity => {
            patterns.extend(args.clone());
        }
        MatchPattern::Wildcard => {
            patterns.extend(std::iter::repeat_n(MatchPattern::Wildcard, arity));
        }
        MatchPattern::Bind(name) => {
            bindings.push((name.clone(), occurrence.clone()));
            patterns.extend(std::iter::repeat_n(MatchPattern::Wildcard, arity));
        }
        _ => return None,
    }
    patterns.extend_from_slice(&row.patterns[column + 1..]);
    Some(MatchRow {
        patterns,
        bindings,
        body: row.body.clone(),
    })
}

fn default_specialize_rows(
    rows: &[MatchRow],
    column: usize,
    occurrence: &tsast::Expr,
) -> Vec<MatchRow> {
    rows.iter()
        .filter_map(|row| {
            let pattern = row.patterns.get(column)?;
            let mut patterns = row.patterns[..column].to_vec();
            let mut bindings = row.bindings.clone();
            match pattern {
                MatchPattern::Wildcard => {}
                MatchPattern::Bind(name) => bindings.push((name.clone(), occurrence.clone())),
                MatchPattern::Ctor { .. } => return None,
            }
            patterns.extend_from_slice(&row.patterns[column + 1..]);
            Some(MatchRow {
                patterns,
                bindings,
                body: row.body.clone(),
            })
        })
        .collect()
}

fn specialize_occurrences_for_ctor(
    occurrences: &[tsast::Expr],
    column: usize,
    arity: usize,
) -> Vec<tsast::Expr> {
    let mut out = occurrences[..column].to_vec();
    let occurrence = &occurrences[column];
    out.extend((0..arity).map(|index| payload_access_expr(occurrence, index)));
    out.extend_from_slice(&occurrences[column + 1..]);
    out
}

fn default_specialize_occurrences(occurrences: &[tsast::Expr], column: usize) -> Vec<tsast::Expr> {
    let mut out = occurrences[..column].to_vec();
    out.extend_from_slice(&occurrences[column + 1..]);
    out
}

fn lower_match_decision(
    error_value: &tsast::Expr,
    decision: MatchDecision,
    lower_body: &dyn Fn(&lir::Expr, Vec<(String, tsast::Expr)>) -> tsast::Expr,
) -> tsast::Expr {
    match decision {
        MatchDecision::Fail => runtime_call("__lumo_match_error", vec![error_value.clone()]),
        MatchDecision::Leaf { bindings, body } => lower_body(&body, bindings),
        MatchDecision::Switch {
            occurrence,
            cases,
            default,
        } => {
            let mut folded = lower_match_decision(error_value, *default, lower_body);
            for case in cases.into_iter().rev() {
                let cond = runtime_call(
                    "__lumo_is",
                    vec![occurrence.clone(), tsast::Expr::String(case.ctor_name)],
                );
                folded = tsast::Expr::IfElse {
                    cond: Box::new(cond),
                    then_expr: Box::new(lower_match_decision(
                        error_value,
                        case.subtree,
                        lower_body,
                    )),
                    else_expr: Box::new(folded),
                };
            }
            folded
        }
    }
}

// --- CPS codegen for cap handlers with resume ---

/// Decompose a perform call pattern in LIR:
/// Apply*(Member(Perform(E), op), args) → Some((E, op, args))
/// Member(Perform(E), op) with no args → Some((E, op, []))
/// Peel Apply nodes off an expression, returning (root, args) in application order.
fn unwrap_apply_chain<'a>(expr: &'a lir::Expr) -> (&'a lir::Expr, Vec<&'a lir::Expr>) {
    let mut args = Vec::new();
    let mut cursor = expr;
    while let lir::Expr::Apply { callee, arg, .. } = cursor {
        args.push(arg.as_ref());
        cursor = callee.as_ref();
    }
    args.reverse();
    (cursor, args)
}

fn decompose_perform_call<'a>(
    expr: &'a lir::Expr,
) -> Option<(String, &'a str, Vec<&'a lir::Expr>)> {
    let (root, args) = unwrap_apply_chain(expr);
    // Unwrap Force wrapper: LIR produces Apply*(Force(Member(Perform(cap), op)), args)
    let root = if let lir::Expr::Force { expr, .. } = root {
        expr.as_ref()
    } else {
        root
    };
    if let lir::Expr::Member { object, field, .. } = root {
        if let lir::Expr::Perform { cap, type_args, .. } = object.as_ref() {
            return Some((cap_runtime_name(cap, type_args), field.as_str(), args));
        }
    }
    None
}

/// Decompose a function call pattern in LIR:
/// Apply*(Force(Ident(name)), args) → Some((name, args))
/// Force(Ident(name)) with no args → Some((name, []))
fn decompose_fn_call<'a>(expr: &'a lir::Expr) -> Option<(&'a str, Vec<&'a lir::Expr>)> {
    let (root, args) = unwrap_apply_chain(expr);
    if let lir::Expr::Force { expr, .. } = root {
        if let lir::Expr::Ident { name, .. } = expr.as_ref() {
            return Some((name.as_str(), args));
        }
    }
    None
}

/// Compile handle expression with CPS for resume support
fn lower_cps_handle(
    cap: &str,
    type_args: &[String],
    handler: &lir::Expr,
    body: &lir::Expr,
    ctx: &LoweringContext,
) -> tsast::Expr {
    let runtime_name = cap_runtime_name(cap, type_args);
    let param_name = runtime_name.clone();
    let identity_k = tsast::Expr::Arrow {
        params: vec![tsast::Param::new("__v")],
        return_type: None,
        body: Box::new(tsast::FunctionBody::Expr(Box::new(tsast::Expr::Ident(
            "__v".to_owned(),
        )))),
    };
    let cps_body = lower_cps_expr(body, identity_k, &[runtime_name], ctx);
    let handler_lowered = lower_handler_with_resume(handler, ctx);
    let bound = iife(&param_name, cps_body, handler_lowered);
    // Wrap in trampoline to evaluate CPS thunks iteratively
    tsast::Expr::Call {
        callee: Box::new(tsast::Expr::Ident("__trampoline".to_owned())),
        args: vec![bound],
    }
}

/// CPS-transform an expression within a handle body.
/// `k` is the continuation expression (a JS function taking a value).
/// `handled_caps` are the caps available for CPS threading.
/// Recursively check whether a LIR expression contains any effectful computation
/// that requires CPS sequencing within the current handled-cap context.
///
/// **General rule**: In CPS mode, every sub-expression in value position (function
/// arguments, match scrutinees, `produce` bodies) must be checked with this function.
/// If it returns true, the sub-expression must be CPS-lowered and bound to a temp
/// variable before use, rather than being lowered with `lower_expr`.
fn is_effectful_expr(
    expr: &lir::Expr,
    handled_caps: &[String],
    ctx: &LoweringContext,
) -> bool {
    // Decompose as perform call: Apply*(Member(Perform(E), op), args)
    if let Some((cap, _, args)) = decompose_perform_call(expr) {
        if handled_caps.iter().any(|c| c == &cap) {
            return true;
        }
        // Even if this cap isn't handled, args might be effectful
        return args
            .iter()
            .any(|a| is_effectful_expr(a, handled_caps, ctx));
    }
    // Decompose as effectful function call: Apply*(Force(Ident(f)), args)
    if let Some((fn_name, args)) = decompose_fn_call(expr) {
        if ctx
            .fn_caps
            .get(fn_name)
            .map_or(false, |c| !c.is_empty())
        {
            return true;
        }
        return args
            .iter()
            .any(|a| is_effectful_expr(a, handled_caps, ctx));
    }
    // Recurse into compound forms
    match expr {
        lir::Expr::Apply { callee, arg, .. } => {
            is_effectful_expr(callee, handled_caps, ctx)
                || is_effectful_expr(arg, handled_caps, ctx)
        }
        lir::Expr::Force { expr, .. } => is_effectful_expr(expr, handled_caps, ctx),
        lir::Expr::Match {
            scrutinee, arms, ..
        } => {
            is_effectful_expr(scrutinee, handled_caps, ctx)
                || arms
                    .iter()
                    .any(|a| is_effectful_expr(&a.body, handled_caps, ctx))
        }
        lir::Expr::Let { value, body, .. } => {
            is_effectful_expr(value, handled_caps, ctx)
                || is_effectful_expr(body, handled_caps, ctx)
        }
        lir::Expr::Produce { expr, .. } => is_effectful_expr(expr, handled_caps, ctx),
        lir::Expr::Member { object, .. } => is_effectful_expr(object, handled_caps, ctx),
        lir::Expr::Handle { .. } => true,
        // Leaf values are never effectful
        lir::Expr::Ident { .. }
        | lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Perform { .. } // bare Perform (no member/apply) is just a value ref
        | lir::Expr::Bundle { .. }
        | lir::Expr::Lambda { .. }
        | lir::Expr::Error { .. } => false,
        // Wrappers: recurse into inner expression
        lir::Expr::Ctor { args, .. } => args.iter().any(|a| is_effectful_expr(a, handled_caps, ctx)),
        lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Thunk { expr, .. }
        | lir::Expr::Ann { expr, .. } => is_effectful_expr(expr, handled_caps, ctx),
    }
}

/// Lower a value-position expression within CPS context.
///
/// If the expression is effectful (contains perform/effectful-call), CPS-sequence
/// it: lower it with `lower_cps_expr` into a fresh temp variable, then build the
/// continuation with that variable.
///
/// If it is pure, lower it directly with `lower_expr` and build immediately.
fn lower_cps_value(
    expr: &lir::Expr,
    handled_caps: &[String],
    ctx: &LoweringContext,
    then: impl FnOnce(tsast::Expr) -> tsast::Expr,
) -> tsast::Expr {
    if is_effectful_expr(expr, handled_caps, ctx) {
        static COUNTER: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let tmp = format!(
            "__cps_v_{}",
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        let body = then(tsast::Expr::Ident(tmp.clone()));
        let inner_k = tsast::Expr::Arrow {
            params: vec![tsast::Param::new(&tmp)],
            return_type: None,
            body: Box::new(tsast::FunctionBody::Expr(Box::new(body))),
        };
        lower_cps_expr(expr, inner_k, handled_caps, ctx)
    } else {
        then(lower_expr(expr, ctx))
    }
}

/// Lower a list of value-position expressions within CPS context, left-to-right.
/// Each effectful expression is CPS-sequenced before the next one is evaluated.
fn lower_cps_values(
    exprs: &[&lir::Expr],
    idx: usize,
    acc: Vec<tsast::Expr>,
    handled_caps: &[String],
    ctx: &LoweringContext,
    then: &dyn Fn(Vec<tsast::Expr>) -> tsast::Expr,
) -> tsast::Expr {
    if idx >= exprs.len() {
        return then(acc);
    }
    lower_cps_value(exprs[idx], handled_caps, ctx, |val| {
        let mut next_acc = acc;
        next_acc.push(val);
        lower_cps_values(exprs, idx + 1, next_acc, handled_caps, ctx, then)
    })
}

fn lower_cps_expr(
    expr: &lir::Expr,
    k: tsast::Expr,
    handled_caps: &[String],
    ctx: &LoweringContext,
) -> tsast::Expr {
    lower_cps_expr_inner(expr, k, handled_caps, ctx)
}

fn lower_cps_expr_inner(
    expr: &lir::Expr,
    k: tsast::Expr,
    handled_caps: &[String],
    ctx: &LoweringContext,
) -> tsast::Expr {
    // Check if this is a perform of a handled cap
    if let Some((cap_runtime, op, args)) = decompose_perform_call(expr) {
        if handled_caps.iter().any(|c| c == &cap_runtime) {
            let op = op.to_owned();
            // CPS-sequence all arguments, then emit the perform call
            return lower_cps_values(&args, 0, Vec::new(), handled_caps, ctx, &|ts_args| {
                let callee = tsast::Expr::Member {
                    object: Box::new(tsast::Expr::Ident(cap_runtime.clone())),
                    property: op.clone(),
                };
                let mut final_args = ts_args;
                final_args.push(k.clone());
                tsast::Expr::Call {
                    callee: Box::new(callee),
                    args: final_args,
                }
            });
        }
    }

    match expr {
        lir::Expr::Produce { expr: inner, .. } => {
            // produce v → CPS-lower v (in case it's effectful), then k(result)
            lower_cps_value(inner, handled_caps, ctx, |val| tsast::Expr::Call {
                callee: Box::new(k),
                args: vec![val],
            })
        }
        lir::Expr::Let {
            name, value, body, ..
        } => {
            // let x = comp in body → CPS[comp]((x) => CPS[body](k))
            let body_cps = lower_cps_expr(body, k, handled_caps, ctx);
            let inner_k = tsast::Expr::Arrow {
                params: vec![tsast::Param::new(name)],
                return_type: None,
                body: Box::new(tsast::FunctionBody::Expr(Box::new(body_cps))),
            };
            lower_cps_expr(value, inner_k, handled_caps, ctx)
        }
        lir::Expr::Match {
            scrutinee, arms, ..
        } => lower_cps_match_expr(scrutinee, arms, k, handled_caps, ctx),
        // Ctor with effectful args: CPS-sequence each arg, then construct
        lir::Expr::Ctor { name, args, .. } => {
            let arg_refs: Vec<&lir::Expr> = args.iter().collect();
            let name = name.clone();
            lower_cps_values(&arg_refs, 0, Vec::new(), handled_caps, ctx, &|ts_args| {
                let callee = if let Some((owner, variant)) = name.split_once('.') {
                    tsast::Expr::Index {
                        object: Box::new(tsast::Expr::Ident(owner.to_owned())),
                        index: Box::new(tsast::Expr::String(variant.to_owned())),
                    }
                } else {
                    tsast::Expr::Ident(name.clone())
                };
                let ctor_expr = if ts_args.is_empty() {
                    callee
                } else {
                    tsast::Expr::Call {
                        callee: Box::new(callee),
                        args: ts_args,
                    }
                };
                tsast::Expr::Call {
                    callee: Box::new(k.clone()),
                    args: vec![ctor_expr],
                }
            })
        }
        // Transparent wrappers: recurse into inner expression for CPS
        lir::Expr::Unroll { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Ann { expr, .. } => lower_cps_expr(expr, k, handled_caps, ctx),
        lir::Expr::Handle {
            cap, type_args, handler, body, ..
        } => {
            // handle Cap with handler in body → bind handler, CPS-lower body with cap added
            let runtime_name = cap_runtime_name(cap, type_args);
            let handler_lowered = lower_handler_with_resume(handler, ctx);
            let mut extended_caps: Vec<String> =
                handled_caps.iter().cloned().collect();
            extended_caps.push(runtime_name.clone());
            let cps_body = lower_cps_expr(body, k, &extended_caps, ctx);
            iife(&runtime_name, cps_body, handler_lowered)
        }
        // Check for effectful function calls: Apply*(Force(Ident(f)), args)
        _ => {
            if let Some((fn_name, args)) = decompose_fn_call(expr) {
                if let Some(caps) = ctx.fn_caps.get(fn_name) {
                    if !caps.is_empty() {
                        let fn_name = fn_name.to_owned();
                        let caps = caps.clone();
                        // CPS-sequence all arguments, then emit the effectful call
                        return lower_cps_values(
                            &args,
                            0,
                            Vec::new(),
                            handled_caps,
                            ctx,
                            &|ts_args| {
                                let mut final_args = ts_args;
                                for cap in &caps {
                                    final_args
                                        .push(tsast::Expr::Ident(cap.clone()));
                                }
                                final_args.push(k.clone());
                                tsast::Expr::Call {
                                    callee: Box::new(tsast::Expr::Ident(fn_name.clone())),
                                    args: final_args,
                                }
                            },
                        );
                    }
                }
            }
            // Opaque expression: lower directly and pass result to k.
            // All effectful expression forms (Perform, Handle, Match with effects, etc.)
            // must be handled in explicit branches above — the fallthrough is for pure values.
            tsast::Expr::Call {
                callee: Box::new(k),
                args: vec![lower_expr(expr, ctx)],
            }
        }
    }
}

/// CPS-transform a match expression.
/// The scrutinee is CPS-lowered via `lower_cps_value` so any effectful
/// sub-expression is correctly sequenced before the match body runs.
fn lower_cps_match_expr(
    scrutinee: &lir::Expr,
    arms: &[lir::MatchArm],
    k: tsast::Expr,
    handled_caps: &[String],
    ctx: &LoweringContext,
) -> tsast::Expr {
    let k_name = ctx.next_k_name();
    let k_ident = tsast::Expr::Ident(k_name.clone());

    let scrutinee_name = ctx.next_match_name();
    let scrutinee_expr = tsast::Expr::Ident(scrutinee_name.clone());

    let rows = arms
        .iter()
        .map(|arm| MatchRow {
            patterns: vec![pattern_to_match_pattern(&arm.pattern)],
            bindings: Vec::new(),
            body: arm.body.clone(),
        })
        .collect::<Vec<_>>();
    let decision = build_match_decision(vec![scrutinee_expr.clone()], rows);
    let lowered = lower_match_decision(&scrutinee_expr, decision, &|body, bindings| {
        wrap_bindings(
            lower_cps_expr(body, k_ident.clone(), handled_caps, ctx),
            bindings,
        )
    });

    lower_cps_value(scrutinee, handled_caps, ctx, |lowered_scrutinee| {
        let inner = iife(&scrutinee_name, lowered, lowered_scrutinee);
        iife(&k_name, inner, k.clone())
    })
}

/// Compile handler entries with resume support.
/// Each entry gets an extra `__k` parameter.
/// - Entries using `resume`: `(params, __k) => ((resume) => body)(() => __k)`
/// - Tail-resumptive entries: `(params, __k) => __k(body_result)`
fn lower_handler_with_resume(handler: &lir::Expr, ctx: &LoweringContext) -> tsast::Expr {
    let lir::Expr::Bundle { entries, .. } = handler else {
        return lower_expr(handler, ctx);
    };
    let props = entries
        .iter()
        .map(|entry| {
            let body_lowered = lower_expr(&entry.body, ctx);
            let mut params: Vec<tsast::Param> = entry
                .params
                .iter()
                .map(|p| tsast::Param::new(&p.name))
                .collect();
            params.push(tsast::Param::new("__k"));

            let raw_body = if lir::expr_references_name(&entry.body, "resume") {
                // ((resume) => body)(() => __k)
                let resume_binding = tsast::Expr::Arrow {
                    params: Vec::new(),
                    return_type: None,
                    body: Box::new(tsast::FunctionBody::Expr(Box::new(
                        tsast::Expr::Ident("__k".to_owned()),
                    ))),
                };
                iife("resume", body_lowered, resume_binding)
            } else {
                // __k(body_result)
                tsast::Expr::Call {
                    callee: Box::new(tsast::Expr::Ident("__k".to_owned())),
                    args: vec![body_lowered],
                }
            };
            // Wrap in __thunk for trampoline bounce
            let inner_body = tsast::Expr::Call {
                callee: Box::new(tsast::Expr::Ident("__thunk".to_owned())),
                args: vec![tsast::Expr::Arrow {
                    params: vec![],
                    return_type: None,
                    body: Box::new(tsast::FunctionBody::Expr(Box::new(raw_body))),
                }],
            };

            tsast::ObjectProp {
                key: tsast::ObjectKey::Ident(entry.name.clone()),
                value: tsast::Expr::Arrow {
                    params,
                    return_type: None,
                    body: Box::new(tsast::FunctionBody::Expr(Box::new(inner_body))),
                },
            }
        })
        .collect();
    tsast::Expr::Object(props)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MatchPattern {
    Wildcard,
    Bind(String),
    Ctor {
        name: String,
        args: Vec<MatchPattern>,
    },
}

/// Convert from shared `Pattern` type to backend-local `MatchPattern`.
fn pattern_to_match_pattern(pattern: &Pattern) -> MatchPattern {
    match pattern {
        Pattern::Wildcard => MatchPattern::Wildcard,
        Pattern::Bind(name) => MatchPattern::Bind(name.clone()),
        Pattern::Ctor { name, args } => MatchPattern::Ctor {
            name: name.clone(),
            args: args.iter().map(pattern_to_match_pattern).collect(),
        },
    }
}

fn runtime_call(name: &str, args: Vec<tsast::Expr>) -> tsast::Expr {
    tsast::Expr::Call {
        callee: Box::new(tsast::Expr::Ident(name.to_owned())),
        args,
    }
}
