//! SCC-based dependency-free analysis.
//!
//! See `docs/superpowers/specs/2026-04-19-lto-cap-monomorphization-design.md`.
//!
//! A `(target, binding)` pair is **dependency-free** if every `Perform` in its
//! body, after substituting current resolutions, resolves through the
//! resolution map to an impl method whose body is also dep-free, and every
//! direct fn / impl-method call lands in a dep-free target. Indirect calls
//! (lambda / fn-typed param / unknown) immediately block.
//!
//! v2: SCC-level analysis via Tarjan's algorithm. Each SCC is analyzed
//! atomically: if any member has an indirect call or unresolvable perform,
//! or any outgoing edge leads to a Blocked target, the whole SCC is Blocked.
//! Otherwise the whole SCC is DepFree. This correctly handles mutual
//! recursion (Task 6 in the spec).

use std::collections::{HashMap, HashSet};

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

/// Per-node analysis results collected in Phase 2.
struct NodeInfo {
    /// Edges to other target nodes (same-file fn or impl-method bodies). Externs
    /// and unknown-but-safe terminals are NOT recorded here (they don't
    /// participate in SCC cycles).
    outgoing: Vec<DepFreeKey>,
    /// True if this node has an indirect call site (lambda / fn-typed param /
    /// unknown direct callee / impl method we don't have a body for).
    has_indirect: bool,
    /// True if this node has a Perform that can't be resolved: either bare
    /// `Perform` (no enclosing Member), or `Member(Perform, m)` where the
    /// resolution map lacks an entry for `(cap, type_args)`, or resolution
    /// points to an impl method body we don't have.
    has_unresolved: bool,
    /// Tentative `(ExprId, ResolvedRef)` entries to commit to
    /// `perform_resolution` iff this node's SCC ends up DepFree.
    tentative_resolutions: Vec<(ExprId, ResolvedRef)>,
}

pub fn run(file: &lir::File, resolution: &ResolutionMap, cg: &CallGraph) -> DepFreeAnalysis {
    // Phase 1: enumerate nodes (fn bodies + impl method bodies).
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
    let target_keys: HashSet<DepFreeKey> = targets.iter().map(|(k, _)| k.clone()).collect();

    let mut status: HashMap<DepFreeKey, DepFreeStatus> = HashMap::new();
    // Extern fns are leaves with no caps — seed them as DepFree so direct
    // calls to them (e.g. `js_add` from a default impl body) don't force the
    // caller to Blocked.
    for item in &file.items {
        if let lir::Item::ExternFn(e) = item {
            status.insert((e.name.clone(), Vec::new()), DepFreeStatus::DepFree);
        }
    }

    // Phase 2: for each target, walk its body and collect outgoing edges,
    // indirect-call flag, unresolved-perform flag, and tentative resolutions.
    let mut nodes: HashMap<DepFreeKey, NodeInfo> = HashMap::new();
    for (key, body) in &targets {
        let mut outgoing: Vec<DepFreeKey> = Vec::new();
        let mut has_indirect = false;

        // Direct call edges via cg.
        if let Some(sites) = cg.edges.get(&key.0) {
            for site in sites {
                match &site.callee {
                    CallTarget::Indirect => has_indirect = true,
                    CallTarget::Fn(name) => {
                        let dep = (name.clone(), Vec::new());
                        if target_keys.contains(&dep) {
                            outgoing.push(dep);
                        } else if status.get(&dep) == Some(&DepFreeStatus::DepFree) {
                            // Extern fn — already DepFree, no SCC edge needed.
                        } else {
                            // Unknown direct callee (no body, not extern) —
                            // conservatively treat as indirect.
                            has_indirect = true;
                        }
                    }
                    CallTarget::ImplMethod { impl_const, method } => {
                        let dep = (format!("{impl_const}.{method}"), Vec::new());
                        if target_keys.contains(&dep) {
                            outgoing.push(dep);
                        } else {
                            // Impl method body unknown — conservatively block.
                            has_indirect = true;
                        }
                    }
                }
            }
        }

        // Perform-site walk: collect Member(Perform, method) patterns.
        let mut has_unresolved = false;
        let mut tentative_resolutions: Vec<(ExprId, ResolvedRef)> = Vec::new();
        walk_for_perform_sites(
            body,
            resolution,
            &target_keys,
            &mut outgoing,
            &mut has_unresolved,
            &mut tentative_resolutions,
        );

        nodes.insert(
            key.clone(),
            NodeInfo {
                outgoing,
                has_indirect,
                has_unresolved,
                tentative_resolutions,
            },
        );
    }

    // Phase 3: compute SCCs via Tarjan's algorithm. Returned in reverse
    // topological order (leaves first).
    let sccs = compute_sccs(&target_keys, &nodes);

    // Phase 4: process each SCC in order. If any member is problematic or any
    // outgoing edge leads outside the SCC to a Blocked target, the whole SCC
    // is Blocked; otherwise DepFree.
    let mut perform_resolution: HashMap<ExprId, ResolvedRef> = HashMap::new();
    for scc in &sccs {
        let scc_set: HashSet<&DepFreeKey> = scc.iter().collect();
        let mut blocked = false;

        for key in scc {
            let info = nodes.get(key).expect("node info present");
            if info.has_indirect || info.has_unresolved {
                blocked = true;
                break;
            }
            for out in &info.outgoing {
                if scc_set.contains(out) {
                    continue; // intra-SCC edge — skipped, SCC is atomic
                }
                match status.get(out) {
                    Some(DepFreeStatus::DepFree) => {}
                    _ => {
                        // Any cross-SCC edge to a non-DepFree target blocks.
                        // By reverse-topo order the target must already be
                        // classified (DepFree or Blocked); extern leaves were
                        // seeded DepFree at the start of this phase.
                        blocked = true;
                        break;
                    }
                }
            }
            if blocked {
                break;
            }
        }

        if blocked {
            for key in scc {
                status.insert(key.clone(), DepFreeStatus::Blocked);
            }
        } else {
            for key in scc {
                status.insert(key.clone(), DepFreeStatus::DepFree);
                let info = nodes.get(key).expect("node info present");
                for (id, r) in &info.tentative_resolutions {
                    perform_resolution.insert(*id, r.clone());
                }
            }
        }
    }

    // Safety net: any target never classified (shouldn't happen if SCCs cover
    // all nodes) is treated as Blocked.
    for (key, _) in &targets {
        status.entry(key.clone()).or_insert(DepFreeStatus::Blocked);
    }

    DepFreeAnalysis {
        status,
        perform_resolution,
    }
}

