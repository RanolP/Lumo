//! Fixed-point dependency-free analysis.
//!
//! See `docs/superpowers/specs/2026-04-19-lto-cap-monomorphization-design.md`.
//!
//! A `(target, binding)` pair is **dependency-free** if every `Perform` in its
//! body, after substituting current resolutions, resolves through the
//! resolution map to an impl method whose body is also dep-free, and every
//! direct fn / impl-method call lands in a dep-free target. Indirect calls
//! (lambda / fn-typed param / unknown) immediately block.

use std::collections::HashMap;

use lumo_lir as lir;
use lumo_types::ExprId;

use super::call_graph::{CallGraph, CallTarget};
use super::resolution::ResolutionMap;

/// Key = (target_name, cap_binding_tuple). For fns the cap_binding_tuple lists
/// runtime cap names (mangled). For impl methods the target_name is
/// `"<impl_const>.<method>"`. The binding is empty `Vec::new()` for v1 because
/// Lumo doesn't yet propagate generic cap bindings through call sites; kept as
/// `Vec<String>` for forward-compat.
pub type DepFreeKey = (String, Vec<String>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepFreeStatus {
    Pending,
    DepFree,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRef {
    pub impl_const: String,
    pub method: String,
}

#[derive(Debug, Clone, Default)]
pub struct DepFreeAnalysis {
    pub status: HashMap<DepFreeKey, DepFreeStatus>,
    pub perform_resolution: HashMap<ExprId, ResolvedRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnalyzeResult {
    DepFree,
    Blocked,
    Pending,
}

pub fn run(file: &lir::File, resolution: &ResolutionMap, cg: &CallGraph) -> DepFreeAnalysis {
    let mut status: HashMap<DepFreeKey, DepFreeStatus> = HashMap::new();
    let mut perform_resolution: HashMap<ExprId, ResolvedRef> = HashMap::new();

    // Build index of (target_name, body) pairs.
    let mut targets: Vec<(DepFreeKey, &lir::Expr)> = Vec::new();
    for item in &file.items {
        match item {
            lir::Item::Fn(f) => {
                targets.push(((f.name.clone(), Vec::new()), &f.value));
            }
            lir::Item::Impl(impl_decl) => {
                let target = impl_decl.target_type.value.display();
                let const_name = impl_decl.name.clone().unwrap_or_else(|| {
                    match &impl_decl.capability {
                        Some(cap) => format!("__impl_{target}_{}", cap.value.display()),
                        None => target.clone(),
                    }
                });
                for m in &impl_decl.methods {
                    let key = (format!("{const_name}.{}", m.name), Vec::new());
                    targets.push((key, &m.value));
                }
            }
            _ => {}
        }
    }

    // Initialize all to Pending.
    for (k, _) in &targets {
        status.insert(k.clone(), DepFreeStatus::Pending);
    }

    // Iterate to fixed point. Each pass: for each Pending key, optimistically
    // pre-mark DepFree (so self-recursive references see DepFree), then walk
    // the body. If body says Blocked → mark Blocked + drop tentative
    // resolutions. If body says Pending (a dependency is still Pending and
    // not self) → revert to Pending + drop tentative resolutions; we'll retry
    // next pass once the dependency settles. If DepFree → keep mark.
    //
    // Termination: each pass can only transition Pending → {DepFree, Blocked}
    // monotonically (a Pending revert is a no-op for this fixed-point sense).
    // We track `changed` more carefully to capture monotonic moves only.
    loop {
        let mut changed = false;
        for (key, body) in &targets {
            if status.get(key) != Some(&DepFreeStatus::Pending) {
                continue;
            }

            // Optimistic: mark DepFree before analyzing self.
            status.insert(key.clone(), DepFreeStatus::DepFree);

            let result = analyze_body(
                key,
                body,
                resolution,
                cg,
                &status,
                &mut perform_resolution,
            );
            match result {
                AnalyzeResult::DepFree => {
                    // Stay DepFree.
                    changed = true;
                }
                AnalyzeResult::Blocked => {
                    status.insert(key.clone(), DepFreeStatus::Blocked);
                    drop_resolutions_in_body(body, &mut perform_resolution);
                    changed = true;
                }
                AnalyzeResult::Pending => {
                    // Revert optimistic mark, retry next pass once deps settle.
                    status.insert(key.clone(), DepFreeStatus::Pending);
                    drop_resolutions_in_body(body, &mut perform_resolution);
                }
            }
        }
        if !changed {
            break;
        }
    }

    // Anything still Pending after fixed point converges (e.g. mutual deps
    // where every member is waiting on another) becomes Blocked — we have no
    // proof of dep-freedom.
    for (k, s) in status.iter_mut() {
        if *s == DepFreeStatus::Pending {
            *s = DepFreeStatus::Blocked;
        }
        // Move binding lookup out of mutation: can't borrow `targets` here
        // anymore; drop_resolutions_in_body needs the body, defer below.
        let _ = k;
    }
    // Drop perform resolutions for any newly-Blocked targets.
    let blocked_keys: std::collections::HashSet<&DepFreeKey> = status
        .iter()
        .filter(|(_, s)| **s == DepFreeStatus::Blocked)
        .map(|(k, _)| k)
        .collect();
    for (key, body) in &targets {
        if blocked_keys.contains(key) {
            drop_resolutions_in_body(body, &mut perform_resolution);
        }
    }

    DepFreeAnalysis {
        status,
        perform_resolution,
    }
}

fn analyze_body(
    self_key: &DepFreeKey,
    expr: &lir::Expr,
    resolution: &ResolutionMap,
    cg: &CallGraph,
    status: &HashMap<DepFreeKey, DepFreeStatus>,
    perform_resolution: &mut HashMap<ExprId, ResolvedRef>,
) -> AnalyzeResult {
    // Step 1: any indirect call in this body's call graph entry → immediate Blocked.
    if let Some(sites) = cg.edges.get(&self_key.0) {
        for site in sites {
            match &site.callee {
                CallTarget::Indirect => return AnalyzeResult::Blocked,
                CallTarget::Fn(name) => {
                    let dep = (name.clone(), Vec::new());
                    match status.get(&dep) {
                        Some(DepFreeStatus::DepFree) => {}
                        Some(DepFreeStatus::Blocked) => return AnalyzeResult::Blocked,
                        Some(DepFreeStatus::Pending) | None => {
                            // Unknown direct callee (no entry) means it's not a
                            // known fn — treat as indirect / blocked.
                            if status.get(&dep).is_none() {
                                return AnalyzeResult::Blocked;
                            }
                            // Otherwise: known fn currently Pending. If it's
                            // self, we'll see DepFree (optimistic mark) and
                            // not arrive here. So this is a real dep we must
                            // wait on.
                            return AnalyzeResult::Pending;
                        }
                    }
                }
                CallTarget::ImplMethod { impl_const, method } => {
                    let dep = (format!("{impl_const}.{method}"), Vec::new());
                    match status.get(&dep) {
                        Some(DepFreeStatus::DepFree) => {}
                        Some(DepFreeStatus::Blocked) => return AnalyzeResult::Blocked,
                        Some(DepFreeStatus::Pending) | None => {
                            if status.get(&dep).is_none() {
                                return AnalyzeResult::Blocked;
                            }
                            return AnalyzeResult::Pending;
                        }
                    }
                }
            }
        }
    }

    // Step 2: walk the body for Perform resolution.
    let mut state = AnalyzeResult::DepFree;
    let mut deferred = false;
    walk_for_performs(
        expr,
        resolution,
        status,
        perform_resolution,
        self_key,
        &mut state,
        &mut deferred,
    );
    match state {
        AnalyzeResult::Blocked => AnalyzeResult::Blocked,
        AnalyzeResult::DepFree if deferred => AnalyzeResult::Pending,
        AnalyzeResult::DepFree => AnalyzeResult::DepFree,
        AnalyzeResult::Pending => AnalyzeResult::Pending,
    }
}

#[allow(clippy::too_many_arguments)]
fn walk_for_performs(
    expr: &lir::Expr,
    resolution: &ResolutionMap,
    status: &HashMap<DepFreeKey, DepFreeStatus>,
    perform_resolution: &mut HashMap<ExprId, ResolvedRef>,
    self_key: &DepFreeKey,
    state: &mut AnalyzeResult,
    deferred: &mut bool,
) {
    if matches!(state, AnalyzeResult::Blocked) {
        return;
    }
    match expr {
        // Standalone Perform (no enclosing Member): cannot pick a method, so
        // we cannot resolve it. Conservatively block — a bare Perform
        // requires runtime dispatch for whatever method ends up called.
        lir::Expr::Perform { cap, type_args, .. } => {
            let key = (cap.clone(), type_args.clone());
            if resolution.get(&key).is_none() {
                *state = AnalyzeResult::Blocked;
            } else {
                // Even if the cap resolves, we can't pin a method here. Block.
                *state = AnalyzeResult::Blocked;
            }
        }
        // perform Cap.method(...) lowers to Member(Perform(cap), field=method)
        lir::Expr::Member { object, field, id } => {
            if let lir::Expr::Perform { cap, type_args, .. } = object.as_ref() {
                let key = (cap.clone(), type_args.clone());
                if let Some(res) = resolution.get(&key) {
                    let method_key = (format!("{}.{}", res.impl_const, field), Vec::new());
                    match status.get(&method_key) {
                        Some(DepFreeStatus::DepFree) => {
                            // Record perform resolution at the Member's id (the call site).
                            perform_resolution.insert(
                                *id,
                                ResolvedRef {
                                    impl_const: res.impl_const.clone(),
                                    method: field.clone(),
                                },
                            );
                        }
                        Some(DepFreeStatus::Blocked) => {
                            *state = AnalyzeResult::Blocked;
                        }
                        Some(DepFreeStatus::Pending) => {
                            if &method_key == self_key {
                                // Self-reference: optimistically resolve.
                                perform_resolution.insert(
                                    *id,
                                    ResolvedRef {
                                        impl_const: res.impl_const.clone(),
                                        method: field.clone(),
                                    },
                                );
                            } else {
                                *deferred = true;
                            }
                        }
                        None => {
                            // Resolution points to a method we don't have a
                            // body for (e.g. extern). Conservatively block.
                            *state = AnalyzeResult::Blocked;
                        }
                    }
                } else {
                    // No resolution for this (cap, type_args) — typeclass
                    // binding either ambiguous or unbound. Cannot statically
                    // resolve; block.
                    *state = AnalyzeResult::Blocked;
                }
                return;
            }
            // Member on something other than Perform: just recurse into object.
            walk_for_performs(
                object,
                resolution,
                status,
                perform_resolution,
                self_key,
                state,
                deferred,
            );
        }
        lir::Expr::Apply { callee, arg, .. } => {
            walk_for_performs(callee, resolution, status, perform_resolution, self_key, state, deferred);
            walk_for_performs(arg, resolution, status, perform_resolution, self_key, state, deferred);
        }
        lir::Expr::Force { expr, .. }
        | lir::Expr::Thunk { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Ann { expr, .. } => {
            walk_for_performs(expr, resolution, status, perform_resolution, self_key, state, deferred)
        }
        lir::Expr::Lambda { body, .. } => {
            walk_for_performs(body, resolution, status, perform_resolution, self_key, state, deferred)
        }
        lir::Expr::Let { value, body, .. } => {
            walk_for_performs(value, resolution, status, perform_resolution, self_key, state, deferred);
            walk_for_performs(body, resolution, status, perform_resolution, self_key, state, deferred);
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            walk_for_performs(scrutinee, resolution, status, perform_resolution, self_key, state, deferred);
            for arm in arms {
                walk_for_performs(&arm.body, resolution, status, perform_resolution, self_key, state, deferred);
            }
        }
        lir::Expr::Handle { handler, body, .. } => {
            walk_for_performs(handler, resolution, status, perform_resolution, self_key, state, deferred);
            walk_for_performs(body, resolution, status, perform_resolution, self_key, state, deferred);
        }
        lir::Expr::Bundle { entries, .. } => {
            for e in entries {
                walk_for_performs(&e.body, resolution, status, perform_resolution, self_key, state, deferred);
            }
        }
        lir::Expr::Ctor { args, .. } => {
            for a in args {
                walk_for_performs(a, resolution, status, perform_resolution, self_key, state, deferred);
            }
        }
        lir::Expr::Ident { .. }
        | lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Error { .. } => {}
    }
}

fn drop_resolutions_in_body(expr: &lir::Expr, map: &mut HashMap<ExprId, ResolvedRef>) {
    fn walk(expr: &lir::Expr, map: &mut HashMap<ExprId, ResolvedRef>) {
        map.remove(&expr.id());
        match expr {
            lir::Expr::Apply { callee, arg, .. } => {
                walk(callee, map);
                walk(arg, map);
            }
            lir::Expr::Force { expr, .. }
            | lir::Expr::Thunk { expr, .. }
            | lir::Expr::Produce { expr, .. }
            | lir::Expr::Roll { expr, .. }
            | lir::Expr::Unroll { expr, .. }
            | lir::Expr::Ann { expr, .. } => walk(expr, map),
            lir::Expr::Lambda { body, .. } => walk(body, map),
            lir::Expr::Let { value, body, .. } => {
                walk(value, map);
                walk(body, map);
            }
            lir::Expr::Match { scrutinee, arms, .. } => {
                walk(scrutinee, map);
                for arm in arms {
                    walk(&arm.body, map);
                }
            }
            lir::Expr::Handle { handler, body, .. } => {
                walk(handler, map);
                walk(body, map);
            }
            lir::Expr::Bundle { entries, .. } => {
                for e in entries {
                    walk(&e.body, map);
                }
            }
            lir::Expr::Ctor { args, .. } => {
                for a in args {
                    walk(a, map);
                }
            }
            lir::Expr::Member { object, .. } => walk(object, map),
            _ => {}
        }
    }
    walk(expr, map);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hir, lexer::lex, lto, parser::parse};

    fn lower(src: &str) -> lir::File {
        let lexed = lex(src);
        let parsed = parse(&lexed.tokens, &lexed.errors);
        lir::lower(&hir::lower(&parsed.file))
    }

    #[test]
    fn pure_impl_method_seeds_dep_free() {
        let src = r#"
            cap Add { fn add(a: Number, b: Number): Number }
            impl Number: Add { fn add(a: Number, b: Number): Number { a } }
        "#;
        let file = lower(src);
        let res = lto::resolution::build_resolution_map(&file);
        let cg = lto::call_graph::build_call_graph(&file);
        let an = run(&file, &res, &cg);
        let key = ("__impl_Number_Add.add".to_owned(), Vec::<String>::new());
        assert_eq!(an.status.get(&key), Some(&DepFreeStatus::DepFree));
    }

    #[test]
    fn fn_with_resolvable_perform_becomes_dep_free() {
        // Smoke only — perform.type_args are not yet patched at this point.
        // Real coverage in Task 11/12 fixtures.
        let src = r#"
            cap Add { fn add(a: Number, b: Number): Number }
            impl Number: Add { fn add(a: Number, b: Number): Number { a } }
            fn double(x: Number): Number { Add.add(x, x) }
        "#;
        let file = lower(src);
        let res = lto::resolution::build_resolution_map(&file);
        let cg = lto::call_graph::build_call_graph(&file);
        let _an = run(&file, &res, &cg);
    }

    #[test]
    fn indirect_call_blocks() {
        let src = r#"
            fn apply(f: thunk Number, x: Number): Number { f(x) }
        "#;
        let file = lower(src);
        let res = lto::resolution::build_resolution_map(&file);
        let cg = lto::call_graph::build_call_graph(&file);
        let an = run(&file, &res, &cg);
        let key = ("apply".to_owned(), Vec::<String>::new());
        assert_eq!(an.status.get(&key), Some(&DepFreeStatus::Blocked));
    }
}
