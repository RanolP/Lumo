use crate::{
    BundleEntry, CapDecl, DataDecl, ExternFnDecl, ExternTypeDecl, Expr, File, FnDecl, ImplDecl,
    ImplMethodDecl, Item, MatchArm, OperationDecl, Param, UseDecl, VariantDecl,
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
    // Ensure trailing newline
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
    print_generics(p, &data.generics);
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
    print_expr(p, &func.body);
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
    print_expr(p, &method.body);
    p.dedent();
}

// ---------------------------------------------------------------------------
// Expressions
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
        Expr::Call { callee, args, .. } => {
            print_expr_atom(p, callee);
            p.push("(");
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    p.push(", ");
                }
                print_expr(p, arg);
            }
            p.push(")");
        }
        Expr::Member { object, member, .. } => {
            print_expr_atom(p, object);
            p.push(".");
            p.push(member);
        }
        Expr::Produce { expr: inner, .. } => {
            p.push("produce ");
            print_expr(p, inner);
        }
        Expr::Thunk { expr: inner, .. } => {
            p.push("thunk ");
            print_expr(p, inner);
        }
        Expr::Force { expr: inner, .. } => {
            p.push("force ");
            print_expr_atom(p, inner);
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
        Expr::Perform { cap, .. } => {
            p.push("perform ");
            p.push(cap);
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
        Expr::Ann { expr: inner, ty, .. } => {
            p.push("(");
            print_expr(p, inner);
            p.push(" : ");
            p.push(&ty.value.display());
            p.push(")");
        }
        Expr::Error { .. } => p.push("<error>"),
    }
}

