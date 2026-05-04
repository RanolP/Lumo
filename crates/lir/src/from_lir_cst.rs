use lumo_span::Span;
use lumo_types::{CapEntry, CapRef, ContentHash, ExprId, Pattern, Spanned, TypeExpr};

use crate::{
    ast::{self, AstNode},
    lossless::{self, SyntaxElement, SyntaxNode},
    syntax_kind::SyntaxKind,
    BundleEntry, CapDecl, DataDecl, ExternFnDecl, ExternTypeDecl, Expr, File, FnDecl,
    GenericParam, ImplDecl, ImplMethodDecl, Item, MatchArm, OperationDecl, Param, UseDecl,
    VariantDecl,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn parse(source: &str) -> Result<File, Vec<ParseError>> {
    let output = lossless::parse(source);

    let mut errors: Vec<ParseError> = output
        .errors
        .iter()
        .map(|e| ParseError {
            span: e.span,
            message: e.message.clone(),
        })
        .collect();

    let file_node = &output.root;
    let ast_file = match ast::File::cast(file_node) {
        Some(f) => f,
        None => {
            errors.push(ParseError {
                span: Span::new(0, 0),
                message: "expected FILE node at root".into(),
            });
            return Err(errors);
        }
    };

    let mut items = Vec::new();
    let mut spans: Vec<Span> = Vec::new();
    let mut lowering_errors: Vec<ParseError> = Vec::new();

    for ast_item in ast_file.items() {
        match lower_item(&ast_item, &mut spans, &mut lowering_errors) {
            Some(item) => items.push(item),
            None => {}
        }
    }

    errors.extend(lowering_errors);

    if errors.is_empty() {
        Ok(File {
            items,
            content_hash: ContentHash(0),
            spans,
        })
    } else {
        Err(errors)
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn find_child_node<'a>(node: &'a SyntaxNode, kind: SyntaxKind) -> Option<&'a SyntaxNode> {
    node.children.iter().find_map(|c| match c {
        SyntaxElement::Node(n) if n.kind == kind => Some(n.as_ref()),
        _ => None,
    })
}

fn child_nodes<'a>(
    node: &'a SyntaxNode,
    kind: SyntaxKind,
) -> impl Iterator<Item = &'a SyntaxNode> + 'a {
    node.children.iter().filter_map(move |c| match c {
        SyntaxElement::Node(n) if n.kind == kind => Some(n.as_ref()),
        _ => None,
    })
}

fn child_tokens_of_kind<'a>(
    node: &'a SyntaxNode,
    kind: SyntaxKind,
) -> impl Iterator<Item = &'a lossless::LosslessToken> + 'a {
    node.children.iter().filter_map(move |c| match c {
        SyntaxElement::Token(t) if t.kind == kind => Some(t),
        _ => None,
    })
}

fn node_span(node: &SyntaxNode) -> Span {
    Span::new(node.span.start, node.span.end)
}

fn strip_string_quotes(s: &str) -> String {
    let inner = s
        .strip_prefix('"')
        .unwrap_or(s)
        .strip_suffix('"')
        .unwrap_or(s);
    unescape_string(inner)
}