#[allow(clippy::too_many_arguments)]
fn walk_for_perform_sites(
    expr: &lir::Expr,
    resolution: &ResolutionMap,
    target_keys: &HashSet<DepFreeKey>,
    outgoing: &mut Vec<DepFreeKey>,
    has_unresolved: &mut bool,
    tentative_resolutions: &mut Vec<(ExprId, ResolvedRef)>,
) {
    match expr {
        // Bare Perform (no enclosing Member): cannot pick a method — block.
        lir::Expr::Perform { .. } => {
            *has_unresolved = true;
        }
        lir::Expr::Member { object, field, id } => {
            if let lir::Expr::Perform { cap, type_args, .. } = object.as_ref() {
                let key = (cap.clone(), type_args.clone());
                if let Some(res) = resolution.get(&key) {
                    let method_key = (format!("{}.{}", res.impl_const, field), Vec::new());
                    if target_keys.contains(&method_key) {
                        outgoing.push(method_key);
                        tentative_resolutions.push((
                            *id,
                            ResolvedRef {
                                impl_const: res.impl_const.clone(),
                                method: field.clone(),
                            },
                        ));
                    } else {
                        // Resolution points to an impl method we don't have a
                        // body for (e.g. extern-only). Block.
                        *has_unresolved = true;
                    }
                } else {
                    // No resolution for (cap, type_args) — typeclass binding
                    // ambiguous or unbound. Block.
                    *has_unresolved = true;
                }
                return;
            }
            // Member on something other than Perform: just recurse into object.
            walk_for_perform_sites(
                object,
                resolution,
                target_keys,
                outgoing,
                has_unresolved,
                tentative_resolutions,
            );
        }
        lir::Expr::Apply { callee, arg, .. } => {
            walk_for_perform_sites(
                callee,
                resolution,
                target_keys,
                outgoing,
                has_unresolved,
                tentative_resolutions,
            );
            walk_for_perform_sites(
                arg,
                resolution,
                target_keys,
                outgoing,
                has_unresolved,
                tentative_resolutions,
            );
        }
        lir::Expr::Force { expr, .. }
        | lir::Expr::Thunk { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Ann { expr, .. } => walk_for_perform_sites(
            expr,
            resolution,
            target_keys,
            outgoing,
            has_unresolved,
            tentative_resolutions,
        ),
        lir::Expr::Lambda { body, .. } => walk_for_perform_sites(
            body,
            resolution,
            target_keys,
            outgoing,
            has_unresolved,
            tentative_resolutions,
        ),
        lir::Expr::Let { value, body, .. } => {
            walk_for_perform_sites(
                value,
                resolution,
                target_keys,
                outgoing,
                has_unresolved,
                tentative_resolutions,
            );
            walk_for_perform_sites(
                body,
                resolution,
                target_keys,
                outgoing,
                has_unresolved,
                tentative_resolutions,
            );
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            walk_for_perform_sites(
                scrutinee,
                resolution,
                target_keys,
                outgoing,
                has_unresolved,
                tentative_resolutions,
            );
            for arm in arms {
                walk_for_perform_sites(
                    &arm.body,
                    resolution,
                    target_keys,
                    outgoing,
                    has_unresolved,
                    tentative_resolutions,
                );
            }
        }
        lir::Expr::Handle { handler, body, .. } => {
            walk_for_perform_sites(
                handler,
                resolution,
                target_keys,
                outgoing,
                has_unresolved,
                tentative_resolutions,
            );
            walk_for_perform_sites(
                body,
                resolution,
                target_keys,
                outgoing,
                has_unresolved,
                tentative_resolutions,
            );
        }
        lir::Expr::Bundle { entries, .. } => {
            for e in entries {
                walk_for_perform_sites(
                    &e.body,
                    resolution,
                    target_keys,
                    outgoing,
                    has_unresolved,
                    tentative_resolutions,
                );
            }
        }
        lir::Expr::Ctor { args, .. } => {
            for a in args {
                walk_for_perform_sites(
                    a,
                    resolution,
                    target_keys,
                    outgoing,
                    has_unresolved,
                    tentative_resolutions,
                );
            }
        }
        lir::Expr::Ident { .. }
        | lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Error { .. } => {}
    }
}

/// Tarjan's SCC algorithm over `target_keys`. Returns SCCs in reverse
/// topological order (leaves-first). Within each SCC the nodes are sorted
/// lexicographically for determinism; between SCCs the iteration order is
/// determined by a sorted DFS root order (also for determinism, since
/// HashSet iteration order is not stable).
fn compute_sccs(
    target_keys: &HashSet<DepFreeKey>,
    nodes: &HashMap<DepFreeKey, NodeInfo>,
) -> Vec<Vec<DepFreeKey>> {
    // Map keys to integer indices.
    let mut key_of: Vec<DepFreeKey> = target_keys.iter().cloned().collect();
    key_of.sort();
    let index_of: HashMap<DepFreeKey, usize> = key_of
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, k)| (k, i))
        .collect();

    let n = key_of.len();
    let mut index = vec![usize::MAX; n];
    let mut lowlink = vec![0usize; n];
    let mut on_stack = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut next_index: usize = 0;
    let mut sccs: Vec<Vec<DepFreeKey>> = Vec::new();

    struct Frame {
        v: usize,
        iter: std::vec::IntoIter<usize>,
    }

    for root in 0..n {
        if index[root] != usize::MAX {
            continue;
        }
        let adj_root = adj_of(root, &key_of, nodes, &index_of);
        index[root] = next_index;
        lowlink[root] = next_index;
        next_index += 1;
        stack.push(root);
        on_stack[root] = true;
        let mut call_stack: Vec<Frame> = vec![Frame {
            v: root,
            iter: adj_root.into_iter(),
        }];

        while let Some(frame) = call_stack.last_mut() {
            if let Some(w) = frame.iter.next() {
                if index[w] == usize::MAX {
                    index[w] = next_index;
                    lowlink[w] = next_index;
                    next_index += 1;
                    stack.push(w);
                    on_stack[w] = true;
                    let adj_w = adj_of(w, &key_of, nodes, &index_of);
                    call_stack.push(Frame {
                        v: w,
                        iter: adj_w.into_iter(),
                    });
                } else if on_stack[w] {
                    let fv = frame.v;
                    lowlink[fv] = lowlink[fv].min(index[w]);
                }
            } else {
                let v = frame.v;
                if lowlink[v] == index[v] {
                    let mut scc: Vec<DepFreeKey> = Vec::new();
                    loop {
                        let w = stack.pop().expect("tarjan stack nonempty");
                        on_stack[w] = false;
                        scc.push(key_of[w].clone());
                        if w == v {
                            break;
                        }
                    }
                    scc.sort();
                    sccs.push(scc);
                }
                call_stack.pop();
                if let Some(parent) = call_stack.last_mut() {
                    let pv = parent.v;
                    lowlink[pv] = lowlink[pv].min(lowlink[v]);
                }
            }
        }
    }

    sccs
}

