use crate::{
    backend::{Backend, BackendError, BackendKind, CodegenTarget},
    lir,
};
use simple_ts_ast as tsast;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct TypeScriptBackend;

struct LoweringContext {
    direct_callable_arities: HashMap<String, usize>,
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
        };

        for item in &file.items {
            match item {
                lir::Item::ExternType(ext) => {
                    body.push(tsast::Stmt::TypeAlias(tsast::TypeAlias {
                        export: true,
                        name: ext.name.clone(),
                        type_params: Vec::new(),
                        ty: ts_type_from_extern_name(ext),
                    }));
                }
                lir::Item::ExternFn(func) => {
                    let params = func
                        .params
                        .iter()
                        .map(|param| {
                            let ty = parse_lumo_type_text(&param.ty_repr);
                            tsast::Param::new(&param.name)
                                .with_type(lower_lumo_type_to_ts_type(&ty))
                        })
                        .collect::<Vec<_>>();
                    let return_ty = func
                        .return_type_repr
                        .as_ref()
                        .map(|repr| parse_lumo_type_text(repr))
                        .unwrap_or_else(|| LumoType::Named("Unit".to_owned()));
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
                        return_type: Some(lower_lumo_type_to_ts_type(&return_ty)),
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
                lir::Item::Effect(_) => {
                    // Effect declarations are type-only; no TS output needed
                }
                lir::Item::Fn(func) => {
                    body.push(tsast::Stmt::Function(lower_fn_decl(func, &ctx)?));
                }
            }
        }

        let mut program = tsast::Program::new(body);
        specialize_operator_extern_wrappers(&mut program, &extern_names);
        tsast::lower_expression_bodies(&mut program);
        tsast::flatten_iifes(&mut program);
        tsast::return_lifting(&mut program);
        tsast::return_lifting(&mut program);
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
                out.insert(func.name.clone(), func.params.len());
            }
            _ => {}
        }
    }
    out
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

    let params = func
        .params
        .iter()
        .zip(lowered_params.iter())
        .map(|(param, lowered_name)| {
            let ty = parse_lumo_type_text(&param.ty_repr);
            tsast::Param::new(lowered_name).with_type(lower_lumo_type_to_ts_type(&ty))
        })
        .collect::<Vec<_>>();

    let return_ty = func
        .return_type_repr
        .as_ref()
        .map(|repr| parse_lumo_type_text(repr))
        .unwrap_or_else(|| LumoType::Named("Unit".to_owned()));

    Ok(tsast::FunctionDecl {
        export: true,
        name: func.name.clone(),
        type_params: func.generics.clone(),
        params,
        return_type: Some(lower_lumo_type_to_ts_type(&return_ty)),
        body: tsast::FunctionBody::Expr(Box::new(lower_expr(lowered_body, ctx))),
    })
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

