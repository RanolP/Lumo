//! Elab emitter (M1 step 6, D-21): one generated module per `from A to
//! B` pair. Patterns compile to kind checks + accessor-scheme child
//! lookups; constructions compile to builder calls that render B-text;
//! sort mismatches route through the externs' `coerce` hook (D-38), and
//! pending binders flush at binder-site nodes. Extern rules and passes
//! (D-01) are trait methods without defaults, so a missing Rust
//! implementation fails the build — that is the coverage check.

use std::collections::BTreeMap;

use crate::project::fields::{kind_set, node_fields, node_info, FieldTarget, NodeInfo};
use crate::project::model::{Definition, ElabDef, Language};
use crate::project::praat::{classify_row, RowKind, TailPart};
use crate::syntax::ast::{Con, Pat};

use super::naming::{kind_name, module_name, snake};
use super::Buf;

pub fn pair_module(from: &str, to: &str) -> String {
    format!("{}_to_{}", module_name(from), module_name(to))
}

pub fn generate_mod(def: &Definition) -> String {
    let mut buf = Buf::new();
    buf.line("//! Generated elab modules, one per `from A to B` pair.");
    buf.blank();
    for (from, to) in def.elabs.keys() {
        buf.line(&format!("pub mod {};", pair_module(from, to)));
    }
    buf.finish()
}

pub fn generate_pair(def: &Definition, from: &str, to: &str, elab: &ElabDef) -> String {
    let from_lang = &def.languages[from];
    let to_lang = &def.languages[to];
    let g = Gen {
        from,
        to,
        from_lang,
        to_lang,
        from_mod: module_name(from),
        to_mod: module_name(to),
        elab,
        passes: def.extern_passes.iter().map(|(n, _)| n.clone()).collect(),
    };
    let mut buf = Buf::new();
    buf.line("#![allow(dead_code, unused_variables, unused_mut, clippy::all)]");
    buf.blank();
    if g.passes.is_empty() {
        buf.line("use langue_rt::{ElabCtx, ElabReport};");
    } else {
        buf.line("use langue_rt::{ElabCtx, ElabReport, PassPhase};");
    }
    buf.blank();
    buf.line(&format!("use crate::{}::lossless::SyntaxNode as FromNode;", g.from_mod));
    buf.line(&format!("use crate::{}::syntax_kind::SyntaxKind as FromKind;", g.from_mod));
    buf.line(&format!("use crate::{}::builder;", g.to_mod));
    buf.line(&format!("use crate::{}::syntax_kind::SyntaxKind as ToKind;", g.to_mod));
    buf.blank();
    buf.line("pub type ToFrag = langue_rt::Frag<ToKind>;");

    g.emit_trait(&mut buf);
    g.emit_pipeline(&mut buf);
    g.emit_dispatch(&mut buf);
    g.emit_expect_kind(&mut buf);
    for (i, rule) in elab.rules.iter().enumerate() {
        g.emit_rule(&mut buf, i, &rule.pattern, &rule.construction);
    }
    buf.finish()
}

struct Gen<'d> {
    from: &'d str,
    to: &'d str,
    from_lang: &'d Language,
    to_lang: &'d Language,
    from_mod: String,
    to_mod: String,
    elab: &'d ElabDef,
    passes: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BindKind {
    TokenScalar,
    NodeScalar,
    TokenList,
    NodeList,
}

