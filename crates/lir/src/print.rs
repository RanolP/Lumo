use crate::{
    BundleEntry, CapDecl, DataDecl, ExternFnDecl, ExternTypeDecl, Expr, File, FnDecl, GenericParam,
    ImplDecl, ImplMethodDecl, Item, MatchArm, OperationDecl, Param, UseDecl, VariantDecl,
};
use lumo_types::{CapRef, Pattern};

// ---------------------------------------------------------------------------
// Printer
// ---------------------------------------------------------------------------

struct Printer {
    buf: String,
    indent: usize,
}

impl Printer {
    fn new() -> Self {
        Self {
            buf: String::new(),
            indent: 0,
        }
    }

    fn push(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    fn newline(&mut self) {
        self.buf.push('\n');
        for _ in 0..self.indent {
            self.buf.push_str("  ");
        }
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn print_file(file: &File) -> String {
    let mut p = Printer::new();
    for (i, item) in file.items.iter().enumerate() {
        if i > 0 {
            p.push("\n");
            p.newline();
        }
        print_item(&mut p, item);
    }
    if !p.buf.is_empty() && !p.buf.ends_with('\n') {
        p.buf.push('\n');
    }
    p.buf
}

// ---------------------------------------------------------------------------
// Items
// ---------------------------------------------------------------------------

fn print_item(p: &mut Printer, item: &Item) {
    match item {
        Item::ExternType(ext) => print_extern_type(p, ext),
        Item::ExternFn(ext) => print_extern_fn(p, ext),
        Item::Data(data) => print_data(p, data),
        Item::Cap(cap) => print_cap(p, cap),
        Item::Fn(func) => print_fn(p, func),
        Item::Use(u) => print_use(p, u),
        Item::Impl(impl_decl) => print_impl(p, impl_decl),
    }
}

fn print_extern_type(p: &mut Printer, ext: &ExternTypeDecl) {
    p.push("extern type ");
    p.push(&ext.name);
    if let Some(extern_name) = &ext.extern_name {
        p.push(" as \"");
        p.push(extern_name);
        p.push("\"");
    }
}

fn print_extern_fn(p: &mut Printer, ext: &ExternFnDecl) {
    if ext.inline {
        p.push("#[inline(always)] ");
    }
    p.push("extern fn ");
    p.push(&ext.name);
    print_param_list(p, &ext.params);
    if let Some(ret) = &ext.return_type {
        p.push(": ");
        p.push(&ret.value.display());
    }
    print_cap_annotation(p, &ext.cap);
    if let Some(extern_name) = &ext.extern_name {
        p.push(" as \"");
        p.push(extern_name);
        p.push("\"");
    }
}

fn print_data(p: &mut Printer, data: &DataDecl) {
    p.push("data ");
    p.push(&data.name);
    print_str_generics(p, &data.generics);
    p.push(" {");
    for (i, v) in data.variants.iter().enumerate() {
        if i > 0 {
            p.push(",");
        }
        p.push(" ");
        print_variant(p, v);
    }
    if !data.variants.is_empty() {
        p.push(" ");
    }
    p.push("}");
}

fn print_variant(p: &mut Printer, v: &VariantDecl) {
    p.push(".");
    p.push(&v.name);
    if !v.payload.is_empty() {
        p.push("(");
        for (i, ty) in v.payload.iter().enumerate() {
            if i > 0 {
                p.push(", ");
            }
            p.push(&ty.value.display());
        }
        p.push(")");
    }
}

fn print_cap(p: &mut Printer, cap: &CapDecl) {
    p.push("cap ");
    p.push(&cap.name);
    p.push(" {");
    p.indent();
    for op in &cap.operations {
        p.newline();
        print_operation(p, op);
    }
    p.dedent();
    if !cap.operations.is_empty() {
        p.newline();
    }
    p.push("}");
}

fn print_operation(p: &mut Printer, op: &OperationDecl) {
    p.push("fn ");
    p.push(&op.name);
    print_param_list(p, &op.params);
    if let Some(ret) = &op.return_type {
        p.push(": ");
        p.push(&ret.value.display());
    }
}

fn print_fn(p: &mut Printer, func: &FnDecl) {
    p.push("fn ");
    p.push(&func.name);
    print_generics(p, &func.generics);
    print_param_list(p, &func.params);
    if let Some(ret) = &func.return_type {
        p.push(": ");
        p.push(&ret.value.display());
    }
    print_cap_annotation(p, &func.cap);
    p.push(" :=");
    p.indent();
    p.newline();
    print_expr(p, &func.value);
    p.dedent();
}

fn print_use(p: &mut Printer, u: &UseDecl) {
    p.push("use ");
    p.push(&u.path.join("."));
    if let Some(names) = &u.names {
        p.push(".{");
        p.push(&names.join(", "));
        p.push("}");
    }
    p.push(";");
}

fn print_impl(p: &mut Printer, impl_decl: &ImplDecl) {
    p.push("impl");
    print_generics(p, &impl_decl.generics);
    if let Some(name) = &impl_decl.name {
        p.push(" ");
        p.push(name);
        p.push(" =");
    }
    p.push(" ");
    p.push(&impl_decl.target_type.value.display());
    if let Some(cap) = &impl_decl.capability {
        p.push(": ");
        p.push(&cap.value.display());
    }
    p.push(" {");
    p.indent();
    for method in &impl_decl.methods {
        p.newline();
        print_impl_method(p, method);
    }
    p.dedent();
    if !impl_decl.methods.is_empty() {
        p.newline();
    }
    p.push("}");
}

fn print_impl_method(p: &mut Printer, method: &ImplMethodDecl) {
    p.push("fn ");
    p.push(&method.name);
    print_param_list(p, &method.params);
    if let Some(ret) = &method.return_type {
        p.push(": ");
        p.push(&ret.value.display());
    }
    p.push(" :=");
    p.indent();
    p.newline();
    print_expr(p, &method.value);
    p.dedent();
}

// ---------------------------------------------------------------------------
// Expressions — LIR-specific
// ---------------------------------------------------------------------------

fn print_expr(p: &mut Printer, expr: &Expr) {
    match expr {
        Expr::Ident { name, .. } => p.push(name),
        Expr::String { value, .. } => {
            p.push("\"");
            p.push(&escape_string(value));
            p.push("\"");
        }
        Expr::Number { value, .. } => p.push(value),
        Expr::Ctor {
            name, called, args, ..
        } => {
            p.push("ctor ");
            p.push(name);
            if *called {
                p.push("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        p.push(", ");
                    }
                    print_expr(p, arg);
                }
                p.push(")");
            }
        }
        Expr::Thunk { expr: inner, .. } => {
            p.push("thunk ");
            print_expr(p, inner);
        }
        Expr::Roll { expr: inner, .. } => {
            p.push("roll ");
            print_expr_atom(p, inner);
        }
        Expr::Bundle { entries, .. } => {
            p.push("bundle {");
            p.indent();
            for entry in entries {
                p.newline();
                print_bundle_entry(p, entry);
            }
            p.dedent();
            if !entries.is_empty() {
                p.newline();
            }
            p.push("}");
        }
        Expr::Produce { expr: inner, .. } => {
            p.push("produce ");
            print_expr(p, inner);
        }
        Expr::Force { expr: inner, .. } => {
            p.push("force ");
            print_expr_atom(p, inner);
        }
        Expr::Lambda { param, body, .. } => {
            p.push("lambda ");
            p.push(param);
            p.push(". ");
            print_expr(p, body);
        }
        Expr::Apply { callee, arg, .. } => {
            print_expr_apply(p, callee);
            p.push("(");
            print_expr(p, arg);
            p.push(")");
        }
        Expr::Let {
            name, value, body, ..
        } => {
            p.push("let ");
            p.push(name);
            p.push(" = ");
            print_expr(p, value);
            p.push(" in");
            p.newline();
            print_expr(p, body);
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            p.push("match ");
            print_expr(p, scrutinee);
            p.push(" {");
            p.indent();
            for arm in arms {
                p.newline();
                print_match_arm(p, arm);
            }
            p.dedent();
            if !arms.is_empty() {
                p.newline();
            }
            p.push("}");
        }
        Expr::Unroll { expr: inner, .. } => {
            p.push("unroll ");
            print_expr_atom(p, inner);
        }
        Expr::Perform { cap, type_args, .. } => {
            p.push("perform ");
            p.push(cap);
            if !type_args.is_empty() {
                p.push("[");
                p.push(&type_args.join(", "));
                p.push("]");
            }
        }
        Expr::Handle {
            cap, type_args, handler, body, ..
        } => {
            p.push("handle ");
            p.push(cap);
            if !type_args.is_empty() {
                p.push("[");
                p.push(&type_args.join(", "));
                p.push("]");
            }
            p.push(" with ");
            print_expr(p, handler);
            p.push(" in");
            p.indent();
            p.newline();
            print_expr(p, body);
            p.dedent();
        }
        Expr::Member { object, field, .. } => {
            print_expr_atom(p, object);
            p.push(".");
            p.push(field);
        }
        Expr::Ann { expr: inner, ty, .. } => {
            p.push("(");
            print_expr(p, inner);
            p.push(" : ");
            p.push(&ty.display());
            p.push(")");
        }
        Expr::Error { .. } => p.push("<error>"),
    }
}

/// Print an expression in apply-callee position.
/// Apply chains like `f(a)(b)` print naturally; other compound exprs get parens.
fn print_expr_apply(p: &mut Printer, expr: &Expr) {
    match expr {
        Expr::Apply { .. } | Expr::Force { .. } | Expr::Ident { .. } | Expr::Member { .. } => {
            print_expr(p, expr)
        }
        _ => {
            p.push("(");
            print_expr(p, expr);
            p.push(")");
        }
    }
}

/// Print an expression as an atom (wraps compound exprs in parens).
fn print_expr_atom(p: &mut Printer, expr: &Expr) {
    match expr {
        Expr::Ident { .. }
        | Expr::String { .. }
        | Expr::Number { .. }
        | Expr::Error { .. } => print_expr(p, expr),
        Expr::Member { .. } | Expr::Apply { .. } => print_expr(p, expr),
        _ => {
            p.push("(");
            print_expr(p, expr);
            p.push(")");
        }
    }
}

fn print_match_arm(p: &mut Printer, arm: &MatchArm) {
    print_pattern(p, &arm.pattern);
    p.push(" =>");
    p.indent();
    p.newline();
    print_expr(p, &arm.body);
    p.push(";");
    p.dedent();
}

fn print_bundle_entry(p: &mut Printer, entry: &BundleEntry) {
    p.push("fn ");
    p.push(&entry.name);
    print_param_list(p, &entry.params);
    p.push(" :=");
    p.indent();
    p.newline();
    print_expr(p, &entry.body);
    p.dedent();
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn print_str_generics(p: &mut Printer, generics: &[String]) {
    if !generics.is_empty() {
        p.push("[");
        p.push(&generics.join(", "));
        p.push("]");
    }
}

fn print_generics(p: &mut Printer, generics: &[GenericParam]) {
    if !generics.is_empty() {
        p.push("[");
        for (i, g) in generics.iter().enumerate() {
            if i > 0 { p.push(", "); }
            if g.is_cap_row() {
                p.push("cap ");
            }
            p.push(g.name());
        }
        p.push("]");
    }
}

fn print_param_list(p: &mut Printer, params: &[Param]) {
    p.push("(");
    for (i, param) in params.iter().enumerate() {
        if i > 0 {
            p.push(", ");
        }
        p.push(&param.name);
        p.push(": ");
        p.push(&param.ty.value.display());
    }
    p.push(")");
}

fn print_cap_annotation(p: &mut Printer, cap: &Option<CapRef>) {
    if let Some(cap) = cap {
        if cap.is_empty() {
            p.push(" / {}");
        } else {
            p.push(" / {");
            for (i, entry) in cap.iter().enumerate() {
                if i > 0 {
                    p.push(", ");
                }
                p.push(&entry.display());
            }
            p.push("}");
        }
    }
}

fn print_pattern(p: &mut Printer, pat: &Pattern) {
    p.push(&pat.display());
}

fn escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use lumo_span::Span;
    use lumo_types::{ContentHash, ExprId, Spanned, TypeExpr};

    fn dummy_span() -> Span {
        Span::new(0, 0)
    }

    fn spanned<T>(value: T) -> Spanned<T> {
        Spanned {
            value,
            span: dummy_span(),
        }
    }

    fn id(n: u32) -> ExprId {
        ExprId(n)
    }

    #[test]
    fn print_lambda_chain() {
        let expr = Expr::Thunk {
            id: id(0),
            expr: Box::new(Expr::Lambda {
                id: id(1),
                param: "x".into(),
                body: Box::new(Expr::Lambda {
                    id: id(2),
                    param: "y".into(),
                    body: Box::new(Expr::Produce {
                        id: id(3),
                        expr: Box::new(Expr::Ident {
                            id: id(4),
                            name: "x".into(),
                        }),
                    }),
                }),
            }),
        };
        let mut p = Printer::new();
        print_expr(&mut p, &expr);
        assert_eq!(p.buf, "thunk lambda x. lambda y. produce x");
    }

    #[test]
    fn print_apply_chain() {
        // (force f)(a)(b)
        let expr = Expr::Apply {
            id: id(0),
            callee: Box::new(Expr::Apply {
                id: id(1),
                callee: Box::new(Expr::Force {
                    id: id(2),
                    expr: Box::new(Expr::Ident {
                        id: id(3),
                        name: "f".into(),
                    }),
                }),
                arg: Box::new(Expr::Ident {
                    id: id(4),
                    name: "a".into(),
                }),
            }),
            arg: Box::new(Expr::Ident {
                id: id(5),
                name: "b".into(),
            }),
        };
        let mut p = Printer::new();
        print_expr(&mut p, &expr);
        assert_eq!(p.buf, "force f(a)(b)");
    }

    #[test]
    fn print_ctor_uncalled() {
        let expr = Expr::Roll {
            id: id(0),
            expr: Box::new(Expr::Ctor {
                id: id(1),
                name: "Bool.true".into(),
                called: false,
                args: vec![],
            }),
        };
        let mut p = Printer::new();
        print_expr(&mut p, &expr);
        assert_eq!(p.buf, "roll (ctor Bool.true)");
    }

    #[test]
    fn print_ctor_called() {
        let expr = Expr::Roll {
            id: id(0),
            expr: Box::new(Expr::Ctor {
                id: id(1),
                name: "List.cons".into(),
                called: true,
                args: vec![
                    Expr::Ident {
                        id: id(2),
                        name: "x".into(),
                    },
                    Expr::Ident {
                        id: id(3),
                        name: "xs".into(),
                    },
                ],
            }),
        };
        let mut p = Printer::new();
        print_expr(&mut p, &expr);
        assert_eq!(p.buf, "roll (ctor List.cons(x, xs))");
    }

    #[test]
    fn print_unroll_match() {
        let expr = Expr::Match {
            id: id(0),
            scrutinee: Box::new(Expr::Unroll {
                id: id(1),
                expr: Box::new(Expr::Ident {
                    id: id(2),
                    name: "b".into(),
                }),
            }),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Ctor {
                        name: "true".into(),
                        args: vec![],
                    },
                    body: Expr::Produce {
                        id: id(3),
                        expr: Box::new(Expr::Number {
                            id: id(4),
                            value: "1".into(),
                        }),
                    },
                    span: dummy_span(),
                },
                MatchArm {
                    pattern: Pattern::Ctor {
                        name: "false".into(),
                        args: vec![],
                    },
                    body: Expr::Produce {
                        id: id(5),
                        expr: Box::new(Expr::Number {
                            id: id(6),
                            value: "0".into(),
                        }),
                    },
                    span: dummy_span(),
                },
            ],
        };
        let mut p = Printer::new();
        print_expr(&mut p, &expr);
        assert!(p.buf.contains("match unroll b {"));
        assert!(p.buf.contains(".true =>"));
        assert!(p.buf.contains(".false =>"));
    }