"
        }
        tsast::EmitTarget::JavaScript => {
            "const LUMO_TAG = Symbol.for(\"Lumo/tag\");
const __lumo_is = (value, pattern) =>
  !!value && typeof value === \"object\" && LUMO_TAG in value && value[LUMO_TAG] === pattern;
const __lumo_match_error = (value) => { throw new Error(\"non-exhaustive match: \" + JSON.stringify(value)); };
const __lumo_error = () => { throw new Error(\"lumo runtime error\"); };

"
        }
        tsast::EmitTarget::TypeScriptDefinition => "",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LumoType {
    Named(String),
    App { head: String, args: Vec<LumoType> },
    Produce(Box<LumoType>),
    Thunk(Box<LumoType>),
}

fn parse_lumo_type_text(text: &str) -> LumoType {
    let text = text.trim();
    if let Some(rest) = text.strip_prefix("produce") {
        return LumoType::Produce(Box::new(parse_lumo_type_text(rest)));
    }
    if let Some(rest) = text.strip_prefix("thunk") {
        return LumoType::Thunk(Box::new(parse_lumo_type_text(rest)));
    }
    parse_named_type_text(&canonicalize_lumo_named_type(&lumo_type_text_to_ts(text)))
}

fn lumo_type_text_to_ts(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '[' => '<',
            ']' => '>',
            _ => ch,
        })
        .collect()
}

fn parse_named_type_text(text: &str) -> LumoType {
    let canonical = canonicalize_lumo_named_type(text);
    let (head, args_text) = split_type_args(&canonical);
    if args_text.is_empty() {
        LumoType::Named(head)
    } else {
        LumoType::App {
            head,
            args: args_text
                .into_iter()
                .map(|arg| parse_named_type_text(&arg))
                .collect(),
        }
    }
}

fn lower_lumo_type_to_ts_type(ty: &LumoType) -> tsast::TsType {
    match ty {
        LumoType::Named(name) if name == "Unit" => tsast::TsType::Void,
        LumoType::Named(name) => tsast::TsType::TypeRef(name.clone()),
        LumoType::App { head, args } => tsast::TsType::TypeRef(format!(
            "{head}<{}>",
            args.iter()
                .map(lower_lumo_type_to_ts_text)
                .collect::<Vec<_>>()
                .join(", ")
        )),
        LumoType::Produce(inner) => lower_lumo_type_to_ts_type(inner),
        LumoType::Thunk(inner) => tsast::TsType::Func {
            params: Vec::new(),
            ret: Box::new(lower_lumo_type_to_ts_type(inner)),
        },
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
            if variant.payload_types.is_empty() {
                format!("{{ [LUMO_TAG]: '{}' }}", variant.name)
            } else {
                let payloads = variant
                    .payload_types
                    .iter()
                    .map(|payload| lower_lumo_type_to_ts_text(&parse_lumo_type_text(payload)))
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
        let Some(expr) = specialize_operator_wrapper_expr(extern_name, &func.params) else {
            continue;
        };
        func.body = tsast::FunctionBody::Expr(Box::new(expr));
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
    if variant.payload_types.is_empty() {
        let fields = vec![tsast::ObjectProp {
            key: tsast::ObjectKey::Computed(Box::new(tsast::Expr::Ident("LUMO_TAG".to_owned()))),
            value: tsast::Expr::String(variant.name.clone()),
        }];
        return tsast::Expr::Object(fields);
    }

    let params = variant
        .payload_types
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
                .payload_types
                .iter()
                .enumerate()
                .map(|(index, payload)| {
                    let payload = lower_lumo_type_to_ts_text(&parse_lumo_type_text(payload));
                    format!("arg{index}: {payload}")
                })
                .collect::<Vec<_>>()
                .join(", ");
            if variant.payload_types.is_empty() {
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

fn lower_lumo_type_to_ts_text(ty: &LumoType) -> String {
    match ty {
        LumoType::Named(name) if name == "Unit" => "void".to_owned(),
        LumoType::Named(name) => name.clone(),
        LumoType::App { head, args } => format!(
            "{head}<{}>",
            args.iter()
                .map(lower_lumo_type_to_ts_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        LumoType::Produce(inner) => lower_lumo_type_to_ts_text(inner),
        LumoType::Thunk(inner) => format!("() => {}", lower_lumo_type_to_ts_text(inner)),
    }
}

fn canonicalize_lumo_named_type(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn split_type_args(text: &str) -> (String, Vec<String>) {
    let text = text.trim();
    let Some(start) = text.find('<') else {
        return (text.to_owned(), Vec::new());
    };
    if !text.ends_with('>') {
        return (text.to_owned(), Vec::new());
    }
    let head = text[..start].to_owned();
    let inner = &text[start + 1..text.len() - 1];
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut begin = 0usize;
    for (idx, ch) in inner.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                out.push(canonicalize_lumo_named_type(&inner[begin..idx]));
                begin = idx + 1;
            }
            _ => {}
        }
    }
    let tail = canonicalize_lumo_named_type(&inner[begin..]);
    if !tail.is_empty() {
        out.push(tail);
    }
    (head, out)
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

fn lower_expr(expr: &lir::Expr, ctx: &LoweringContext) -> tsast::Expr {
    match expr {
        lir::Expr::Ident { name, .. } if name == "Unit" => {
            tsast::Expr::Void(Box::new(tsast::Expr::Number(0.0)))
        }
        lir::Expr::Ident { name, .. } => tsast::Expr::Ident(name.clone()),
        lir::Expr::String { value, .. } => tsast::Expr::String(value.clone()),
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
        lir::Expr::LetIn {
            name, value, body, ..
        } => {
            let lambda = tsast::Expr::Arrow {
                params: vec![tsast::Param::new(name)],
                return_type: None,
                body: Box::new(tsast::FunctionBody::Expr(Box::new(lower_expr(body, ctx)))),
            };
            tsast::Expr::Call {
                callee: Box::new(lambda),
                args: vec![lower_expr(value, ctx)],
            }
        }
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
        lir::Expr::Perform { effect, .. } => {
            tsast::Expr::Ident(format!("__effect_{effect}"))
        }
        lir::Expr::Handle {
            effect, handler, body, ..
        } => {
            // ((__effect_E) => body)(handler)
            let param_name = format!("__effect_{effect}");
            let lambda = tsast::Expr::Arrow {
                params: vec![tsast::Param::new(&param_name)],
                return_type: None,
                body: Box::new(tsast::FunctionBody::Expr(Box::new(lower_expr(body, ctx)))),
            };
            tsast::Expr::Call {
                callee: Box::new(lambda),
                args: vec![lower_expr(handler, ctx)],
            }
        }
        lir::Expr::Ann { expr, .. } => lower_expr(expr, ctx),
        lir::Expr::Error { .. } => runtime_call("__lumo_error", Vec::new()),
        lir::Expr::Bundle { entries, .. } => {
            let props = entries
                .iter()
                .map(|entry| {
                    let body = lower_expr(&entry.body, ctx);
                    let value = if entry.params.is_empty() {
                        tsast::Expr::Arrow {
                            params: Vec::new(),
                            return_type: None,
                            body: Box::new(tsast::FunctionBody::Expr(Box::new(body))),
                        }
                    } else {
                        tsast::Expr::Arrow {
                            params: entry
                                .params
                                .iter()
                                .map(|p| tsast::Param::new(&p.name))
                                .collect(),
                            return_type: None,
                            body: Box::new(tsast::FunctionBody::Expr(Box::new(body))),
                        }
                    };
                    tsast::ObjectProp {
                        key: tsast::ObjectKey::Ident(entry.name.clone()),
                        value,
                    }
                })
                .collect();
            tsast::Expr::Object(props)
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

fn lower_match_expr(
    scrutinee: &lir::Expr,
    arms: &[lir::MatchArm],
    ctx: &LoweringContext,
) -> tsast::Expr {
    let lowered_scrutinee = lower_expr(scrutinee, ctx);
    let scrutinee_name = "__match";
    let scrutinee_expr = tsast::Expr::Ident(scrutinee_name.to_owned());
    let rows = arms
        .iter()
        .map(|arm| MatchRow {
            patterns: vec![parse_match_pattern(&arm.pattern).unwrap_or(MatchPattern::Wildcard)],
            bindings: Vec::new(),
            body: arm.body.clone(),
        })
        .collect::<Vec<_>>();
    let decision = build_match_decision(vec![scrutinee_expr.clone()], rows);
    let lowered = lower_match_decision(&scrutinee_expr, decision, ctx);

    tsast::Expr::Call {
        callee: Box::new(tsast::Expr::Arrow {
            params: vec![tsast::Param::new(scrutinee_name)],
            return_type: None,
            body: Box::new(tsast::FunctionBody::Expr(Box::new(lowered))),
        }),
        args: vec![lowered_scrutinee],
    }
}

fn lower_match_arm_body(
    body: &lir::Expr,
    bindings: Vec<(String, tsast::Expr)>,
    ctx: &LoweringContext,
) -> tsast::Expr {
    let mut expr = lower_expr(body, ctx);
    for (name, value) in bindings.into_iter().rev() {
        expr = tsast::Expr::Call {
            callee: Box::new(tsast::Expr::Arrow {
                params: vec![tsast::Param::new(name)],
                return_type: None,
                body: Box::new(tsast::FunctionBody::Expr(Box::new(expr))),
            }),
            args: vec![value],
        };
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
    ctx: &LoweringContext,
) -> tsast::Expr {
    match decision {
        MatchDecision::Fail => runtime_call("__lumo_match_error", vec![error_value.clone()]),
        MatchDecision::Leaf { bindings, body } => lower_match_arm_body(&body, bindings, ctx),
        MatchDecision::Switch {
            occurrence,
            cases,
            default,
        } => {
            let mut folded = lower_match_decision(error_value, *default, ctx);
            for case in cases.into_iter().rev() {
                let cond = runtime_call(
                    "__lumo_is",
                    vec![occurrence.clone(), tsast::Expr::String(case.ctor_name)],
                );
                folded = tsast::Expr::IfElse {
                    cond: Box::new(cond),
                    then_expr: Box::new(lower_match_decision(error_value, case.subtree, ctx)),
                    else_expr: Box::new(folded),
                };
            }
            folded
        }
    }
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

fn parse_match_pattern(pattern: &str) -> Option<MatchPattern> {
    let mut parser = MatchPatternParser::new(pattern);
    let pat = parser.parse_pattern();
    if parser.failed || parser.peek().is_some() {
        None
    } else {
        Some(pat)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MatchPatternToken {
    Ident(String),
    Underscore,
    Dot,
    LParen,
    RParen,
    Comma,
}

struct MatchPatternParser {
    tokens: Vec<MatchPatternToken>,
    index: usize,
    failed: bool,
}

impl MatchPatternParser {
    fn new(text: &str) -> Self {
        Self {
            tokens: lex_match_pattern(text),
            index: 0,
            failed: false,
        }
    }

    fn parse_pattern(&mut self) -> MatchPattern {
        let Some(token) = self.bump() else {
            self.failed = true;
            return MatchPattern::Wildcard;
        };
        match token {
            MatchPatternToken::Underscore => MatchPattern::Wildcard,
            MatchPatternToken::Dot => {
                let Some(MatchPatternToken::Ident(name)) = self.bump() else {
                    self.failed = true;
                    return MatchPattern::Wildcard;
                };
                self.parse_ctor_pattern(name)
            }
            MatchPatternToken::Ident(head) => {
                if head == "let" || head == "mut" {
                    let Some(MatchPatternToken::Ident(name)) = self.bump() else {
                        self.failed = true;
                        return MatchPattern::Wildcard;
                    };
                    if is_pattern_binding_name(&name) {
                        MatchPattern::Bind(name)
                    } else {
                        self.failed = true;
                        MatchPattern::Wildcard
                    }
                } else if is_pattern_binding_name(&head) {
                    MatchPattern::Bind(head)
                } else {
                    self.failed = true;
                    MatchPattern::Wildcard
                }
            }
            _ => {
                self.failed = true;
                MatchPattern::Wildcard
            }
        }
    }

    fn parse_ctor_pattern(&mut self, name: String) -> MatchPattern {
        if self.peek() == Some(&MatchPatternToken::LParen) {
            self.bump();
            let mut args = Vec::new();
            if self.peek() != Some(&MatchPatternToken::RParen) {
                loop {
                    args.push(self.parse_pattern());
                    if self.peek() == Some(&MatchPatternToken::Comma) {
                        self.bump();
                        continue;
                    }
                    break;
                }
            }
            if self.peek() == Some(&MatchPatternToken::RParen) {
                self.bump();
                MatchPattern::Ctor { name, args }
            } else {
                self.failed = true;
                MatchPattern::Wildcard
            }
        } else {
            MatchPattern::Ctor {
                name,
                args: Vec::new(),
            }
        }
    }

    fn peek(&self) -> Option<&MatchPatternToken> {
        self.tokens.get(self.index)
    }

    fn bump(&mut self) -> Option<MatchPatternToken> {
        let out = self.tokens.get(self.index).cloned();
        if out.is_some() {
            self.index += 1;
        }
        out
    }
}

fn lex_match_pattern(text: &str) -> Vec<MatchPatternToken> {
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
                out.push(MatchPatternToken::Underscore);
                i += 1;
            }
            '.' => {
                out.push(MatchPatternToken::Dot);
                i += 1;
            }
            '(' => {
                out.push(MatchPatternToken::LParen);
                i += 1;
            }
            ')' => {
                out.push(MatchPatternToken::RParen);
                i += 1;
            }
            ',' => {
                out.push(MatchPatternToken::Comma);
                i += 1;
            }
            _ => {
                if ch == '_' || ch.is_alphabetic() {
                    let start = i;
                    i += ch.len_utf8();
                    while i < bytes.len() {
                        let next = text[i..].chars().next().unwrap_or('\0');
                        if next == '_' || next.is_alphanumeric() {
                            i += next.len_utf8();
                        } else {
                            break;
                        }
                    }
                    out.push(MatchPatternToken::Ident(text[start..i].to_owned()));
                } else {
                    i += ch.len_utf8();
                }
            }
        }
    }
    out
}

fn is_pattern_binding_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_alphabetic()) && chars.all(|ch| ch == '_' || ch.is_alphanumeric())
}

fn runtime_call(name: &str, args: Vec<tsast::Expr>) -> tsast::Expr {
    tsast::Expr::Call {
        callee: Box::new(tsast::Expr::Ident(name.to_owned())),
        args,
    }
}
