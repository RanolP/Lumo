use std::collections::HashMap;

use lumo_lir as lir;
use lumo_types::{ExprId, Pattern};

use super::dep_free::{DepFreeAnalysis, DepFreeStatus, ResolvedRef};
use super::resolution::ResolutionMap;

// ---------------------------------------------------------------------------
// Alpha-renaming for inlined bodies
// ---------------------------------------------------------------------------
//
// Multiple inlinings of the same impl method (or fn) in the same caller scope
// produce duplicate param names (e.g. `self`, `other`) which collide once the
// TS backend flattens IIFEs into the enclosing function body. The fix is to
// mint fresh `__lto_<name>_<N>` names at each inline site for both params and
// the body's internal Let/Lambda bindings.
//
// The counter is per `transform` invocation (not a process-global AtomicU64)
// so output is deterministic across repeated compilations.

#[derive(Default)]
struct AlphaCtx {
    counter: u64,
}

impl AlphaCtx {
    fn fresh(&mut self, base: &str) -> String {
        let n = self.counter;
        self.counter += 1;
        format!("__lto_{base}_{n}")
    }
}

/// Rename every binding (`Let.name`, `Lambda.param`, pattern bindings) inside
/// `expr` to a fresh unique name, and rewire in-scope references accordingly.
/// References to names introduced OUTSIDE `expr` (free vars) are untouched.
fn alpha_rename_bindings(expr: &mut lir::Expr, ctx: &mut AlphaCtx) {
    fn walk_pattern(
        pattern: &mut Pattern,
        map: &mut Vec<(String, String)>,
        ctx: &mut AlphaCtx,
    ) -> usize {
        match pattern {
            Pattern::Wildcard => 0,
            Pattern::Bind(name) => {
                let fresh = ctx.fresh(name);
                let old = std::mem::replace(name, fresh.clone());
                map.push((old, fresh));
                1
            }
            Pattern::Ctor { args, .. } => {
                let mut pushed = 0;
                for a in args {
                    pushed += walk_pattern(a, map, ctx);
                }
                pushed
            }
        }
    }

    fn walk(expr: &mut lir::Expr, map: &mut Vec<(String, String)>, ctx: &mut AlphaCtx) {
        match expr {
            lir::Expr::Ident { name, .. } => {
                if let Some((_, new)) = map.iter().rev().find(|(old, _)| old == name) {
                    *name = new.clone();
                }
            }
            lir::Expr::Let { name, value, body, .. } => {
                walk(value, map, ctx);
                let fresh = ctx.fresh(name);
                let old = std::mem::replace(name, fresh.clone());
                map.push((old, fresh));
                walk(body, map, ctx);
                map.pop();
            }
            lir::Expr::Lambda { param, body, .. } => {
                let fresh = ctx.fresh(param);
                let old = std::mem::replace(param, fresh.clone());
                map.push((old, fresh));
                walk(body, map, ctx);
                map.pop();
            }
            lir::Expr::Apply { callee, arg, .. } => {
                walk(callee, map, ctx);
                walk(arg, map, ctx);
            }
            lir::Expr::Force { expr, .. }
            | lir::Expr::Thunk { expr, .. }
            | lir::Expr::Produce { expr, .. }
            | lir::Expr::Roll { expr, .. }
            | lir::Expr::Unroll { expr, .. }
            | lir::Expr::Ann { expr, .. } => walk(expr, map, ctx),
            lir::Expr::Match { scrutinee, arms, .. } => {
                walk(scrutinee, map, ctx);
                for arm in arms {
                    let pushed = walk_pattern(&mut arm.pattern, map, ctx);
                    walk(&mut arm.body, map, ctx);
                    for _ in 0..pushed {
                        map.pop();
                    }
                }
            }
            lir::Expr::Handle { handler, body, .. } => {
                walk(handler, map, ctx);
                walk(body, map, ctx);
            }
            lir::Expr::Bundle { entries, .. } => {
                for e in entries {
                    walk(&mut e.body, map, ctx);
                }
            }
            lir::Expr::Ctor { args, .. } => {
                for a in args {
                    walk(a, map, ctx);
                }
            }
            lir::Expr::Member { object, .. } => walk(object, map, ctx),
            lir::Expr::Perform { .. }
            | lir::Expr::String { .. }
            | lir::Expr::Number { .. }
            | lir::Expr::Error { .. } => {}
        }
    }
    let mut map: Vec<(String, String)> = Vec::new();
    walk(expr, &mut map, ctx);
}