    #[test]
    fn print_fn_decl_curried() {
        let file = File {
            items: vec![Item::Fn(FnDecl {
                name: "add".into(),
                generics: vec![],
                params: vec![
                    Param {
                        name: "a".into(),
                        ty: spanned(TypeExpr::Named("Number".into())),
                        span: dummy_span(),
                    },
                    Param {
                        name: "b".into(),
                        ty: spanned(TypeExpr::Named("Number".into())),
                        span: dummy_span(),
                    },
                ],
                return_type: Some(spanned(TypeExpr::Produce(Box::new(TypeExpr::Named(
                    "Number".into(),
                ))))),
                cap: None,
                value: Expr::Thunk {
                    id: id(0),
                    expr: Box::new(Expr::Lambda {
                        id: id(1),
                        param: "a".into(),
                        body: Box::new(Expr::Lambda {
                            id: id(2),
                            param: "b".into(),
                            body: Box::new(Expr::Produce {
                                id: id(3),
                                expr: Box::new(Expr::Ident {
                                    id: id(4),
                                    name: "a".into(),
                                }),
                            }),
                        }),
                    }),
                },
                inline: false,
                span: dummy_span(),
            })],
            content_hash: ContentHash(0),
            spans: vec![dummy_span(); 5],
        };
        let output = print_file(&file);
        assert!(output.contains("fn add(a: Number, b: Number): produce Number :="));
        assert!(output.contains("thunk lambda a. lambda b. produce a"));
    }
}
