use derive_more::Debug;

use crate::{lexer::Span, lst, parser};

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
    pub body: Expr,
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
    Member {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        object: Box<Expr>,
        member: String,
    },
    Call {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        #[debug(skip)]
        source_span: Span,
        callee: Box<Expr>,
        args: Vec<Expr>,
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
        entries: Vec<HirBundleEntry>,
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
pub struct HirBundleEntry {
    #[debug(skip)]
    pub source_span: Span,
    pub name: String,
    pub params: Vec<ParamDecl>,
    pub body: Expr,
}

pub fn lower_lossless(parsed: &crate::lst::lossless::ParseOutput) -> File {
    let parsed = parser::parse_lossless(parsed);
    lower(&parsed.file)
}

pub fn lower(file: &lst::File) -> File {
    let items = file
        .items
        .iter()
        .map(|item| match item {
            lst::Item::ExternType(ext) => Item::ExternType(lower_extern_type(ext)),
            lst::Item::ExternFn(ext) => Item::ExternFn(lower_extern_fn(ext)),
            lst::Item::Data(data) => Item::Data(lower_data(data)),
            lst::Item::Effect(effect) => Item::Effect(lower_effect(effect)),
            lst::Item::Fn(func) => Item::Fn(lower_fn(func)),
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

fn lower_effect(effect: &lst::EffectDecl) -> EffectDecl {
    let operations = effect
        .operations
        .iter()
        .map(|op| {
            let params = op
                .params
                .iter()
                .map(|param| ParamDecl {
                    source_span: param.span,
                    name: param.name.clone(),
                    ty_repr: param.ty.repr.trim().to_owned(),
                })
                .collect::<Vec<_>>();
            let return_type_repr = op.return_type.as_ref().map(|ty| ty.repr.trim().to_owned());
            let return_type_span = op.return_type.as_ref().map(|ty| ty.span);
            let id = source_id("operation", op.span);
            let mut hasher = Hasher::new();
            hasher.write_tag("operation");
            hasher.write_str(&op.name);
            for param in &params {
                hasher.write_str(&param.name);
                hasher.write_str(&param.ty_repr);
            }
            if let Some(ret) = &return_type_repr {
                hasher.write_str(ret);
            }
            let structural_hash = ContentHash(hasher.finish());
            OperationDecl {
                id,
                structural_hash,
                source_span: op.span,
                name: op.name.clone(),
                params,
                return_type_repr,
                return_type_span,
            }
        })
        .collect::<Vec<_>>();

    let id = source_id("effect", effect.span);
    let mut hasher = Hasher::new();
    hasher.write_tag("effect");
    hasher.write_str(&effect.name);
    for op in &operations {
        hasher.write_u64(op.structural_hash.0);
    }
    let structural_hash = ContentHash(hasher.finish());

    EffectDecl {
        name: effect.name.clone(),
        id,
        structural_hash,
        source_span: effect.span,
        operations,
    }
}

fn lower_data(data: &lst::DataDecl) -> DataDecl {
    let variants = data.variants.iter().map(lower_variant).collect::<Vec<_>>();

    let id = source_id("data", data.span);
    let mut hasher = Hasher::new();
    hasher.write_tag("data");
    for variant in &variants {
        hasher.write_u64(variant.structural_hash.0);
    }
    let structural_hash = ContentHash(hasher.finish());

    DataDecl {
        name: data.name.clone(),
        generics: data.generics.iter().map(|g| g.name.clone()).collect(),
        id,
        structural_hash,
        source_span: data.span,
        variants,
    }
}

fn lower_extern_type(ext: &lst::ExternTypeDecl) -> ExternTypeDecl {
    let extern_name = find_extern_name(&ext.attrs);
    let id = source_id("extern-type", ext.span);
    let mut hasher = Hasher::new();
    hasher.write_tag("extern-type");
    hasher.write_str(&ext.name);
    if let Some(name) = &extern_name {
        hasher.write_str(name);
    }
    let structural_hash = ContentHash(hasher.finish());
    ExternTypeDecl {
        id,
        structural_hash,
        source_span: ext.span,
        name: ext.name.clone(),
        extern_name,
    }
}

fn lower_extern_fn(ext: &lst::ExternFnDecl) -> ExternFnDecl {
    let extern_name = find_extern_name(&ext.attrs);
    let params = ext
        .params
        .iter()
        .map(|param| ParamDecl {
            source_span: param.span,
            name: param.name.clone(),
            ty_repr: param.ty.repr.trim().to_owned(),
        })
        .collect::<Vec<_>>();
    let return_type_repr = ext.return_type.as_ref().map(|ty| ty.repr.trim().to_owned());
    let return_type_span = ext.return_type.as_ref().map(|ty| ty.span);
    let effect_repr = ext
        .effect
        .as_ref()
        .map(|effect| effect.repr.trim().to_owned());
    let effect_span = ext.effect.as_ref().map(|effect| effect.span);
    let id = source_id("extern-fn", ext.span);
    let mut hasher = Hasher::new();
    hasher.write_tag("extern-fn");
    hasher.write_str(&ext.name);
    if let Some(name) = &extern_name {
        hasher.write_str(name);
    }
    for param in &params {
        hasher.write_str(&param.name);
        hasher.write_str(&param.ty_repr);
    }
    if let Some(ret) = &return_type_repr {
        hasher.write_str(ret);
    }
    if let Some(effect) = &effect_repr {
        hasher.write_str(effect);
    }
    let structural_hash = ContentHash(hasher.finish());
    ExternFnDecl {
        id,
        structural_hash,
        source_span: ext.span,
        name: ext.name.clone(),
        extern_name,
        params,
        return_type_repr,
        return_type_span,
        effect_repr,
        effect_span,
    }
}

fn lower_variant(variant: &lst::VariantDecl) -> VariantDecl {
    let id = source_id("variant", variant.span);
    let mut hasher = Hasher::new();
    hasher.write_tag("variant");
    hasher.write_str(&variant.name);
    let structural_hash = ContentHash(hasher.finish());

    VariantDecl {
        id,
        structural_hash,
        source_span: variant.span,
        name: variant.name.clone(),
        payload_types: variant
            .payload
            .iter()
            .map(|ty| ty.repr.trim().to_owned())
            .collect(),
        payload_spans: variant.payload.iter().map(|ty| ty.span).collect(),
    }
}

fn lower_fn(func: &lst::FnDecl) -> FnDecl {
    let body = lower_expr(&func.body);
    let params = func
        .params
        .iter()
        .map(|param| ParamDecl {
            source_span: param.span,
            name: param.name.clone(),
            ty_repr: param.ty.repr.trim().to_owned(),
        })
        .collect::<Vec<_>>();
    let return_type_repr = func
        .return_type
        .as_ref()
        .map(|ty| ty.repr.trim().to_owned());
    let effect_repr = func
        .effect
        .as_ref()
        .map(|effect| effect.repr.trim().to_owned());
    let return_type_span = func.return_type.as_ref().map(|ty| ty.span);
    let effect_span = func.effect.as_ref().map(|effect| effect.span);

    let id = source_id("fn", func.span);
    let mut hasher = Hasher::new();
    hasher.write_tag("fn");
    hasher.write_u64(expr_structural_hash(&body).0);
    let structural_hash = ContentHash(hasher.finish());

    FnDecl {
        name: func.name.clone(),
        generics: func.generics.iter().map(|g| g.name.clone()).collect(),
        id,
        structural_hash,
        source_span: func.span,
        params,
        return_type_repr,
        return_type_span,
        effect_repr,
        effect_span,
        body,
    }
}

fn lower_expr(expr: &lst::Expr) -> Expr {
    match expr {
        lst::Expr::Ident { name, .. } => {
            let span = expr_source_span(expr);
            let id = source_id("ident", span);
            let mut hasher = Hasher::new();
            hasher.write_tag("ident");
            hasher.write_str(name);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Ident {
                id,
                structural_hash,
                source_span: span,
                name: name.clone(),
            }
        }
        lst::Expr::String { value, .. } => {
            let span = expr_source_span(expr);
            let id = source_id("string", span);
            let mut hasher = Hasher::new();
            hasher.write_tag("string");
            hasher.write_str(value);
            let structural_hash = ContentHash(hasher.finish());
            Expr::String {
                id,
                structural_hash,
                source_span: span,
                value: value.clone(),
            }
        }
        lst::Expr::Member { object, member, .. } => {
            let span = expr_source_span(expr);
            let object = Box::new(lower_expr(object));
            let id = source_expr_id("member", span, &[expr_id(&object)]);
            let mut hasher = Hasher::new();
            hasher.write_tag("member");
            hasher.write_u64(expr_structural_hash(&object).0);
            hasher.write_str(member);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Member {
                id,
                structural_hash,
                source_span: span,
                object,
                member: member.clone(),
            }
        }
        lst::Expr::Call { callee, args, .. } => {
            let span = expr_source_span(expr);
            let callee = Box::new(lower_expr(callee));
            let args = args.iter().map(lower_expr).collect::<Vec<_>>();
            let mut source_children = vec![expr_id(&callee)];
            source_children.extend(args.iter().map(expr_id));
            let id = source_expr_id("call", span, &source_children);
            let mut hasher = Hasher::new();
            hasher.write_tag("call");
            hasher.write_u64(expr_structural_hash(&callee).0);
            for arg in &args {
                hasher.write_u64(expr_structural_hash(arg).0);
            }
            let structural_hash = ContentHash(hasher.finish());
            Expr::Call {
                id,
                structural_hash,
                source_span: span,
                callee,
                args,
            }
        }
        lst::Expr::Produce { expr: inner, .. } => {
            let span = expr_source_span(expr);
            let expr = Box::new(lower_expr(inner));
            let id = source_expr_id("produce", span, &[expr_id(&expr)]);
            let mut hasher = Hasher::new();
            hasher.write_tag("produce");
            hasher.write_u64(expr_structural_hash(&expr).0);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Produce {
                id,
                structural_hash,
                source_span: span,
                expr,
            }
        }
        lst::Expr::Thunk { expr: inner, .. } => {
            let span = expr_source_span(expr);
            let expr = Box::new(lower_expr(inner));
            let id = source_expr_id("thunk", span, &[expr_id(&expr)]);
            let mut hasher = Hasher::new();
            hasher.write_tag("thunk");
            hasher.write_u64(expr_structural_hash(&expr).0);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Thunk {
                id,
                structural_hash,
                source_span: span,
                expr,
            }
        }
        lst::Expr::Force { expr: inner, .. } => {
            let span = expr_source_span(expr);
            let expr = Box::new(lower_expr(inner));
            let id = source_expr_id("force", span, &[expr_id(&expr)]);
            let mut hasher = Hasher::new();
            hasher.write_tag("force");
            hasher.write_u64(expr_structural_hash(&expr).0);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Force {
                id,
                structural_hash,
                source_span: span,
                expr,
            }
        }
        lst::Expr::LetIn {
            name, value, body, ..
        } => {
            let span = expr_source_span(expr);
            let value = Box::new(lower_expr(value));
            let body = Box::new(lower_expr(body));
            let id = source_expr_id("let-in", span, &[expr_id(&value), expr_id(&body)]);
            let mut hasher = Hasher::new();
            hasher.write_tag("let-in");
            hasher.write_str(name);
            hasher.write_u64(expr_structural_hash(&value).0);
            hasher.write_u64(expr_structural_hash(&body).0);
            let structural_hash = ContentHash(hasher.finish());
            Expr::LetIn {
                id,
                structural_hash,
                source_span: span,
                name: name.clone(),
                value,
                body,
            }
        }
        lst::Expr::Match {
            scrutinee, arms, ..
        } => {
            let span = expr_source_span(expr);
            let scrutinee = Box::new(lower_expr(scrutinee));
            let arms = arms
                .iter()
                .map(|arm| MatchArm {
                    source_span: arm.span,
                    pattern: arm.pattern.clone(),
                    body: lower_expr(&arm.body),
                })
                .collect::<Vec<_>>();
            let mut source_children = vec![expr_id(&scrutinee)];
            source_children.extend(arms.iter().map(|arm| expr_id(&arm.body)));
            let id = source_expr_id("match", span, &source_children);
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
                source_span: span,
                scrutinee,
                arms,
            }
        }
        lst::Expr::Perform { effect, .. } => {
            let span = expr_source_span(expr);
            let id = source_id("perform", span);
            let mut hasher = Hasher::new();
            hasher.write_tag("perform");
            hasher.write_str(effect);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Perform {
                id,
                structural_hash,
                source_span: span,
                effect: effect.clone(),
            }
        }
        lst::Expr::Handle {
            effect, handler, body, ..
        } => {
            let span = expr_source_span(expr);
            let handler = Box::new(lower_expr(handler));
            let body = Box::new(lower_expr(body));
            let id =
                source_expr_id("handle", span, &[expr_id(&handler), expr_id(&body)]);
            let mut hasher = Hasher::new();
            hasher.write_tag("handle");
            hasher.write_str(effect);
            hasher.write_u64(expr_structural_hash(&handler).0);
            hasher.write_u64(expr_structural_hash(&body).0);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Handle {
                id,
                structural_hash,
                source_span: span,
                effect: effect.clone(),
                handler,
                body,
            }
        }
        lst::Expr::Bundle { entries, .. } => {
            let span = expr_source_span(expr);
            let hir_entries = entries
                .iter()
                .map(|e| {
                    let params = e
                        .params
                        .iter()
                        .map(|p| ParamDecl {
                            source_span: p.span,
                            name: p.name.clone(),
                            ty_repr: p.ty.repr.trim().to_owned(),
                        })
                        .collect::<Vec<_>>();
                    HirBundleEntry {
                        source_span: e.span,
                        name: e.name.clone(),
                        params,
                        body: lower_expr(&e.body),
                    }
                })
                .collect::<Vec<_>>();
            let source_children: Vec<ContentHash> =
                hir_entries.iter().map(|e| expr_id(&e.body)).collect();
            let id = source_expr_id("bundle", span, &source_children);
            let mut hasher = Hasher::new();
            hasher.write_tag("bundle");
            for e in &hir_entries {
                hasher.write_str(&e.name);
                hasher.write_u64(expr_structural_hash(&e.body).0);
            }
            let structural_hash = ContentHash(hasher.finish());
            Expr::Bundle {
                id,
                structural_hash,
                source_span: span,
                entries: hir_entries,
            }
        }
        lst::Expr::Ann { expr: inner, ty, .. } => {
            let span = expr_source_span(expr);
            let inner = Box::new(lower_expr(inner));
            let id = source_expr_id("ann", span, &[expr_id(&inner)]);
            let mut hasher = Hasher::new();
            hasher.write_tag("ann");
            hasher.write_str(&ty.repr);
            hasher.write_u64(expr_structural_hash(&inner).0);
            let structural_hash = ContentHash(hasher.finish());
            Expr::Ann {
                id,
                structural_hash,
                source_span: span,
                expr: inner,
                ty_repr: ty.repr.trim().to_owned(),
            }
        }
        lst::Expr::Error { span } => error_expr(*span),
    }
}

fn find_extern_name(attrs: &[lst::Attribute]) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.name == "extern")
        .and_then(|attr| {
            if let Some(value) = &attr.value {
                if let Some(text) = expr_string_literal(value) {
                    return Some(text.to_owned());
                }
            }
            attr.args
                .iter()
                .find(|arg| arg.key == "name")
                .and_then(|arg| expr_string_literal(&arg.value))
                .map(ToOwned::to_owned)
        })
}

fn expr_string_literal(expr: &lst::Expr) -> Option<&str> {
    match expr {
        lst::Expr::String { value, .. } => Some(value.as_str()),
        _ => None,
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
        Expr::Member { id, .. } => *id,
        Expr::Call { id, .. } => *id,
        Expr::Produce { id, .. } => *id,
        Expr::Thunk { id, .. } => *id,
        Expr::Force { id, .. } => *id,
        Expr::LetIn { id, .. } => *id,
        Expr::Match { id, .. } => *id,
        Expr::Perform { id, .. } => *id,
        Expr::Handle { id, .. } => *id,
        Expr::Bundle { id, .. } => *id,
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
        Expr::Member {
            structural_hash, ..
        } => *structural_hash,
        Expr::Call {
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
        Expr::LetIn {
            structural_hash, ..
        } => *structural_hash,
        Expr::Match {
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
        Expr::Ann {
            structural_hash, ..
        } => *structural_hash,
        Expr::Error {
            structural_hash, ..
        } => *structural_hash,
    }
}

fn expr_source_span(expr: &lst::Expr) -> Span {
    match expr {
        lst::Expr::Ident { span, .. }
        | lst::Expr::String { span, .. }
        | lst::Expr::Member { span, .. }
        | lst::Expr::Call { span, .. }
        | lst::Expr::Produce { span, .. }
        | lst::Expr::Thunk { span, .. }
        | lst::Expr::Force { span, .. }
        | lst::Expr::LetIn { span, .. }
        | lst::Expr::Match { span, .. }
        | lst::Expr::Perform { span, .. }
        | lst::Expr::Handle { span, .. }
        | lst::Expr::Bundle { span, .. }
        | lst::Expr::Ann { span, .. }
        | lst::Expr::Error { span } => *span,
    }
}

fn source_id(tag: &str, span: Span) -> ContentHash {
    let mut hasher = Hasher::new();
    hasher.write_tag(tag);
    hasher.write_u64(span.start as u64);
    hasher.write_u64(span.end as u64);
    ContentHash(hasher.finish())
}

fn source_expr_id(tag: &str, span: Span, children: &[ContentHash]) -> ContentHash {
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