/// Rename free occurrences of the names in `map` inside `expr`. An occurrence
/// is "free" if it is NOT shadowed by a Let/Lambda/pattern binding above it
/// within `expr`. Used to rename param references in a method body AFTER the
/// outer `Lambda(p, ...)` wrappers have been stripped.
fn rename_free_idents(expr: &mut lir::Expr, map: &[(String, String)]) {
    fn walk_pattern(pattern: &Pattern, shadowed: &mut Vec<String>) -> usize {
        match pattern {
            Pattern::Wildcard => 0,
            Pattern::Bind(name) => {
                shadowed.push(name.clone());
                1
            }
            Pattern::Ctor { args, .. } => {
                let mut pushed = 0;
                for a in args {
                    pushed += walk_pattern(a, shadowed);
                }
                pushed
            }
        }
    }

    fn walk(expr: &mut lir::Expr, map: &[(String, String)], shadowed: &mut Vec<String>) {
        match expr {
            lir::Expr::Ident { name, .. } => {
                if shadowed.iter().rev().any(|s| s == name) {
                    return;
                }
                if let Some((_, new)) = map.iter().find(|(old, _)| old == name) {
                    *name = new.clone();
                }
            }
            lir::Expr::Let { name, value, body, .. } => {
                walk(value, map, shadowed);
                shadowed.push(name.clone());
                walk(body, map, shadowed);
                shadowed.pop();
            }
            lir::Expr::Lambda { param, body, .. } => {
                shadowed.push(param.clone());
                walk(body, map, shadowed);
                shadowed.pop();
            }
            lir::Expr::Apply { callee, arg, .. } => {
                walk(callee, map, shadowed);
                walk(arg, map, shadowed);
            }
            lir::Expr::Force { expr, .. }
            | lir::Expr::Thunk { expr, .. }
            | lir::Expr::Produce { expr, .. }
            | lir::Expr::Roll { expr, .. }
            | lir::Expr::Unroll { expr, .. }
            | lir::Expr::Ann { expr, .. } => walk(expr, map, shadowed),
            lir::Expr::Match { scrutinee, arms, .. } => {
                walk(scrutinee, map, shadowed);
                for arm in arms {
                    let pushed = walk_pattern(&arm.pattern, shadowed);
                    walk(&mut arm.body, map, shadowed);
                    for _ in 0..pushed {
                        shadowed.pop();
                    }
                }
            }
            lir::Expr::Handle { handler, body, .. } => {
                walk(handler, map, shadowed);
                walk(body, map, shadowed);
            }
            lir::Expr::Bundle { entries, .. } => {
                for e in entries {
                    walk(&mut e.body, map, shadowed);
                }
            }
            lir::Expr::Ctor { args, .. } => {
                for a in args {
                    walk(a, map, shadowed);
                }
            }
            lir::Expr::Member { object, .. } => walk(object, map, shadowed),
            lir::Expr::Perform { .. }
            | lir::Expr::String { .. }
            | lir::Expr::Number { .. }
            | lir::Expr::Error { .. } => {}
        }
    }
    let mut shadowed = Vec::new();
    walk(expr, map, &mut shadowed);
}

const INLINE_SIZE_THRESHOLD: usize = 16;