impl Gen<'_> {
    fn emit_trait(&self, buf: &mut Buf) {
        buf.blank();
        buf.line("/// Extern hooks (D-01/D-38). Rules and passes declared in the");
        buf.line("/// definition have no default body — the Rust implementation is");
        buf.line("/// mandatory, which is the extern coverage check. Provide it as");
        buf.line(&format!(
            "/// `crate::elab_externs::{}() -> Box<dyn Externs>`.",
            pair_module(self.from, self.to)
        ));
        buf.open("pub trait Externs {");
        buf.line("/// Called with the parsed source before rule dispatch.");
        buf.line("fn init(&mut self, _root: &FromNode) {}");
        buf.blank();
        buf.line("/// D-38 sort coercion: turn `frag` into something acceptable where");
        buf.line("/// `expected` is required (typically bind a computation to a fresh");
        buf.line("/// value via `ctx.fresh()` and return the variable as a frag whose");
        buf.line("/// `pending` carries the binder). `None` = not coercible.");
        buf.open(
            "fn coerce(&mut self, _ctx: &mut ElabCtx, _expected: &'static str, _frag: &ToFrag) -> Option<ToFrag> {",
        );
        buf.line("None");
        buf.close("}");
        buf.blank();
        buf.line("/// Nodes at which pending coercion binders are inserted.");
        buf.open("fn is_binder_site(&mut self, _kind: ToKind) -> bool {");
        buf.line("false");
        buf.close("}");
        buf.blank();
        buf.line("/// Wrap `body` in binders for `pending` (outermost first); returns");
        buf.line("/// the new text and its root kind.");
        buf.open(
            "fn wrap_pending(&mut self, _pending: &[(String, String)], body: &str, kind: ToKind) -> (String, ToKind) {",
        );
        buf.line("(body.to_owned(), kind)");
        buf.close("}");
        for (name, _) in &self.elab.extern_rules {
            buf.blank();
            buf.line(&format!("/// `extern rule {name} from {} to {}`", self.from, self.to));
            buf.line(&format!(
                "fn rule_{name}(&mut self, ctx: &mut ElabCtx, node: &FromNode) -> Option<ToFrag>;"
            ));
        }
        for name in &self.passes {
            buf.blank();
            buf.line(&format!("/// `extern pass {name}` — offered both phases; `None` = skip."));
            buf.line(&format!(
                "fn pass_{name}(&mut self, phase: PassPhase, text: &str) -> Option<String>;"
            ));
        }
        buf.close("}");
    }

    fn emit_pipeline(&self, buf: &mut Buf) {
        buf.blank();
        buf.line(&format!("/// Run `elab {} to {}` end to end over source text.", self.from, self.to));
        buf.open("pub fn elab(text: &str, externs: &mut dyn Externs) -> ElabReport {");
        buf.line("let mut errors: Vec<String> = Vec::new();");
        buf.line("let mut src = text.to_owned();");
        for name in &self.passes {
            buf.open(&format!("if let Some(t) = externs.pass_{name}(PassPhase::PreSource, &src) {{"));
            buf.line("src = t;");
            buf.close("}");
        }
        buf.line(&format!("let parsed = crate::{}::parser::parse(&src);", self.from_mod));
        buf.line(&format!(
            "errors.extend(parsed.errors.iter().map(|e| format!(\"{}: {{}} at {{}}\", e.message, e.span)));",
            self.from
        ));
        buf.line("let mut ctx = ElabCtx::new();");
        buf.line("externs.init(&parsed.root);");
        buf.line("let mut output = String::new();");
        buf.open("match elab_node(&mut ctx, externs, &parsed.root) {");
        buf.open("Some(frag) => {");
        buf.open("if !frag.pending.is_empty() {");
        buf.line("ctx.error(\"dangling coercion binders at the root\".to_owned());");
        buf.close("}");
        buf.line("output = frag.text;");
        buf.close("}");
        buf.line("None => ctx.error(\"elaboration produced no output\".to_owned()),");
        buf.close("}");
        buf.line("errors.append(&mut ctx.errors);");
        for name in &self.passes {
            buf.open(&format!(
                "if let Some(t) = externs.pass_{name}(PassPhase::PostTarget, &output) {{"
            ));
            buf.line("output = t;");
            buf.close("}");
        }
        buf.line(&format!("let reparsed = crate::{}::parser::parse(&output);", self.to_mod));
        buf.line(&format!(
            "errors.extend(reparsed.errors.iter().map(|e| format!(\"{}: {{}} at {{}} in `{{output}}`\", e.message, e.span)));",
            self.to
        ));
        buf.open("ElabReport {");
        buf.line(&format!("output: crate::{}::printer::canonical(&reparsed.root),", self.to_mod));
        buf.line("errors,");
        buf.close("}");
        buf.close("}");
    }

    fn emit_dispatch(&self, buf: &mut Buf) {
        let mut by_root: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, rule) in self.elab.rules.iter().enumerate() {
            if let Pat::Node { name, .. } = &rule.pattern {
                by_root.entry(kind_name(name)).or_default().push(i);
            }
        }
        buf.blank();
        buf.line("/// Top-down dispatch by root kind. Extern rules go first, in");
        buf.line("/// declaration order (they see every node and decline with `None`).");
        buf.open(
            "pub fn elab_node(ctx: &mut ElabCtx, externs: &mut dyn Externs, node: &FromNode) -> Option<ToFrag> {",
        );
        for (name, _) in &self.elab.extern_rules {
            buf.open(&format!("if let Some(frag) = externs.rule_{name}(ctx, node) {{"));
            buf.line("return Some(frag);");
            buf.close("}");
        }
        buf.open("match node.kind {");
        for (kind, rules) in &by_root {
            buf.open(&format!("FromKind::{kind} => {{"));
            for i in rules {
                buf.open(&format!("if let Some(frag) = rule_{i}(ctx, externs, node) {{"));
                buf.line("return Some(frag);");
                buf.close("}");
            }
            buf.line(&format!(
                "ctx.error(format!(\"no rule matched `{kind}`: {{}}\", crate::{}::printer::canonical(node)));",
                self.from_mod
            ));
            buf.line("None");
            buf.close("}");
        }
        buf.open("other => {");
        buf.line("ctx.error(format!(\"no elab rule for `{other:?}`\"));");
        buf.line("None");
        buf.close("}");
        buf.close("}");
        buf.close("}");
    }

    fn emit_expect_kind(&self, buf: &mut Buf) {
        buf.blank();
        buf.line("/// A frag placed where `kinds` (the concrete kinds of `expected`)");
        buf.line("/// are required: pass through, coerce (D-38), or error.");
        buf.open(
            "fn expect_kind(ctx: &mut ElabCtx, externs: &mut dyn Externs, frag: ToFrag, kinds: &[ToKind], expected: &'static str) -> Option<ToFrag> {",
        );
        buf.open("match frag.kind {");
        buf.line("None => Some(frag),");
        buf.line("Some(k) if kinds.contains(&k) => Some(frag),");
        buf.open("Some(k) => match externs.coerce(ctx, expected, &frag) {");
        buf.open("Some(mut coerced) => {");
        buf.line("let mut pending = frag.pending;");
        buf.line("pending.append(&mut coerced.pending);");
        buf.line("coerced.pending = pending;");
        buf.line("Some(coerced)");
        buf.close("}");
        buf.open("None => {");
        buf.line(
            "ctx.error(format!(\"cannot place `{k:?}` where `{expected}` is expected: {}\", frag.text));",
        );
        buf.line("None");
        buf.close("}");
        buf.close("},");
        buf.close("}");
        buf.close("}");
        buf.blank();
        buf.line("/// Flush pending binders at a binder-site node (D-38).");
        buf.open("fn flush_binders(externs: &mut dyn Externs, frag: &mut ToFrag) {");
        buf.open("if let Some(kind) = frag.kind {");
        buf.open("if !frag.pending.is_empty() && externs.is_binder_site(kind) {");
        buf.line("let (text, new_kind) = externs.wrap_pending(&frag.pending, &frag.text, kind);");
        buf.line("frag.text = text;");
        buf.line("frag.kind = Some(new_kind);");
        buf.line("frag.pending.clear();");
        buf.close("}");
        buf.close("}");
        buf.close("}");
    }

    // === rules ===

    fn emit_rule(&self, buf: &mut Buf, i: usize, pat: &Pat, con: &Con) {
        let Pat::Node { name, fields, .. } = pat else {
            return; // check rejects non-node roots
        };
        buf.blank();
        buf.line(&format!("/// Rule {i}: `{}` ==> …", name));
        buf.open(&format!(
            "fn rule_{i}(ctx: &mut ElabCtx, externs: &mut dyn Externs, node: &FromNode) -> Option<ToFrag> {{"
        ));
        buf.line(&format!("if node.kind != FromKind::{} {{", kind_name(name)));
        buf.line("    return None;");
        buf.line("}");
        let mut bindings = BTreeMap::new();
        let mut counter = 0usize;
        self.emit_pat_fields(buf, "node", name, fields, &mut bindings, &mut counter);
        let result = self.emit_con(buf, con, Expected::Root, &bindings, &mut counter);
        buf.line(&format!("Some({result})"));
        buf.close("}");
    }

    fn emit_pat_fields(
        &self,
        buf: &mut Buf,
        var: &str,
        node_name: &str,
        fields: &[(String, Pat)],
        bindings: &mut BTreeMap<String, BindKind>,
        counter: &mut usize,
    ) {
        let table = node_fields(self.from_lang, node_name).expect("checked: concrete node");
        for (label, sub) in fields {
            let field = table.iter().find(|f| &f.label == label).expect("checked: field exists");
            match &field.target {
                FieldTarget::Token(t) if t == "<op>" => {
                    let tv = fresh(counter, "t");
                    buf.line(&format!(
                        "let {tv} = langue_rt::first_token({var}, FromKind::is_trivia)?;"
                    ));
                    self.emit_token_pat(buf, &tv, sub, bindings);
                }
                FieldTarget::Token(_) | FieldTarget::LitToken(_) => {
                    let kind = self.from_token_kind(&field.target);
                    if field.many {
                        let tv = fresh(counter, "ts");
                        buf.line(&format!(
                            "let {tv} = langue_rt::tokens_of({var}, FromKind::{kind}, {});",
                            field.skip
                        ));
                        if let Pat::ListVar { name, .. } = sub {
                            buf.line(&format!("let b_{name} = {tv};"));
                            bindings.insert(name.clone(), BindKind::TokenList);
                        }
                    } else {
                        let tv = fresh(counter, "t");
                        buf.line(&format!(
                            "let {tv} = langue_rt::nth_token_of({var}, FromKind::{kind}, {})?;",
                            field.skip
                        ));
                        self.emit_token_pat(buf, &tv, sub, bindings);
                    }
                }
                FieldTarget::Node(rule) => {
                    let kinds = self.from_kind_slice(rule);
                    match sub {
                        Pat::Var { name, .. } => {
                            buf.line(&format!(
                                "let b_{name} = langue_rt::nth_node_in({var}, {kinds}, {})?;",
                                field.skip
                            ));
                            bindings.insert(name.clone(), BindKind::NodeScalar);
                        }
                        Pat::ListVar { name, .. } => {
                            buf.line(&format!(
                                "let b_{name} = langue_rt::nodes_in({var}, {kinds}, {});",
                                field.skip
                            ));
                            bindings.insert(name.clone(), BindKind::NodeList);
                        }
                        Pat::Node { name: sub_name, fields: sub_fields, .. } => {
                            let nv = fresh(counter, "v");
                            buf.line(&format!(
                                "let {nv} = langue_rt::nth_node_in({var}, {kinds}, {})?;",
                                field.skip
                            ));
                            buf.line(&format!(
                                "if {nv}.kind != FromKind::{} {{",
                                kind_name(sub_name)
                            ));
                            buf.line("    return None;");
                            buf.line("}");
                            self.emit_pat_fields(buf, &nv, sub_name, sub_fields, bindings, counter);
                        }
                        Pat::Lit { .. } => {
                            buf.line("return None; // checked: literal on a node field");
                        }
                    }
                }
            }
        }
    }

    fn emit_token_pat(
        &self,
        buf: &mut Buf,
        token_var: &str,
        pat: &Pat,
        bindings: &mut BTreeMap<String, BindKind>,
    ) {
        match pat {
            Pat::Var { name, .. } => {
                buf.line(&format!("let b_{name} = {token_var};"));
                bindings.insert(name.clone(), BindKind::TokenScalar);
            }
            Pat::Lit { text, .. } => {
                buf.line(&format!("if {token_var}.text != {text:?} {{"));
                buf.line("    return None;");
                buf.line("}");
            }
            _ => {
                buf.line("return None; // checked: node pattern on a token field");
            }
        }
    }

    fn from_token_kind(&self, target: &FieldTarget) -> String {
        match target {
            FieldTarget::Token(t) => kind_name(t),
            FieldTarget::LitToken(text) => kind_name(
                &self
                    .from_lang
                    .literal_token(text)
                    .expect("checked: literal token exists")
                    .name,
            ),
            FieldTarget::Node(_) => unreachable!(),
        }
    }

    fn from_kind_slice(&self, rule: &str) -> String {
        let kinds: Vec<String> = kind_set(self.from_lang, rule)
            .iter()
            .map(|k| format!("FromKind::{}", kind_name(k)))
            .collect();
        format!("&[{}]", kinds.join(", "))
    }

    fn to_kind_slice(&self, rule: &str) -> String {
        let kinds: Vec<String> = kind_set(self.to_lang, rule)
            .iter()
            .map(|k| format!("ToKind::{}", kind_name(k)))
            .collect();
        format!("&[{}]", kinds.join(", "))
    }

    // === constructions ===

    /// Emits statements computing a `ToFrag` local; returns its name.
    fn emit_con(
        &self,
        buf: &mut Buf,
        con: &Con,
        expected: Expected<'_>,
        bindings: &BTreeMap<String, BindKind>,
        counter: &mut usize,
    ) -> String {
        let cv = fresh(counter, "c");
        match con {
            Con::Var { name, .. } => match bindings.get(name) {
                Some(BindKind::TokenScalar) => {
                    buf.line(&format!("let mut {cv} = ToFrag::token(b_{name}.text.clone());"));
                }
                _ => {
                    buf.line(&format!(
                        "let mut {cv} = ToFrag::token(crate::{}::printer::canonical(b_{name}));",
                        self.from_mod
                    ));
                }
            },
            Con::Lit { text, .. } => {
                buf.line(&format!("let mut {cv} = ToFrag::token({text:?}.to_owned());"));
            }
            Con::VarTo { name, .. } => {
                buf.line(&format!("let mut {cv} = elab_node(ctx, externs, b_{name})?;"));
            }
            Con::Subst { .. } => {
                buf.line(
                    "ctx.error(\"subst is only available in `between` relations (M3)\".to_owned());",
                );
                buf.line("return None;");
                buf.line(&format!("let mut {cv} = ToFrag::token(String::new());"));
            }
            Con::ListVarTo { .. } => {
                // Only valid under a many field; handled by emit_con_list.
                buf.line("return None; // checked: list construction outside a list field");
                buf.line(&format!("let mut {cv} = ToFrag::token(String::new());"));
            }
            Con::Node { name, fields, .. } => {
                return self.emit_con_node(buf, name, fields, expected, bindings, counter);
            }
        }
        // Kind-check scalar placements into node fields.
        if let Expected::NodeField { rule } = expected {
            let kinds = self.to_kind_slice(rule);
            let rule_lit = format!("{rule:?}");
            buf.line(&format!(
                "let mut {cv} = expect_kind(ctx, externs, {cv}, {kinds}, {rule_lit})?;"
            ));
        }
        cv
    }

    /// A list-valued construction (`[$x* to L]` or a spliced list
    /// binding) for a many field; returns the name of a `Vec<ToFrag>`.
    fn emit_con_list(
        &self,
        buf: &mut Buf,
        con: &Con,
        expected_rule: Option<&str>,
        bindings: &BTreeMap<String, BindKind>,
        counter: &mut usize,
    ) -> String {
        let lv = fresh(counter, "l");
        match con {
            Con::ListVarTo { name, .. } => {
                buf.line(&format!("let mut {lv}: Vec<ToFrag> = Vec::new();"));
                buf.open(&format!("for n in &b_{name} {{"));
                buf.line("let f = elab_node(ctx, externs, n)?;");
                if let Some(rule) = expected_rule {
                    let kinds = self.to_kind_slice(rule);
                    buf.line(&format!(
                        "let f = expect_kind(ctx, externs, f, {kinds}, {:?})?;",
                        rule
                    ));
                }
                buf.line(&format!("{lv}.push(f);"));
                buf.close("}");
            }
            Con::Var { name, .. } => match bindings.get(name) {
                Some(BindKind::TokenList) => {
                    buf.line(&format!(
                        "let mut {lv}: Vec<ToFrag> = b_{name}.iter().map(|t| ToFrag::token(t.text.clone())).collect();"
                    ));
                }
                _ => {
                    buf.line(&format!(
                        "let mut {lv}: Vec<ToFrag> = b_{name}.iter().map(|n| ToFrag::token(crate::{}::printer::canonical(n))).collect();",
                        self.from_mod
                    ));
                }
            },
            _ => {
                buf.line("return None; // checked: scalar construction on a list field");
                buf.line(&format!("let mut {lv}: Vec<ToFrag> = Vec::new();"));
            }
        }
        lv
    }

    fn emit_con_node(
        &self,
        buf: &mut Buf,
        name: &str,
        fields: &[(String, Con)],
        _expected: Expected<'_>,
        bindings: &BTreeMap<String, BindKind>,
        counter: &mut usize,
    ) -> String {
        match node_info(self.to_lang, name).expect("checked: concrete target node") {
            NodeInfo::Struct(_) => {
                self.emit_struct_con(buf, name, fields, bindings, counter)
            }
            NodeInfo::PraatRow { rule, praat, placement } => self.emit_row_con(
                buf, name, &rule, praat, placement, fields, bindings, counter,
            ),
        }
    }

    fn emit_struct_con(
        &self,
        buf: &mut Buf,
        name: &str,
        fields: &[(String, Con)],
        bindings: &BTreeMap<String, BindKind>,
        counter: &mut usize,
    ) -> String {
        let table = node_fields(self.to_lang, name).expect("checked: concrete node");
        // Children in construction order.
        let mut scalar_locals: BTreeMap<&str, String> = BTreeMap::new();
        let mut list_locals: BTreeMap<&str, String> = BTreeMap::new();
        for (label, sub) in fields {
            let field = table.iter().find(|f| &f.label == label).expect("checked: field exists");
            if field.many {
                let expected_rule = match &field.target {
                    FieldTarget::Node(rule) => Some(rule.as_str()),
                    _ => None,
                };
                let lv = self.emit_con_list(buf, sub, expected_rule, bindings, counter);
                list_locals.insert(label.as_str(), lv);
            } else {
                let expected = match &field.target {
                    FieldTarget::Node(rule) => Expected::NodeField { rule },
                    _ => Expected::TokenField,
                };
                let cv = self.emit_con(buf, sub, expected, bindings, counter);
                scalar_locals.insert(label.as_str(), cv);
            }
        }
        // Builder arguments in grammar field order.
        let mut args: Vec<String> = Vec::new();
        let mut text_vecs: Vec<(String, String)> = Vec::new();
        for field in &table {
            let provided_scalar = scalar_locals.get(field.label.as_str());
            let provided_list = list_locals.get(field.label.as_str());
            if field.many {
                match provided_list {
                    Some(lv) => {
                        let tv = fresh(counter, "texts");
                        text_vecs.push((tv.clone(), lv.clone()));
                        args.push(format!("&{tv}"));
                    }
                    None => args.push("&[]".to_owned()),
                }
            } else if field.required {
                let cv = provided_scalar.expect("checked: required field provided");
                args.push(format!("&{cv}.text"));
            } else {
                match provided_scalar {
                    Some(cv) => args.push(format!("Some(&{cv}.text)")),
                    None => args.push("None".to_owned()),
                }
            }
        }
        for (tv, lv) in &text_vecs {
            buf.line(&format!(
                "let {tv}: Vec<&str> = {lv}.iter().map(|f| f.text.as_str()).collect();"
            ));
        }
        let rv = fresh(counter, "r");
        buf.line(&format!(
            "let mut {rv} = ToFrag::node(ToKind::{}, builder::{}({}));",
            kind_name(name),
            snake(name),
            args.join(", ")
        ));
        for cv in scalar_locals.values() {
            buf.line(&format!("{rv}.absorb(&mut {cv});"));
        }
        for lv in list_locals.values() {
            buf.open(&format!("for f in &mut {lv} {{"));
            buf.line(&format!("{rv}.absorb(f);"));
            buf.close("}");
        }
        buf.line(&format!("flush_binders(externs, &mut {rv});"));
        rv
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_row_con(
        &self,
        buf: &mut Buf,
        name: &str,
        rule: &str,
        praat: &crate::syntax::ast::Praat,
        placement: &str,
        fields: &[(String, Con)],
        bindings: &BTreeMap<String, BindKind>,
        counter: &mut usize,
    ) -> String {
        let rows: Vec<RowKind> = praat
            .rows
            .iter()
            .filter_map(|r| classify_row(r).ok())
            .filter(|k| {
                matches!(
                    (k, placement),
                    (RowKind::Prefix { .. }, "Prefix")
                        | (RowKind::Infix { .. }, "Infix")
                        | (RowKind::Postfix { .. }, "Postfix")
                        | (RowKind::Mixfix { .. }, "Mixfix")
                )
            })
            .collect();
        // Select the row by payload labels and the `op` literal.
        let payload_labels: Vec<&str> = fields
            .iter()
            .map(|(l, _)| l.as_str())
            .filter(|l| !matches!(*l, "op" | "expr" | "lhs" | "rhs"))
            .collect();
        let op_lit = fields.iter().find(|(l, _)| l == "op").and_then(|(_, c)| match c {
            Con::Lit { text, .. } => Some(text.as_str()),
            _ => None,
        });
        let matching: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                let payloads: Vec<String> = match row {
                    RowKind::Postfix { tail, .. } => tail
                        .iter()
                        .filter_map(|p| match p {
                            TailPart::Node(r) => Some(snake(r)),
                            TailPart::Toks(_) => None,
                        })
                        .collect(),
                    _ => Vec::new(),
                };
                let mut sorted_have: Vec<&str> = payload_labels.clone();
                sorted_have.sort_unstable();
                let mut sorted_want: Vec<&str> = payloads.iter().map(String::as_str).collect();
                sorted_want.sort_unstable();
                if sorted_have != sorted_want {
                    return false;
                }
                match op_lit {
                    Some(op) => row.lead_toks().iter().any(|t| t == op),
                    None => true,
                }
            })
            .map(|(i, _)| i)
            .collect();
        let [row_i] = matching.as_slice() else {
            // Check guarantees fields exist; ambiguity is a definition
            // bug we surface at generation time.
            panic!(
                "construction of `{name}`: {} row(s) of `{rule}` match payloads {payload_labels:?} / op {op_lit:?}",
                matching.len()
            );
        };
        let row = &rows[*row_i];
        let fn_name = if rows.len() > 1 {
            format!("{}_{}_{row_i}", snake(rule), placement.to_ascii_lowercase())
        } else {
            format!("{}_{}", snake(rule), placement.to_ascii_lowercase())
        };

        // Operand and payload children.
        let child = |buf: &mut Buf, label: &str, counter: &mut usize| -> Option<String> {
            let (_, sub) = fields.iter().find(|(l, _)| l == label)?;
            let expected_rule = match label {
                "expr" | "lhs" | "rhs" => rule.to_owned(),
                payload => {
                    // payload label is snake(payload rule); recover the rule.
                    match row {
                        RowKind::Postfix { tail, .. } => tail
                            .iter()
                            .find_map(|p| match p {
                                TailPart::Node(r) if snake(r) == payload => Some(r.clone()),
                                _ => None,
                            })
                            .expect("payload label maps to a tail node"),
                        _ => unreachable!("payloads only in postfix tails"),
                    }
                }
            };
            Some(self.emit_con(
                buf,
                sub,
                Expected::NodeField { rule: &expected_rule },
                bindings,
                counter,
            ))
        };
        let op_arg = |row: &RowKind| -> String {
            match op_lit {
                Some(op) => format!("{op:?}"),
                None => {
                    let toks = row.lead_toks();
                    assert!(
                        toks.len() == 1,
                        "construction of `{name}` needs an `op` literal to pick among {toks:?}"
                    );
                    format!("{:?}", toks[0])
                }
            }
        };
        let operand = |cv: &str| format!("builder::Operand {{ text: &{cv}.text, kind: {cv}.kind }}");

        let rv = fresh(counter, "r");
        let mut absorbs: Vec<String> = Vec::new();
        let call = match row {
            RowKind::Prefix { .. } => {
                let e = child(buf, "expr", counter).expect("checked: prefix has expr");
                absorbs.push(e.clone());
                format!("builder::{fn_name}({}, {})", op_arg(row), operand(&e))
            }
            RowKind::Infix { .. } => {
                let l = child(buf, "lhs", counter).expect("checked: infix has lhs");
                let r = child(buf, "rhs", counter).expect("checked: infix has rhs");
                absorbs.push(l.clone());
                absorbs.push(r.clone());
                format!(
                    "builder::{fn_name}({}, {}, {})",
                    operand(&l),
                    op_arg(row),
                    operand(&r)
                )
            }
            RowKind::Postfix { tail, .. } => {
                let e = child(buf, "expr", counter).expect("checked: postfix has expr");
                absorbs.push(e.clone());
                let mut args = vec![operand(&e)];
                for part in tail {
                    if let TailPart::Node(r) = part {
                        let p = child(buf, &snake(r), counter)
                            .expect("checked: payload field provided");
                        args.push(format!("&{p}.text"));
                        absorbs.push(p);
                    }
                }
                format!("builder::{fn_name}({})", args.join(", "))
            }
            RowKind::Mixfix { .. } => {
                panic!("mixfix constructions are not supported in M1 (`{name}`)")
            }
        };
        buf.line(&format!("let mut {rv} = ToFrag::node(ToKind::{}, {call});", kind_name(name)));
        for cv in &absorbs {
            buf.line(&format!("{rv}.absorb(&mut {cv});"));
        }
        buf.line(&format!("flush_binders(externs, &mut {rv});"));
        rv
    }
}

enum Expected<'e> {
    Root,
    TokenField,
    NodeField { rule: &'e str },
}

fn fresh(counter: &mut usize, prefix: &str) -> String {
    let v = format!("{prefix}{counter}");
    *counter += 1;
    v
}
