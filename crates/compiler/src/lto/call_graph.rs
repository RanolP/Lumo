use std::collections::HashMap;

use lumo_lir as lir;
use lumo_span::Span;

use super::resolution::ResolutionMap;

#[derive(Debug, Clone)]
pub struct CallGraph {
    /// caller fn name (or "<impl_const>.<method>") → call sites in its body
    pub edges: HashMap<String, Vec<CallSite>>,
    /// callee fn name → caller names (for DCE reachability)
    pub callers: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallSite {
    pub callee: CallTarget,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallTarget {
    Fn(String),
    ImplMethod { impl_const: String, method: String },
    Indirect,
}

fn collect_apply_chain<'a>(expr: &'a lir::Expr) -> Option<(&'a lir::Expr, Vec<&'a lir::Expr>)> {
    let mut args = Vec::new();
    let mut cur = expr;
    while let lir::Expr::Apply { callee, arg, .. } = cur {
        args.push(arg.as_ref());
        cur = callee.as_ref();
    }
    if args.is_empty() {
        return None;
    }
    args.reverse();
    Some((cur, args))
}

/// Classifies whether `head` is a known fn / impl-method call or an
/// indirect/opaque callee. `Force(Ident("resume"))` returns `None` —
/// `resume` is a CPS plumbing construct (identity in the dep-free path)
/// and should not generate a call-graph edge that blocks dep-free analysis.
fn classify_callee(
    head: &lir::Expr,
    fn_names: &std::collections::HashSet<String>,
    resolution: Option<&ResolutionMap>,
) -> Option<CallTarget> {
    match head {
        lir::Expr::Force { expr, .. } => classify_callee(expr, fn_names, resolution),
        lir::Expr::Ident { name, .. } => {
            if name == "resume" {
                None
            } else if fn_names.contains(name) {
                Some(CallTarget::Fn(name.clone()))
            } else {
                Some(CallTarget::Indirect)
            }
        }
        lir::Expr::Member { object, field, .. } => {
            if let lir::Expr::Ident { name, .. } = object.as_ref() {
                Some(CallTarget::ImplMethod {
                    impl_const: name.clone(),
                    method: field.clone(),
                })
            } else if let lir::Expr::Perform { cap, type_args, .. } = object.as_ref() {
                // `Member(Perform(cap), method)` is a typeclass dispatch.
                // If the resolution map has a default impl for (cap, type_args),
                // classify as a direct ImplMethod call.
                if let Some(map) = resolution {
                    if let Some(res) = map.get(&(cap.clone(), type_args.clone())) {
                        return Some(CallTarget::ImplMethod {
                            impl_const: res.impl_const.clone(),
                            method: field.clone(),
                        });
                    }
                }
                Some(CallTarget::Indirect)
            } else {
                Some(CallTarget::Indirect)
            }
        }
        _ => Some(CallTarget::Indirect),
    }
}

fn walk_expr(
    expr: &lir::Expr,
    file: &lir::File,
    fn_names: &std::collections::HashSet<String>,
    resolution: Option<&ResolutionMap>,
    out: &mut Vec<CallSite>,
) {
    // Record this Apply chain (if it's the head of one).
    if let Some((head, _args)) = collect_apply_chain(expr) {
        if let Some(callee) = classify_callee(head, fn_names, resolution) {
            out.push(CallSite {
                callee,
                span: file.span_of(expr.id()),
            });
        }
    }
    // Record zero-arg calls: `Force(Ident(name))` with no surrounding Apply.
    // These are top-level calls to zero-parameter functions (e.g. `sum()`).
    if let lir::Expr::Force { id, expr: inner } = expr {
        if let lir::Expr::Ident { name, .. } = inner.as_ref() {
            if fn_names.contains(name) {
                out.push(CallSite {
                    callee: CallTarget::Fn(name.clone()),
                    span: file.span_of(*id),
                });
            }
        }
    }
    // Recurse into all subexpressions. Even if `expr` is itself an Apply chain,
    // we descend so nested calls (in args) are also captured.
    match expr {
        lir::Expr::Apply { callee, arg, .. } => {
            walk_expr(callee, file, fn_names, resolution, out);
            walk_expr(arg, file, fn_names, resolution, out);
        }
        lir::Expr::Force { expr, .. }
        | lir::Expr::Thunk { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Ann { expr, .. } => walk_expr(expr, file, fn_names, resolution, out),
        lir::Expr::Lambda { body, .. } => walk_expr(body, file, fn_names, resolution, out),
        lir::Expr::Let { value, body, .. } => {
            walk_expr(value, file, fn_names, resolution, out);
            walk_expr(body, file, fn_names, resolution, out);
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            walk_expr(scrutinee, file, fn_names, resolution, out);
            for arm in arms {
                walk_expr(&arm.body, file, fn_names, resolution, out);
            }
        }
        lir::Expr::Handle { handler, body, .. } => {
            walk_expr(handler, file, fn_names, resolution, out);
            walk_expr(body, file, fn_names, resolution, out);
        }
        lir::Expr::Bundle { entries, .. } => {
            for e in entries {
                walk_expr(&e.body, file, fn_names, resolution, out);
            }
        }
        lir::Expr::Ctor { args, .. } => {
            for a in args {
                walk_expr(a, file, fn_names, resolution, out);
            }
        }
        lir::Expr::Member { object, .. } => walk_expr(object, file, fn_names, resolution, out),
        lir::Expr::Perform { .. }
        | lir::Expr::Ident { .. }
        | lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Error { .. } => {}
    }
}

pub fn build_call_graph(file: &lir::File) -> CallGraph {
    build_call_graph_with_resolution(file, None)
}

/// Build the call graph with an optional resolution map. When provided,
/// `Member(Perform(cap), method)` chains whose `(cap, type_args)` resolves
/// in the map are classified as `ImplMethod` instead of `Indirect`.
pub fn build_call_graph_with_resolution(
    file: &lir::File,
    resolution: Option<&ResolutionMap>,
) -> CallGraph {
    let fn_names: std::collections::HashSet<String> = file
        .items
        .iter()
        .filter_map(|item| match item {
            lir::Item::Fn(f) => Some(f.name.clone()),
            lir::Item::ExternFn(e) => Some(e.name.clone()),
            _ => None,
        })
        .collect();

    let mut edges: HashMap<String, Vec<CallSite>> = HashMap::new();
    let mut callers: HashMap<String, Vec<String>> = HashMap::new();

    for item in &file.items {
        let lir::Item::Fn(f) = item else { continue };
        let mut sites = Vec::new();
        walk_expr(&f.value, file, &fn_names, resolution, &mut sites);
        // Dedupe consecutive identical sites (the walker may visit Apply sub-chains).
        sites.dedup();
        for cs in &sites {
            if let CallTarget::Fn(callee) = &cs.callee {
                callers
                    .entry(callee.clone())
                    .or_default()
                    .push(f.name.clone());
            }
        }
        edges.insert(f.name.clone(), sites);
    }

    // Also walk impl method bodies — clones may be invoked through them.
    for item in &file.items {
        let lir::Item::Impl(impl_decl) = item else {
            continue;
        };
        let target = impl_decl.target_type.value.display();
        let const_name = impl_decl.name.clone().unwrap_or_else(|| {
            match &impl_decl.capability {
                Some(cap) => format!("__impl_{target}_{}", cap.value.display()),
                None => target,
            }
        });
        for method in &impl_decl.methods {
            let key = format!("{const_name}.{}", method.name);
            let mut sites = Vec::new();
            walk_expr(&method.value, file, &fn_names, resolution, &mut sites);
            sites.dedup();
            for cs in &sites {
                if let CallTarget::Fn(callee) = &cs.callee {
                    callers
                        .entry(callee.clone())
                        .or_default()
                        .push(key.clone());
                }
            }
            edges.insert(key, sites);
        }
    }

    CallGraph { edges, callers }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hir, lexer::lex, parser::parse};

    fn lower(src: &str) -> lir::File {
        let lexed = lex(src);
        let parsed = parse(&lexed.tokens, &lexed.errors);
        lir::lower(&hir::lower(&parsed.file))
    }

    #[test]
    fn direct_call_creates_edge() {
        let src = r#"
            fn helper(x: Number): Number { x }
            fn caller(): Number { helper(1) }
        "#;
        let file = lower(src);
        let cg = build_call_graph(&file);
        let edges = cg.edges.get("caller").expect("caller has edges");
        assert!(
            edges.iter().any(|cs| cs.callee == CallTarget::Fn("helper".to_owned())),
            "expected Fn(\"helper\") in caller's edges, got {:?}", edges
        );
        assert!(
            cg.callers.get("helper").map(|v| v.contains(&"caller".to_owned())).unwrap_or(false),
            "expected helper to have caller \"caller\""
        );
    }

    #[test]
    fn indirect_call_via_param_is_marked_indirect() {
        // A Force(Ident("f")) where "f" is a param (not a known fn) → Indirect.
        let src = r#"
            fn apply(f: thunk Number, x: Number): Number { f(x) }
        "#;
        let file = lower(src);
        let cg = build_call_graph(&file);
        let edges = cg.edges.get("apply").expect("apply has edges");
        assert!(
            edges.iter().any(|cs| matches!(cs.callee, CallTarget::Indirect)),
            "expected Indirect call site, got {:?}", edges
        );
    }
}