fn unescape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(c) => {
                    out.push('\\');
                    out.push(c);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn error_span(errors: &mut Vec<ParseError>, span: Span, msg: &str) {
    errors.push(ParseError {
        span,
        message: msg.into(),
    });
}

fn alloc_span(spans: &mut Vec<Span>, span: Span) -> ExprId {
    let id = ExprId(spans.len() as u32);
    spans.push(span);
    id
}

// ---------------------------------------------------------------------------
// Items
// ---------------------------------------------------------------------------

fn lower_item(
    item: &ast::Item,
    spans: &mut Vec<Span>,
    errors: &mut Vec<ParseError>,
) -> Option<Item> {
    match item {
        ast::Item::ExternTypeDecl(n) => lower_extern_type(n, errors).map(Item::ExternType),
        ast::Item::ExternFnDecl(n) => lower_extern_fn(n, errors).map(Item::ExternFn),
        ast::Item::DataDecl(n) => lower_data_decl(n, errors).map(Item::Data),
        ast::Item::CapDecl(n) => lower_cap_decl(n, errors).map(Item::Cap),
        ast::Item::FnDecl(n) => lower_fn_decl(n, spans, errors).map(Item::Fn),
        ast::Item::UseDecl(n) => lower_use_decl(n, errors).map(Item::Use),
        ast::Item::ImplDecl(n) => lower_impl_decl(n, spans, errors).map(Item::Impl),
    }
}

fn lower_extern_type(
    node: &ast::ExternTypeDecl,
    errors: &mut Vec<ParseError>,
) -> Option<ExternTypeDecl> {
    let span = node_span(node.syntax());
    let name_tok = node.name().or_else(|| {
        error_span(errors, span, "extern type: missing name");
        None
    })?;
    let name = name_tok.text.clone();
    let extern_name = node
        .extern_as()
        .and_then(|ea| ea.value())
        .map(|t| strip_string_quotes(&t.text));
    Some(ExternTypeDecl {
        name,
        extern_name,
        span,
    })
}

fn lower_extern_fn(
    node: &ast::ExternFnDecl,
    errors: &mut Vec<ParseError>,
) -> Option<ExternFnDecl> {
    let span = node_span(node.syntax());
    let inline = node.inline().is_some();
    let name_tok = node.name().or_else(|| {
        error_span(errors, span, "extern fn: missing name");
        None
    })?;
    let name = name_tok.text.clone();
    let params = lower_param_list_opt(node.param_list(), errors);
    let return_type = node
        .return_ann()
        .and_then(|ra| ra.ty())
        .and_then(|ty| lower_type_expr(&ty, errors));
    let cap = node
        .cap_annotation()
        .and_then(|ca| lower_cap_annotation(ca.syntax(), errors));
    let extern_name = node
        .extern_as()
        .and_then(|ea| ea.value())
        .map(|t| strip_string_quotes(&t.text));
    Some(ExternFnDecl {
        name,
        extern_name,
        link_module: None,
        inline,
        params,
        return_type,
        cap,
        span,
    })
}

fn lower_data_decl(node: &ast::DataDecl, errors: &mut Vec<ParseError>) -> Option<DataDecl> {
    let span = node_span(node.syntax());
    let name_tok = node.name().or_else(|| {
        error_span(errors, span, "data decl: missing name");
        None
    })?;
    let name = name_tok.text.clone();
    let generics = lower_generic_params_to_strings(node.generic_params());
    let variants = lower_variant_items(node.variants(), errors);
    Some(DataDecl {
        name,
        generics,
        variants,
        span,
    })
}

fn lower_generic_params_to_strings(gp: Option<ast::GenericParams>) -> Vec<String> {
    let gp = match gp {
        Some(g) => g,
        None => return vec![],
    };
    let items_node = match find_child_node(gp.syntax(), SyntaxKind::GENERIC_PARAM_ITEMS) {
        Some(n) => n,
        None => return vec![],
    };
    child_nodes(items_node, SyntaxKind::GENERIC_PARAM)
        .filter_map(|param_node| {
            let param = ast::GenericParam::cast(param_node)?;
            Some(param.name()?.text.clone())
        })
        .collect()
}

fn lower_generic_params(gp: Option<ast::GenericParams>) -> Vec<GenericParam> {
    let gp = match gp {
        Some(g) => g,
        None => return vec![],
    };
    let items_node = match find_child_node(gp.syntax(), SyntaxKind::GENERIC_PARAM_ITEMS) {
        Some(n) => n,
        None => return vec![],
    };
    child_nodes(items_node, SyntaxKind::GENERIC_PARAM)
        .filter_map(|param_node| {
            let param = ast::GenericParam::cast(param_node)?;
            let name = param.name()?.text.clone();
            // Check if there's a CAP_KW token child → CapRow
            let is_cap_row = param_node
                .children
                .iter()
                .any(|c| matches!(c, SyntaxElement::Token(t) if t.kind == SyntaxKind::CAP_KW));
            if is_cap_row {
                Some(GenericParam::CapRow(name))
            } else {
                // Check for BOUND_LIST
                let bounds = match find_child_node(param_node, SyntaxKind::BOUND_LIST) {
                    Some(bl) => child_tokens_of_kind(bl, SyntaxKind::IDENT)
                        .map(|t| t.text.clone())
                        .collect(),
                    None => vec![],
                };
                Some(GenericParam::Type(name, bounds))
            }
        })
        .collect()
}

fn lower_variant_items(
    vi: Option<ast::VariantItems>,
    errors: &mut Vec<ParseError>,
) -> Vec<VariantDecl> {
    let vi = match vi {
        Some(v) => v,
        None => return vec![],
    };
    child_nodes(vi.syntax(), SyntaxKind::VARIANT)
        .filter_map(|vn| {
            let variant = ast::Variant::cast(vn)?;
            lower_variant(&variant, errors)
        })
        .collect()
}

fn lower_variant(node: &ast::Variant, errors: &mut Vec<ParseError>) -> Option<VariantDecl> {
    let span = node_span(node.syntax());
    let name = node.name()?.text.clone();
    let payload = match node.variant_fields() {
        Some(vf) => {
            match find_child_node(vf.syntax(), SyntaxKind::VARIANT_FIELD_ITEMS) {
                Some(items_node) => child_nodes(items_node, SyntaxKind::TYPE_EXPR)
                    .filter_map(|tn| {
                        let ty = ast::TypeExpr::cast(tn)?;
                        lower_type_expr(&ty, errors)
                    })
                    .collect(),
                None => vec![],
            }
        }
        None => vec![],
    };
    Some(VariantDecl {
        name,
        payload,
        as_raw: None,
        span,
    })
}

fn lower_cap_decl(node: &ast::CapDecl, errors: &mut Vec<ParseError>) -> Option<CapDecl> {
    let span = node_span(node.syntax());
    let name_tok = node.name().or_else(|| {
        error_span(errors, span, "cap decl: missing name");
        None
    })?;
    let name = name_tok.text.clone();
    let operations = node
        .operations()
        .filter_map(|op| lower_operation_decl(&op, errors))
        .collect();
    Some(CapDecl {
        name,
        operations,
        span,
    })
}

fn lower_operation_decl(
    node: &ast::OperationDecl,
    errors: &mut Vec<ParseError>,
) -> Option<OperationDecl> {
    let span = node_span(node.syntax());
    let name = node.name()?.text.clone();
    let params = lower_param_list_opt(node.param_list(), errors);
    let return_type = node
        .return_ann()
        .and_then(|ra| ra.ty())
        .and_then(|ty| lower_type_expr(&ty, errors));
    Some(OperationDecl {
        name,
        params,
        return_type,
        span,
    })
}

fn lower_fn_decl(
    node: &ast::FnDecl,
    spans: &mut Vec<Span>,
    errors: &mut Vec<ParseError>,
) -> Option<FnDecl> {
    let span = node_span(node.syntax());
    let name_tok = node.name().or_else(|| {
        error_span(errors, span, "fn decl: missing name");
        None
    })?;
    let name = name_tok.text.clone();
    let generics = lower_generic_params(node.generic_params());
    let params = lower_param_list_opt(node.param_list(), errors);
    let return_type = node
        .return_ann()
        .and_then(|ra| ra.ty())
        .and_then(|ty| lower_type_expr(&ty, errors));
    let cap = node
        .cap_annotation()
        .and_then(|ca| lower_cap_annotation(ca.syntax(), errors));
    let value = node.value().and_then(|e| lower_expr(&e, spans, errors)).or_else(|| {
        error_span(errors, span, "fn decl: missing value");
        None
    })?;
    Some(FnDecl {
        name,
        generics,
        params,
        return_type,
        cap,
        value,
        inline: false,
        span,
    })
}

fn lower_use_decl(node: &ast::UseDecl, errors: &mut Vec<ParseError>) -> Option<UseDecl> {
    let span = node_span(node.syntax());
    let head = node.head().or_else(|| {
        error_span(errors, span, "use decl: missing head");
        None
    })?;
    let mut path = vec![head.text.clone()];
    for seg in node.rest() {
        if let Some(tok) = seg.seg() {
            path.push(tok.text.clone());
        }
    }
    let names = node.tree().and_then(|tree| {
        let names_node = tree.names()?;
        let names: Vec<String> = child_tokens_of_kind(names_node.syntax(), SyntaxKind::IDENT)
            .map(|t| t.text.clone())
            .collect();
        Some(names)
    });
    Some(UseDecl { path, names, span })
}

fn lower_impl_decl(
    node: &ast::ImplDecl,
    spans: &mut Vec<Span>,
    errors: &mut Vec<ParseError>,
) -> Option<ImplDecl> {
    let span = node_span(node.syntax());
    let generics = lower_generic_params(node.generic_params());

    // The generated parser greedily parses ImplName? even when there's no '='.
    // For `impl Number: Add { ... }` it consumes `Number` as ImplName (without the `=`).
    // Detect this: if ImplName has no EQUALS token, the ident is really the target type.
    let (name, target_type) = match node.impl_name() {
        Some(impl_name_node) => {
            let has_equals = impl_name_node
                .syntax()
                .children
                .iter()
                .any(|c| matches!(c, SyntaxElement::Token(t) if t.kind == SyntaxKind::EQUALS));
            if has_equals {
                // Real named impl: `impl Name = Target: Cap { ... }`
                let name = impl_name_node.name().map(|t| t.text.clone());
                let target_type = node
                    .target()
                    .and_then(|ty| lower_type_expr(&ty, errors))
                    .or_else(|| {
                        error_span(errors, span, "impl decl: missing target type");
                        None
                    })?;
                (name, target_type)
            } else {
                // The greedy parse consumed the target's ident as ImplName — recover it
                let target_name = impl_name_node.name()?.text.clone();
                let target_span = node_span(impl_name_node.syntax());
                let target_type = Spanned {
                    value: TypeExpr::Named(target_name),
                    span: target_span,
                };
                (None, target_type)
            }
        }
        None => {
            let target_type = node
                .target()
                .and_then(|ty| lower_type_expr(&ty, errors))
                .or_else(|| {
                    error_span(errors, span, "impl decl: missing target type");
                    None
                })?;
            (None, target_type)
        }
    };

    let capability = node
        .impl_cap()
        .and_then(|ic| ic.cap())
        .and_then(|ty| lower_type_expr(&ty, errors));
    let methods = node
        .methods()
        .filter_map(|m| lower_impl_method(&m, spans, errors))
        .collect();
    Some(ImplDecl {
        name,
        generics,
        target_type,
        capability,
        methods,
        span,
    })
}

fn lower_impl_method(
    node: &ast::ImplMethod,
    spans: &mut Vec<Span>,
    errors: &mut Vec<ParseError>,
) -> Option<ImplMethodDecl> {
    let span = node_span(node.syntax());
    let name = node.name()?.text.clone();
    let params = lower_param_list_opt(node.param_list(), errors);
    let return_type = node
        .return_ann()
        .and_then(|ra| ra.ty())
        .and_then(|ty| lower_type_expr(&ty, errors));
    let value = node.value().and_then(|e| lower_expr(&e, spans, errors)).or_else(|| {
        error_span(errors, span, "impl method: missing value");
        None
    })?;
    Some(ImplMethodDecl {
        name,
        params,
        return_type,
        value,
        span,
    })
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn lower_param_list_opt(pl: Option<ast::ParamList>, errors: &mut Vec<ParseError>) -> Vec<Param> {
    let pl = match pl {
        Some(p) => p,
        None => return vec![],
    };
    let items_node = match find_child_node(pl.syntax(), SyntaxKind::PARAM_ITEMS) {
        Some(n) => n,
        None => return vec![],
    };
    child_nodes(items_node, SyntaxKind::PARAM)
        .filter_map(|pn| {
            let param = ast::Param::cast(pn)?;
            lower_param(&param, errors)
        })
        .collect()
}

fn lower_param(node: &ast::Param, errors: &mut Vec<ParseError>) -> Option<Param> {
    let span = node_span(node.syntax());
    let name = node.name()?.text.clone();
    let ty_ast = node.ty().or_else(|| {
        error_span(errors, span, "param: missing type");
        None
    })?;
    let ty = lower_type_expr(&ty_ast, errors)?;
    Some(Param { name, ty, span })
}

fn lower_cap_annotation(
    node: &SyntaxNode,
    errors: &mut Vec<ParseError>,
) -> Option<CapRef> {
    let items_node = find_child_node(node, SyntaxKind::CAP_ITEMS)?;
    let entries: Vec<CapEntry> = child_nodes(items_node, SyntaxKind::TYPE_EXPR)
        .filter_map(|tn| {
            let ty_ast = ast::TypeExpr::cast(tn)?;
            lower_type_expr(&ty_ast, errors).map(|t| CapEntry::Cap(t.value))
        })
        .collect();
    Some(entries)
}

fn lower_type_expr(
    node: &ast::TypeExpr,
    errors: &mut Vec<ParseError>,
) -> Option<Spanned<TypeExpr>> {
    let span = node_span(node.syntax());
    let name_tok = node.name().or_else(|| {
        error_span(errors, span, "type expr: missing name");
        None
    })?;
    let name = name_tok.text.clone();
    let value = match node.generic_args() {
        Some(ga) => {
            let args = lower_generic_args(&ga, errors);
            TypeExpr::App { head: name, args }
        }
        None => TypeExpr::Named(name),
    };
    Some(Spanned { value, span })
}

fn lower_generic_args(node: &ast::GenericArgs, errors: &mut Vec<ParseError>) -> Vec<TypeExpr> {
    let items_node = match find_child_node(node.syntax(), SyntaxKind::GENERIC_ARG_ITEMS) {
        Some(n) => n,
        None => return vec![],
    };
    child_nodes(items_node, SyntaxKind::TYPE_EXPR)
        .filter_map(|tn| {
            let ty = ast::TypeExpr::cast(tn)?;
            lower_type_expr(&ty, errors).map(|t| t.value)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

fn lower_expr(
    expr: &ast::Expr,
    spans: &mut Vec<Span>,
    errors: &mut Vec<ParseError>,
) -> Option<Expr> {
    match expr {
        ast::Expr::IdentExpr(n) => {
            let span = node_span(n.syntax());
            let name = n.name()?.text.clone();
            let id = alloc_span(spans, span);
            Some(Expr::Ident { id, name })
        }
        ast::Expr::StringExpr(n) => {
            let span = node_span(n.syntax());
            let tok = n.value()?;
            let value = strip_string_quotes(&tok.text);
            let id = alloc_span(spans, span);
            Some(Expr::String { id, value })
        }
        ast::Expr::NumberExpr(n) => {
            let span = node_span(n.syntax());
            let value = n.value()?.text.clone();
            let id = alloc_span(spans, span);
            Some(Expr::Number { id, value })
        }
        ast::Expr::ProduceExpr(n) => {
            let span = node_span(n.syntax());
            let inner = n.value().and_then(|e| lower_expr(&e, spans, errors))?;
            let id = alloc_span(spans, span);
            Some(Expr::Produce {
                id,
                expr: Box::new(inner),
            })
        }
        ast::Expr::ThunkExpr(n) => {
            let span = node_span(n.syntax());
            let inner = n.body().and_then(|e| lower_expr(&e, spans, errors))?;
            let id = alloc_span(spans, span);
            Some(Expr::Thunk {
                id,
                expr: Box::new(inner),
            })
        }
        ast::Expr::ForceExpr(n) => {
            let span = node_span(n.syntax());
            let inner = n.expr().and_then(|e| lower_expr(&e, spans, errors))?;
            let id = alloc_span(spans, span);
            Some(Expr::Force {
                id,
                expr: Box::new(inner),
            })
        }
        ast::Expr::LambdaExpr(n) => {
            let span = node_span(n.syntax());
            let param = n.param()?.text.clone();
            let body = n.body().and_then(|e| lower_expr(&e, spans, errors))?;
            let id = alloc_span(spans, span);
            Some(Expr::Lambda {
                id,
                param,
                body: Box::new(body),
            })
        }
        ast::Expr::RollExpr(n) => {
            let span = node_span(n.syntax());
            let inner = n.expr().and_then(|e| lower_expr(&e, spans, errors))?;
            let id = alloc_span(spans, span);
            Some(Expr::Roll {
                id,
                expr: Box::new(inner),
            })
        }
        ast::Expr::UnrollExpr(n) => {
            let span = node_span(n.syntax());
            let inner = n.expr().and_then(|e| lower_expr(&e, spans, errors))?;
            let id = alloc_span(spans, span);
            Some(Expr::Unroll {
                id,
                expr: Box::new(inner),
            })
        }
        ast::Expr::CtorExpr(n) => {
            let span = node_span(n.syntax());
            let name = n.name()?.text.clone();
            let ctor_args = n.ctor_args();
            let called = ctor_args.is_some();
            let args = match ctor_args {
                Some(ca) => {
                    match find_child_node(ca.syntax(), SyntaxKind::CTOR_ARG_ITEMS) {
                        Some(items_node) => items_node
                            .children
                            .iter()
                            .filter_map(|c| match c {
                                SyntaxElement::Node(nd) => {
                                    ast::Expr::cast(nd.as_ref())
                                        .and_then(|e| lower_expr(&e, spans, errors))
                                }
                                _ => None,
                            })
                            .collect(),
                        None => vec![],
                    }
                }
                None => vec![],
            };
            let id = alloc_span(spans, span);
            Some(Expr::Ctor { id, name, called, args })
        }
        ast::Expr::ApplyExpr(n) => {
            let span = node_span(n.syntax());
            // Walk children directly — callee() and arg() both return first Expr child
            let mut expr_iter = n.syntax().children.iter().filter_map(|c| match c {
                SyntaxElement::Node(nd) => ast::Expr::cast(nd.as_ref()),
                _ => None,
            });
            let callee_ast = expr_iter.next()?;
            let arg_ast = expr_iter.next()?;
            let callee = lower_expr(&callee_ast, spans, errors)?;
            let arg = lower_expr(&arg_ast, spans, errors)?;
            let id = alloc_span(spans, span);
            Some(Expr::Apply {
                id,
                callee: Box::new(callee),
                arg: Box::new(arg),
            })
        }
        ast::Expr::LetExpr(n) => {
            let span = node_span(n.syntax());
            let name = n.name()?.text.clone();
            // Walk children directly for the two Expr nodes (generated bug: value() and body() both return first)
            let mut expr_nodes = n.syntax().children.iter().filter_map(|c| match c {
                SyntaxElement::Node(nd) => ast::Expr::cast(nd.as_ref()),
                _ => None,
            });
            let value_ast = expr_nodes.next()?;
            let body_ast = expr_nodes.next()?;
            let value = lower_expr(&value_ast, spans, errors)?;
            let body = lower_expr(&body_ast, spans, errors)?;
            let id = alloc_span(spans, span);
            Some(Expr::Let {
                id,
                name,
                value: Box::new(value),
                body: Box::new(body),
            })
        }
        ast::Expr::MatchExpr(n) => {
            let span = node_span(n.syntax());
            let scrutinee_ast = n.scrutinee()?;
            let scrutinee = lower_expr(&scrutinee_ast, spans, errors)?;
            let arms = n
                .arms()
                .filter_map(|arm| lower_match_arm(&arm, spans, errors))
                .collect();
            let id = alloc_span(spans, span);
            Some(Expr::Match {
                id,
                scrutinee: Box::new(scrutinee),
                arms,
            })
        }
        ast::Expr::PerformExpr(n) => {
            let span = node_span(n.syntax());
            let cap = n.cap()?.text.clone();
            let type_args: Vec<String> = match n.type_args() {
                Some(ga) => {
                    let items = lower_generic_args(&ga, errors);
                    items
                        .into_iter()
                        .filter_map(|ty| match ty {
                            TypeExpr::Named(name) => Some(name),
                            TypeExpr::App { head, .. } => Some(head),
                            _ => None,
                        })
                        .collect()
                }
                None => vec![],
            };
            let id = alloc_span(spans, span);
            Some(Expr::Perform { id, cap, type_args })
        }
        ast::Expr::HandleExpr(n) => {
            let span = node_span(n.syntax());
            let cap = n.cap()?.text.clone();
            let type_args: Vec<String> = match n.type_args() {
                Some(ga) => {
                    let items = lower_generic_args(&ga, errors);
                    items
                        .into_iter()
                        .filter_map(|ty| match ty {
                            TypeExpr::Named(name) => Some(name),
                            TypeExpr::App { head, .. } => Some(head),
                            _ => None,
                        })
                        .collect()
                }
                None => vec![],
            };
            // handler and body: walk children directly (generated bug: handler() and body() both return first Expr)
            let mut expr_nodes = n.syntax().children.iter().filter_map(|c| match c {
                SyntaxElement::Node(nd) => ast::Expr::cast(nd.as_ref()),
                _ => None,
            });
            let handler_ast = expr_nodes.next()?;
            let body_ast = expr_nodes.next()?;
            let handler = lower_expr(&handler_ast, spans, errors)?;
            let body = lower_expr(&body_ast, spans, errors)?;
            let id = alloc_span(spans, span);
            Some(Expr::Handle {
                id,
                cap,
                type_args,
                handler: Box::new(handler),
                body: Box::new(body),
            })
        }
        ast::Expr::BundleExpr(n) => {
            let span = node_span(n.syntax());
            let entries = n
                .entries()
                .filter_map(|e| lower_bundle_entry(&e, spans, errors))
                .collect();
            let id = alloc_span(spans, span);
            Some(Expr::Bundle { id, entries })
        }
        ast::Expr::MemberExpr(n) => {
            let span = node_span(n.syntax());
            let object_ast = n.object()?;
            let object = lower_expr(&object_ast, spans, errors)?;
            let field = n.field()?.text.clone();
            let id = alloc_span(spans, span);
            Some(Expr::Member {
                id,
                object: Box::new(object),
                field,
            })
        }
        ast::Expr::AnnExpr(n) => {
            let span = node_span(n.syntax());
            let inner_ast = n.expr()?;
            let inner = lower_expr(&inner_ast, spans, errors)?;
            let ty_ast = n.ty()?;
            let ty = lower_type_expr(&ty_ast, errors)?;
            let id = alloc_span(spans, span);
            Some(Expr::Ann {
                id,
                expr: Box::new(inner),
                ty: ty.value,
            })
        }
        ast::Expr::ErrorExpr(_n) => {
            let span = node_span(_n.syntax());
            let id = alloc_span(spans, span);
            Some(Expr::Error { id })
        }
    }
}

fn lower_match_arm(
    node: &ast::MatchArm,
    spans: &mut Vec<Span>,
    errors: &mut Vec<ParseError>,
) -> Option<MatchArm> {
    let span = node_span(node.syntax());
    let pattern_ast = node.pattern()?;
    let pattern = lower_pattern(&pattern_ast, errors)?;
    let body_ast = node.body()?;
    let body = lower_expr(&body_ast, spans, errors)?;
    Some(MatchArm {
        pattern,
        body,
        span,
    })
}

fn lower_bundle_entry(
    node: &ast::BundleEntry,
    spans: &mut Vec<Span>,
    errors: &mut Vec<ParseError>,
) -> Option<BundleEntry> {
    let span = node_span(node.syntax());
    let name = node.name()?.text.clone();
    let params = lower_param_list_opt(node.param_list(), errors);
    let body_ast = node.body()?;
    let body = lower_expr(&body_ast, spans, errors)?;
    Some(BundleEntry {
        name,
        params,
        body,
        span,
    })
}

fn lower_pattern(pat: &ast::Pattern, errors: &mut Vec<ParseError>) -> Option<Pattern> {
    match pat {
        ast::Pattern::VariantPattern(n) => {
            let name = n.name()?.text.clone();
            let args = match find_child_node(n.syntax(), SyntaxKind::PATTERN_FIELD_ITEMS) {
                Some(items_node) => items_node
                    .children
                    .iter()
                    .filter_map(|c| match c {
                        SyntaxElement::Node(nd) => {
                            ast::Pattern::cast(nd.as_ref())
                                .and_then(|p| lower_pattern(&p, errors))
                        }
                        _ => None,
                    })
                    .collect(),
                None => vec![],
            };
            Some(Pattern::Ctor { name, args })
        }
        ast::Pattern::BindPattern(n) => {
            let name = n.name()?.text.clone();
            Some(Pattern::Bind(name))
        }
        ast::Pattern::WildcardPattern(_) => Some(Pattern::Wildcard),
    }
}
