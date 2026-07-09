use crate::{lir, lir_memaware};

/// Count the number of times `name` is used as a free variable inside `expr`.
/// Shadowing is respected: a nested `Let` or `Lambda` that rebinds `name` stops
/// the count in its scope.
fn count_uses(expr: &lir::Expr, name: &str) -> usize {
    match expr {
        lir::Expr::Ident { name: n, .. } => usize::from(n == name),
        lir::Expr::Let { name: n, value, body, .. } => {
            let in_value = count_uses(value, name);
            let in_body = if n == name { 0 } else { count_uses(body, name) };
            in_value + in_body
        }
        lir::Expr::Lambda { param, body, .. } => {
            if param == name { 0 } else { count_uses(body, name) }
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            count_uses(scrutinee, name)
                + arms.iter().map(|a| {
                    if a.pattern.bindings().iter().any(|b| b == name) {
                        0
                    } else {
                        count_uses(&a.body, name)
                    }
                }).sum::<usize>()
        }
        lir::Expr::Apply { callee, arg, .. } => count_uses(callee, name) + count_uses(arg, name),
        lir::Expr::Force { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Thunk { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Ann { expr, .. } => count_uses(expr, name),
        lir::Expr::Ctor { args, .. } => args.iter().map(|a| count_uses(a, name)).sum(),
        lir::Expr::Bundle { entries, .. } => {
            entries.iter().map(|e| {
                if e.params.iter().any(|p| p.name == name) { 0 } else { count_uses(&e.body, name) }
            }).sum()
        }
        lir::Expr::Handle { handler, body, .. } => {
            count_uses(handler, name) + count_uses(body, name)
        }
        lir::Expr::Member { object, .. } => count_uses(object, name),
        lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Perform { .. }
        | lir::Expr::Error { .. } => 0,
    }
}

/// Elaborate a top-level `lir::Expr` into a `lir_memaware::Expr`.
///
/// Only top-level `Let` nodes receive Dup/Drop treatment in this pass.
/// Deeper nesting remains `Pure(…)` until a future compound-variant pass.
fn elaborate_expr(expr: &lir::Expr) -> lir_memaware::Expr {
    match expr {
        lir::Expr::Let { id, name, value: _, body } => {
            let uses = count_uses(body, name);
            if uses == 0 {
                lir_memaware::Expr::Drop {
                    id: *id,
                    name: name.clone(),
                    body: Box::new(elaborate_expr(body)),
                }
            } else if uses >= 2 {
                lir_memaware::Expr::Dup {
                    id: *id,
                    expr: Box::new(lir_memaware::Expr::Pure(expr.clone())),
                }
            } else {
                lir_memaware::Expr::Pure(expr.clone())
            }
        }
        _ => lir_memaware::Expr::Pure(expr.clone()),
    }
}

/// Convert a pure-functional `lir::File` into a `lir_memaware::File`.
///
/// Top-level `Let` bindings are annotated with Dup (used ≥ 2 times) or Drop
/// (used 0 times). Single-use and non-`Let` bodies remain wrapped in `Pure(…)`.
pub fn elaborate(file: &lir::File) -> lir_memaware::File {
    lir_memaware::File {
        content_hash: file.content_hash.clone(),
        spans: file.spans.clone(),
        items: file.items.iter().map(elaborate_item).collect(),
    }
}

fn elaborate_item(item: &lir::Item) -> lir_memaware::Item {
    match item {
        lir::Item::Fn(f)         => lir_memaware::Item::Fn(elaborate_fn(f)),
        lir::Item::Impl(i)       => lir_memaware::Item::Impl(elaborate_impl(i)),
        lir::Item::ExternType(e) => lir_memaware::Item::ExternType(e.clone()),
        lir::Item::ExternFn(e)   => lir_memaware::Item::ExternFn(e.clone()),
        lir::Item::Data(d)       => lir_memaware::Item::Data(d.clone()),
        lir::Item::Cap(c)        => lir_memaware::Item::Cap(c.clone()),
        lir::Item::Use(u)        => lir_memaware::Item::Use(u.clone()),
    }
}

fn elaborate_fn(f: &lir::FnDecl) -> lir_memaware::FnDecl {
    lir_memaware::FnDecl {
        name:        f.name.clone(),
        generics:    f.generics.clone(),
        params:      f.params.clone(),
        return_type: f.return_type.clone(),
        cap:         f.cap.clone(),
        inline:      f.inline,
        span:        f.span,
        value:       elaborate_expr(&f.value),
    }
}

fn elaborate_impl(i: &lir::ImplDecl) -> lir_memaware::ImplDecl {
    lir_memaware::ImplDecl {
        name:         i.name.clone(),
        generics:     i.generics.clone(),
        target_type:  i.target_type.clone(),
        capability:   i.capability.clone(),
        assoc_types:  i.assoc_types.clone(),
        span:         i.span,
        methods:      i.methods.iter().map(elaborate_method).collect(),
    }
}

fn elaborate_method(m: &lir::ImplMethodDecl) -> lir_memaware::ImplMethodDecl {
    lir_memaware::ImplMethodDecl {
        name:        m.name.clone(),
        params:      m.params.clone(),
        return_type: m.return_type.clone(),
        span:        m.span,
        value:       elaborate_expr(&m.value),
    }
}
