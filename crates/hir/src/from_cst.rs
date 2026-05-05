/// Direct lowering from lossless surface CST (`lst::lossless::ParseOutput`) to `hir::File`.
use lumo_lst::{
    ast::{self, AstNode},
    lossless::{ParseOutput, SyntaxElement, SyntaxNode},
    SyntaxKind,
};
use lumo_span::Span;
use lumo_types::{ContentHash, Pattern, Spanned, TypeExpr};

use crate::{
    BundleEntry, CapDecl, DataDecl, Expr, ExternFnDecl, ExternTypeDecl, FnDecl, GenericParam,
    HirError, ImplDecl, ImplMethodDecl, Item, MatchArm, OperationDecl, Param, UseDecl,
    VariantDecl,
};

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn lower_from_cst(parsed: &ParseOutput) -> crate::File {
    let mut ctx = LowerCtx { errors: Vec::new() };

    let root = match ast::File::cast(&parsed.root) {
        Some(f) => f,
        None => {
            return crate::File {
                items: vec![],
                content_hash: ContentHash(0),
                errors: vec![HirError {
                    span: parsed.root.span,
                    message: "root node is not FILE".into(),
                }],
            };
        }
    };

    let mut items: Vec<Item> = Vec::new();
    for item in root.items() {
        lower_item(&item, &mut items, &mut ctx);
    }

    let items = crate::dedupe_data_with_as_raw(items);
    let content_hash = crate::hash_file_pub(&items);
    crate::File {
        items,
        content_hash,
        errors: ctx.errors,
    }
}

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

struct LowerCtx {
    errors: Vec<HirError>,
}

// ---------------------------------------------------------------------------
// Attribute helpers
// ---------------------------------------------------------------------------

fn attr_name(attr: &ast::Attribute) -> Option<String> {
    // attr.name() looks for ATTR_NAME, but keyword tokens like `extern` keep their keyword kind.
    // Walk children: first non-trivia token after `[` is the attribute name.
    let mut past_bracket = false;
    for child in &attr.syntax().children {
        if let SyntaxElement::Token(t) = child {
            if !past_bracket {
                if t.kind == SyntaxKind::L_BRACKET {
                    past_bracket = true;
                }
            } else if t.kind != SyntaxKind::WHITESPACE && t.kind != SyntaxKind::NEWLINE {
                return Some(t.text.clone());
            }
        }
    }
    None
}

fn attr_args<'a>(attr: &'a ast::Attribute) -> impl Iterator<Item = ast::AttributeArgItem<'a>> + 'a {
    attr.syntax()
        .children
        .iter()
        .find_map(|c| match c {
            SyntaxElement::Node(n) => ast::AttributeArgs::cast(n),
            _ => None,
        })
        .into_iter()
        .flat_map(|args| args.items().collect::<Vec<_>>())
}

fn attr_arg_str(attr: &ast::Attribute, key: &str) -> Option<String> {
    for arg in attr_args(attr) {
        // arg.name() looks for ATTR_NAME but keys are lexed as IDENT; scan children directly.
        let k = arg_item_key_text(&arg);
        if k.as_deref() == Some(key) {
            if let Some(val_expr) = arg.value() {
                if let Some(s) = extract_string_expr(&val_expr) {
                    return Some(s);
                }
            }
        }
    }
    None
}

fn arg_item_key_text(arg: &ast::AttributeArgItem) -> Option<String> {
    // The key token is the first non-trivia token in the arg item (stored as IDENT or ATTR_NAME).
    for child in &arg.syntax().children {
        if let SyntaxElement::Token(t) = child {
            if t.kind != SyntaxKind::WHITESPACE && t.kind != SyntaxKind::NEWLINE {
                return Some(t.text.clone());
            }
        }
    }
    None
}

fn unescape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('0') => out.push('\0'),
                Some(c) => { out.push('\\'); out.push(c); }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn strip_string_quotes(text: &str) -> String {
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        unescape_string(&text[1..text.len() - 1])
    } else {
        text.to_owned()
    }
}

fn extract_string_expr(expr: &ast::Expr) -> Option<String> {
    match expr {
        ast::Expr::StringExpr(s) => s.value().map(|t| strip_string_quotes(&t.text)),
        _ => None,
    }
}

