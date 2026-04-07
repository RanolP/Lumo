use crate::{
    backend::{Backend, BackendError, BackendKind, CodegenTarget},
    lir,
    types::{Pattern, TypeExpr},
};

#[derive(Debug, Default)]
pub struct RustBackend;

impl RustBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Backend for RustBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Rust
    }

    fn supports(&self, target: CodegenTarget) -> bool {
        matches!(target, CodegenTarget::Rust)
    }

    fn emit(&self, file: &lir::File, _target: CodegenTarget) -> Result<String, BackendError> {
        emit_file(file)
    }
}

// ---------------------------------------------------------------------------
// File-level emission
// ---------------------------------------------------------------------------

/// Rename whole-word occurrences of `from` to `to` in a Rust expression string.
/// Only replaces when `from` is bounded by non-identifier characters.
fn rename_ident(s: &str, from: &str, to: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let from_bytes = from.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + from_bytes.len() <= bytes.len() && &bytes[i..i + from_bytes.len()] == from_bytes {
            let before_ok =
                i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_';
            let after_ok = i + from_bytes.len() >= bytes.len()
                || !bytes[i + from_bytes.len()].is_ascii_alphanumeric()
                    && bytes[i + from_bytes.len()] != b'_';
            if before_ok && after_ok {
                result.push_str(to);
                i += from_bytes.len();
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn emit_file(file: &lir::File) -> Result<String, BackendError> {
    let mut out = String::new();

    // Prelude
    out.push_str("#![allow(dead_code, unused_variables, non_camel_case_types)]\n\n");

    // Collect context: which names are data types, extern fns, etc.
    let ctx = LoweringContext::from_file(file);

    // Extern types (skip built-in String/Number)
    for item in &file.items {
        if let lir::Item::ExternType(ext) = item {
            if let Some(s) = emit_extern_type(ext) {
                out.push_str(&s);
                out.push('\n');
            }
        }
    }

    // Data declarations
    for item in &file.items {
        if let lir::Item::Data(data) = item {
            out.push_str(&emit_data_decl(data));
            out.push('\n');
        }
    }

    // Extern functions
    for item in &file.items {
        if let lir::Item::ExternFn(func) = item {
            out.push_str(&emit_extern_fn(func));
            out.push('\n');
        }
    }

    // User functions
    for item in &file.items {
        if let lir::Item::Fn(func) = item {
            if func.name == "main" {
                continue; // emit main() separately at the end
            }
            out.push_str(&emit_fn_decl(func, &ctx)?);
            out.push('\n');
        }
    }

    // Impl methods (emitted as standalone functions)
    for item in &file.items {
        if let lir::Item::Impl(impl_decl) = item {
            out.push_str(&emit_impl_decl(impl_decl, &ctx)?);
            out.push('\n');
        }
    }

    // Main function wrapper
    if let Some(main_fn) = file.items.iter().find_map(|item| match item {
        lir::Item::Fn(f) if f.name == "main" => Some(f),
        _ => None,
    }) {
        out.push_str(&emit_main_fn(main_fn, &ctx)?);
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

struct LoweringContext {
    /// Maps data type name -> list of (variant_name, payload_count)
    data_types: std::collections::HashMap<String, Vec<(String, usize)>>,
    /// Known extern function names
    extern_fns: std::collections::HashSet<String>,
    /// Maps data type name -> set of (variant_index, field_index) for recursive fields
    recursive_fields: std::collections::HashMap<String, std::collections::HashSet<(usize, usize)>>,
}

impl LoweringContext {
    fn from_file(file: &lir::File) -> Self {
        let mut data_types = std::collections::HashMap::new();
        let mut extern_fns = std::collections::HashSet::new();
        let mut recursive_fields = std::collections::HashMap::new();
        for item in &file.items {
            match item {
                lir::Item::Data(data) => {
                    let variants = data
                        .variants
                        .iter()
                        .map(|v| (v.name.clone(), v.payload.len()))
                        .collect();
                    data_types.insert(data.name.clone(), variants);
                    let rec = find_recursive_fields(data);
                    if !rec.is_empty() {
                        recursive_fields.insert(data.name.clone(), rec);
                    }
                }
                lir::Item::ExternFn(func) => {
                    extern_fns.insert(func.name.clone());
                }
                _ => {}
            }
        }
        Self {
            data_types,
            extern_fns,
            recursive_fields,
        }
    }
}

// ---------------------------------------------------------------------------
// Extern types
// ---------------------------------------------------------------------------

fn emit_extern_type(ext: &lir::ExternTypeDecl) -> Option<String> {
    let extern_name = ext.extern_name.as_deref().unwrap_or("");
    match extern_name {
        "string" | "number" => None, // built-in, no alias needed
        _ => Some(format!(
            "type {} = {};\n",
            ext.name,
            rust_type_for_extern(extern_name)
        )),
    }
}

fn rust_type_for_extern(extern_name: &str) -> &str {
    match extern_name {
        "string" => "String",
        "number" => "f64",
        "bool" => "bool",
        _ => "() /* unknown extern type */",
    }
}

// ---------------------------------------------------------------------------
// Recursive ADT detection
// ---------------------------------------------------------------------------

/// Find fields in an ADT that reference the enclosing type (directly or via generics).
/// Returns a set of (variant_index, field_index) pairs that need `Box<>` wrapping.
fn find_recursive_fields(data: &lir::DataDecl) -> std::collections::HashSet<(usize, usize)> {
    let mut result = std::collections::HashSet::new();
    for (vi, variant) in data.variants.iter().enumerate() {
        for (fi, spanned_ty) in variant.payload.iter().enumerate() {
            if spanned_ty.value.references_name(&data.name) {
                result.insert((vi, fi));
            }
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Type conversion: TypeExpr -> Rust type string
// ---------------------------------------------------------------------------

fn type_expr_to_rust(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named(name) => match name.as_str() {
            "String" => "String".to_string(),
            "Number" => "f64".to_string(),
            "Unit" => "()".to_string(),
            _ => name.clone(),
        },
        TypeExpr::App { head, args } => {
            let args_str: Vec<String> = args.iter().map(|a| type_expr_to_rust(a)).collect();
            format!("{}<{}>", head, args_str.join(", "))
        }
        TypeExpr::Produce(inner) | TypeExpr::Thunk(inner) => type_expr_to_rust(inner),
    }
}

// ---------------------------------------------------------------------------
// Data declarations -> Rust enums
// ---------------------------------------------------------------------------

fn emit_data_decl(data: &lir::DataDecl) -> String {
    let mut out = String::new();
    let recursive = find_recursive_fields(data);

    // #[derive(Debug, Clone)]
    // enum Name<A: Clone, B: Clone> { ... }
    out.push_str("#[derive(Debug, Clone)]\n");
    out.push_str("enum ");
    out.push_str(&data.name);
    if !data.generics.is_empty() {
        out.push('<');
        for (i, g) in data.generics.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(g);
            out.push_str(": Clone + std::fmt::Debug");
        }
        out.push('>');
    }
    out.push_str(" {\n");

    for (vi, variant) in data.variants.iter().enumerate() {
        out.push_str("    ");
        out.push_str(&to_pascal_case(&variant.name));
        if !variant.payload.is_empty() {
            out.push('(');
            for (fi, spanned_ty) in variant.payload.iter().enumerate() {
                if fi > 0 {
                    out.push_str(", ");
                }
                let rust_ty = type_expr_to_rust(&spanned_ty.value);
                if recursive.contains(&(vi, fi)) {
                    out.push_str(&format!("Box<{}>", rust_ty));
                } else {
                    out.push_str(&rust_ty);
                }
            }
            out.push(')');
        }
        out.push_str(",\n");
    }

    out.push_str("}\n");
    out
}

// ---------------------------------------------------------------------------
// Extern functions
// ---------------------------------------------------------------------------

fn emit_extern_fn(func: &lir::ExternFnDecl) -> String {
    let extern_name = func.extern_name.as_deref().unwrap_or(&func.name);
    let params = emit_param_list(&func.params);
    let ret = func
        .return_type
        .as_ref()
        .map(|r| type_expr_to_rust(&r.value))
        .unwrap_or_else(|| "()".to_string());

    let p = |i: usize| -> &str { &func.params[i].name };
    let body = match extern_name {
        // I/O
        "console.log" => {
            if func.params.is_empty() {
                "println!();".to_string()
            } else {
                format!("println!(\"{{}}\", {});", p(0))
            }
        }

        // String operations
        "str.len" => format!("({}.len() as f64)", p(0)),
        "str.char_at" => format!(
            "{}.chars().nth({} as usize).map(|c| c.to_string()).unwrap_or_default()",
            p(0), p(1)
        ),
        "str.slice" => format!(
            "{}.chars().skip({} as usize).take(({} as usize) - ({} as usize)).collect::<String>()",
            p(0), p(1), p(2), p(1)
        ),
        "str.concat" => format!("format!(\"{{}}{{}}\", {}, {})", p(0), p(1)),
        "str.eq" => format!(
            "if {} == {} {{ Bool::True }} else {{ Bool::False }}",
            p(0), p(1)
        ),
        "str.starts_with" => format!(
            "if {}.starts_with({}.as_str()) {{ Bool::True }} else {{ Bool::False }}",
            p(0), p(1)
        ),
        "str.contains" => format!(
            "if {}.contains({}.as_str()) {{ Bool::True }} else {{ Bool::False }}",
            p(0), p(1)
        ),
        "str.index_of" => format!(
            "{}.find({}.as_str()).map(|i| i as f64).unwrap_or(-1.0)",
            p(0), p(1)
        ),
        "str.trim" => format!("{}.trim().to_string()", p(0)),
        "str.char_code_at" => format!(
            "{}.chars().nth({} as usize).map(|c| c as u32 as f64).unwrap_or(-1.0)",
            p(0), p(1)
        ),
        "str.from_char_code" => format!(
            "char::from_u32({} as u32).map(|c| c.to_string()).unwrap_or_default()",
            p(0)
        ),
        "str.replace_all" => format!(
            "{}.replace({}.as_str(), {}.as_str())",
            p(0), p(1), p(2)
        ),
        "num.to_string" => format!("{}.to_string()", p(0)),

        // Number operations
        "num.eq" => format!(
            "if {} == {} {{ Bool::True }} else {{ Bool::False }}",
            p(0), p(1)
        ),
        "num.lt" => format!(
            "if {} < {} {{ Bool::True }} else {{ Bool::False }}",
            p(0), p(1)
        ),
        "num.gt" => format!(
            "if {} > {} {{ Bool::True }} else {{ Bool::False }}",
            p(0), p(1)
        ),
        "num.lte" => format!(
            "if {} <= {} {{ Bool::True }} else {{ Bool::False }}",
            p(0), p(1)
        ),
        "num.gte" => format!(
            "if {} >= {} {{ Bool::True }} else {{ Bool::False }}",
            p(0), p(1)
        ),
        "num.add" => format!("{} + {}", p(0), p(1)),
        "num.sub" => format!("{} - {}", p(0), p(1)),
        "num.mul" => format!("{} * {}", p(0), p(1)),
        "num.floor" => format!("{}.floor()", p(0)),

        // File I/O
        "fs.read_file" => format!(
            "std::fs::read_to_string(&{}).expect(\"failed to read file\")",
            p(0)
        ),
        "fs.write_file" => format!(
            "std::fs::write(&{}, &{}).expect(\"failed to write file\")",
            p(0), p(1)
        ),

        // Process
        "process.arg_at" => format!(
            "std::env::args().nth({} as usize).unwrap_or_default()",
            p(0)
        ),
        "process.args_count" => "std::env::args().count() as f64".to_string(),
        "process.exit" => format!("std::process::exit({} as i32)", p(0)),
        "process.panic" => format!("panic!(\"{{}}\", {})", p(0)),

        _ => format!("todo!(\"extern: {}\")", extern_name),
    };

    format!("fn {}({}) -> {} {{\n    {}\n}}\n", func.name, params, ret, body)
}

// ---------------------------------------------------------------------------
// User functions
// ---------------------------------------------------------------------------

fn emit_fn_decl(func: &lir::FnDecl, ctx: &LoweringContext) -> Result<String, BackendError> {
    let (param_names, body) = unwrap_fn_value(&func.value)?;

    if param_names.len() != func.params.len() {
        return Err(BackendError::EmitFailed(format!(
            "function `{}` lowered to {} lambda params but signature has {} params",
            func.name,
            param_names.len(),
            func.params.len()
        )));
    }

    let mut params = String::new();
    for (i, (param, name)) in func.params.iter().zip(param_names.iter()).enumerate() {
        if i > 0 {
            params.push_str(", ");
        }
        params.push_str(name);
        params.push_str(": ");
        params.push_str(&type_expr_to_rust(&param.ty.value));
    }

    let ret = func
        .return_type
        .as_ref()
        .map(|r| type_expr_to_rust(&r.value))
        .unwrap_or_else(|| "()".to_string());

    let generics = if func.generics.is_empty() {
        String::new()
    } else {
        let bounded: Vec<String> = func
            .generics
            .iter()
            .map(|g| format!("{}: Clone + std::fmt::Debug", g))
            .collect();
        format!("<{}>", bounded.join(", "))
    };

    let body_str = emit_expr(body, ctx);

    Ok(format!(
        "fn {}{}({}) -> {} {{\n    {}\n}}\n",
        func.name, generics, params, ret, body_str,
    ))
}

fn emit_impl_decl(
    impl_decl: &lir::ImplDecl,
    ctx: &LoweringContext,
) -> Result<String, BackendError> {
    let mut out = String::new();
    let target = impl_decl.target_type.value.display().replace(' ', "").to_lowercase();

    for method in &impl_decl.methods {
        let (param_names, body) = unwrap_fn_value(&method.value)?;
        if param_names.len() != method.params.len() {
            return Err(BackendError::EmitFailed(format!(
                "impl method `{}` lowered to {} lambda params but signature has {} params",
                method.name,
                param_names.len(),
                method.params.len()
            )));
        }

        // Build mangled function name
        let fn_name = if let Some(cap) = &impl_decl.capability {
            let cap_clean = cap.value.display().replace(' ', "").to_lowercase();
            if let Some(name) = &impl_decl.name {
                format!("{}__{}", name.to_lowercase(), method.name)
            } else {
                format!("__impl_{}_{}_{}", target, cap_clean, method.name)
            }
        } else if let Some(name) = &impl_decl.name {
            format!("{}__{}", name.to_lowercase(), method.name)
        } else {
            format!("{}__{}", target, method.name)
        };

        let mut params = String::new();
        for (i, (param, name)) in method.params.iter().zip(param_names.iter()).enumerate() {
            if i > 0 {
                params.push_str(", ");
            }
            // Rename `self` to avoid Rust keyword conflict
            let pname = if name == "self" { "self_" } else { name.as_str() };
            params.push_str(pname);
            params.push_str(": ");
            params.push_str(&type_expr_to_rust(&param.ty.value));
        }

        let ret = method
            .return_type
            .as_ref()
            .map(|r| type_expr_to_rust(&r.value))
            .unwrap_or_else(|| "()".to_string());

        let generics = if impl_decl.generics.is_empty() {
            String::new()
        } else {
            let bounded: Vec<String> = impl_decl
                .generics
                .iter()
                .map(|g| format!("{}: Clone + std::fmt::Debug", g))
                .collect();
            format!("<{}>", bounded.join(", "))
        };

        let body_str = emit_expr(body, ctx);
        // If we renamed `self` -> `self_`, fix references in the body too.
        let has_self_param = param_names.iter().any(|n| n == "self");
        let body_str = if has_self_param {
            rename_ident(&body_str, "self", "self_")
        } else {
            body_str
        };
        out.push_str(&format!(
            "fn {}{}({}) -> {} {{\n    {}\n}}\n\n",
            fn_name, generics, params, ret, body_str,
        ));
    }

    Ok(out)
}

fn emit_main_fn(func: &lir::FnDecl, ctx: &LoweringContext) -> Result<String, BackendError> {
    let (_param_names, body) = unwrap_fn_value(&func.value)?;
    let body_str = emit_expr(body, ctx);
    Ok(format!("fn main() {{\n    {};\n}}\n", body_str))
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

// ---------------------------------------------------------------------------
// Expression emission
// ---------------------------------------------------------------------------

fn emit_expr(expr: &lir::Expr, ctx: &LoweringContext) -> String {
    match expr {
        lir::Expr::Ident { name, .. } if name == "Unit" => "()".to_string(),
        lir::Expr::Ident { name, .. } => name.clone(),

        lir::Expr::String { value, .. } => format!("\"{}\".to_string()", escape_str(value)),

        lir::Expr::Number { value, .. } => {
            // Ensure it's a valid Rust float literal
            if value.contains('.') {
                format!("{}f64", value)
            } else {
                format!("{}.0f64", value)
            }
        }

        lir::Expr::Produce { expr, .. } => emit_expr(expr, ctx),

        lir::Expr::Thunk { expr, .. } => {
            let inner = emit_expr(expr, ctx);
            format!("(|| {{ {} }})", inner)
        }

        lir::Expr::Force { expr, .. } => {
            // If forcing a known function name, just reference it
            if let lir::Expr::Ident { name, .. } = expr.as_ref() {
                if ctx.extern_fns.contains(name) {
                    return name.clone();
                }
            }
            let inner = emit_expr(expr, ctx);
            format!("({})()", inner)
        }

        lir::Expr::Lambda { param, body, .. } => {
            let body_str = emit_expr(body, ctx);
            format!("|{}| {{ {} }}", param, body_str)
        }

        lir::Expr::Apply { .. } => {
            // Collect full apply chain for multi-arg calls
            let (root, args) = collect_apply_chain(expr);
            let root_str = emit_expr(root, ctx);
            let args_str: Vec<String> = args.iter().map(|a| emit_expr(a, ctx)).collect();
            format!("{}({})", root_str, args_str.join(", "))
        }

        lir::Expr::Unroll { expr, .. } | lir::Expr::Roll { expr, .. } => emit_expr(expr, ctx),

        lir::Expr::Let {
            name, value, body, ..
        } => {
            let val_str = emit_expr(value, ctx);
            let body_str = emit_expr(body, ctx);
            format!("{{ let {} = {}; {} }}", name, val_str, body_str)
        }

        lir::Expr::Match {
            scrutinee, arms, ..
        } => emit_match(scrutinee, arms, ctx),

        lir::Expr::Ctor {
            name, args, called, ..
        } => emit_ctor(name, args, *called, ctx),

        lir::Expr::Perform { cap, .. } => {
            format!("todo!(\"perform {}\")", cap)
        }

        lir::Expr::Handle { .. } => "todo!(\"handle\")".to_string(),

        lir::Expr::Bundle { .. } => "todo!(\"bundle\")".to_string(),

        lir::Expr::Member {
            object, field, ..
        } => {
            let obj = emit_expr(object, ctx);
            format!("{}.{}", obj, field)
        }

        lir::Expr::Ann { expr, .. } => emit_expr(expr, ctx),

        lir::Expr::Error { .. } => "panic!(\"lumo runtime error\")".to_string(),
    }
}

/// Collect a chain of Apply nodes into (root_callee, [arg1, arg2, ...])
fn collect_apply_chain(expr: &lir::Expr) -> (&lir::Expr, Vec<&lir::Expr>) {
    let mut args = Vec::new();
    let mut cursor = expr;
    while let lir::Expr::Apply { callee, arg, .. } = cursor {
        args.push(arg.as_ref());
        cursor = callee.as_ref();
    }
    args.reverse();
    (cursor, args)
}

// ---------------------------------------------------------------------------
// Pattern matching
// ---------------------------------------------------------------------------

fn emit_match(
    scrutinee: &lir::Expr,
    arms: &[lir::MatchArm],
    ctx: &LoweringContext,
) -> String {
    let scrut = emit_expr(scrutinee, ctx);
    let mut out = format!("match {} {{\n", scrut);
    for arm in arms {
        let pat = emit_pattern(&arm.pattern, ctx);
        let body = emit_expr(&arm.body, ctx);
        out.push_str(&format!("        {} => {},\n", pat, body));
    }
    out.push_str("    }");
    out
}

fn emit_pattern(pattern: &Pattern, _ctx: &LoweringContext) -> String {
    match pattern {
        Pattern::Wildcard => "_".to_string(),
        Pattern::Bind(name) => name.clone(),
        Pattern::Ctor { name, args } => {
            // Name may be "Type.variant" or just "variant"
            let rust_path = if let Some((type_name, variant_name)) = name.split_once('.') {
                format!("{}::{}", type_name, to_pascal_case(variant_name))
            } else {
                to_pascal_case(name)
            };
            if args.is_empty() {
                rust_path
            } else {
                let inner: Vec<String> = args
                    .iter()
                    .map(|p| emit_pattern(p, _ctx))
                    .collect();
                format!("{}({})", rust_path, inner.join(", "))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Constructor emission
// ---------------------------------------------------------------------------

fn emit_ctor(name: &str, args: &[lir::Expr], _called: bool, ctx: &LoweringContext) -> String {
    let (type_name, variant_name) = name.split_once('.').unwrap_or(("", name));
    let rust_path = if type_name.is_empty() {
        to_pascal_case(variant_name)
    } else {
        format!("{}::{}", type_name, to_pascal_case(variant_name))
    };

    if args.is_empty() {
        rust_path
    } else {
        // Find which fields are recursive and need Box::new() wrapping
        let variant_index = ctx
            .data_types
            .get(type_name)
            .and_then(|variants| {
                variants
                    .iter()
                    .position(|(vn, _)| vn == variant_name)
            });
        let recursive = ctx.recursive_fields.get(type_name);

        let arg_strs: Vec<String> = args
            .iter()
            .enumerate()
            .map(|(fi, a)| {
                let expr = emit_expr(a, ctx);
                let is_recursive = variant_index
                    .and_then(|vi| recursive.map(|r| r.contains(&(vi, fi))))
                    .unwrap_or(false);
                if is_recursive {
                    format!("Box::new({})", expr)
                } else {
                    expr
                }
            })
            .collect();
        format!("{}({})", rust_path, arg_strs.join(", "))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn emit_param_list(params: &[lir::Param]) -> String {
    params
        .iter()
        .map(|p| format!("{}: {}", p.name, type_expr_to_rust(&p.ty.value)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn to_pascal_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for ch in name.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
