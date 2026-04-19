use std::collections::HashSet;

use lumo_lir as lir;

pub fn sweep(file: &mut lir::File) {
    use super::call_graph::{build_call_graph, CallTarget};

    let cg = build_call_graph(file);
    let mut reachable: HashSet<String> = HashSet::new();
    let mut work: Vec<String> = Vec::new();

    // Roots: main if present; otherwise all top-level fns (library mode).
    let has_main = file.items.iter().any(|i| matches!(i, lir::Item::Fn(f) if f.name == "main"));
    if has_main {
        work.push("main".to_owned());
    } else {
        // No main → treat all top-level fns as roots (library mode).
        for item in &file.items {
            if let lir::Item::Fn(f) = item {
                work.push(f.name.clone());
            }
        }
    }

    while let Some(name) = work.pop() {
        if !reachable.insert(name.clone()) {
            continue;
        }
        if let Some(sites) = cg.edges.get(&name) {
            for cs in sites {
                match &cs.callee {
                    CallTarget::Fn(callee) => work.push(callee.clone()),
                    CallTarget::ImplMethod { impl_const, method } => {
                        // Reach the impl method as a virtual fn key
                        let mkey = format!("{impl_const}.{method}");
                        if reachable.insert(mkey.clone()) {
                            if let Some(s) = cg.edges.get(&mkey) {
                                for inner in s {
                                    if let CallTarget::Fn(c) = &inner.callee {
                                        work.push(c.clone());
                                    }
                                }
                            }
                        }
                    }
                    CallTarget::Indirect => {}
                }
            }
        }
    }

    // Drop fns that are not reachable. Keep all non-fn items (data, cap, impl,
    // extern) — DCE for those is out of scope.
    file.items.retain(|item| match item {
        lir::Item::Fn(f) => reachable.contains(&f.name),
        _ => true,
    });
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
    fn unreachable_fn_is_dropped() {
        let src = r#"
            fn dead(): Number { 1 }
            fn main(): Number { 2 }
        "#;
        let mut file = lower(src);
        sweep(&mut file);
        assert!(!file.items.iter().any(|i| matches!(i, lir::Item::Fn(f) if f.name == "dead")));
        assert!(file.items.iter().any(|i| matches!(i, lir::Item::Fn(f) if f.name == "main")));
    }

    #[test]
    fn extern_fn_always_kept() {
        let src = r#"
            extern fn js_print(s: String): Number
            fn main(): Number { 0 }
        "#;
        let mut file = lower(src);
        sweep(&mut file);
        assert!(file.items.iter().any(|i| matches!(i, lir::Item::ExternFn(f) if f.name == "js_print")));
    }

    #[test]
    fn reachable_fn_is_kept() {
        let src = r#"
            fn helper(x: Number): Number { x }
            fn main(): Number { helper(1) }
        "#;
        let mut file = lower(src);
        sweep(&mut file);
        assert!(file.items.iter().any(|i| matches!(i, lir::Item::Fn(f) if f.name == "helper")));
    }
}
