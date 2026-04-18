pub mod check;
pub mod parse;
pub mod print;

use lumo_span::Span;
use lumo_lst as lst;
use lumo_lst::parser;
use lumo_types::{CapRef, ContentHash, Pattern, Spanned, TypeExpr};

// ---------------------------------------------------------------------------
// HIR types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub items: Vec<Item>,
    pub content_hash: ContentHash,
    pub errors: Vec<HirError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirError {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    ExternType(ExternTypeDecl),
    ExternFn(ExternFnDecl),
    Data(DataDecl),
    Cap(CapDecl),
    Fn(FnDecl),
    Use(UseDecl),
    Impl(ImplDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternTypeDecl {
    pub name: String,
    pub extern_name: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternFnDecl {
    pub name: String,
    pub extern_name: Option<String>,
    /// Module import via `#[link(module = "...")]` — (module, js_name).
    pub link_module: Option<(String, String)>,
    pub inline: bool,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub cap: Option<CapRef>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: Vec<VariantDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantDecl {
    pub name: String,
    pub payload: Vec<Spanned<TypeExpr>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapDecl {
    pub name: String,
    pub operations: Vec<OperationDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub cap: Option<CapRef>,
    pub body: Expr,
    pub inline: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseDecl {
    pub path: Vec<String>,
    pub names: Option<Vec<String>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplDecl {
    pub name: Option<String>,
    pub generics: Vec<String>,
    pub target_type: Spanned<TypeExpr>,
    pub capability: Option<Spanned<TypeExpr>>,
    pub methods: Vec<ImplMethodDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplMethodDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Spanned<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Ident { name: String, span: Span },
    String { value: String, span: Span },
    Number { value: String, span: Span },
    Call { callee: Box<Expr>, args: Vec<Expr>, span: Span },
    Member { object: Box<Expr>, member: String, span: Span },
    Produce { expr: Box<Expr>, span: Span },
    Thunk { expr: Box<Expr>, span: Span },
    Force { expr: Box<Expr>, span: Span },
    Let { name: String, value: Box<Expr>, body: Box<Expr>, span: Span },
    Match { scrutinee: Box<Expr>, arms: Vec<MatchArm>, span: Span },
    Perform { cap: String, span: Span },
    Handle { cap: String, type_args: Vec<String>, handler: Box<Expr>, body: Box<Expr>, span: Span },
    Bundle { entries: Vec<BundleEntry>, span: Span },
    Ann { expr: Box<Expr>, ty: Spanned<TypeExpr>, span: Span },
    Error { span: Span },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleEntry {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Expr,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Span accessor
// ---------------------------------------------------------------------------

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Ident { span, .. }
            | Expr::String { span, .. }
            | Expr::Number { span, .. }
            | Expr::Call { span, .. }
            | Expr::Member { span, .. }
            | Expr::Produce { span, .. }
            | Expr::Thunk { span, .. }
            | Expr::Force { span, .. }
            | Expr::Let { span, .. }
            | Expr::Match { span, .. }
            | Expr::Perform { span, .. }
            | Expr::Handle { span, .. }
            | Expr::Bundle { span, .. }
            | Expr::Ann { span, .. }
            | Expr::Error { span } => *span,
        }
    }
}

// ---------------------------------------------------------------------------
// Lowering: LST → HIR
// ---------------------------------------------------------------------------

pub fn lower_lossless(parsed: &crate::lst::lossless::ParseOutput) -> File {
    let parsed = parser::parse_lossless(parsed);
    lower(&parsed.file)
}

pub fn lower(file: &lst::File) -> File {
    let mut ctx = LowerCtx { errors: Vec::new() };
    let items: Vec<Item> = file
        .items
        .iter()
        .map(|item| match item {
            lst::Item::ExternType(ext) => Item::ExternType(lower_extern_type(ext)),
            lst::Item::ExternFn(ext) => Item::ExternFn(lower_extern_fn(ext)),
            lst::Item::Data(data) => Item::Data(lower_data(data)),
            lst::Item::Cap(cap) => Item::Cap(lower_cap(cap)),
            lst::Item::Fn(func) => Item::Fn(lower_fn(func, &mut ctx)),
            lst::Item::Use(u) => Item::Use(UseDecl {
                path: u.path.clone(),
                names: u.names.clone(),
                span: u.span,
            }),
            lst::Item::Impl(impl_decl) => Item::Impl(lower_impl(impl_decl, &mut ctx)),
        })
        .collect();

    let content_hash = hash_file(&items);
    File {
        items,
        content_hash,
        errors: ctx.errors,
    }
}

struct LowerCtx {
    errors: Vec<HirError>,
}

/// Merge multiple HIR files into a single combined File.
pub fn merge_files(files: &[File]) -> File {
    let mut items = Vec::new();
    let mut errors = Vec::new();
    for file in files {
        items.extend(file.items.iter().cloned());
        errors.extend(file.errors.iter().cloned());
    }
    let content_hash = hash_file(&items);
    File {
        items,
        content_hash,
        errors,
    }
}

// ---------------------------------------------------------------------------
// Item lowering
// ---------------------------------------------------------------------------

fn lower_extern_type(ext: &lst::ExternTypeDecl) -> ExternTypeDecl {
    ExternTypeDecl {
        name: ext.name.clone(),
        extern_name: find_extern_name(&ext.attrs, &ext.name),
        span: ext.span,
    }
}

fn lower_extern_fn(ext: &lst::ExternFnDecl) -> ExternFnDecl {
    ExternFnDecl {
        name: ext.name.clone(),
        extern_name: find_extern_name(&ext.attrs, &ext.name),
        link_module: find_link_module(&ext.attrs, &ext.name),
        inline: find_inline_hint(&ext.attrs),
        params: ext.params.iter().map(lower_param).collect(),
        return_type: ext.return_type.as_ref().and_then(lower_type_sig),
        cap: ext.cap.as_ref().map(lower_cap_sig),
        span: ext.span,
    }
}

fn lower_data(data: &lst::DataDecl) -> DataDecl {
    DataDecl {
        name: data.name.clone(),
        generics: data.generics.iter().map(|g| g.name.clone()).collect(),
        variants: data.variants.iter().map(lower_variant).collect(),
        span: data.span,
    }
}

fn lower_variant(variant: &lst::VariantDecl) -> VariantDecl {
    VariantDecl {
        name: variant.name.clone(),
        payload: variant
            .payload
            .iter()
            .filter_map(|ty| lower_type_sig(ty))
            .collect(),
        span: variant.span,
    }
}

fn lower_cap(cap: &lst::CapDecl) -> CapDecl {
    CapDecl {
        name: cap.name.clone(),
        operations: cap.operations.iter().map(lower_operation).collect(),
        span: cap.span,
    }
}

fn lower_operation(op: &lst::OperationDecl) -> OperationDecl {
    OperationDecl {
        name: op.name.clone(),
        params: op.params.iter().map(lower_param).collect(),
        return_type: op.return_type.as_ref().and_then(lower_type_sig),
        span: op.span,
    }
}

fn lower_fn(func: &lst::FnDecl, ctx: &mut LowerCtx) -> FnDecl {
    FnDecl {
        name: func.name.clone(),
        generics: func.generics.iter().map(|g| g.name.clone()).collect(),
        params: func.params.iter().map(lower_param).collect(),
        return_type: func.return_type.as_ref().and_then(lower_type_sig),
        cap: func.cap.as_ref().map(lower_cap_sig),
        body: lower_expr(&func.body, ctx),
        inline: find_inline_hint(&func.attrs),
        span: func.span,
    }
}

fn lower_impl(impl_decl: &lst::ImplDecl, ctx: &mut LowerCtx) -> ImplDecl {
    let target_type = lower_type_sig(&impl_decl.target_type)
        .unwrap_or_else(|| Spanned {
            value: TypeExpr::Named("Unknown".into()),
            span: impl_decl.target_type.span,
        });
    let capability = impl_decl
        .capability
        .as_ref()
        .and_then(lower_type_sig);
    let target_repr = &impl_decl.target_type.repr;

    let methods = impl_decl
        .methods
        .iter()
        .map(|m| {
            let params = m
                .params
                .iter()
                .map(|p| {
                    let resolved_repr = p.ty.repr.replace("Self", target_repr);
                    Param {
                        name: p.name.clone(),
                        ty: lower_type_sig_with_fallback(&resolved_repr, p.ty.span),
                        span: p.span,
                    }
                })
                .collect();
            let return_type = m.return_type.as_ref().and_then(|rt| {
                let resolved = rt.repr.replace("Self", target_repr);
                lower_type_sig(&lst::TypeSig {
                    repr: resolved,
                    span: rt.span,
                })
            });
            ImplMethodDecl {
                name: m.name.clone(),
                params,
                return_type,
                body: lower_expr(&m.body, ctx),
                span: m.span,
            }
        })
        .collect();

    ImplDecl {
        name: impl_decl.name.clone(),
        generics: impl_decl.generics.iter().map(|g| g.name.clone()).collect(),
        target_type,
        capability,
        methods,
        span: impl_decl.span,
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn lower_param(param: &lst::Param) -> Param {
    Param {
        name: param.name.clone(),
        ty: lower_type_sig_with_fallback(&param.ty.repr, param.ty.span),
        span: param.span,
    }
}

fn lower_type_sig(ty: &lst::TypeSig) -> Option<Spanned<TypeExpr>> {
    TypeExpr::parse(&ty.repr).map(|value| Spanned {
        value,
        span: ty.span,
    })
}

fn lower_type_sig_with_fallback(repr: &str, span: Span) -> Spanned<TypeExpr> {
    let value = TypeExpr::parse(repr)
        .unwrap_or_else(|| TypeExpr::Named(repr.trim().to_owned()));
    Spanned { value, span }
}

fn lower_cap_sig(cap: &lst::CapSig) -> CapRef {
    CapRef::parse(&cap.repr)
}

/// Parse a handle's cap string: "Add for Number" → ("Add", vec!["Number"]), "IO" → ("IO", vec![])
fn parse_handle_cap(cap: &str) -> (String, Vec<String>) {
    let s = cap.trim();
    if let Some((name, ty)) = s.split_once(" for ") {
        (name.trim().to_owned(), vec![ty.trim().to_owned()])
    } else {
        (s.to_owned(), vec![])
    }
}

// ---------------------------------------------------------------------------
// Expression lowering
// ---------------------------------------------------------------------------

/// Wrap a value-form expression in `Produce`. Computation forms pass through.
fn maybe_produce(expr: Expr, span: Span) -> Expr {
    match &expr {
        Expr::Produce { .. }
        | Expr::Force { .. }
        | Expr::Call { .. }
        | Expr::Let { .. }
        | Expr::Match { .. }
        | Expr::Perform { .. }
        | Expr::Handle { .. }
        | Expr::Member { .. }
        | Expr::Error { .. } => expr,
        _ => Expr::Produce {
            expr: Box::new(expr),
            span,
        },
    }
}

fn lower_expr(expr: &lst::Expr, ctx: &mut LowerCtx) -> Expr {
    match expr {
        lst::Expr::Ident { name, span } => Expr::Ident {
            name: name.clone(),
            span: *span,
        },
        lst::Expr::String { value, span } => Expr::String {
            value: value.clone(),
            span: *span,
        },
        lst::Expr::Number { value, span } => Expr::Number {
            value: value.clone(),
            span: *span,
        },
        lst::Expr::Member { object, member, span } => Expr::Member {
            object: Box::new(lower_expr(object, ctx)),
            member: member.clone(),
            span: *span,
        },
        lst::Expr::Call { callee, args, span } => Expr::Call {
            callee: Box::new(lower_expr(callee, ctx)),
            args: args.iter().map(|a| lower_expr(a, ctx)).collect(),
            span: *span,
        },
        lst::Expr::Thunk { expr: inner, span } => Expr::Thunk {
            expr: Box::new(lower_expr(inner, ctx)),
            span: *span,
        },
        lst::Expr::Force { expr: inner, span } => Expr::Force {
            expr: Box::new(lower_expr(inner, ctx)),
            span: *span,
        },
        lst::Expr::Let { name, value, span } => {
            // Standalone let without body — only meaningful inside blocks
            // (blocks desugar Let stmts into HIR Let with body).
            // If reached here, treat as a let binding with an error body.
            Expr::Let {
                name: name.clone(),
                value: Box::new(lower_expr(value, ctx)),
                body: Box::new(Expr::Error { span: *span }),
                span: *span,
            }
        },
        lst::Expr::Match { scrutinee, arms, span } => Expr::Match {
            scrutinee: Box::new(lower_expr(scrutinee, ctx)),
            arms: arms
                .iter()
                .map(|arm| {
                    let pattern = match Pattern::parse(&arm.pattern) {
                        Some(p) => p,
                        None => {
                            ctx.errors.push(HirError {
                                span: arm.span,
                                message: format!(
                                    "invalid match pattern `{}`; constructor patterns must start with `.`",
                                    arm.pattern
                                ),
                            });
                            Pattern::Wildcard
                        }
                    };
                    MatchArm {
                        pattern,
                        body: maybe_produce(lower_expr(&arm.body, ctx), arm.span),
                        span: arm.span,
                    }
                })
                .collect(),
            span: *span,
        },
        lst::Expr::Perform { cap, span } => Expr::Perform {
            cap: cap.clone(),
            span: *span,
        },
        lst::Expr::Handle { cap, handler, body, span } => {
            let (cap_name, type_args) = parse_handle_cap(cap);
            Expr::Handle {
                cap: cap_name,
                type_args,
                handler: Box::new(lower_expr(handler, ctx)),
                body: Box::new(maybe_produce(lower_expr(body, ctx), *span)),
                span: *span,
            }
        },
        lst::Expr::Bundle { entries, span } => Expr::Bundle {
            entries: entries
                .iter()
                .map(|e| BundleEntry {
                    name: e.name.clone(),
                    params: e.params.iter().map(lower_param).collect(),
                    body: maybe_produce(lower_expr(&e.body, ctx), e.span),
                    span: e.span,
                })
                .collect(),
            span: *span,
        },
        lst::Expr::Ann { expr: inner, ty, span } => Expr::Ann {
            expr: Box::new(lower_expr(inner, ctx)),
            ty: lower_type_sig_with_fallback(&ty.repr, ty.span),
            span: *span,
        },
        lst::Expr::Binary { left, op, right, span } => {
            let left = lower_expr(left, ctx);
            let right = lower_expr(right, ctx);
            desugar_binary_op(*span, *op, left, right)
        }
        lst::Expr::Unary { op, expr: inner, span } => {
            let (cap_name, method_name) = unary_op_to_cap_method(*op);
            let inner = lower_expr(inner, ctx);
            desugar_unary_call(*span, cap_name, method_name, inner)
        }
        lst::Expr::Assign { name, value, body, span } => Expr::Let {
            name: name.clone(),
            value: Box::new(lower_expr(value, ctx)),
            body: Box::new(lower_expr(body, ctx)),
            span: *span,
        },
        lst::Expr::Block { stmts, result, span } => {
            // Desugar block to nested Let:
            //   { let x = e1; e2; result } → Let(x, e1, Let("_", e2, result))
            let mut body = maybe_produce(lower_expr(result, ctx), *span);
            for stmt in stmts.iter().rev() {
                match stmt {
                    lst::BlockStmt::Let { name, value, span: stmt_span } => {
                        body = Expr::Let {
                            name: name.clone(),
                            value: Box::new(lower_expr(value, ctx)),
                            body: Box::new(body),
                            span: *stmt_span,
                        };
                    }
                    lst::BlockStmt::Expr { expr, span: stmt_span } => {
                        body = Expr::Let {
                            name: "_".to_string(),
                            value: Box::new(lower_expr(expr, ctx)),
                            body: Box::new(body),
                            span: *stmt_span,
                        };
                    }
                }
            }
            // Rewrap with outer span if there were statements
            if !stmts.is_empty() {
                if let Expr::Let { name, value, body: inner, .. } = body {
                    body = Expr::Let { name, value, body: inner, span: *span };
                }
            }
            body
        },
        lst::Expr::IfElse { condition, then_body, else_body, span } => {
            let scrutinee = lower_expr(condition, ctx);
            let then_arm = MatchArm {
                pattern: Pattern::Ctor { name: "true".into(), args: vec![] },
                body: maybe_produce(lower_expr(then_body, ctx), *span),
                span: *span,
            };
            let else_arm = MatchArm {
                pattern: Pattern::Ctor { name: "false".into(), args: vec![] },
                body: match else_body {
                    Some(e) => maybe_produce(lower_expr(e, ctx), *span),
                    None => Expr::Produce {
                        expr: Box::new(Expr::Ident { name: "unit".into(), span: *span }),
                        span: *span,
                    },
                },
                span: *span,
            };
            Expr::Match {
                scrutinee: Box::new(scrutinee),
                arms: vec![then_arm, else_arm],
                span: *span,
            }
        },
        lst::Expr::Error { span } => Expr::Error { span: *span },
    }
}

// ---------------------------------------------------------------------------
// Operator desugaring
// ---------------------------------------------------------------------------

/// Desugar binary operators.
/// Simple ops (arithmetic, ==, &&, ||) → `perform Cap.method(a, b)`
/// `!=` → `match perform PartialEq.eq(a, b) { .true => Bool.false, .false => Bool.true }`
/// `<`, `<=`, `>`, `>=` → `match perform PartialOrd.cmp(a, b) { ... }`
fn desugar_binary_op(span: Span, op: lst::BinaryOp, left: Expr, right: Expr) -> Expr {
    match op {
        lst::BinaryOp::Add => desugar_binary_call(span, "Add", "add", left, right),
        lst::BinaryOp::Sub => desugar_binary_call(span, "Sub", "sub", left, right),
        lst::BinaryOp::Mul => desugar_binary_call(span, "Mul", "mul", left, right),
        lst::BinaryOp::Div => desugar_binary_call(span, "Div", "div", left, right),
        lst::BinaryOp::Mod => desugar_binary_call(span, "Mod", "mod_", left, right),
        lst::BinaryOp::EqEq => desugar_binary_call(span, "PartialEq", "eq", left, right),
        lst::BinaryOp::AndAnd => desugar_binary_call(span, "Bool", "and", left, right),
        lst::BinaryOp::OrOr => desugar_binary_call(span, "Bool", "or", left, right),
        // != → match perform PartialEq.eq(a, b) { .true => Bool.false, .false => Bool.true }
        lst::BinaryOp::NotEq => {
            let eq_call = desugar_binary_call(span, "PartialEq", "eq", left, right);
            desugar_negate_bool(span, eq_call)
        }
        // Comparison → match perform PartialOrd.cmp(a, b) { ... }
        lst::BinaryOp::Lt => {
            let cmp = desugar_binary_call(span, "PartialOrd", "cmp", left, right);
            desugar_ordering_match(span, cmp, true, false, false)
        }
        lst::BinaryOp::LtEq => {
            let cmp = desugar_binary_call(span, "PartialOrd", "cmp", left, right);
            desugar_ordering_match(span, cmp, true, true, false)
        }
        lst::BinaryOp::Gt => {
            let cmp = desugar_binary_call(span, "PartialOrd", "cmp", left, right);
            desugar_ordering_match(span, cmp, false, false, true)
        }
        lst::BinaryOp::GtEq => {
            let cmp = desugar_binary_call(span, "PartialOrd", "cmp", left, right);
            desugar_ordering_match(span, cmp, false, true, true)
        }
    }
}

fn bool_expr(span: Span, val: bool) -> Expr {
    let variant = if val { "true" } else { "false" };
    Expr::Member {
        object: Box::new(Expr::Ident { name: "Bool".into(), span }),
        member: variant.into(),
        span,
    }
}

/// `match scrutinee { .true => produce Bool.false, .false => produce Bool.true }`
fn desugar_negate_bool(span: Span, scrutinee: Expr) -> Expr {
    Expr::Match {
        scrutinee: Box::new(scrutinee),
        arms: vec![
            MatchArm {
                pattern: Pattern::Ctor { name: "true".into(), args: vec![] },
                body: Expr::Produce { expr: Box::new(bool_expr(span, false)), span },
                span,
            },
            MatchArm {
                pattern: Pattern::Ctor { name: "false".into(), args: vec![] },
                body: Expr::Produce { expr: Box::new(bool_expr(span, true)), span },
                span,
            },
        ],
        span,
    }
}

/// `match scrutinee { .less => produce Bool.T/F, .equal => ..., .greater => ... }`
fn desugar_ordering_match(span: Span, scrutinee: Expr, less: bool, equal: bool, greater: bool) -> Expr {
    Expr::Match {
        scrutinee: Box::new(scrutinee),
        arms: vec![
            MatchArm {
                pattern: Pattern::Ctor { name: "less".into(), args: vec![] },
                body: Expr::Produce { expr: Box::new(bool_expr(span, less)), span },
                span,
            },
            MatchArm {
                pattern: Pattern::Ctor { name: "equal".into(), args: vec![] },
                body: Expr::Produce { expr: Box::new(bool_expr(span, equal)), span },
                span,
            },
            MatchArm {
                pattern: Pattern::Ctor { name: "greater".into(), args: vec![] },
                body: Expr::Produce { expr: Box::new(bool_expr(span, greater)), span },
                span,
            },
        ],
        span,
    }
}

fn unary_op_to_cap_method(op: lst::UnaryOp) -> (&'static str, &'static str) {
    match op {
        lst::UnaryOp::Neg => ("Neg", "neg"),
        lst::UnaryOp::Not => ("Not", "not"),
    }
}

/// Desugar `a <op> b` → `Call(Member(Perform(Cap), method), [a, b])`
fn desugar_binary_call(
    span: Span,
    cap_name: &str,
    method_name: &str,
    left: Expr,
    right: Expr,
) -> Expr {
    let perform = Expr::Perform {
        cap: cap_name.to_owned(),
        span,
    };
    let member = Expr::Member {
        object: Box::new(perform),
        member: method_name.to_owned(),
        span,
    };
    Expr::Call {
        callee: Box::new(member),
        args: vec![left, right],
        span,
    }
}

/// Desugar `-a` / `!a` → `Call(Member(Perform(Cap), method), [a])`
fn desugar_unary_call(
    span: Span,
    cap_name: &str,
    method_name: &str,
    operand: Expr,
) -> Expr {
    let perform = Expr::Perform {
        cap: cap_name.to_owned(),
        span,
    };
    let member = Expr::Member {
        object: Box::new(perform),
        member: method_name.to_owned(),
        span,
    };
    Expr::Call {
        callee: Box::new(member),
        args: vec![operand],
        span,
    }
}

// ---------------------------------------------------------------------------
// Attribute helpers
// ---------------------------------------------------------------------------

fn find_inline_hint(attrs: &[lst::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.name == "inline"
            && (attr.args.iter().any(|arg| arg.key == "always")
                || matches!(&attr.value, Some(lst::Expr::String { value, .. }) if value == "always"))
    })
}

/// Read the effective extern JS path from attributes.
///
/// Resolution order:
/// 1. `#[link(expr = "Obj")] + #[extern(property = "P")]`  → `"Obj.prototype.P"` (receiver property access)
/// 2. `#[link(expr = "Obj")] + #[extern(name = "M")]`      → `"Obj.prototype.M()"` (receiver method call)
/// 3. `#[link(module = "X")] + #[extern(name = "J")]`      → `"J()"` (import alias call)
/// 4. `#[link(module = "X")]`                              → `"<fn_name>()"` (direct import call)
/// 5. `#[extern(name = "...")]` or `#[extern = "..."]`     → verbatim path (current behavior)
fn find_extern_name(attrs: &[lst::Attribute], fallback_fn_name: &str) -> Option<String> {
    let link = attrs.iter().find(|a| a.name == "link");
    let extern_attr = attrs.iter().find(|a| a.name == "extern");

    let attr_arg_str = |attr: &lst::Attribute, key: &str| -> Option<String> {
        attr.args.iter().find(|a| a.key == key).and_then(|a| match &a.value {
            lst::Expr::String { value, .. } => Some(value.clone()),
            _ => None,
        })
    };

    if let Some(link) = link {
        if let Some(expr_base) = attr_arg_str(link, "expr") {
            if let Some(extern_attr) = extern_attr {
                if let Some(prop) = attr_arg_str(extern_attr, "property") {
                    return Some(format!("{expr_base}.prototype.{prop}"));
                }
                if let Some(method) = attr_arg_str(extern_attr, "name") {
                    return Some(format!("{expr_base}.prototype.{method}()"));
                }
                if let Some(static_method) = attr_arg_str(extern_attr, "static") {
                    return Some(format!("{expr_base}.{static_method}()"));
                }
                if let Some(static_prop) = attr_arg_str(extern_attr, "static_property") {
                    return Some(format!("{expr_base}.{static_prop}"));
                }
            }
            // No property/name/static specified — fall back to static fn on the link expr.
            return Some(format!("{expr_base}.{fallback_fn_name}()"));
        }
        if attr_arg_str(link, "module").is_some() {
            // The JS import is aliased to `__lumo_<fn_name>` to avoid collision
            // with the Lumo-side `export function <fn_name>`.
            return Some(format!("__lumo_{fallback_fn_name}()"));
        }
    }

    if let Some(attr) = extern_attr {
        if let Some(value) = &attr.value {
            if let lst::Expr::String { value, .. } = value {
                return Some(value.clone());
            }
        }
        if let Some(op) = attr_arg_str(attr, "operator") {
            return Some(operator_attr_to_extern_name(&op));
        }
        if let Some(name) = attr_arg_str(attr, "name") {
            return Some(name);
        }
    }
    None
}

/// Translate `infix<op>` / `prefix<op>` into the internal `_X_` / `X_` extern
/// name form that the backend specializes into operator expressions.
fn operator_attr_to_extern_name(spec: &str) -> String {
    if let Some(op) = spec.strip_prefix("infix") {
        format!("_{op}_")
    } else if let Some(op) = spec.strip_prefix("prefix") {
        format!("{op}_")
    } else {
        spec.to_owned()
    }
}

/// If `#[link(module = "X")]` is present, return (module, js_name) for import emission.
fn find_link_module(attrs: &[lst::Attribute], fallback_fn_name: &str) -> Option<(String, String)> {
    let link = attrs.iter().find(|a| a.name == "link")?;
    let module = link.args.iter().find(|a| a.key == "module").and_then(|a| match &a.value {
        lst::Expr::String { value, .. } => Some(value.clone()),
        _ => None,
    })?;
    let js_name = attrs
        .iter()
        .find(|a| a.name == "extern")
        .and_then(|a| a.args.iter().find(|kv| kv.key == "name"))
        .and_then(|kv| match &kv.value {
            lst::Expr::String { value, .. } => Some(value.clone()),
            _ => None,
        })
        .unwrap_or_else(|| fallback_fn_name.to_owned());
    Some((module, js_name))
}

// ---------------------------------------------------------------------------
// File-level content hashing (FNV-1a)
// ---------------------------------------------------------------------------

fn hash_file(items: &[Item]) -> ContentHash {
    let mut h = FnvHasher::new();
    h.write_tag("file");
    for item in items {
        hash_item(&mut h, item);
    }
    ContentHash(h.finish())
}

fn hash_item(h: &mut FnvHasher, item: &Item) {
    match item {
        Item::ExternType(ext) => {
            h.write_tag("extern-type");
            h.write_str(&ext.name);
            if let Some(name) = &ext.extern_name {
                h.write_str(name);
            }
        }
        Item::ExternFn(ext) => {
            h.write_tag("extern-fn");
            h.write_str(&ext.name);
            if let Some(name) = &ext.extern_name {
                h.write_str(name);
            }
            for param in &ext.params {
                h.write_str(&param.name);
                h.write_str(&param.ty.value.display());
            }
            if let Some(ret) = &ext.return_type {
                h.write_str(&ret.value.display());
            }
        }
        Item::Data(d) => {
            h.write_tag("data");
            h.write_str(&d.name);
            for v in &d.variants {
                h.write_str(&v.name);
                for ty in &v.payload {
                    h.write_str(&ty.value.display());
                }
            }
        }
        Item::Cap(c) => {
            h.write_tag("cap");
            h.write_str(&c.name);
            for op in &c.operations {
                h.write_str(&op.name);
            }
        }
        Item::Fn(f) => {
            h.write_tag("fn");
            h.write_str(&f.name);
            hash_expr(h, &f.body);
        }
        Item::Use(u) => {
            h.write_tag("use");
            for seg in &u.path {
                h.write_str(seg);
            }
            if let Some(names) = &u.names {
                for name in names {
                    h.write_str(name);
                }
            }
        }
        Item::Impl(i) => {
            h.write_tag("impl");
            h.write_str(&i.target_type.value.display());
            if let Some(cap) = &i.capability {
                h.write_str(&cap.value.display());
            }
            for m in &i.methods {
                h.write_str(&m.name);
                hash_expr(h, &m.body);
            }
        }
    }
}

fn hash_expr(h: &mut FnvHasher, expr: &Expr) {
    match expr {
        Expr::Ident { name, .. } => {
            h.write_tag("ident");
            h.write_str(name);
        }
        Expr::String { value, .. } => {
            h.write_tag("string");
            h.write_str(value);
        }
        Expr::Number { value, .. } => {
            h.write_tag("number");
            h.write_str(value);
        }
        Expr::Call { callee, args, .. } => {
            h.write_tag("call");
            hash_expr(h, callee);
            for arg in args {
                hash_expr(h, arg);
            }
        }
        Expr::Member { object, member, .. } => {
            h.write_tag("member");
            hash_expr(h, object);
            h.write_str(member);
        }
        Expr::Produce { expr, .. } => {
            h.write_tag("produce");
            hash_expr(h, expr);
        }
        Expr::Thunk { expr, .. } => {
            h.write_tag("thunk");
            hash_expr(h, expr);
        }
        Expr::Force { expr, .. } => {
            h.write_tag("force");
            hash_expr(h, expr);
        }
        Expr::Let { name, value, body, .. } => {
            h.write_tag("let");
            h.write_str(name);
            hash_expr(h, value);
            hash_expr(h, body);
        }
        Expr::Match { scrutinee, arms, .. } => {
            h.write_tag("match");
            hash_expr(h, scrutinee);
            for arm in arms {
                h.write_str(&arm.pattern.display());
                hash_expr(h, &arm.body);
            }
        }
        Expr::Perform { cap, .. } => {
            h.write_tag("perform");
            h.write_str(cap);
        }
        Expr::Handle { cap, handler, body, .. } => {
            h.write_tag("handle");
            h.write_str(cap);
            hash_expr(h, handler);
            hash_expr(h, body);
        }
        Expr::Bundle { entries, .. } => {
            h.write_tag("bundle");
            for e in entries {
                h.write_str(&e.name);
                hash_expr(h, &e.body);
            }
        }
        Expr::Ann { expr, ty, .. } => {
            h.write_tag("ann");
            h.write_str(&ty.value.display());
            hash_expr(h, expr);
        }
        Expr::Error { .. } => {
            h.write_tag("error");
        }
    }
}

struct FnvHasher {
    state: u64,
}

impl FnvHasher {
    fn new() -> Self {
        Self {
            state: 0xcbf29ce484222325,
        }
    }

    fn write_tag(&mut self, tag: &str) {
        self.write_str(tag);
        self.write_byte(0xff);
    }

    fn write_str(&mut self, value: &str) {
        self.write_u64(value.len() as u64);
        for b in value.as_bytes() {
            self.write_byte(*b);
        }
    }

    fn write_u64(&mut self, value: u64) {
        for b in value.to_le_bytes() {
            self.write_byte(b);
        }
    }

    fn write_byte(&mut self, value: u8) {
        self.state ^= value as u64;
        self.state = self.state.wrapping_mul(0x100000001b3);
    }

    fn finish(&self) -> u64 {
        self.state
    }
}
