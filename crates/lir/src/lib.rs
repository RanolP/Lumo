pub mod parse;
pub mod print;
pub mod validate;

use std::collections::HashSet;

use lumo_hir as hir;
pub use lumo_hir::GenericParam;
use lumo_span::Span;
use lumo_types::{CapRef, ContentHash, ExprId, Pattern, Spanned, TypeExpr};

// ---------------------------------------------------------------------------
// LIR types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub items: Vec<Item>,
    pub content_hash: ContentHash,
    /// Side-table: ExprId indexes into this.
    pub spans: Vec<Span>,
}

impl File {
    pub fn span_of(&self, id: ExprId) -> Span {
        self.spans[id.0 as usize]
    }
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
pub enum AsRawValue {
    True,
    False,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantDecl {
    pub name: String,
    pub payload: Vec<Spanned<TypeExpr>>,
    pub as_raw: Option<AsRawValue>,
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
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub cap: Option<CapRef>,
    pub value: Expr,
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
    pub generics: Vec<GenericParam>,
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
    pub value: Expr,
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
    Ident { id: ExprId, name: String },
    String { id: ExprId, value: String },
    Number { id: ExprId, value: String },
    Ctor { id: ExprId, name: String, called: bool, args: Vec<Expr> },
    Thunk { id: ExprId, expr: Box<Expr> },
    Roll { id: ExprId, expr: Box<Expr> },
    Bundle { id: ExprId, entries: Vec<BundleEntry> },
    Produce { id: ExprId, expr: Box<Expr> },
    Force { id: ExprId, expr: Box<Expr> },
    Lambda { id: ExprId, param: String, body: Box<Expr> },
    Apply { id: ExprId, callee: Box<Expr>, arg: Box<Expr> },
    Let { id: ExprId, name: String, value: Box<Expr>, body: Box<Expr> },
    Match { id: ExprId, scrutinee: Box<Expr>, arms: Vec<MatchArm> },
    Unroll { id: ExprId, expr: Box<Expr> },
    Perform { id: ExprId, cap: String, type_args: Vec<String> },
    Handle { id: ExprId, cap: String, type_args: Vec<String>, handler: Box<Expr>, body: Box<Expr> },
    Member { id: ExprId, object: Box<Expr>, field: String },
    Ann { id: ExprId, expr: Box<Expr>, ty: TypeExpr },
    Error { id: ExprId },
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
// Expr accessors
// ---------------------------------------------------------------------------

impl Expr {
    pub fn id(&self) -> ExprId {
        match self {
            Expr::Ident { id, .. }
            | Expr::String { id, .. }
            | Expr::Number { id, .. }
            | Expr::Ctor { id, .. }
            | Expr::Thunk { id, .. }
            | Expr::Roll { id, .. }
            | Expr::Bundle { id, .. }
            | Expr::Produce { id, .. }
            | Expr::Force { id, .. }
            | Expr::Lambda { id, .. }
            | Expr::Apply { id, .. }
            | Expr::Let { id, .. }
            | Expr::Match { id, .. }
            | Expr::Unroll { id, .. }
            | Expr::Perform { id, .. }
            | Expr::Handle { id, .. }
            | Expr::Member { id, .. }
            | Expr::Ann { id, .. }
            | Expr::Error { id } => *id,
        }
    }
}

pub fn expr_references_name(expr: &Expr, target: &str) -> bool {
    match expr {
        Expr::Ident { name, .. } => name == target,
        Expr::String { .. } | Expr::Number { .. } | Expr::Error { .. } | Expr::Perform { .. } => {
            false
        }
        Expr::Produce { expr, .. }
        | Expr::Thunk { expr, .. }
        | Expr::Force { expr, .. }
        | Expr::Unroll { expr, .. }
        | Expr::Roll { expr, .. }
        | Expr::Ann { expr, .. } => expr_references_name(expr, target),
        Expr::Lambda { param, body, .. } => {
            if param == target {
                false
            } else {
                expr_references_name(body, target)
            }
        }
        Expr::Apply { callee, arg, .. } => {
            expr_references_name(callee, target) || expr_references_name(arg, target)
        }
        Expr::Let {
            name, value, body, ..
        } => {
            expr_references_name(value, target)
                || (name != target && expr_references_name(body, target))
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            expr_references_name(scrutinee, target)
                || arms
                    .iter()
                    .any(|arm| expr_references_name(&arm.body, target))
        }
        Expr::Ctor { args, .. } => args.iter().any(|a| expr_references_name(a, target)),
        Expr::Handle {
            handler, body, ..
        } => expr_references_name(handler, target) || expr_references_name(body, target),
        Expr::Bundle { entries, .. } => entries
            .iter()
            .any(|e| expr_references_name(&e.body, target)),
        Expr::Member { object, .. } => expr_references_name(object, target),
    }
}

// ---------------------------------------------------------------------------
// Lowering: HIR → LIR
// ---------------------------------------------------------------------------

struct LoweringCtx {
    spans: Vec<Span>,
    variants: Vec<(String, String)>,
    caps: HashSet<String>,
}

impl LoweringCtx {
    fn new(file: &hir::File) -> Self {
        Self {
            spans: Vec::new(),
            variants: collect_variants(file),
            caps: collect_caps(file),
        }
    }

    fn alloc(&mut self, span: Span) -> ExprId {
        let id = ExprId(self.spans.len() as u32);
        self.spans.push(span);
        id
    }
}

pub fn lower(file: &hir::File) -> File {
    let mut ctx = LoweringCtx::new(file);
    let items: Vec<Item> = file
        .items
        .iter()
        .map(|item| lower_item(&mut ctx, item))
        .collect();

    let content_hash = file.content_hash;
    File {
        items,
        content_hash,
        spans: ctx.spans,
    }
}

fn lower_item(ctx: &mut LoweringCtx, item: &hir::Item) -> Item {
    match item {
        hir::Item::ExternType(ext) => Item::ExternType(ExternTypeDecl {
            name: ext.name.clone(),
            extern_name: ext.extern_name.clone(),
            span: ext.span,
        }),
        hir::Item::ExternFn(ext) => Item::ExternFn(ExternFnDecl {
            name: ext.name.clone(),
            extern_name: ext.extern_name.clone(),
            link_module: ext.link_module.clone(),
            inline: ext.inline,
            params: ext.params.iter().map(lower_param).collect(),
            return_type: ext.return_type.clone(),
            cap: ext.cap.clone(),
            span: ext.span,
        }),
        hir::Item::Data(data) => Item::Data(DataDecl {
            name: data.name.clone(),
            generics: data.generics.clone(),
            variants: data
                .variants
                .iter()
                .map(|v| VariantDecl {
                    name: v.name.clone(),
                    payload: v.payload.clone(),
                    as_raw: v.as_raw.as_ref().map(|r| match r {
                        hir::AsRawValue::True => AsRawValue::True,
                        hir::AsRawValue::False => AsRawValue::False,
                    }),
                    span: v.span,
                })
                .collect(),
            span: data.span,
        }),
        hir::Item::Cap(cap) => Item::Cap(CapDecl {
            name: cap.name.clone(),
            operations: cap
                .operations
                .iter()
                .map(|op| OperationDecl {
                    name: op.name.clone(),
                    params: op.params.iter().map(lower_param).collect(),
                    return_type: op.return_type.clone(),
                    span: op.span,
                })
                .collect(),
            span: cap.span,
        }),
        hir::Item::Fn(func) => Item::Fn(lower_fn(ctx, func)),
        hir::Item::Use(u) => Item::Use(UseDecl {
            path: u.path.clone(),
            names: u.names.clone(),
            span: u.span,
        }),
        hir::Item::Impl(impl_decl) => Item::Impl(lower_impl(ctx, impl_decl)),
    }
}

fn lower_param(param: &hir::Param) -> Param {
    Param {
        name: param.name.clone(),
        ty: param.ty.clone(),
        span: param.span,
    }
}

fn lower_fn(ctx: &mut LoweringCtx, func: &hir::FnDecl) -> FnDecl {
    let params: Vec<Param> = func.params.iter().map(lower_param).collect();
    let body = lower_expr(ctx, &func.body);
    let value = lower_fn_value(ctx, func.span, &params, body);
    FnDecl {
        name: func.name.clone(),
        generics: func.generics.clone(),
        params,
        return_type: func.return_type.clone(),
        cap: func.cap.clone(),
        value,
        inline: func.inline,
        span: func.span,
    }
}

fn lower_impl(ctx: &mut LoweringCtx, impl_decl: &hir::ImplDecl) -> ImplDecl {
    let methods = impl_decl
        .methods
        .iter()
        .map(|m| {
            let params: Vec<Param> = m.params.iter().map(lower_param).collect();
            let body = lower_expr(ctx, &m.body);
            let value = lower_fn_value(ctx, m.span, &params, body);
            ImplMethodDecl {
                name: m.name.clone(),
                params,
                return_type: m.return_type.clone(),
                value,
                span: m.span,
            }
        })
        .collect();

    ImplDecl {
        name: impl_decl.name.clone(),
        generics: impl_decl.generics.clone(),
        target_type: impl_decl.target_type.clone(),
        capability: impl_decl.capability.clone(),
        methods,
        span: impl_decl.span,
    }
}

// ---------------------------------------------------------------------------
// Expression lowering
// ---------------------------------------------------------------------------

fn lower_expr(ctx: &mut LoweringCtx, expr: &hir::Expr) -> Expr {
    let span = expr.span();
    match expr {
        hir::Expr::Ident { name, .. } => Expr::Ident {
            id: ctx.alloc(span),
            name: name.clone(),
        },
        hir::Expr::String { value, .. } => Expr::String {
            id: ctx.alloc(span),
            value: value.clone(),
        },
        hir::Expr::Number { value, .. } => Expr::Number {
            id: ctx.alloc(span),
            value: value.clone(),
        },
        hir::Expr::Produce { expr, .. } => {
            let inner = Box::new(lower_expr(ctx, expr));
            Expr::Produce {
                id: ctx.alloc(span),
                expr: inner,
            }
        }
        hir::Expr::Thunk { expr, .. } => {
            let inner = Box::new(lower_expr(ctx, expr));
            Expr::Thunk {
                id: ctx.alloc(span),
                expr: inner,
            }
        }
        hir::Expr::Lambda { params, body, .. } => {
            let lowered_body = lower_expr(ctx, body);
            let lambda_chain = params.iter().rev().fold(lowered_body, |acc, (name, _ty)| {
                Expr::Lambda {
                    id: ctx.alloc(span),
                    param: name.clone(),
                    body: Box::new(acc),
                }
            });
            Expr::Thunk {
                id: ctx.alloc(span),
                expr: Box::new(lambda_chain),
            }
        }
        hir::Expr::Force { expr, .. } => {
            let inner = Box::new(lower_expr(ctx, expr));
            Expr::Force {
                id: ctx.alloc(span),
                expr: inner,
            }
        }
        hir::Expr::Let {
            name, value, body, ..
        } => {
            let value = Box::new(lower_expr(ctx, value));
            let body = Box::new(lower_expr(ctx, body));
            Expr::Let {
                id: ctx.alloc(span),
                name: name.clone(),
                value,
                body,
            }
        }
        hir::Expr::Match {
            scrutinee, arms, ..
        } => {
            let lowered_scrutinee = lower_expr(ctx, scrutinee);
            let scrutinee = Box::new(mk_unroll(ctx, span, lowered_scrutinee));
            let arms = arms
                .iter()
                .map(|arm| MatchArm {
                    pattern: arm.pattern.clone(),
                    body: lower_expr(ctx, &arm.body),
                    span: arm.span,
                })
                .collect();
            Expr::Match {
                id: ctx.alloc(span),
                scrutinee,
                arms,
            }
        }
        hir::Expr::Member { object, member, .. } => {
            // Check if this is a data constructor (Ident.Variant pattern)
            if let hir::Expr::Ident { name: owner, .. } = object.as_ref() {
                if ctx
                    .variants
                    .iter()
                    .any(|(o, v)| o == owner && v == member)
                {
                    let ctor = mk_ctor(ctx, span, &format!("{owner}.{member}"), false, Vec::new());
                    return mk_roll(ctx, span, ctor);
                }
                // Check if this is a capability access (Cap.operation pattern)
                if ctx.caps.contains(owner) {
                    return Expr::Member {
                        id: ctx.alloc(span),
                        object: Box::new(Expr::Perform {
                            id: ctx.alloc(span),
                            cap: owner.clone(),
                            type_args: vec![],
                        }),
                        field: member.clone(),
                    };
                }
            }
            let lowered_object = Box::new(lower_expr(ctx, object));
            Expr::Member {
                id: ctx.alloc(span),
                object: lowered_object,
                field: member.clone(),
            }
        }
        hir::Expr::Call { callee, args, .. } => {
            let args: Vec<Expr> = args.iter().map(|arg| lower_expr(ctx, arg)).collect();
            if let hir::Expr::Member { object, member, .. } = callee.as_ref() {
                if let hir::Expr::Ident { name: owner, .. } = object.as_ref() {
                    // Only create Ctor if owner is a known data type
                    if ctx.variants.iter().any(|(o, _)| o == owner) {
                        let ctor =
                            mk_ctor(ctx, span, &format!("{owner}.{member}"), true, args);
                        if ctx
                            .variants
                            .iter()
                            .any(|(o, v)| o == owner && v == member)
                        {
                            return mk_roll(ctx, span, ctor);
                        }
                        return ctor;
                    }
                }
                // Non-data Member: computation callee, no Force needed
                let callee_lowered = lower_expr(ctx, callee);
                return lower_apply_chain(ctx, span, callee_lowered, args);
            }
            let callee_lowered = lower_expr(ctx, callee);
            let forced = mk_force(ctx, span, callee_lowered);
            lower_apply_chain(ctx, span, forced, args)
        }
        hir::Expr::Perform { cap, .. } => Expr::Perform {
            id: ctx.alloc(span),
            cap: cap.clone(),
            type_args: vec![],
        },
        hir::Expr::Handle {
            cap,
            type_args,
            handler,
            body,
            ..
        } => {
            let handler = Box::new(lower_expr(ctx, handler));
            let body = Box::new(lower_expr(ctx, body));
            Expr::Handle {
                id: ctx.alloc(span),
                cap: cap.clone(),
                type_args: type_args.clone(),
                handler,
                body,
            }
        }
        hir::Expr::Bundle { entries, .. } => {
            let lir_entries = entries
                .iter()
                .map(|e| BundleEntry {
                    name: e.name.clone(),
                    params: e.params.iter().map(lower_param).collect(),
                    body: lower_expr(ctx, &e.body),
                    span: e.span,
                })
                .collect();
            Expr::Bundle {
                id: ctx.alloc(span),
                entries: lir_entries,
            }
        }
        hir::Expr::Ann { expr, ty, .. } => {
            let inner = Box::new(lower_expr(ctx, expr));
            Expr::Ann {
                id: ctx.alloc(span),
                expr: inner,
                ty: ty.value.clone(),
            }
        }
        hir::Expr::Error { .. } => Expr::Error {
            id: ctx.alloc(span),
        },
    }
}

// ---------------------------------------------------------------------------
// Expression constructors
// ---------------------------------------------------------------------------

fn lower_fn_value(ctx: &mut LoweringCtx, span: Span, params: &[Param], body: Expr) -> Expr {
    let mut lowered = body;
    for param in params.iter().rev() {
        lowered = mk_lambda(ctx, param.span, &param.name, lowered);
    }
    mk_thunk(ctx, span, lowered)
}

fn mk_thunk(ctx: &mut LoweringCtx, span: Span, expr: Expr) -> Expr {
    Expr::Thunk {
        id: ctx.alloc(span),
        expr: Box::new(expr),
    }
}

fn mk_force(ctx: &mut LoweringCtx, span: Span, expr: Expr) -> Expr {
    Expr::Force {
        id: ctx.alloc(span),
        expr: Box::new(expr),
    }
}

fn mk_lambda(ctx: &mut LoweringCtx, span: Span, param: &str, body: Expr) -> Expr {
    Expr::Lambda {
        id: ctx.alloc(span),
        param: param.to_owned(),
        body: Box::new(body),
    }
}

fn mk_apply(ctx: &mut LoweringCtx, span: Span, callee: Expr, arg: Expr) -> Expr {
    Expr::Apply {
        id: ctx.alloc(span),
        callee: Box::new(callee),
        arg: Box::new(arg),
    }
}

fn lower_apply_chain(ctx: &mut LoweringCtx, span: Span, callee: Expr, args: Vec<Expr>) -> Expr {
    args.into_iter()
        .fold(callee, |callee, arg| mk_apply(ctx, span, callee, arg))
}

fn mk_unroll(ctx: &mut LoweringCtx, span: Span, expr: Expr) -> Expr {
    Expr::Unroll {
        id: ctx.alloc(span),
        expr: Box::new(expr),
    }
}

fn mk_roll(ctx: &mut LoweringCtx, span: Span, expr: Expr) -> Expr {
    Expr::Roll {
        id: ctx.alloc(span),
        expr: Box::new(expr),
    }
}

fn mk_ctor(
    ctx: &mut LoweringCtx,
    span: Span,
    name: &str,
    called: bool,
    args: Vec<Expr>,
) -> Expr {
    Expr::Ctor {
        id: ctx.alloc(span),
        name: name.to_owned(),
        called,
        args,
    }
}

fn collect_variants(file: &hir::File) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for item in &file.items {
        if let hir::Item::Data(d) = item {
            for v in &d.variants {
                out.push((d.name.clone(), v.name.clone()));
            }
        }
    }
    out
}

fn collect_caps(file: &hir::File) -> HashSet<String> {
    file.items
        .iter()
        .filter_map(|item| {
            if let hir::Item::Cap(c) = item {
                Some(c.name.clone())
            } else {
                None
            }
        })
        .collect()
}