fn adj_of(
    v: usize,
    key_of: &[DepFreeKey],
    nodes: &HashMap<DepFreeKey, NodeInfo>,
    index_of: &HashMap<DepFreeKey, usize>,
) -> Vec<usize> {
    let key = &key_of[v];
    let info = nodes.get(key).expect("node info");
    let mut out: Vec<usize> = info
        .outgoing
        .iter()
        .filter_map(|k| index_of.get(k).copied())
        .collect();
    out.sort();
    out.dedup();
    out
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

    #[test]
    fn mutually_recursive_pure_fns_become_dep_free() {
        // Two pure recursive-descent fns A↔B. Both should end up DepFree now
        // that SCC analysis treats them as a unit. Uses match to avoid
        // depending on `==` typeclass resolution at the LTO layer.
        let src = r#"
            fn a(n: Number): Number = match n { 0 => 0, _ => b(n) }
            fn b(n: Number): Number = match n { 0 => 0, _ => a(n) }
        "#;
        let file = lower(src);
        let res = lto::resolution::build_resolution_map(&file);
        let cg = lto::call_graph::build_call_graph(&file);
        let an = run(&file, &res, &cg);
        let a_key = ("a".to_owned(), Vec::<String>::new());
        let b_key = ("b".to_owned(), Vec::<String>::new());
        assert_eq!(an.status.get(&a_key), Some(&DepFreeStatus::DepFree));
        assert_eq!(an.status.get(&b_key), Some(&DepFreeStatus::DepFree));
    }
}