pub fn transform(file: &mut lir::File, analysis: &DepFreeAnalysis, resolution: &ResolutionMap) {
    // Step 1: identify dep-free fns under empty binding (v1).
    let mut dep_free_fns: Vec<String> = analysis
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
    // Sort to guarantee deterministic processing order regardless of HashMap
    // iteration order (which is randomized per process by Rust's default hasher).
    dep_free_fns.sort();

    if dep_free_fns.is_empty() {
        return;
    }

    // Heuristic D inputs: inline flag, body size, caller count.
    let caller_counts = count_callers(file);

    let mut to_inline: Vec<String> = Vec::new(); // C form
    let mut to_clone: Vec<String> = Vec::new(); // B form

    for fn_name in &dep_free_fns {
        let Some(decl) = file.items.iter().find_map(|i| match i {
            lir::Item::Fn(f) if &f.name == fn_name => Some(f),
            _ => None,
        }) else {
            continue;
        };

        let force_inline = decl.inline;
        let small = body_size(&decl.value) <= INLINE_SIZE_THRESHOLD;
        let single_caller = caller_counts.get(fn_name).copied().unwrap_or(0) == 1;
        let has_rewrite = body_has_resolution(&decl.value, &analysis.perform_resolution);

        if force_inline {
            // Always inline at call sites, even if the body has no rewrites.
            // Consistent with #[inline(always)] semantics.
            to_inline.push(fn_name.clone());
        } else if has_rewrite && small && single_caller {
            to_inline.push(fn_name.clone());
        } else if has_rewrite {
            to_clone.push(fn_name.clone());
        }
        // Otherwise: skip (no rewrite to apply, no inline-always hint).
    }

    // Apply clones first — clones rename callees in-place. Inlines replace
    // calls outright and drop the original fn from `file.items`.
    let mut ctx = AlphaCtx::default();
    apply_clones(file, &to_clone, analysis, resolution, &mut ctx);
    apply_inlines(file, &to_inline, analysis, resolution, &mut ctx);
}

fn apply_clones(
    file: &mut lir::File,
    fns: &[String],
    analysis: &DepFreeAnalysis,
    resolution: &ResolutionMap,
    ctx: &mut AlphaCtx,
) {
    if fns.is_empty() {
        return;
    }

    // For zero-caller fns (entry points, exported fns), rewrite in place
    // instead of cloning. A clone would be immediately dropped by DCE since
    // no caller references it.
    let caller_counts = count_callers(file);

    let mut new_items: Vec<lir::Item> = Vec::new();
    let mut clone_names: HashMap<String, String> = HashMap::new();

    // First pass: rewrite zero-caller fns in place.
    for item in file.items.iter_mut() {
        let lir::Item::Fn(f) = item else { continue };
        if !fns.contains(&f.name) {
            continue;
        }
        if caller_counts.get(&f.name).copied().unwrap_or(0) == 0 {
            // Zero callers: rewrite body in place and clear the cap annotation.
            rewrite_performs(
                &mut f.value,
                &analysis.perform_resolution,
                resolution,
                &mut file.spans,
                ctx,
            );
            f.cap = None;
            // No clone needed — nothing to redirect.
        }
    }

    // Second pass (snapshot): create clones for fns that DO have callers.
    let items_snapshot: Vec<lir::Item> = file.items.clone();
    for item in &items_snapshot {
        let lir::Item::Fn(f) = item else { continue };
        if !fns.contains(&f.name) {
            continue;
        }
        if caller_counts.get(&f.name).copied().unwrap_or(0) == 0 {
            // Already handled in place above.
            continue;
        }

        let clone_name = format!("{}__lto", f.name);
        let mut cloned = f.clone();
        cloned.name = clone_name.clone();
        cloned.cap = None;
        rewrite_performs(
            &mut cloned.value,
            &analysis.perform_resolution,
            resolution,
            &mut file.spans,
            ctx,
        );
        new_items.push(lir::Item::Fn(cloned));
        clone_names.insert(f.name.clone(), clone_name);
    }

    file.items.extend(new_items);

    // Redirect call sites in non-cloned fn bodies to point at the clones.
    for item in file.items.iter_mut() {
        if let lir::Item::Fn(f) = item {
            if clone_names.values().any(|cn| cn == &f.name) {
                continue;
            }
            redirect_calls(&mut f.value, &clone_names);
        }
    }
}

