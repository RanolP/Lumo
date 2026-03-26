use derive_more::Debug;

use crate::{hir, lexer::Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub items: Vec<Item>,
    #[debug(skip)]
    pub content_hash: ContentHash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    ExternType(ExternTypeDecl),
    ExternFn(ExternFnDecl),
    Data(DataDecl),
    Effect(EffectDecl),
    Fn(FnDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternTypeDecl {
    #[debug(skip)]
    pub id: ContentHash,
    #[debug(skip)]
    pub structural_hash: ContentHash,
    #[debug(skip)]
    pub source_span: Span,
    pub name: String,
    pub extern_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternFnDecl {
    #[debug(skip)]
    pub id: ContentHash,
    #[debug(skip)]
    pub structural_hash: ContentHash,
    #[debug(skip)]
    pub source_span: Span,
    pub name: String,
    pub extern_name: Option<String>,
    pub params: Vec<ParamDecl>,
    pub return_type_repr: Option<String>,
    #[debug(skip)]
    pub return_type_span: Option<Span>,
    pub effect_repr: Option<String>,
    #[debug(skip)]
    pub effect_span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDecl {
    pub name: String,
    pub generics: Vec<String>,
    #[debug(skip)]
    pub id: ContentHash,
    #[debug(skip)]
    pub structural_hash: ContentHash,
    #[debug(skip)]
    pub source_span: Span,
    pub variants: Vec<VariantDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantDecl {
    #[debug(skip)]
    pub id: ContentHash,
    #[debug(skip)]
    pub structural_hash: ContentHash,
    #[debug(skip)]
    pub source_span: Span,
    pub name: String,
    pub payload_types: Vec<String>,
    #[debug(skip)]
    pub payload_spans: Vec<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectDecl {
    pub name: String,
    #[debug(skip)]
    pub id: ContentHash,
    #[debug(skip)]
    pub structural_hash: ContentHash,
    #[debug(skip)]
    pub source_span: Span,
    pub operations: Vec<OperationDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationDecl {
    #[debug(skip)]
    pub id: ContentHash,
    #[debug(skip)]
    pub structural_hash: ContentHash,
    #[debug(skip)]
    pub source_span: Span,
    pub name: String,
    pub params: Vec<ParamDecl>,
    pub return_type_repr: Option<String>,
    #[debug(skip)]
    pub return_type_span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
    pub name: String,
    pub generics: Vec<String>,
    #[debug(skip)]
    pub id: ContentHash,
    #[debug(skip)]
    pub structural_hash: ContentHash,
    #[debug(skip)]
    pub source_span: Span,
    pub params: Vec<ParamDecl>,
    pub return_type_repr: Option<String>,
    #[debug(skip)]
    pub return_type_span: Option<Span>,
    pub effect_repr: Option<String>,
    #[debug(skip)]
    pub effect_span: Option<Span>,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamDecl {
    #[debug(skip)]
    pub source_span: Span,
    pub name: String,
    pub ty_repr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Ident {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        name: String,
    },
    String {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        value: String,
    },
    Produce {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        expr: Box<Expr>,
    },
    Thunk {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        expr: Box<Expr>,
    },
    Force {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        expr: Box<Expr>,
    },
    Lambda {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        param: String,
        body: Box<Expr>,
    },
    Apply {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        callee: Box<Expr>,
        arg: Box<Expr>,
    },
    Unroll {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        expr: Box<Expr>,
    },
    LetIn {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    Match {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    Ctor {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        name: String,
        called: bool,
        args: Vec<Expr>,
    },
    Roll {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        expr: Box<Expr>,
    },
    Perform {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        effect: String,
    },
    Handle {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        effect: String,
        handler: Box<Expr>,
        body: Box<Expr>,
    },
    Bundle {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        entries: Vec<LirBundleEntry>,
    },
    Member {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        object: Box<Expr>,
        field: String,
    },
    Ann {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        expr: Box<Expr>,
        ty_repr: String,
    },
    Error {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    #[debug(skip)]
    pub source_span: Span,
    pub pattern: String,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirBundleEntry {
    #[debug(skip)]
    pub source_span: Span,
    pub name: String,
    pub params: Vec<ParamDecl>,
    pub body: Expr,
}

pub fn lower(file: &hir::File) -> File {
    let variants = collect_variants(file);
    let items = file
        .items
        .iter()
        .map(|item| match item {
            hir::Item::ExternType(ext) => Item::ExternType(lower_extern_type(ext)),
            hir::Item::ExternFn(ext) => Item::ExternFn(lower_extern_fn(ext)),
            hir::Item::Data(data) => Item::Data(lower_data(data)),
            hir::Item::Effect(effect) => Item::Effect(lower_effect(effect)),
            hir::Item::Fn(func) => Item::Fn(lower_fn(func, &variants)),
        })
        .collect::<Vec<_>>();

    let mut hasher = Hasher::new();
    hasher.write_tag("file");
    for item in &items {
        hasher.write_u64(item_structural_hash(item).0);
    }

    File {
        items,
        content_hash: ContentHash(hasher.finish()),
    }
}

fn lower_extern_type(ext: &hir::ExternTypeDecl) -> ExternTypeDecl {
    ExternTypeDecl {
        id: ContentHash(ext.id.0),
        structural_hash: ContentHash(ext.structural_hash.0),
        source_span: ext.source_span,
        name: ext.name.clone(),
        extern_name: ext.extern_name.clone(),
    }
}

fn lower_extern_fn(ext: &hir::ExternFnDecl) -> ExternFnDecl {
    ExternFnDecl {
        id: ContentHash(ext.id.0),
        structural_hash: ContentHash(ext.structural_hash.0),
        source_span: ext.source_span,
        name: ext.name.clone(),
        extern_name: ext.extern_name.clone(),
        params: ext
            .params
            .iter()
            .map(|param| ParamDecl {
                source_span: param.source_span,
                name: param.name.clone(),
                ty_repr: param.ty_repr.clone(),
            })
            .collect(),
        return_type_repr: ext.return_type_repr.clone(),
        return_type_span: ext.return_type_span,
        effect_repr: ext.effect_repr.clone(),
        effect_span: ext.effect_span,
    }
}

fn lower_effect(effect: &hir::EffectDecl) -> EffectDecl {
    EffectDecl {
        name: effect.name.clone(),
        id: ContentHash(effect.id.0),
        structural_hash: ContentHash(effect.structural_hash.0),
        source_span: effect.source_span,
        operations: effect
            .operations
            .iter()
            .map(|op| OperationDecl {
                id: ContentHash(op.id.0),
                structural_hash: ContentHash(op.structural_hash.0),
                source_span: op.source_span,
                name: op.name.clone(),
                params: op
                    .params
                    .iter()
                    .map(|param| ParamDecl {
                        source_span: param.source_span,
                        name: param.name.clone(),
                        ty_repr: param.ty_repr.clone(),
                    })
                    .collect(),
                return_type_repr: op.return_type_repr.clone(),
                return_type_span: op.return_type_span,
            })
            .collect(),
    }
}

fn lower_data(data: &hir::DataDecl) -> DataDecl {
    DataDecl {
        name: data.name.clone(),
        generics: data.generics.clone(),
        id: ContentHash(data.id.0),
        structural_hash: ContentHash(data.structural_hash.0),
        source_span: data.source_span,
        variants: data
            .variants
            .iter()
            .map(|variant| VariantDecl {
                id: ContentHash(variant.id.0),
                structural_hash: ContentHash(variant.structural_hash.0),
                source_span: variant.source_span,
                name: variant.name.clone(),
                payload_types: variant.payload_types.clone(),
                payload_spans: variant.payload_spans.clone(),
            })
            .collect(),
    }
}

fn lower_fn(func: &hir::FnDecl, variants: &[(String, String)]) -> FnDecl {
    let params = func
        .params
        .iter()
        .map(|param| ParamDecl {
            source_span: param.source_span,
            name: param.name.clone(),
            ty_repr: param.ty_repr.clone(),
        })
        .collect::<Vec<_>>();
    let body = lower_expr(&func.body, variants);
    let value = lower_fn_value(func.source_span, &params, body);
    let id = source_id("fn", func.source_span);
    let mut hasher = Hasher::new();
    hasher.write_tag("fn");
    hasher.write_u64(expr_structural_hash(&value).0);
    let structural_hash = ContentHash(hasher.finish());

    FnDecl {
        name: func.name.clone(),
        generics: func.generics.clone(),
        id,
        structural_hash,
        source_span: func.source_span,
        params,
        return_type_repr: func.return_type_repr.clone(),
        return_type_span: func.return_type_span,
        effect_repr: func.effect_repr.clone(),
        effect_span: func.effect_span,
        value,
    }
}

fn lower_expr(expr: &hir::Expr, variants: &[(String, String)]) -> Expr {
    match expr {
        hir::Expr::Ident {
            name, source_span, ..
        } => {
            let id = source_id("ident", *source_span);
            let mut hasher = Hasher::new();
            hasher.write_tag("ident");
            hasher.write_str(name);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Ident {
                id,
                structural_hash,
                source_span: *source_span,
                name: name.clone(),
            }
        }
        hir::Expr::String {
            value, source_span, ..
        } => {
            let id = source_id("string", *source_span);
            let mut hasher = Hasher::new();
            hasher.write_tag("string");
            hasher.write_str(value);
            let structural_hash = ContentHash(hasher.finish());
            Expr::String {
                id,
                structural_hash,
                source_span: *source_span,
                value: value.clone(),
            }
        }
        hir::Expr::Produce {
            expr, source_span, ..
        } => {
            let expr = Box::new(lower_expr(expr, variants));
            let id = source_expr_id("produce", *source_span, &[expr_id(&expr)]);
            let mut hasher = Hasher::new();
            hasher.write_tag("produce");
            hasher.write_u64(expr_structural_hash(&expr).0);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Produce {
                id,
                structural_hash,
                source_span: *source_span,
                expr,
            }
        }
        hir::Expr::Thunk {
            expr, source_span, ..
        } => {
            let expr = Box::new(lower_expr(expr, variants));
            let id = source_expr_id("thunk", *source_span, &[expr_id(&expr)]);
            let mut hasher = Hasher::new();
            hasher.write_tag("thunk");
            hasher.write_u64(expr_structural_hash(&expr).0);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Thunk {
                id,
                structural_hash,
                source_span: *source_span,
                expr,
            }
        }
        hir::Expr::Force {
            expr, source_span, ..
        } => {
            let expr = Box::new(lower_expr(expr, variants));
            let id = source_expr_id("force", *source_span, &[expr_id(&expr)]);
            let mut hasher = Hasher::new();
            hasher.write_tag("force");
            hasher.write_u64(expr_structural_hash(&expr).0);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Force {
                id,
                structural_hash,
                source_span: *source_span,
                expr,
            }
        }
        hir::Expr::LetIn {
            name,
            value,
            body,
            source_span,
            ..
        } => {
            let value = Box::new(lower_expr(value, variants));
            let body = Box::new(lower_expr(body, variants));
            let id = source_expr_id("let-in", *source_span, &[expr_id(&value), expr_id(&body)]);
            let mut hasher = Hasher::new();
            hasher.write_tag("let-in");
            hasher.write_str(name);
            hasher.write_u64(expr_structural_hash(&value).0);
            hasher.write_u64(expr_structural_hash(&body).0);
            let structural_hash = ContentHash(hasher.finish());
            Expr::LetIn {
                id,
                structural_hash,
                source_span: *source_span,
                name: name.clone(),
                value,
                body,
            }
        }
        hir::Expr::Match {
            scrutinee,
            arms,
            source_span,
            ..
        } => {
            let lowered_scrutinee = lower_expr(scrutinee, variants);
            let scrutinee = Box::new(with_unroll(*source_span, lowered_scrutinee));
            let arms = arms
                .iter()
                .map(|arm| MatchArm {
                    source_span: arm.source_span,
                    pattern: arm.pattern.clone(),
                    body: lower_expr(&arm.body, variants),
                })
                .collect::<Vec<_>>();
            let mut source_children = vec![expr_id(&scrutinee)];
            source_children.extend(arms.iter().map(|arm| expr_id(&arm.body)));
            let id = source_expr_id("match", *source_span, &source_children);
            let mut hasher = Hasher::new();
            hasher.write_tag("match");
            hasher.write_u64(expr_structural_hash(&scrutinee).0);
            for arm in &arms {
                hasher.write_str(&arm.pattern);
                hasher.write_u64(expr_structural_hash(&arm.body).0);
            }
            let structural_hash = ContentHash(hasher.finish());
            Expr::Match {
                id,
                structural_hash,
                source_span: *source_span,
                scrutinee,
                arms,
            }
        }
        hir::Expr::Member {
            object,
            member,
            source_span,
            ..
        } => {
            // Check if this is a data constructor (Ident.Variant pattern)
            if let hir::Expr::Ident { name: owner, .. } = object.as_ref() {
                if variants.iter().any(|(o, v)| o == owner && v == member) {
                    let ctor = ctor_expr(
                        *source_span,
                        &format!("{owner}.{member}"),
                        false,
                        Vec::new(),
                    );
                    return with_roll(*source_span, ctor);
                }
                // Check if it could be a non-recursive data constructor
                // (existing behavior for non-variant Ident.member)
                // Fall through to general member access below
            }

            // General member access (e.g., (perform E).op)
            let lowered_object = Box::new(lower_expr(object, variants));
            let id = source_expr_id("member", *source_span, &[expr_id(&lowered_object)]);
            let mut hasher = Hasher::new();
            hasher.write_tag("member");
            hasher.write_u64(expr_structural_hash(&lowered_object).0);
            hasher.write_str(member);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Member {
                id,
                structural_hash,
                source_span: *source_span,
                object: lowered_object,
                field: member.clone(),
            }
        }
        hir::Expr::Call {
            callee,
            args,
            source_span,
            ..
        } => {
            let args = args
                .iter()
                .map(|arg| lower_expr(arg, variants))
                .collect::<Vec<_>>();
            if let hir::Expr::Member { object, member, .. } = callee.as_ref() {
                if let hir::Expr::Ident { name: owner, .. } = object.as_ref() {
                    // Only create Ctor if owner is a known data type
                    if variants.iter().any(|(o, _)| o == owner) {
                        let ctor =
                            ctor_expr(*source_span, &format!("{owner}.{member}"), true, args);
                        if variants.iter().any(|(o, v)| o == owner && v == member) {
                            return with_roll(*source_span, ctor);
                        }
                        return ctor;
                    }
                }
                // Non-data Member: computation callee, no Force needed
                let callee_lowered = lower_expr(callee, variants);
                return lower_apply_chain(*source_span, callee_lowered, args);
            }
            let callee = force_expr(*source_span, lower_expr(callee, variants));
            lower_apply_chain(*source_span, callee, args)
        }
        hir::Expr::Perform {
            effect,
            source_span,
            ..
        } => {
            let id = source_id("perform", *source_span);
            let mut hasher = Hasher::new();
            hasher.write_tag("perform");
            hasher.write_str(effect);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Perform {
                id,
                structural_hash,
                source_span: *source_span,
                effect: effect.clone(),
            }
        }
        hir::Expr::Handle {
            effect,
            handler,
            body,
            source_span,
            ..
        } => {
            let handler = Box::new(lower_expr(handler, variants));
            let body = Box::new(lower_expr(body, variants));
            let id = source_expr_id(
                "handle",
                *source_span,
                &[expr_id(&handler), expr_id(&body)],
            );
            let mut hasher = Hasher::new();
            hasher.write_tag("handle");
            hasher.write_str(effect);
            hasher.write_u64(expr_structural_hash(&handler).0);
            hasher.write_u64(expr_structural_hash(&body).0);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Handle {
                id,
                structural_hash,
                source_span: *source_span,
                effect: effect.clone(),
                handler,
                body,
            }
        }
        hir::Expr::Bundle {
            entries,
            source_span,
            ..
        } => {
            let lir_entries = entries
                .iter()
                .map(|e| {
                    let params = e
                        .params
                        .iter()
                        .map(|p| ParamDecl {
                            source_span: p.source_span,
                            name: p.name.clone(),
                            ty_repr: p.ty_repr.clone(),
                        })
                        .collect::<Vec<_>>();
                    LirBundleEntry {
                        source_span: e.source_span,
                        name: e.name.clone(),
                        params,
                        body: lower_expr(&e.body, variants),
                    }
                })
                .collect::<Vec<_>>();
            let source_children: Vec<ContentHash> =
                lir_entries.iter().map(|e| expr_id(&e.body)).collect();
            let id = source_expr_id("bundle", *source_span, &source_children);
            let mut hasher = Hasher::new();
            hasher.write_tag("bundle");
            for e in &lir_entries {
                hasher.write_str(&e.name);
                hasher.write_u64(expr_structural_hash(&e.body).0);
            }
            let structural_hash = ContentHash(hasher.finish());
            Expr::Bundle {
                id,
                structural_hash,
                source_span: *source_span,
                entries: lir_entries,
            }
        }
        hir::Expr::Ann {
            expr,
            ty_repr,
            source_span,
            ..
        } => {
            let inner = Box::new(lower_expr(expr, variants));
            let id = source_expr_id("ann", *source_span, &[expr_id(&inner)]);
            let mut hasher = Hasher::new();
            hasher.write_tag("ann");
            hasher.write_str(ty_repr);
            hasher.write_u64(expr_structural_hash(&inner).0);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Ann {
                id,
                structural_hash,
                source_span: *source_span,
                expr: inner,
                ty_repr: ty_repr.clone(),
            }
        }
        hir::Expr::Error { source_span, .. } => error_expr(*source_span),
    }
}

fn lower_fn_value(source_span: Span, params: &[ParamDecl], body: Expr) -> Expr {
    let mut lowered = body;
    for param in params.iter().rev() {
        lowered = lambda_expr(param.source_span, &param.name, lowered);
    }
    thunk_expr(source_span, lowered)
}

fn thunk_expr(source_span: Span, expr: Expr) -> Expr {
    let expr = Box::new(expr);
    let id = source_expr_id("thunk", source_span, &[expr_id(&expr)]);
    let mut hasher = Hasher::new();
    hasher.write_tag("thunk");
    hasher.write_u64(expr_structural_hash(&expr).0);
    let structural_hash = ContentHash(hasher.finish());
    Expr::Thunk {
        id,
        structural_hash,
        source_span,
        expr,
    }
}

fn force_expr(source_span: Span, expr: Expr) -> Expr {
    let expr = Box::new(expr);
    let id = source_expr_id("force", source_span, &[expr_id(&expr)]);
    let mut hasher = Hasher::new();
    hasher.write_tag("force");
    hasher.write_u64(expr_structural_hash(&expr).0);
    let structural_hash = ContentHash(hasher.finish());
    Expr::Force {
        id,
        structural_hash,
        source_span,
        expr,
    }
}

fn lambda_expr(source_span: Span, param: &str, body: Expr) -> Expr {
    let body = Box::new(body);
    let id = source_expr_id("lambda", source_span, &[expr_id(&body)]);
    let mut hasher = Hasher::new();
    hasher.write_tag("lambda");
    hasher.write_str(param);
    hasher.write_u64(expr_structural_hash(&body).0);
    let structural_hash = ContentHash(hasher.finish());
    Expr::Lambda {
        id,
        structural_hash,
        source_span,
        param: param.to_owned(),
        body,
    }
}

fn apply_expr(source_span: Span, callee: Expr, arg: Expr) -> Expr {
    let callee = Box::new(callee);
    let arg = Box::new(arg);
    let id = source_expr_id("apply", source_span, &[expr_id(&callee), expr_id(&arg)]);
    let mut hasher = Hasher::new();
    hasher.write_tag("apply");
    hasher.write_u64(expr_structural_hash(&callee).0);
    hasher.write_u64(expr_structural_hash(&arg).0);
    let structural_hash = ContentHash(hasher.finish());
    Expr::Apply {
        id,
        structural_hash,
        source_span,
        callee,
        arg,
    }
}

fn lower_apply_chain(source_span: Span, callee: Expr, args: Vec<Expr>) -> Expr {
    args.into_iter()
        .fold(callee, |callee, arg| apply_expr(source_span, callee, arg))
}

fn with_unroll(source_span: Span, expr: Expr) -> Expr {
    let expr = Box::new(expr);
    let id = source_expr_id("unroll", source_span, &[expr_id(&expr)]);
    let mut hasher = Hasher::new();
    hasher.write_tag("unroll");
    hasher.write_u64(expr_structural_hash(&expr).0);
    let structural_hash = ContentHash(hasher.finish());
    Expr::Unroll {
        id,
        structural_hash,
        source_span,
        expr,
    }
}

// Data constructor bundles return already-rolled recursive data values.
// We inline that constructor body in LIR, so using `T.v(...)` lowers to
// `Roll(Ctor("T.v", ...))` rather than exposing a user-level `roll`.
fn with_roll(source_span: Span, expr: Expr) -> Expr {
    let expr = Box::new(expr);
    let id = source_expr_id("roll", source_span, &[expr_id(&expr)]);
    let mut hasher = Hasher::new();
    hasher.write_tag("roll");
    hasher.write_u64(expr_structural_hash(&expr).0);
    let structural_hash = ContentHash(hasher.finish());
    Expr::Roll {
        id,
        structural_hash,
        source_span,
        expr,
    }
}

fn ctor_expr(source_span: Span, name: &str, called: bool, args: Vec<Expr>) -> Expr {
    let arg_ids = args.iter().map(expr_id).collect::<Vec<_>>();
    let id = source_expr_id("ctor", source_span, &arg_ids);
    let mut hasher = Hasher::new();
    hasher.write_tag("ctor");
    hasher.write_str(name);
    for arg in &args {
        hasher.write_u64(expr_structural_hash(arg).0);
    }
    let structural_hash = ContentHash(hasher.finish());
    Expr::Ctor {
        id,
        structural_hash,
        source_span,
        name: name.to_owned(),
        called,
        args,
    }
}

fn error_expr(source_span: Span) -> Expr {
    let id = source_id("error", source_span);
    let mut hasher = Hasher::new();
    hasher.write_tag("error");
    let structural_hash = ContentHash(hasher.finish());
    Expr::Error {
        id,
        structural_hash,
        source_span,
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

fn item_structural_hash(item: &Item) -> ContentHash {
    match item {
        Item::ExternType(ext) => ext.structural_hash,
        Item::ExternFn(ext) => ext.structural_hash,
        Item::Data(d) => d.structural_hash,
        Item::Effect(e) => e.structural_hash,
        Item::Fn(f) => f.structural_hash,
    }
}

fn expr_id(expr: &Expr) -> ContentHash {
    match expr {
        Expr::Ident { id, .. } => *id,
        Expr::String { id, .. } => *id,
        Expr::Produce { id, .. } => *id,
        Expr::Thunk { id, .. } => *id,
        Expr::Force { id, .. } => *id,
        Expr::Lambda { id, .. } => *id,
        Expr::Apply { id, .. } => *id,
        Expr::Unroll { id, .. } => *id,
        Expr::LetIn { id, .. } => *id,
        Expr::Match { id, .. } => *id,
        Expr::Ctor { id, .. } => *id,
        Expr::Roll { id, .. } => *id,
        Expr::Perform { id, .. } => *id,
        Expr::Handle { id, .. } => *id,
        Expr::Bundle { id, .. } => *id,
        Expr::Member { id, .. } => *id,
        Expr::Ann { id, .. } => *id,
        Expr::Error { id, .. } => *id,
    }
}

fn expr_structural_hash(expr: &Expr) -> ContentHash {
    match expr {
        Expr::Ident {
            structural_hash, ..
        } => *structural_hash,
        Expr::String {
            structural_hash, ..
        } => *structural_hash,
        Expr::Produce {
            structural_hash, ..
        } => *structural_hash,
        Expr::Thunk {
            structural_hash, ..
        } => *structural_hash,
        Expr::Force {
            structural_hash, ..
        } => *structural_hash,
        Expr::Lambda {
            structural_hash, ..
        } => *structural_hash,
        Expr::Apply {
            structural_hash, ..
        } => *structural_hash,
        Expr::Unroll {
            structural_hash, ..
        } => *structural_hash,
        Expr::LetIn {
            structural_hash, ..
        } => *structural_hash,
        Expr::Match {
            structural_hash, ..
        } => *structural_hash,
        Expr::Ctor {
            structural_hash, ..
        } => *structural_hash,
        Expr::Roll {
            structural_hash, ..
        } => *structural_hash,
        Expr::Perform {
            structural_hash, ..
        } => *structural_hash,
        Expr::Handle {
            structural_hash, ..
        } => *structural_hash,
        Expr::Bundle {
            structural_hash, ..
        } => *structural_hash,
        Expr::Member {
            structural_hash, ..
        } => *structural_hash,
        Expr::Ann {
            structural_hash, ..
        } => *structural_hash,
        Expr::Error {
            structural_hash, ..
        } => *structural_hash,
    }
}

pub(crate) fn source_id(tag: &str, span: Span) -> ContentHash {
    let mut hasher = Hasher::new();
    hasher.write_tag(tag);
    hasher.write_u64(span.start as u64);
    hasher.write_u64(span.end as u64);
    ContentHash(hasher.finish())
}

pub(crate) fn source_expr_id(tag: &str, span: Span, children: &[ContentHash]) -> ContentHash {
    let mut hasher = Hasher::new();
    hasher.write_tag(tag);
    hasher.write_u64(span.start as u64);
    hasher.write_u64(span.end as u64);
    for child in children {
        hasher.write_u64(child.0);
    }
    ContentHash(hasher.finish())
}

struct Hasher {
    state: u64,
}

impl Hasher {
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
