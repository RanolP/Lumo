use std::collections::HashMap;

use lumo_lir as lir;
use lumo_types::ExprId;

use super::dep_free::{DepFreeAnalysis, DepFreeStatus, ResolvedRef};

pub fn transform(file: &mut lir::File, analysis: &DepFreeAnalysis) {
    // Step 1: identify dep-free fns under empty binding (v1).
    let dep_free_fns: Vec<String> = analysis
        .status
        .iter()
        .filter(|(_, s)| matches!(**s, DepFreeStatus::DepFree))
        .filter_map(|((name, args), _)| {
            // Skip impl methods (their key contains '.') — only top-level fns.
            if args.is_empty() && !name.contains('.') {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect();

    if dep_free_fns.is_empty() {
        return;
    }

    // Step 2: build clones with rewritten Performs.
    let mut new_items: Vec<lir::Item> = Vec::new();
    let mut clone_names: HashMap<String, String> = HashMap::new();

    let items_snapshot: Vec<lir::Item> = file.items.clone();
    for item in &items_snapshot {
        let lir::Item::Fn(f) = item else { continue };
        if !dep_free_fns.contains(&f.name) {
            continue;
        }
        // Skip fns whose body has no resolvable Perform — cloning them would
        // produce a bit-identical duplicate. The dep-free analysis marks
        // even pure fns as DepFree, but there's nothing to monomorphize.
        if !body_has_resolution(&f.value, &analysis.perform_resolution) {
            continue;
        }

        let clone_name = format!("{}__lto", f.name);
        let mut cloned = f.clone();
        cloned.name = clone_name.clone();
        cloned.cap = None;
        rewrite_performs(&mut cloned.value, &analysis.perform_resolution, &mut file.spans);
        new_items.push(lir::Item::Fn(cloned));
        clone_names.insert(f.name.clone(), clone_name);
    }

    file.items.extend(new_items);

    // Step 3: redirect call sites in non-cloned fn bodies.
    for item in file.items.iter_mut() {
        if let lir::Item::Fn(f) = item {
            // Don't rewrite inside the clone itself (its calls were already
            // covered by rewrite_performs; ident-renaming would only matter for
            // mutual-recursion cases, deferred to follow-up).
            if clone_names.values().any(|cn| cn == &f.name) {
                continue;
            }
            redirect_calls(&mut f.value, &clone_names);
        }
    }
}

fn rewrite_performs(
    expr: &mut lir::Expr,
    resolutions: &HashMap<ExprId, ResolvedRef>,
    spans: &mut Vec<lumo_span::Span>,
) {
    rewrite_walk(expr, resolutions, spans);
}

fn rewrite_walk(
    expr: &mut lir::Expr,
    resolutions: &HashMap<ExprId, ResolvedRef>,
    spans: &mut Vec<lumo_span::Span>,
) {
    // For `Member { id, object: Perform(cap), field: method }` whose id has a
    // resolution: replace with `Member { id, object: Ident(impl_const), field: method }`.
    if let lir::Expr::Member { id, object, field: _ } = expr {
        if let lir::Expr::Perform { .. } = object.as_ref() {
            if let Some(res) = resolutions.get(id) {
                let span = spans[id.0 as usize];
                let new_id_ident = alloc_id(spans, span);
                let new_id_member = alloc_id(spans, span);
                *expr = lir::Expr::Member {
                    id: new_id_member,
                    object: Box::new(lir::Expr::Ident {
                        id: new_id_ident,
                        name: res.impl_const.clone(),
                    }),
                    field: res.method.clone(),
                };
                return;
            }
        }
    }
    match expr {
        lir::Expr::Member { object, .. } => rewrite_walk(object, resolutions, spans),
        lir::Expr::Apply { callee, arg, .. } => {
            rewrite_walk(callee, resolutions, spans);
            rewrite_walk(arg, resolutions, spans);
        }
        lir::Expr::Force { expr, .. }
        | lir::Expr::Thunk { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Ann { expr, .. } => rewrite_walk(expr, resolutions, spans),
        lir::Expr::Lambda { body, .. } => rewrite_walk(body, resolutions, spans),
        lir::Expr::Let { value, body, .. } => {
            rewrite_walk(value, resolutions, spans);
            rewrite_walk(body, resolutions, spans);
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            rewrite_walk(scrutinee, resolutions, spans);
            for arm in arms {
                rewrite_walk(&mut arm.body, resolutions, spans);
            }
        }
        lir::Expr::Handle { handler, body, .. } => {
            rewrite_walk(handler, resolutions, spans);
            rewrite_walk(body, resolutions, spans);
        }
        lir::Expr::Bundle { entries, .. } => {
            for e in entries {
                rewrite_walk(&mut e.body, resolutions, spans);
            }
        }
        lir::Expr::Ctor { args, .. } => {
            for a in args {
                rewrite_walk(a, resolutions, spans);
            }
        }
        lir::Expr::Perform { .. }
        | lir::Expr::Ident { .. }
        | lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Error { .. } => {}
    }
}

fn redirect_calls(expr: &mut lir::Expr, clone_names: &HashMap<String, String>) {
    // Walk; if the inner of a Force is `Ident(name)` and `name ∈ clone_names`,
    // rename to the clone name.
    match expr {
        lir::Expr::Force { expr: inner, .. } => {
            if let lir::Expr::Ident { name, .. } = inner.as_mut() {
                if let Some(clone) = clone_names.get(name.as_str()) {
                    *name = clone.clone();
                }
            } else {
                redirect_calls(inner, clone_names);
            }
        }
        lir::Expr::Apply { callee, arg, .. } => {
            redirect_calls(callee, clone_names);
            redirect_calls(arg, clone_names);
        }
        lir::Expr::Thunk { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Ann { expr, .. } => redirect_calls(expr, clone_names),
        lir::Expr::Lambda { body, .. } => redirect_calls(body, clone_names),
        lir::Expr::Let { value, body, .. } => {
            redirect_calls(value, clone_names);
            redirect_calls(body, clone_names);
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            redirect_calls(scrutinee, clone_names);
            for arm in arms {
                redirect_calls(&mut arm.body, clone_names);
            }
        }
        lir::Expr::Handle { handler, body, .. } => {
            redirect_calls(handler, clone_names);
            redirect_calls(body, clone_names);
        }
        lir::Expr::Bundle { entries, .. } => {
            for e in entries {
                redirect_calls(&mut e.body, clone_names);
            }
        }
        lir::Expr::Ctor { args, .. } => {
            for a in args {
                redirect_calls(a, clone_names);
            }
        }
        lir::Expr::Member { object, .. } => redirect_calls(object, clone_names),
        _ => {}
    }
}

fn alloc_id(spans: &mut Vec<lumo_span::Span>, span: lumo_span::Span) -> ExprId {
    let id = ExprId(spans.len() as u32);
    spans.push(span);
    id
}

fn body_has_resolution(expr: &lir::Expr, resolutions: &HashMap<ExprId, ResolvedRef>) -> bool {
    if resolutions.contains_key(&expr.id()) {
        return true;
    }
    match expr {
        lir::Expr::Apply { callee, arg, .. } => {
            body_has_resolution(callee, resolutions) || body_has_resolution(arg, resolutions)
        }
        lir::Expr::Force { expr, .. }
        | lir::Expr::Thunk { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Ann { expr, .. } => body_has_resolution(expr, resolutions),
        lir::Expr::Lambda { body, .. } => body_has_resolution(body, resolutions),
        lir::Expr::Let { value, body, .. } => {
            body_has_resolution(value, resolutions) || body_has_resolution(body, resolutions)
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            body_has_resolution(scrutinee, resolutions)
                || arms.iter().any(|a| body_has_resolution(&a.body, resolutions))
        }
        lir::Expr::Handle { handler, body, .. } => {
            body_has_resolution(handler, resolutions) || body_has_resolution(body, resolutions)
        }
        lir::Expr::Bundle { entries, .. } => {
            entries.iter().any(|e| body_has_resolution(&e.body, resolutions))
        }
        lir::Expr::Ctor { args, .. } => args.iter().any(|a| body_has_resolution(a, resolutions)),
        lir::Expr::Member { object, .. } => body_has_resolution(object, resolutions),
        lir::Expr::Perform { .. }
        | lir::Expr::Ident { .. }
        | lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Error { .. } => false,
    }
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
    fn empty_analysis_is_noop() {
        let src = "fn id(x: Number): Number { x }";
        let mut file = lower(src);
        let an = DepFreeAnalysis::default();
        let before = file.clone();
        transform(&mut file, &an);
        assert_eq!(file, before);
    }
}