fn apply_inlines(
    file: &mut lir::File,
    fns: &[String],
    analysis: &DepFreeAnalysis,
    resolution: &ResolutionMap,
    ctx: &mut AlphaCtx,
) {
    if fns.is_empty() {
        return;
    }

    // Snapshot fn bodies (post-perform-rewrite) we'll inline.
    let mut bodies: HashMap<String, (Vec<lir::Param>, lir::Expr)> = HashMap::new();
    let items_snapshot = file.items.clone();
    for item in &items_snapshot {
        if let lir::Item::Fn(f) = item {
            if fns.contains(&f.name) {
                let mut body = f.value.clone();
                rewrite_performs(
                    &mut body,
                    &analysis.perform_resolution,
                    resolution,
                    &mut file.spans,
                    ctx,
                );
                bodies.insert(
                    f.name.clone(),
                    (f.params.clone(), strip_thunk_lambdas(body, f.params.len())),
                );
            }
        }
    }

    let inline_set: std::collections::HashSet<String> = fns.iter().cloned().collect();
    for item in file.items.iter_mut() {
        let lir::Item::Fn(f) = item else { continue };
        if inline_set.contains(&f.name) {
            continue;
        }
        inline_calls(&mut f.value, &bodies, &mut file.spans, ctx);
    }

    // Drop inlined fns themselves.
    file.items.retain(|item| match item {
        lir::Item::Fn(f) => !inline_set.contains(&f.name),
        _ => true,
    });
}

fn strip_thunk_lambdas(mut expr: lir::Expr, n_params: usize) -> lir::Expr {
    if let lir::Expr::Thunk { expr: inner, .. } = expr {
        expr = *inner;
    }
    for _ in 0..n_params {
        if let lir::Expr::Lambda { body, .. } = expr {
            expr = *body;
        } else {
            break;
        }
    }
    expr
}