fn attrs_find_inline(attrs: &[ast::Attribute<'_>]) -> bool {
    attrs.iter().any(|attr| {
        if attr_name(attr).as_deref() != Some("inline") {
            return false;
        }
        // Check for `always` flag-style arg or value.
        // Use arg_item_key_text (not arg.name()) since the token is IDENT-kinded.
        for arg in attr_args(attr) {
            let key = arg_item_key_text(&arg).unwrap_or_default();
            if key == "always" {
                return true;
            }
        }
        // Also check if the attribute has a value that is "always"
        // (legacy: #[inline = "always"])
        false
    })
}

/// Translate `infix<op>` / `prefix<op>` into internal extern name form.
fn operator_attr_to_extern_name(spec: &str) -> String {
    if let Some(op) = spec.strip_prefix("infix") {
        format!("_{op}_")
    } else if let Some(op) = spec.strip_prefix("prefix") {
        format!("{op}_")
    } else {
        spec.to_owned()
    }
}

fn attrs_find_extern_name<'a>(
    attrs: &[ast::Attribute<'a>],
    fallback_fn_name: &str,
) -> Option<String> {
    let link = attrs.iter().find(|a| attr_name(a).as_deref() == Some("link"));
    let extern_attr = attrs.iter().find(|a| attr_name(a).as_deref() == Some("extern"));

    if let Some(link) = link {
        if let Some(expr_base) = attr_arg_str(link, "expr") {
            if let Some(ext) = extern_attr {
                if let Some(prop) = attr_arg_str(ext, "property") {
                    return Some(format!("{expr_base}.prototype.{prop}"));
                }
                if let Some(method) = attr_arg_str(ext, "name") {
                    return Some(format!("{expr_base}.prototype.{method}()"));
                }
                if let Some(static_method) = attr_arg_str(ext, "static") {
                    return Some(format!("{expr_base}.{static_method}()"));
                }
                if let Some(static_prop) = attr_arg_str(ext, "static_property") {
                    return Some(format!("{expr_base}.{static_prop}"));
                }
            }
            return Some(format!("{expr_base}.{fallback_fn_name}()"));
        }
        if attr_arg_str(link, "module").is_some() {
            return Some(format!("__lumo_{fallback_fn_name}()"));
        }
    }

    if let Some(attr) = extern_attr {
        // Check for direct value: `#[extern = "string"]` form
        if let Some(direct_val) = attr.direct_value() {
            if let Some(s) = extract_string_expr(&direct_val) {
                return Some(s);
            }
        }
        // Check for operator arg
        if let Some(op) = attr_arg_str(attr, "operator") {
            return Some(operator_attr_to_extern_name(&op));
        }
        if let Some(name) = attr_arg_str(attr, "name") {
            return Some(name);
        }
    }

    None
}

fn attrs_find_link_module<'a>(
    attrs: &[ast::Attribute<'a>],
    fallback_fn_name: &str,
) -> Option<(String, String)> {
    let link = attrs.iter().find(|a| attr_name(a).as_deref() == Some("link"))?;
    let module = attr_arg_str(link, "module")?;
    let js_name = attrs
        .iter()
        .find(|a| attr_name(a).as_deref() == Some("extern"))
        .and_then(|ext| attr_arg_str(ext, "name"))
        .unwrap_or_else(|| fallback_fn_name.to_owned());
    Some((module, js_name))
}

fn attrs_find_as_raw(attrs: &[ast::Attribute<'_>]) -> Option<crate::AsRawValue> {
    attrs.iter().find_map(|attr| {
        if attr_name(attr).as_deref() != Some("as__raw") {
            return None;
        }
        // Look for a flag-style arg (key without value, name == "true" or "false").
        // Use arg_item_key_text rather than arg.name() because the key token is
        // IDENT-kinded (not ATTR_NAME) in the lossless tree.
        for arg in attr_args(attr) {
            let key = arg_item_key_text(&arg).unwrap_or_default();
            if key == "true" {
                return Some(crate::AsRawValue::True);
            } else if key == "false" {
                return Some(crate::AsRawValue::False);
            }
        }
        None
    })
}

// ---------------------------------------------------------------------------
// Type expression helpers
// ---------------------------------------------------------------------------

fn type_expr_repr(ty: &ast::TypeExpr) -> String {
    match ty {
        ast::TypeExpr::ThunkTypeExpr(t) => {
            let inner = t.inner().map(|i| type_expr_repr(&i)).unwrap_or_default();
            format!("thunk {}", inner)
        }
        ast::TypeExpr::FnTypeExpr(f) => {
            let params: Vec<String> = f.param_types().map(|p| type_expr_repr(&p)).collect();
            let ret = f.return_type().map(|r| type_expr_repr(&r)).unwrap_or_default();
            let cap_str = if let Some(cap_ann) = f.cap_annotation() {
                if let Some(cap_ref) = lower_cap_annotation(&cap_ann) {
                    if cap_ref.is_empty() {
                        " / {}".into()
                    } else {
                        let parts: Vec<String> = cap_ref.iter().map(|e| e.display()).collect();
                        format!(" / {{ {} }}", parts.join(", "))
                    }
                } else {
                    String::new() // `..` wildcard: omit cap constraint
                }
            } else {
                String::new()
            };
            format!("({}) -> {}{}", params.join(", "), ret, cap_str)
        }
        ast::TypeExpr::ProjTypeExpr(p) => {
            let base = p.head()
                .map(|h| type_expr_repr(&ast::TypeExpr::SimpleTypeExpr(h)))
                .unwrap_or_default();
            let assoc = p.assoc().map(|t| t.text.clone()).unwrap_or_default();
            format!("{base}.{assoc}")
        }
        ast::TypeExpr::SimpleTypeExpr(s) => {
            let name = s.name().map(|t| t.text.clone()).unwrap_or_default();
            if let Some(generic_args) = s.generic_args() {
                if let Some(arg_items) = generic_args.args() {
                    // tail() returns all items (head+tail) since both map over all children
                    let args: Vec<String> = arg_items.tail().map(|a| type_expr_repr(&a)).collect();
                    if !args.is_empty() {
                        return format!("{}[{}]", name, args.join(", "));
                    }
                }
            }
            name
        }
    }
}

fn lower_type_expr(ty: &ast::TypeExpr) -> Option<Spanned<TypeExpr>> {
    let repr = type_expr_repr(ty);
    TypeExpr::parse(&repr).map(|value| Spanned {
        value,
        span: ty.syntax().span,
    })
}

fn lower_type_expr_with_fallback(ty: &ast::TypeExpr) -> Spanned<TypeExpr> {
    let repr = type_expr_repr(ty);
    let span = ty.syntax().span;
    let value = TypeExpr::parse(&repr).unwrap_or_else(|| TypeExpr::Named(repr.trim().to_owned()));
    Spanned { value, span }
}

fn lower_type_expr_repr_with_fallback(repr: &str, span: Span) -> Spanned<TypeExpr> {
    let value = TypeExpr::parse(repr).unwrap_or_else(|| TypeExpr::Named(repr.trim().to_owned()));
    Spanned { value, span }
}

// ---------------------------------------------------------------------------
// CapRef / cap annotation
// ---------------------------------------------------------------------------

fn lower_cap_annotation(ann: &ast::CapAnnotation) -> Option<lumo_types::CapRef> {
    let cap_set = match ann.cap() {
        Some(cs) => cs,
        None => return Some(vec![]), // `/ {}` — explicitly empty cap set
    };
    let has_infer = cap_set.syntax().children.iter().any(|c| matches!(c, SyntaxElement::Token(t) if t.text == ".."));
    let mut parts: Vec<String> = if has_infer { vec!["..".into()] } else { vec![] };
    for sig in cap_set.sigs() {
        let name = sig.name().map(|t| t.text.clone()).unwrap_or_default();
        if let Some(generic_args) = sig.generic_args() {
            if let Some(arg_items) = generic_args.args() {
                let args: Vec<String> = arg_items.tail().map(|a| type_expr_repr(&a)).collect();
                if !args.is_empty() {
                    parts.push(format!("{}[{}]", name, args.join(", ")));
                    continue;
                }
            }
        }
        parts.push(name);
    }
    Some(lumo_types::parse_cap_ref(&parts.join(", ")))
}

// ---------------------------------------------------------------------------
// Generic params
// ---------------------------------------------------------------------------

fn lower_generic_params(gp: &ast::GenericParams) -> Vec<GenericParam> {
    let items = match gp.params() {
        Some(items) => items,
        None => return vec![],
    };
    items
        .tail()
        .map(|param| {
            let name = param.name().map(|t| t.text.clone()).unwrap_or_default();
            if param.is_cap_param() {
                return GenericParam::CapRow(name);
            }
            if let Some(bound_list) = param.constraint() {
                // Collect all bound types: `A: Eq + Add` has two bounds.
                let bounds: Vec<String> = bound_list
                    .tail()
                    .map(|ty| type_expr_repr(&ty))
                    .filter(|s| !s.is_empty())
                    .collect();
                // Cap-row constraint has a single bound starting with `{`
                if bounds.len() == 1 && bounds[0].starts_with('{') {
                    GenericParam::CapRow(name)
                } else {
                    GenericParam::Type(name, bounds)
                }
            } else {
                GenericParam::Type(name, vec![])
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Params
// ---------------------------------------------------------------------------

fn lower_param_list(pl: &ast::ParamList, self_type_repr: Option<&str>) -> Vec<Param> {
    let items = match pl.params() {
        Some(items) => items,
        None => return vec![],
    };
    items
        .tail()
        .map(|param| lower_param(&param, self_type_repr))
        .collect()
}

fn lower_param(param: &ast::Param, self_type_repr: Option<&str>) -> Param {
    let name = param.name().map(|t| t.text.clone()).unwrap_or_default();
    let span = param.syntax().span;
    let ty = match param.ty() {
        Some(ty_node) => {
            let mut repr = type_expr_repr(&ty_node);
            if let Some(target_repr) = self_type_repr {
                repr = repr.replace("Self", target_repr);
            }
            lower_type_expr_repr_with_fallback(&repr, ty_node.syntax().span)
        }
        None => Spanned {
            value: TypeExpr::Named("Unknown".into()),
            span,
        },
    };
    Param { name, ty, span }
}

// ---------------------------------------------------------------------------
// Items
// ---------------------------------------------------------------------------

fn lower_item(item: &ast::Item, out: &mut Vec<Item>, ctx: &mut LowerCtx) {
    let attrs: Vec<ast::Attribute> = item.attrs().collect();
    match item.body() {
        Some(ast::ItemBody::ExternDecl(ext_decl)) => {
            lower_extern_decl(&ext_decl, &attrs, out);
        }
        Some(ast::ItemBody::DataDecl(data_decl)) => {
            out.push(Item::Data(lower_data_decl(&data_decl)));
        }
        Some(ast::ItemBody::CapDecl(cap_decl)) => {
            out.push(Item::Cap(lower_cap_decl(&cap_decl)));
        }
        Some(ast::ItemBody::FnDecl(fn_decl)) => {
            out.push(Item::Fn(lower_fn_decl(&fn_decl, &attrs, ctx)));
        }
        Some(ast::ItemBody::UseDecl(use_decl)) => {
            out.push(Item::Use(lower_use_decl(&use_decl)));
        }
        Some(ast::ItemBody::ImplDecl(impl_decl)) => {
            out.push(Item::Impl(lower_impl_decl(&impl_decl, ctx)));
        }
        None => {}
    }
}

// ---------------------------------------------------------------------------
// ExternDecl
// ---------------------------------------------------------------------------

fn lower_extern_decl(
    ext: &ast::ExternDecl,
    item_attrs: &[ast::Attribute],
    out: &mut Vec<Item>,
) {
    match ext.rest() {
        Some(ast::ExternRest::ExternTypeTail(tail)) => {
            let name = tail.name().map(|t| t.text.clone()).unwrap_or_default();
            let extern_name = attrs_find_extern_name(item_attrs, &name);
            out.push(Item::ExternType(ExternTypeDecl {
                name,
                extern_name,
                span: ext.syntax().span,
            }));
        }
        Some(ast::ExternRest::ExternFnTail(tail)) => {
            let fn_item = lower_extern_fn_tail(&tail, item_attrs, ext.syntax().span);
            out.push(Item::ExternFn(fn_item));
        }
        Some(ast::ExternRest::ExternBlockTail(block_tail)) => {
            for block_item in block_tail.items() {
                // Merge block-level item attrs with outer item attrs
                let item_extra_attrs: Vec<ast::Attribute> = block_item.attrs().collect();
                match block_item.body() {
                    Some(ast::ExternBlockItemBody::ExternTypeTail(tail)) => {
                        let name = tail.name().map(|t| t.text.clone()).unwrap_or_default();
                        // Collect all attrs: outer item_attrs + block_item attrs
                        // We merge by collecting the underlying SyntaxNode pointers.
                        let merged_nodes: Vec<&SyntaxNode> = item_attrs
                            .iter()
                            .map(|a| a.syntax())
                            .chain(item_extra_attrs.iter().map(|a| a.syntax()))
                            .collect();
                        let merged_attrs: Vec<ast::Attribute> = merged_nodes
                            .iter()
                            .filter_map(|n| ast::Attribute::cast(n))
                            .collect();
                        let extern_name = attrs_find_extern_name(&merged_attrs, &name);
                        out.push(Item::ExternType(ExternTypeDecl {
                            name,
                            extern_name,
                            span: tail.syntax().span,
                        }));
                    }
                    Some(ast::ExternBlockItemBody::ExternFnTail(tail)) => {
                        let merged_nodes: Vec<&SyntaxNode> = item_attrs
                            .iter()
                            .map(|a| a.syntax())
                            .chain(item_extra_attrs.iter().map(|a| a.syntax()))
                            .collect();
                        let merged_attrs: Vec<ast::Attribute> = merged_nodes
                            .iter()
                            .filter_map(|n| ast::Attribute::cast(n))
                            .collect();
                        let fn_item = lower_extern_fn_tail(&tail, &merged_attrs, tail.syntax().span);
                        out.push(Item::ExternFn(fn_item));
                    }
                    None => {}
                }
            }
        }
        None => {}
    }
}

fn lower_extern_fn_tail(
    tail: &ast::ExternFnTail,
    attrs: &[ast::Attribute<'_>],
    span: Span,
) -> ExternFnDecl {
    let name = tail.name().map(|t| t.text.clone()).unwrap_or_default();
    let extern_name = attrs_find_extern_name(attrs, &name);
    let link_module = attrs_find_link_module(attrs, &name);
    let inline = attrs_find_inline(attrs);
    let params = tail
        .param_list()
        .map(|pl| lower_param_list(&pl, None))
        .unwrap_or_default();
    let return_type = tail.return_type().and_then(|ty| lower_type_expr(&ty));
    let cap = tail.cap_annotation().and_then(|ann| lower_cap_annotation(&ann));
    ExternFnDecl {
        name,
        extern_name,
        link_module,
        inline,
        params,
        return_type,
        cap,
        span,
    }
}

// ---------------------------------------------------------------------------
// DataDecl
// ---------------------------------------------------------------------------

fn lower_data_decl(data: &ast::DataDecl) -> DataDecl {
    let name = data.name().map(|t| t.text.clone()).unwrap_or_default();
    let generics: Vec<String> = data
        .generic_params()
        .and_then(|gp| gp.params())
        .map(|items| items.tail().map(|p| p.name().map(|t| t.text.clone()).unwrap_or_default()).collect())
        .unwrap_or_default();
    let variants: Vec<VariantDecl> = data.variants().map(|v| lower_variant(&v)).collect();
    DataDecl {
        name,
        generics,
        variants,
        span: data.syntax().span,
    }
}

fn lower_variant(variant: &ast::Variant) -> VariantDecl {
    let name = variant.name().map(|t| t.text.clone()).unwrap_or_default();
    let attrs: Vec<ast::Attribute> = variant.attrs().collect();
    let as_raw = attrs_find_as_raw(&attrs);
    let payload: Vec<Spanned<TypeExpr>> = variant
        .variant_fields()
        .and_then(|vf| vf.fields())
        .map(|items| {
            items
                .tail()
                .filter_map(|ty| lower_type_expr(&ty))
                .collect()
        })
        .unwrap_or_default();
    VariantDecl {
        name,
        payload,
        as_raw,
        span: variant.syntax().span,
    }
}

// ---------------------------------------------------------------------------
// CapDecl
// ---------------------------------------------------------------------------

fn lower_cap_decl(cap: &ast::CapDecl) -> CapDecl {
    let name = cap.name().map(|t| t.text.clone()).unwrap_or_default();
    let operations: Vec<OperationDecl> = cap.operations().map(|op| lower_operation(&op)).collect();
    CapDecl {
        name,
        operations,
        span: cap.syntax().span,
    }
}

fn lower_operation(op: &ast::OperationDecl) -> OperationDecl {
    let name = op.name().map(|t| t.text.clone()).unwrap_or_default();
    let params = op.param_list().map(|pl| lower_param_list(&pl, None)).unwrap_or_default();
    let return_type = op.return_type().and_then(|ty| lower_type_expr(&ty));
    OperationDecl {
        name,
        params,
        return_type,
        span: op.syntax().span,
    }
}

// ---------------------------------------------------------------------------
// FnDecl
// ---------------------------------------------------------------------------

fn lower_fn_decl(
    fn_decl: &ast::FnDecl,
    attrs: &[ast::Attribute<'_>],
    ctx: &mut LowerCtx,
) -> FnDecl {
    let name = fn_decl.name().map(|t| t.text.clone()).unwrap_or_default();
    let generics = fn_decl
        .generic_params()
        .map(|gp| lower_generic_params(&gp))
        .unwrap_or_default();
    let params = fn_decl
        .param_list()
        .map(|pl| lower_param_list(&pl, None))
        .unwrap_or_default();
    let return_type = fn_decl.return_type().and_then(|ty| lower_type_expr(&ty));
    let cap = fn_decl.cap_annotation().and_then(|ann| lower_cap_annotation(&ann));
    let body = match fn_decl.body() {
        Some(ast::FnBody::BlockExpr(block)) => lower_block_expr(&block, ctx),
        Some(ast::FnBody::ExprBody(expr_body)) => {
            expr_body
                .body()
                .map(|e| lower_expr(&e, ctx))
                .unwrap_or(Expr::Error { span: fn_decl.syntax().span })
        }
        None => Expr::Error { span: fn_decl.syntax().span },
    };
    let inline = attrs_find_inline(attrs);
    FnDecl {
        name,
        generics,
        params,
        return_type,
        cap,
        body,
        inline,
        span: fn_decl.syntax().span,
    }
}

// ---------------------------------------------------------------------------
// UseDecl
// ---------------------------------------------------------------------------

fn lower_use_decl(use_decl: &ast::UseDecl) -> UseDecl {
    let span = use_decl.syntax().span;
    let (path, names) = match use_decl.path() {
        Some(p) => collect_use_path(&p),
        None => (vec![], None),
    };
    UseDecl { path, names, span }
}

/// Walk `UsePath` recursively to extract (path_segments, Option<names>).
/// Grammar:
///   UsePath = head:Ident rest:UsePathRest?
///   UsePathRest = '.' item:UsePathItem
///   UsePathItem = UsePathBranch | UseTree
///   UsePathBranch = next:Ident cont:UsePathRest?
///   UseTree = '{' names:UseNameItems? '}'
///   UseNameItems = head:UseNameItem (',' UseNameItem)* ','?
///   UseNameItem = name:Ident
fn collect_use_path(path_node: &ast::UsePath) -> (Vec<String>, Option<Vec<String>>) {
    let mut segments = Vec::new();
    if let Some(head) = path_node.head() {
        segments.push(head.text.clone());
    }
    let names = collect_use_path_rest(path_node.rest(), &mut segments);
    (segments, names)
}

fn collect_use_path_rest(
    rest: Option<ast::UsePathRest>,
    segments: &mut Vec<String>,
) -> Option<Vec<String>> {
    let rest = rest?;
    match rest.item() {
        Some(ast::UsePathItem::UsePathBranch(branch)) => {
            if let Some(next) = branch.next() {
                segments.push(next.text.clone());
            }
            collect_use_path_rest(branch.cont(), segments)
        }
        Some(ast::UsePathItem::UseTree(tree)) => {
            let names: Vec<String> = tree
                .names()
                .map(|items| {
                    items
                        .tail()
                        .filter_map(|ni| ni.name().map(|t| t.text.clone()))
                        .collect()
                })
                .unwrap_or_default();
            Some(names)
        }
        None => None,
    }
}

// ---------------------------------------------------------------------------
// ImplDecl
// ---------------------------------------------------------------------------

fn lower_impl_decl(impl_decl: &ast::ImplDecl, ctx: &mut LowerCtx) -> ImplDecl {
    let span = impl_decl.syntax().span;
    let generics = impl_decl
        .generic_params()
        .map(|gp| lower_generic_params(&gp))
        .unwrap_or_default();

    // Collect TypeExpr children in order (the CST has both target and cap as TypeExpr children)
    let type_exprs: Vec<ast::TypeExpr> = impl_decl
        .syntax()
        .children
        .iter()
        .filter_map(|c| match c {
            SyntaxElement::Node(n) => ast::TypeExpr::cast(n),
            _ => None,
        })
        .collect();

    let target_type_node = type_exprs.first();
    let cap_type_node = type_exprs.get(1);

    let target_repr = target_type_node.map(|ty| type_expr_repr(ty)).unwrap_or_else(|| "Unknown".into());
    let target_span = target_type_node.map(|ty| ty.syntax().span).unwrap_or(span);
    let target_type = lower_type_expr_repr_with_fallback(&target_repr, target_span);

    let capability = cap_type_node.and_then(|ty| lower_type_expr(ty));

    // For `self` parameter type substitution
    // Named impls (impl Name = ...) are not in the surface grammar; name is always None from CST.
    let name: Option<String> = None;

    let methods: Vec<ImplMethodDecl> = impl_decl
        .methods()
        .map(|m| lower_impl_method(&m, &target_repr, ctx))
        .collect();

    ImplDecl {
        name,
        generics,
        target_type,
        capability,
        methods,
        span,
    }
}

fn lower_impl_method(
    method: &ast::ImplMethod,
    target_repr: &str,
    ctx: &mut LowerCtx,
) -> ImplMethodDecl {
    let name = method.name().map(|t| t.text.clone()).unwrap_or_default();
    let span = method.syntax().span;
    let params = method
        .param_list()
        .map(|pl| lower_param_list(&pl, Some(target_repr)))
        .unwrap_or_default();
    let return_type = method.return_type().and_then(|ty| {
        let repr = type_expr_repr(&ty).replace("Self", target_repr);
        Some(lower_type_expr_repr_with_fallback(&repr, ty.syntax().span))
    });
    let body = match method.body() {
        Some(ast::FnBody::BlockExpr(block)) => lower_block_expr(&block, ctx),
        Some(ast::FnBody::ExprBody(expr_body)) => {
            expr_body
                .body()
                .map(|e| lower_expr(&e, ctx))
                .unwrap_or(Expr::Error { span })
        }
        None => Expr::Error { span },
    };
    ImplMethodDecl {
        name,
        params,
        return_type,
        body,
        span,
    }
}

// ---------------------------------------------------------------------------
// Block expression
// ---------------------------------------------------------------------------

fn lower_block_expr(block: &ast::BlockExpr, ctx: &mut LowerCtx) -> Expr {
    let span = block.syntax().span;
    let stmts: Vec<ast::BlockStmt> = block.stmts().collect();

    // The last ExprStmt without semicolon is the "result" expression.
    // The grammar always emits ExprStmt for trailing exprs — so the last stmt is the result.
    // All other stmts become nested Let bindings.
    if stmts.is_empty() {
        return Expr::Produce {
            expr: Box::new(Expr::Ident { name: "unit".into(), span }),
            span,
        };
    }

    // We desugar the block:
    // { let x = e1; e2; result } → Let(x, e1, Let("_", e2, result))
    let (init_stmts, last) = stmts.split_at(stmts.len() - 1);

    // Lower result (last statement)
    let result = match &last[0] {
        ast::BlockStmt::ExprStmt(es) => {
            let expr = es.expr().map(|e| lower_expr(&e, ctx)).unwrap_or(Expr::Error { span });
            maybe_produce(expr, span)
        }
        ast::BlockStmt::LetStmt(ls) => {
            // Lone let at end of block — treat as Let with Error body
            let let_name = ls.name().map(|t| t.text.clone()).unwrap_or_else(|| "_".into());
            let let_val = ls.value().map(|e| lower_expr(&e, ctx)).unwrap_or(Expr::Error { span });
            Expr::Let {
                name: let_name,
                value: Box::new(let_val),
                body: Box::new(Expr::Error { span }),
                span,
            }
        }
    };

    // Fold init stmts in reverse
    let mut body = result;
    for stmt in init_stmts.iter().rev() {
        match stmt {
            ast::BlockStmt::LetStmt(ls) => {
                let let_name = ls.name().map(|t| t.text.clone()).unwrap_or_else(|| "_".into());
                let let_val = ls.value().map(|e| lower_expr(&e, ctx)).unwrap_or(Expr::Error { span });
                let stmt_span = ls.syntax().span;
                body = Expr::Let {
                    name: let_name,
                    value: Box::new(let_val),
                    body: Box::new(body),
                    span: stmt_span,
                };
            }
            ast::BlockStmt::ExprStmt(es) => {
                let expr = es.expr().map(|e| lower_expr(&e, ctx)).unwrap_or(Expr::Error { span });
                let stmt_span = es.syntax().span;
                body = Expr::Let {
                    name: "_".into(),
                    value: Box::new(expr),
                    body: Box::new(body),
                    span: stmt_span,
                };
            }
        }
    }

    // If we folded any stmts, rewrap outer span
    if !init_stmts.is_empty() {
        if let Expr::Let { name, value, body: inner, .. } = body {
            body = Expr::Let { name, value, body: inner, span };
        }
    }

    body
}

// ---------------------------------------------------------------------------
// Expression lowering
// ---------------------------------------------------------------------------

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

/// Get the operator token text from a `BinaryExpr` or `UnaryExpr` node.
fn find_op_token(node: &SyntaxNode) -> Option<String> {
    for child in &node.children {
        if let SyntaxElement::Token(tok) = child {
            match tok.kind {
                SyntaxKind::PLUS
                | SyntaxKind::MINUS
                | SyntaxKind::STAR
                | SyntaxKind::SLASH
                | SyntaxKind::PERCENT
                | SyntaxKind::EQ_EQ
                | SyntaxKind::BANG_EQ
                | SyntaxKind::LT
                | SyntaxKind::LT_EQ
                | SyntaxKind::GT
                | SyntaxKind::GT_EQ
                | SyntaxKind::AMP_AMP
                | SyntaxKind::PIPE_PIPE
                | SyntaxKind::BANG => return Some(tok.text.clone()),
                _ => {}
            }
        }
    }
    None
}

fn get_expr_children<'a>(node: &'a SyntaxNode) -> Vec<ast::Expr<'a>> {
    node.children
        .iter()
        .filter_map(|c| match c {
            SyntaxElement::Node(n) => ast::Expr::cast(n),
            _ => None,
        })
        .collect()
}

fn lower_expr(expr: &ast::Expr, ctx: &mut LowerCtx) -> Expr {
    match expr {
        ast::Expr::IdentExpr(e) => {
            let name = e.name().map(|t| t.text.clone()).unwrap_or_default();
            Expr::Ident { name, span: e.syntax().span }
        }
        ast::Expr::StringExpr(e) => {
            let raw = e.value().map(|t| t.text.clone()).unwrap_or_default();
            let value = strip_string_quotes(&raw);
            Expr::String { value, span: e.syntax().span }
        }
        ast::Expr::NumberExpr(e) => {
            let value = e.value().map(|t| t.text.clone()).unwrap_or_default();
            Expr::Number { value, span: e.syntax().span }
        }
        ast::Expr::LetExpr(e) => {
            // Standalone let — only meaningful inside block; body is Error
            let name = e.name().map(|t| t.text.clone()).unwrap_or_default();
            let value = e.value().map(|v| lower_expr(&v, ctx)).unwrap_or(Expr::Error { span: e.syntax().span });
            Expr::Let {
                name,
                value: Box::new(value),
                body: Box::new(Expr::Error { span: e.syntax().span }),
                span: e.syntax().span,
            }
        }
        ast::Expr::ThunkExpr(e) => {
            let span = e.syntax().span;
            let inner = e.body().map(|b| lower_expr(&b, ctx)).unwrap_or(Expr::Error { span });
            Expr::Thunk { expr: Box::new(inner), span }
        }
        ast::Expr::ForceExpr(e) => {
            let span = e.syntax().span;
            let inner = e.expr().map(|b| lower_expr(&b, ctx)).unwrap_or(Expr::Error { span });
            Expr::Force { expr: Box::new(inner), span }
        }
        ast::Expr::MatchExpr(e) => {
            let span = e.syntax().span;
            let scrutinee = e
                .scrutinee()
                .map(|s| lower_expr(&s, ctx))
                .unwrap_or(Expr::Error { span });
            let arms: Vec<MatchArm> = e
                .arms()
                .map(|arm| {
                    let arm_span = arm.syntax().span;
                    // Detect `ident(...)` — bare constructor without leading `.`
                    let is_malformed_ctor = arm.pattern().map(|p| {
                        matches!(&p, ast::Pattern::BindPattern(bp) if bp.has_call_args())
                    }).unwrap_or(false);
                    let pat_text = arm.pattern().map(|p| pattern_to_str(&p)).unwrap_or_default();
                    let pattern = if is_malformed_ctor {
                        ctx.errors.push(HirError {
                            span: arm_span,
                            message: format!(
                                "invalid match pattern `{}`; constructor patterns must start with `.`",
                                pat_text
                            ),
                        });
                        Pattern::Wildcard
                    } else {
                        match Pattern::parse(&pat_text) {
                            Some(p) => p,
                            None => {
                                ctx.errors.push(HirError {
                                    span: arm_span,
                                    message: format!(
                                        "invalid match pattern `{}`; constructor patterns must start with `.`",
                                        pat_text
                                    ),
                                });
                                Pattern::Wildcard
                            }
                        }
                    };
                    let body = arm
                        .body()
                        .map(|b| maybe_produce(lower_expr(&b, ctx), arm_span))
                        .unwrap_or(Expr::Error { span: arm_span });
                    MatchArm { pattern, body, span: arm_span }
                })
                .collect();
            Expr::Match { scrutinee: Box::new(scrutinee), arms, span }
        }
        ast::Expr::PerformExpr(e) => {
            let cap = e.name().map(|t| t.text.clone()).unwrap_or_default();
            Expr::Perform { cap, span: e.syntax().span }
        }
        ast::Expr::HandleExpr(e) => {
            let span = e.syntax().span;
            let cap_name = e.cap_name().map(|t| t.text.clone()).unwrap_or_default();
            let type_args: Vec<String> = e
                .cap_type()
                .map(|ty| vec![type_expr_repr(&ty)])
                .unwrap_or_default();

            // Walk children to collect Expr children in order (handler then body).
            let exprs = get_expr_children(e.syntax());
            let handler = exprs
                .first()
                .map(|ex| lower_expr(ex, ctx))
                .unwrap_or(Expr::Error { span });
            let body_raw = exprs
                .get(1)
                .map(|ex| lower_expr(ex, ctx))
                .unwrap_or(Expr::Error { span });
            let body = maybe_produce(body_raw, span);

            Expr::Handle {
                cap: cap_name,
                type_args,
                handler: Box::new(handler),
                body: Box::new(body),
                span,
            }
        }
        ast::Expr::BundleExpr(e) => {
            let span = e.syntax().span;
            let entries: Vec<BundleEntry> = e
                .entries()
                .map(|entry| lower_bundle_entry(&entry, ctx))
                .collect();
            Expr::Bundle { entries, span }
        }
        ast::Expr::IfElseExpr(e) => {
            let span = e.syntax().span;
            lower_if_else(e, ctx, span)
        }
        ast::Expr::BlockExpr(e) => lower_block_expr(e, ctx),
        ast::Expr::ParenExpr(e) => {
            let span = e.syntax().span;
            e.inner()
                .map(|inner| lower_expr(&inner, ctx))
                .unwrap_or(Expr::Error { span })
        }
        ast::Expr::AnnotationExpr(e) => {
            let span = e.syntax().span;
            let inner = e.expr().map(|ex| lower_expr(&ex, ctx)).unwrap_or(Expr::Error { span });
            let ty = e
                .ty()
                .map(|ty| lower_type_expr_with_fallback(&ty))
                .unwrap_or(Spanned { value: TypeExpr::Named("Unknown".into()), span });
            Expr::Ann { expr: Box::new(inner), ty, span }
        }
        ast::Expr::LambdaExpr(e) => {
            let span = e.syntax().span;
            let hir_params: Vec<(String, Option<Spanned<TypeExpr>>)> = e
                .param_list()
                .and_then(|pl| pl.params())
                .map(|items| {
                    items
                        .tail()
                        .map(|p| {
                            let name = p.name().map(|t| t.text.clone()).unwrap_or_default();
                            let ty_ann = p.ty().and_then(|ty| lower_type_expr(&ty));
                            (name, ty_ann)
                        })
                        .collect()
                })
                .unwrap_or_default();
            let body = match e.body() {
                Some(ast::FnBody::BlockExpr(block)) => lower_block_expr(&block, ctx),
                Some(ast::FnBody::ExprBody(expr_body)) => {
                    expr_body
                        .body()
                        .map(|ex| lower_expr(&ex, ctx))
                        .unwrap_or(Expr::Error { span })
                }
                None => Expr::Error { span },
            };
            Expr::Lambda { params: hir_params, body: Box::new(body), span }
        }
        ast::Expr::UnaryExpr(e) => {
            let span = e.syntax().span;
            let op = find_op_token(e.syntax()).unwrap_or_default();
            let (cap_name, method_name) = match op.as_str() {
                "-" => ("Neg", "neg"),
                "!" => ("Not", "not"),
                _ => ("Neg", "neg"),
            };
            let exprs = get_expr_children(e.syntax());
            let operand = exprs
                .first()
                .map(|ex| lower_expr(ex, ctx))
                .unwrap_or(Expr::Error { span });
            desugar_unary_call(span, cap_name, method_name, operand)
        }
        ast::Expr::MemberExpr(e) => {
            let span = e.syntax().span;
            let member = e.member().map(|t| t.text.clone()).unwrap_or_default();
            // The object is the first Expr child
            let exprs = get_expr_children(e.syntax());
            let object = exprs
                .first()
                .map(|ex| lower_expr(ex, ctx))
                .unwrap_or(Expr::Error { span });
            Expr::Member { object: Box::new(object), member, span }
        }
        ast::Expr::CallExpr(e) => {
            let span = e.syntax().span;
            // Callee is the first Expr child (before the CallArgItems)
            let exprs = get_expr_children(e.syntax());
            let callee = exprs
                .first()
                .map(|ex| lower_expr(ex, ctx))
                .unwrap_or(Expr::Error { span });
            let args: Vec<Expr> = e
                .args()
                .map(|items| items.tail().map(|a| lower_expr(&a, ctx)).collect())
                .unwrap_or_default();
            Expr::Call { callee: Box::new(callee), args, span }
        }
        ast::Expr::BinaryExpr(e) => {
            let span = e.syntax().span;
            let op_text = find_op_token(e.syntax()).unwrap_or_default();
            let exprs = get_expr_children(e.syntax());
            let left = exprs
                .first()
                .map(|ex| lower_expr(ex, ctx))
                .unwrap_or(Expr::Error { span });
            let right = exprs
                .get(1)
                .map(|ex| lower_expr(ex, ctx))
                .unwrap_or(Expr::Error { span });
            desugar_binary_op_str(span, &op_text, left, right, ctx)
        }
        ast::Expr::AssignExpr(e) => {
            // The CST AssignExpr represents `name = value; body` (assign-then-bind).
            // But the grammar `bp(1) '=' value:Expr ';' body:Expr` means:
            // left-hand side is an Expr before the `=` token, value is the RHS Expr after `=`,
            // body is after the `;`.
            // Bug: value() and body() both return the same first Expr child.
            // Walk children directly.
            let span = e.syntax().span;
            let exprs = get_expr_children(e.syntax());
            // exprs[0] = LHS (name or pattern), exprs[1] = value, exprs[2] = body
            let name = exprs
                .first()
                .and_then(|ex| match ex {
                    ast::Expr::IdentExpr(id) => id.name().map(|t| t.text.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "_".into());
            let value = exprs
                .get(1)
                .map(|ex| lower_expr(ex, ctx))
                .unwrap_or(Expr::Error { span });
            let body = exprs
                .get(2)
                .map(|ex| lower_expr(ex, ctx))
                .unwrap_or(Expr::Error { span });
            Expr::Let {
                name,
                value: Box::new(value),
                body: Box::new(body),
                span,
            }
        }
    }
}

fn lower_bundle_entry(entry: &ast::BundleEntry, ctx: &mut LowerCtx) -> BundleEntry {
    let name = entry.name().map(|t| t.text.clone()).unwrap_or_default();
    let span = entry.syntax().span;
    let params = entry
        .param_list()
        .map(|pl| lower_param_list(&pl, None))
        .unwrap_or_default();
    let body = match entry.body() {
        Some(ast::FnBody::BlockExpr(block)) => maybe_produce(lower_block_expr(&block, ctx), span),
        Some(ast::FnBody::ExprBody(expr_body)) => {
            let e = expr_body
                .body()
                .map(|ex| lower_expr(&ex, ctx))
                .unwrap_or(Expr::Error { span });
            maybe_produce(e, span)
        }
        None => Expr::Error { span },
    };
    BundleEntry { name, params, body, span }
}

fn lower_if_else(e: &ast::IfElseExpr, ctx: &mut LowerCtx, span: Span) -> Expr {
    let cond = e
        .condition()
        .map(|c| lower_expr(&c, ctx))
        .unwrap_or(Expr::Error { span });

    let then_body = e
        .then_body()
        .map(|b| maybe_produce(lower_block_expr(&b, ctx), span))
        .unwrap_or(Expr::Error { span });

    let else_body = match e.else_clause() {
        Some(ast::ElseClause::BlockExpr(block)) => {
            maybe_produce(lower_block_expr(&block, ctx), span)
        }
        Some(ast::ElseClause::IfElseExpr(if_else)) => lower_if_else(&if_else, ctx, span),
        None => Expr::Produce {
            expr: Box::new(Expr::Ident { name: "unit".into(), span }),
            span,
        },
    };

    Expr::Match {
        scrutinee: Box::new(cond),
        arms: vec![
            MatchArm {
                pattern: Pattern::Ctor { name: "true".into(), args: vec![] },
                body: then_body,
                span,
            },
            MatchArm {
                pattern: Pattern::Ctor { name: "false".into(), args: vec![] },
                body: else_body,
                span,
            },
        ],
        span,
    }
}

// ---------------------------------------------------------------------------
// Pattern helpers
// ---------------------------------------------------------------------------

fn pattern_to_str(pat: &ast::Pattern) -> String {
    match pat {
        ast::Pattern::VariantPattern(vp) => {
            let name = vp.name().map(|t| t.text.clone()).unwrap_or_default();
            if let Some(fields) = vp.fields() {
                let field_strs: Vec<String> = fields.tail().map(|p| pattern_to_str(&p)).collect();
                if field_strs.is_empty() {
                    format!(".{}", name)
                } else {
                    format!(".{}({})", name, field_strs.join(", "))
                }
            } else {
                format!(".{}", name)
            }
        }
        ast::Pattern::BindPattern(bp) => {
            bp.name().map(|t| t.text.clone()).unwrap_or_else(|| "_".into())
        }
        ast::Pattern::WildcardPattern(_) => "_".into(),
    }
}

// ---------------------------------------------------------------------------
// Operator desugaring
// ---------------------------------------------------------------------------

fn bool_expr(span: Span, val: bool) -> Expr {
    let variant = if val { "true" } else { "false" };
    Expr::Member {
        object: Box::new(Expr::Ident { name: "Bool".into(), span }),
        member: variant.into(),
        span,
    }
}

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

fn desugar_binary_call(span: Span, cap_name: &str, method_name: &str, left: Expr, right: Expr) -> Expr {
    let perform = Expr::Perform { cap: cap_name.to_owned(), span };
    let member = Expr::Member { object: Box::new(perform), member: method_name.to_owned(), span };
    Expr::Call { callee: Box::new(member), args: vec![left, right], span }
}

fn desugar_unary_call(span: Span, cap_name: &str, method_name: &str, operand: Expr) -> Expr {
    let perform = Expr::Perform { cap: cap_name.to_owned(), span };
    let member = Expr::Member { object: Box::new(perform), member: method_name.to_owned(), span };
    Expr::Call { callee: Box::new(member), args: vec![operand], span }
}

fn desugar_binary_op_str(span: Span, op: &str, left: Expr, right: Expr, ctx: &mut LowerCtx) -> Expr {
    match op {
        "+" => desugar_binary_call(span, "Add", "add", left, right),
        "-" => desugar_binary_call(span, "Sub", "sub", left, right),
        "*" => desugar_binary_call(span, "Mul", "mul", left, right),
        "/" => desugar_binary_call(span, "Div", "div", left, right),
        "%" => desugar_binary_call(span, "Mod", "mod_", left, right),
        "==" => desugar_binary_call(span, "PartialEq", "eq", left, right),
        "&&" => desugar_binary_call(span, "Bool", "and", left, right),
        "||" => desugar_binary_call(span, "Bool", "or", left, right),
        "!=" => {
            let eq_call = desugar_binary_call(span, "PartialEq", "eq", left, right);
            desugar_negate_bool(span, eq_call)
        }
        "<" => {
            let cmp = desugar_binary_call(span, "PartialOrd", "cmp", left, right);
            desugar_ordering_match(span, cmp, true, false, false)
        }
        "<=" => {
            let cmp = desugar_binary_call(span, "PartialOrd", "cmp", left, right);
            desugar_ordering_match(span, cmp, true, true, false)
        }
        ">" => {
            let cmp = desugar_binary_call(span, "PartialOrd", "cmp", left, right);
            desugar_ordering_match(span, cmp, false, false, true)
        }
        ">=" => {
            let cmp = desugar_binary_call(span, "PartialOrd", "cmp", left, right);
            desugar_ordering_match(span, cmp, false, true, true)
        }
        _ => {
            ctx.errors.push(crate::HirError {
                span,
                message: format!("unknown binary operator: `{op}`"),
            });
            Expr::Error { span }
        }
    }
}