/// Print an expression that needs to be atomic (no ambiguity).
/// Wraps compound expressions in parentheses.
fn print_expr_atom(p: &mut Printer, expr: &Expr) {
    match expr {
        Expr::Ident { .. }
        | Expr::String { .. }
        | Expr::Number { .. }
        | Expr::Error { .. } => print_expr(p, expr),
        Expr::Member { .. } | Expr::Call { .. } => print_expr(p, expr),
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

fn print_generics(p: &mut Printer, generics: &[String]) {
    if !generics.is_empty() {
        p.push("[");
        p.push(&generics.join(", "));
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
        match cap {
            CapRef::Pure => p.push(" / {}"),
            CapRef::Named(entries) => {
                p.push(" / {");
                for (i, entry) in entries.iter().enumerate() {
                    if i > 0 {
                        p.push(", ");
                    }
                    p.push(&entry.display());
                }
                p.push("}");
            }
            CapRef::Infer(entries) => {
                p.push(" / { ..");
                for entry in entries {
                    p.push(", ");
                    p.push(&entry.display());
                }
                p.push(" }");
            }
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
    use lumo_types::{Spanned, TypeExpr};

    fn dummy_span() -> Span {
        Span::new(0, 0)
    }

    fn spanned<T>(value: T) -> Spanned<T> {
        Spanned {
            value,
            span: dummy_span(),
        }
    }

    #[test]
    fn print_extern_type_simple() {
        let file = File {
            items: vec![Item::ExternType(ExternTypeDecl {
                name: "String".into(),
                extern_name: None,
                span: dummy_span(),
            })],
            content_hash: lumo_types::ContentHash(0),
            errors: vec![],
        };
        assert_eq!(print_file(&file), "extern type String\n");
    }

    #[test]
    fn print_extern_type_with_name() {
        let file = File {
            items: vec![Item::ExternType(ExternTypeDecl {
                name: "Number".into(),
                extern_name: Some("number".into()),
                span: dummy_span(),
            })],
            content_hash: lumo_types::ContentHash(0),
            errors: vec![],
        };
        assert_eq!(print_file(&file), "extern type Number as \"number\"\n");
    }

    #[test]
    fn print_data_decl() {
        let file = File {
            items: vec![Item::Data(DataDecl {
                name: "Bool".into(),
                generics: vec![],
                variants: vec![
                    VariantDecl {
                        name: "true".into(),
                        payload: vec![],
                        span: dummy_span(),
                    },
                    VariantDecl {
                        name: "false".into(),
                        payload: vec![],
                        span: dummy_span(),
                    },
                ],
                span: dummy_span(),
            })],
            content_hash: lumo_types::ContentHash(0),
            errors: vec![],
        };
        assert_eq!(print_file(&file), "data Bool { .true, .false }\n");
    }

    #[test]
    fn print_data_generic() {
        let file = File {
            items: vec![Item::Data(DataDecl {
                name: "List".into(),
                generics: vec!["A".into()],
                variants: vec![
                    VariantDecl {
                        name: "nil".into(),
                        payload: vec![],
                        span: dummy_span(),
                    },
                    VariantDecl {
                        name: "cons".into(),
                        payload: vec![
                            spanned(TypeExpr::Named("A".into())),
                            spanned(TypeExpr::App {
                                head: "List".into(),
                                args: vec![TypeExpr::Named("A".into())],
                            }),
                        ],
                        span: dummy_span(),
                    },
                ],
                span: dummy_span(),
            })],
            content_hash: lumo_types::ContentHash(0),
            errors: vec![],
        };
        assert_eq!(
            print_file(&file),
            "data List[A] { .nil, .cons(A, List[A]) }\n"
        );
    }

    #[test]
    fn print_fn_decl() {
        let file = File {
            items: vec![Item::Fn(FnDecl {
                name: "id".into(),
                generics: vec![],
                params: vec![Param {
                    name: "x".into(),
                    ty: spanned(TypeExpr::Named("Bool".into())),
                    span: dummy_span(),
                }],
                return_type: Some(spanned(TypeExpr::Produce(Box::new(TypeExpr::Named(
                    "Bool".into(),
                ))))),
                cap: None,
                body: Expr::Produce {
                    expr: Box::new(Expr::Ident {
                        name: "x".into(),
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                },
                inline: false,
                span: dummy_span(),
            })],
            content_hash: lumo_types::ContentHash(0),
            errors: vec![],
        };
        let output = print_file(&file);
        assert!(output.contains("fn id(x: Bool): produce Bool :="));
        assert!(output.contains("produce x"));
    }

    #[test]
    fn print_let_expr() {
        let expr = Expr::Let {
            name: "x".into(),
            value: Box::new(Expr::Number {
                value: "42".into(),
                span: dummy_span(),
            }),
            body: Box::new(Expr::Produce {
                expr: Box::new(Expr::Ident {
                    name: "x".into(),
                    span: dummy_span(),
                }),
                span: dummy_span(),
            }),
            span: dummy_span(),
        };
        let mut p = Printer::new();
        print_expr(&mut p, &expr);
        assert_eq!(p.buf, "let x = 42 in\nproduce x");
    }

    #[test]
    fn print_match_expr() {
        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Ident {
                name: "b".into(),
                span: dummy_span(),
            }),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Ctor {
                        name: "true".into(),
                        args: vec![],
                    },
                    body: Expr::Produce {
                        expr: Box::new(Expr::Number {
                            value: "1".into(),
                            span: dummy_span(),
                        }),
                        span: dummy_span(),
                    },
                    span: dummy_span(),
                },
                MatchArm {
                    pattern: Pattern::Ctor {
                        name: "false".into(),
                        args: vec![],
                    },
                    body: Expr::Produce {
                        expr: Box::new(Expr::Number {
                            value: "0".into(),
                            span: dummy_span(),
                        }),
                        span: dummy_span(),
                    },
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };
        let mut p = Printer::new();
        print_expr(&mut p, &expr);
        assert!(p.buf.contains("match b {"));
        assert!(p.buf.contains(".true =>"));
        assert!(p.buf.contains("produce 1;"));
        assert!(p.buf.contains(".false =>"));
        assert!(p.buf.contains("produce 0;"));
    }

    #[test]
    fn print_call_member() {
        let expr = Expr::Call {
            callee: Box::new(Expr::Member {
                object: Box::new(Expr::Perform {
                    cap: "Add".into(),
                    span: dummy_span(),
                }),
                member: "add".into(),
                span: dummy_span(),
            }),
            args: vec![
                Expr::Ident {
                    name: "a".into(),
                    span: dummy_span(),
                },
                Expr::Ident {
                    name: "b".into(),
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };
        let mut p = Printer::new();
        print_expr(&mut p, &expr);
        assert_eq!(p.buf, "(perform Add).add(a, b)");
    }

    #[test]
    fn print_handle_bundle() {
        let expr = Expr::Handle {
            cap: "IO".into(),
            type_args: vec![],
            handler: Box::new(Expr::Bundle {
                entries: vec![BundleEntry {
                    name: "log".into(),
                    params: vec![Param {
                        name: "msg".into(),
                        ty: spanned(TypeExpr::Named("String".into())),
                        span: dummy_span(),
                    }],
                    body: Expr::Produce {
                        expr: Box::new(Expr::Ident {
                            name: "msg".into(),
                            span: dummy_span(),
                        }),
                        span: dummy_span(),
                    },
                    span: dummy_span(),
                }],
                span: dummy_span(),
            }),
            body: Box::new(Expr::Ident {
                name: "main".into(),
                span: dummy_span(),
            }),
            span: dummy_span(),
        };
        let mut p = Printer::new();
        print_expr(&mut p, &expr);
        assert!(p.buf.contains("handle IO with"));
        assert!(p.buf.contains("bundle {"));
        assert!(p.buf.contains("fn log(msg: String) :="));
    }
}