fn inline_calls(
    expr: &mut lir::Expr,
    bodies: &HashMap<String, (Vec<lir::Param>, lir::Expr)>,
    spans: &mut Vec<lumo_span::Span>,
    ctx: &mut AlphaCtx,
) {
    if let Some((head_name, args)) = match_call_chain(expr) {
        if let Some((params, body)) = bodies.get(&head_name) {
            assert_eq!(
                params.len(),
                args.len(),
                "inline param/arg count mismatch for `{head_name}`: {} params vs {} args",
                params.len(),
                args.len()
            );
            let mut new_expr = body.clone();

            // Alpha-rename params (free after Lambda stripping) and internal
            // bindings so multiple inline sites don't collide in the caller's
            // flattened scope.
            let fresh_params: Vec<String> =
                params.iter().map(|p| ctx.fresh(&p.name)).collect();
            let param_renames: Vec<(String, String)> = params
                .iter()
                .zip(&fresh_params)
                .map(|(p, fresh)| (p.name.clone(), fresh.clone()))
                .collect();
            rename_free_idents(&mut new_expr, &param_renames);
            alpha_rename_bindings(&mut new_expr, ctx);

            for (fresh, a) in fresh_params.iter().zip(args.iter()).rev() {
                let span = spans[expr.id().0 as usize];
                let id = alloc_id(spans, span);
                new_expr = lir::Expr::Let {
                    id,
                    name: fresh.clone(),
                    value: Box::new(a.clone()),
                    body: Box::new(new_expr),
                };
            }
            *expr = new_expr;
            // Recurse on the substituted result — args themselves may be
            // calls to inline.
            inline_calls(expr, bodies, spans, ctx);
            return;
        }
    }
    match expr {
        lir::Expr::Apply { callee, arg, .. } => {
            inline_calls(callee, bodies, spans, ctx);
            inline_calls(arg, bodies, spans, ctx);
        }
        lir::Expr::Force { expr, .. }
        | lir::Expr::Thunk { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Ann { expr, .. } => inline_calls(expr, bodies, spans, ctx),
        lir::Expr::Lambda { body, .. } => inline_calls(body, bodies, spans, ctx),
        lir::Expr::Let { value, body, .. } => {
            inline_calls(value, bodies, spans, ctx);
            inline_calls(body, bodies, spans, ctx);
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            inline_calls(scrutinee, bodies, spans, ctx);
            for arm in arms {
                inline_calls(&mut arm.body, bodies, spans, ctx);
            }
        }
        lir::Expr::Handle { handler, body, .. } => {
            inline_calls(handler, bodies, spans, ctx);
            inline_calls(body, bodies, spans, ctx);
        }
        lir::Expr::Bundle { entries, .. } => {
            for e in entries {
                inline_calls(&mut e.body, bodies, spans, ctx);
            }
        }
        lir::Expr::Ctor { args, .. } => {
            for a in args {
                inline_calls(a, bodies, spans, ctx);
            }
        }
        lir::Expr::Member { object, .. } => inline_calls(object, bodies, spans, ctx),
        _ => {}
    }
}

fn match_call_chain(expr: &lir::Expr) -> Option<(String, Vec<lir::Expr>)> {
    let mut args = Vec::new();
    let mut cur = expr;
    while let lir::Expr::Apply { callee, arg, .. } = cur {
        args.push((**arg).clone());
        cur = callee;
    }
    args.reverse();
    // Accept both n-ary calls (`Apply…Force(Ident)`) and zero-arg calls
    // (`Force(Ident)` directly) so zero-param fns can be inlined too.
    if let lir::Expr::Force { expr: inner, .. } = cur {
        if let lir::Expr::Ident { name, .. } = inner.as_ref() {
            return Some((name.clone(), args));
        }
    }
    None
}

fn body_size(expr: &lir::Expr) -> usize {
    let mut count = 1;
    match expr {
        lir::Expr::Apply { callee, arg, .. } => count += body_size(callee) + body_size(arg),
        lir::Expr::Force { expr, .. }
        | lir::Expr::Thunk { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Ann { expr, .. } => count += body_size(expr),
        lir::Expr::Lambda { body, .. } => count += body_size(body),
        lir::Expr::Let { value, body, .. } => count += body_size(value) + body_size(body),
        lir::Expr::Match { scrutinee, arms, .. } => {
            count += body_size(scrutinee);
            for a in arms {
                count += body_size(&a.body);
            }
        }
        lir::Expr::Handle { handler, body, .. } => count += body_size(handler) + body_size(body),
        lir::Expr::Bundle { entries, .. } => {
            for e in entries {
                count += body_size(&e.body);
            }
        }
        lir::Expr::Ctor { args, .. } => {
            for a in args {
                count += body_size(a);
            }
        }
        lir::Expr::Member { object, .. } => count += body_size(object),
        _ => {}
    }
    count
}

fn count_callers(file: &lir::File) -> HashMap<String, usize> {
    let cg = super::call_graph::build_call_graph(file);
    let mut out: HashMap<String, usize> = HashMap::new();
    for sites in cg.edges.values() {
        for site in sites {
            if let super::call_graph::CallTarget::Fn(name) = &site.callee {
                *out.entry(name.clone()).or_default() += 1;
            }
        }
    }
    out
}

fn rewrite_performs(
    expr: &mut lir::Expr,
    resolutions: &HashMap<ExprId, ResolvedRef>,
    resolution: &ResolutionMap,
    spans: &mut Vec<lumo_span::Span>,
    ctx: &mut AlphaCtx,
) {
    rewrite_walk(expr, resolutions, resolution, spans, ctx);
}

fn rewrite_walk(
    expr: &mut lir::Expr,
    resolutions: &HashMap<ExprId, ResolvedRef>,
    resolution: &ResolutionMap,
    spans: &mut Vec<lumo_span::Span>,
    ctx: &mut AlphaCtx,
) {
    // Recursive monomorphization: match the Apply chain rooted at `expr`
    // whose callee chain terminates in `Member(Perform(cap, type_args),
    // method)` with a resolution. Inline the impl method body at that site,
    // binding each formal param to the corresponding apply-chain arg via a
    // Let-chain, and rewrite `resume(v)` → `v` since there is no handler
    // context in statically-dispatched LTO code.
    if try_inline_perform_call(expr, resolutions, resolution, spans, ctx) {
        // The inlined body may itself contain further Perform calls that
        // should be monomorphized — recurse on the replacement.
        rewrite_walk(expr, resolutions, resolution, spans, ctx);
        return;
    }
    match expr {
        lir::Expr::Member { object, .. } => rewrite_walk(object, resolutions, resolution, spans, ctx),
        lir::Expr::Apply { callee, arg, .. } => {
            rewrite_walk(callee, resolutions, resolution, spans, ctx);
            rewrite_walk(arg, resolutions, resolution, spans, ctx);
        }
        lir::Expr::Force { expr, .. }
        | lir::Expr::Thunk { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Ann { expr, .. } => rewrite_walk(expr, resolutions, resolution, spans, ctx),
        lir::Expr::Lambda { body, .. } => rewrite_walk(body, resolutions, resolution, spans, ctx),
        lir::Expr::Let { value, body, .. } => {
            rewrite_walk(value, resolutions, resolution, spans, ctx);
            rewrite_walk(body, resolutions, resolution, spans, ctx);
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            rewrite_walk(scrutinee, resolutions, resolution, spans, ctx);
            for arm in arms {
                rewrite_walk(&mut arm.body, resolutions, resolution, spans, ctx);
            }
        }
        lir::Expr::Handle { handler, body, .. } => {
            rewrite_walk(handler, resolutions, resolution, spans, ctx);
            rewrite_walk(body, resolutions, resolution, spans, ctx);
        }
        lir::Expr::Bundle { entries, .. } => {
            for e in entries {
                rewrite_walk(&mut e.body, resolutions, resolution, spans, ctx);
            }
        }
        lir::Expr::Ctor { args, .. } => {
            for a in args {
                rewrite_walk(a, resolutions, resolution, spans, ctx);
            }
        }
        lir::Expr::Perform { .. }
        | lir::Expr::Ident { .. }
        | lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Error { .. } => {}
    }
}

/// If `expr` is an Apply chain terminating in `Member(Perform(cap, type_args),
/// method)` with a resolution, replace it with the impl method's inlined body
/// and return true. The body is cloned, `resume(v)` is stripped to `v`, and
/// each formal param is bound to the corresponding arg via a Let wrapper.
fn try_inline_perform_call(
    expr: &mut lir::Expr,
    resolutions: &HashMap<ExprId, ResolvedRef>,
    resolution: &ResolutionMap,
    spans: &mut Vec<lumo_span::Span>,
    ctx: &mut AlphaCtx,
) -> bool {
    // Walk down the apply chain to collect args and find the terminal Member.
    let mut args: Vec<lir::Expr> = Vec::new();
    let mut cur: &lir::Expr = expr;
    while let lir::Expr::Apply { callee, arg, .. } = cur {
        args.push((**arg).clone());
        cur = callee;
    }
    args.reverse();

    let (member_id, perform_cap, perform_type_args, field) = match cur {
        lir::Expr::Member { id, object, field } => match object.as_ref() {
            lir::Expr::Perform { cap, type_args, .. } => {
                (*id, cap.clone(), type_args.clone(), field.clone())
            }
            _ => return false,
        },
        _ => return false,
    };

    // Must have a resolution recorded at the Member's id and a body in the
    // resolution map. (dep_free only records resolutions when the impl method
    // is itself dep-free, so this is a safe statically-dispatched target.)
    let resolved_ref = match resolutions.get(&member_id) {
        Some(r) => r.clone(),
        None => return false,
    };
    let impl_res = match resolution.get(&(perform_cap, perform_type_args)) {
        Some(r) => r,
        None => return false,
    };
    debug_assert_eq!(impl_res.impl_const, resolved_ref.impl_const);
    let method_info = match impl_res.methods.get(&field) {
        Some(m) => m,
        None => return false,
    };

    // Param count must match arg count; otherwise we'd emit ill-formed code.
    if method_info.params.len() != args.len() {
        return false;
    }

    // Clone the method body, strip outer `Thunk(Lambda(p1, Lambda(p2, ...)))`
    // to reach the raw body, then rewrite `resume(v) → v`.
    let stripped = strip_thunk_lambdas(method_info.body.clone(), method_info.params.len());
    let mut inlined = stripped;
    strip_resume(&mut inlined);

    // Alpha-rename everything so multiple inline sites don't collide. Param
    // references in `inlined` are FREE (the Lambda wrappers that bound them
    // were stripped above), so we rename them explicitly via a shadow-aware
    // pass. Internal bindings are renamed via the general alpha walk.
    let fresh_params: Vec<String> = method_info
        .params
        .iter()
        .map(|p| ctx.fresh(&p.name))
        .collect();
    let param_renames: Vec<(String, String)> = method_info
        .params
        .iter()
        .zip(&fresh_params)
        .map(|(p, fresh)| (p.name.clone(), fresh.clone()))
        .collect();
    rename_free_idents(&mut inlined, &param_renames);
    alpha_rename_bindings(&mut inlined, ctx);

    // Wrap in Let-chain binding fresh params to args in order (left-to-right).
    let span = spans[member_id.0 as usize];
    for (fresh, a) in fresh_params.iter().zip(args.into_iter()).rev() {
        let let_id = alloc_id(spans, span);
        inlined = lir::Expr::Let {
            id: let_id,
            name: fresh.clone(),
            value: Box::new(a),
            body: Box::new(inlined),
        };
    }

    *expr = inlined;
    true
}

/// Replace `Apply(Force(Ident("resume")), x)` with `x` throughout `expr`.
///
/// In inlined impl bodies under LTO's statically-dispatched path, there is
/// no surrounding handler: the method body runs directly, so `resume(v)`
/// reduces to `v` (the direct return of the method's result to its caller).
fn strip_resume(expr: &mut lir::Expr) {
    if let lir::Expr::Apply { callee, arg, .. } = expr {
        if let lir::Expr::Force { expr: inner, .. } = callee.as_ref() {
            if let lir::Expr::Ident { name, .. } = inner.as_ref() {
                if name == "resume" {
                    let arg_inner = std::mem::replace(
                        arg.as_mut(),
                        lir::Expr::Error { id: ExprId(0) },
                    );
                    *expr = arg_inner;
                    strip_resume(expr);
                    return;
                }
            }
        }
    }
    match expr {
        lir::Expr::Apply { callee, arg, .. } => {
            strip_resume(callee);
            strip_resume(arg);
        }
        lir::Expr::Force { expr, .. }
        | lir::Expr::Thunk { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Ann { expr, .. } => strip_resume(expr),
        lir::Expr::Lambda { body, .. } => strip_resume(body),
        lir::Expr::Let { value, body, .. } => {
            strip_resume(value);
            strip_resume(body);
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            strip_resume(scrutinee);
            for arm in arms {
                strip_resume(&mut arm.body);
            }
        }
        lir::Expr::Handle { handler, body, .. } => {
            strip_resume(handler);
            strip_resume(body);
        }
        lir::Expr::Bundle { entries, .. } => {
            for e in entries {
                strip_resume(&mut e.body);
            }
        }
        lir::Expr::Ctor { args, .. } => {
            for a in args {
                strip_resume(a);
            }
        }
        lir::Expr::Member { object, .. } => strip_resume(object),
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
        let res = crate::lto::resolution::build_resolution_map(&file);
        let before = file.clone();
        transform(&mut file, &an, &res);
        assert_eq!(file, before);
    }

    #[test]
    fn inline_always_attribute_forces_callsite_inline() {
        let src = r#"
            cap Add { fn add(a: Number, b: Number): Number }
            impl Number: Add { fn add(a: Number, b: Number): Number { a } }
            #[inline(always)]
            fn double(x: Number): Number { Add.add(x, x) }
            fn main(): Number { double(2) }
        "#;
        let mut file = lower(src);
        // Run real analysis pipeline (resolution → call graph → dep_free) so
        // that perform_resolution + DepFree status are correctly populated for
        // the #[inline(always)] candidate.
        let resolution = crate::lto::resolution::build_resolution_map(&file);
        let cg = crate::lto::call_graph::build_call_graph(&file);
        let mut an = crate::lto::dep_free::run(&file, &resolution, &cg);
        // The dep-free analysis depends on Perform.type_args being patched,
        // which only happens in the full lower_module pipeline. In this raw
        // test path type_args stay empty so `double`'s Perform of `Add`
        // doesn't resolve. Inject the DepFree status manually so the inline
        // path under #[inline(always)] is exercised here.
        an.status.insert(
            ("double".to_owned(), Vec::<String>::new()),
            DepFreeStatus::DepFree,
        );
        transform(&mut file, &an, &resolution);
        let has_double = file.items.iter().any(|i| matches!(i, lir::Item::Fn(f) if f.name == "double"));
        assert!(!has_double, "inline(always) fn should be removed after inlining");
        let has_double_clone = file.items.iter().any(|i| matches!(i, lir::Item::Fn(f) if f.name.starts_with("double__")));
        assert!(!has_double_clone, "inline(always) fn should not produce a clone");
    }
}
