//! Parser emitter: lookahead-1 `can_parse_X`/`parse_X` per rule (the
//! legacy generated-parser shape), `sep(X, s)` loops with optional
//! trailing separator, and a Pratt loop per praat rule.
//!
//! Pratt convention (see `project::praat`): an infix/mixfix row binds
//! while `rbp > min_bp` and parses its right side at `min_bp = lbp`;
//! postfix rows bind while `lbp > min_bp`; prefix rows parse their
//! operand at `min_bp = rbp` in atom position.

use std::collections::BTreeSet;

use crate::project::first::{first_sets, shape_first, FirstSets};
use crate::project::model::{Language, START_RULE};
use crate::project::praat::{classify_row, RowKind, TailPart};
use crate::syntax::ast::{Praat, RuleBody, Shape, ShapeKind};

use super::naming::{kind_name, snake};
use super::Buf;

pub fn generate(lang: &Language) -> String {
    let g = Gen { lang, sets: first_sets(lang) };
    let mut buf = Buf::new();
    buf.line("#![allow(dead_code)]");
    buf.blank();
    buf.line("use langue_rt::Cursor;");
    buf.blank();
    buf.line("use super::lossless::{ParseOutput, SyntaxElement, SyntaxNode};");
    buf.line("use super::syntax_kind::SyntaxKind;");
    buf.blank();
    buf.open("pub fn parse(text: &str) -> ParseOutput {");
    buf.line("let tokens = super::lexer::lex(text);");
    buf.line("let mut p = Parser { c: Cursor::new(tokens, SyntaxKind::is_trivia) };");
    buf.line(&format!("let mut root = p.parse_{}();", snake(START_RULE)));
    buf.line("// Losslessness: sweep whatever the start rule left behind.");
    buf.line("let mut rest = Vec::new();");
    buf.line("p.c.drain_rest_into(SyntaxKind::ERROR, &mut rest);");
    buf.open("if !rest.is_empty() {");
    buf.line("let mut children = std::mem::take(&mut root.children);");
    buf.line("children.extend(rest);");
    buf.line("root = SyntaxNode::from_children(root.kind, children);");
    buf.close("}");
    buf.line("ParseOutput { root, errors: std::mem::take(&mut p.c.errors) }");
    buf.close("}");
    buf.blank();
    buf.open("struct Parser {");
    buf.line("c: Cursor<SyntaxKind>,");
    buf.close("}");
    buf.blank();
    buf.open("impl Parser {");
    let mut first_rule = true;
    for (name, rule) in &g.lang.rules {
        if !first_rule {
            buf.blank();
        }
        first_rule = false;
        match &rule.body {
            RuleBody::Plain(shape) => g.plain_rule(&mut buf, name, shape),
            RuleBody::Praat(praat) => g.praat_rule(&mut buf, name, praat),
        }
    }
    buf.close("}");
    buf.finish()
}

struct Gen<'d> {
    lang: &'d Language,
    sets: FirstSets,
}

impl Gen<'_> {
    /// `self.c.at(K)` / `self.c.at_any(&[..])` over a shape's FIRST set.
    fn at_cond(&self, shape: &Shape) -> String {
        self.at_cond_kinds(&shape_first(self.lang, &self.sets, shape).tokens)
    }

    fn at_cond_kinds(&self, tokens: &BTreeSet<String>) -> String {
        let kinds: Vec<String> =
            tokens.iter().map(|t| format!("SyntaxKind::{}", kind_name(t))).collect();
        match kinds.as_slice() {
            [] => "false".to_owned(),
            [one] => format!("self.c.at({one})"),
            many => format!("self.c.at_any(&[{}])", many.join(", ")),
        }
    }

    fn rule_first_kinds(&self, name: &str) -> BTreeSet<String> {
        self.sets.get(name).map(|f| f.tokens.clone()).unwrap_or_default()
    }

    /// The SyntaxKind of the token a grammar literal refers to. Check
    /// guarantees existence before codegen runs.
    fn literal_kind(&self, text: &str) -> String {
        let token = self
            .lang
            .literal_token(text)
            .unwrap_or_else(|| panic!("checked: literal `'{text}'` has a token"));
        format!("SyntaxKind::{}", kind_name(&token.name))
    }

    /// A group of literal alternatives (`'+' | '-'`): consume one of
    /// them or error.
    fn emit_tok_group(&self, buf: &mut Buf, toks: &[String]) {
        match toks {
            [one] => buf.line(&format!(
                "self.c.expect_into({}, &mut children);",
                self.literal_kind(one)
            )),
            many => {
                let kinds: Vec<String> = many.iter().map(|t| self.literal_kind(t)).collect();
                buf.open(&format!("if self.c.at_any(&[{}]) {{", kinds.join(", ")));
                buf.line("self.c.bump_into(&mut children);");
                buf.else_open("} else {");
                buf.line(&format!(
                    "self.c.error_here(\"expected one of {}\".to_owned());",
                    many.join(", ")
                ));
                buf.close("}");
            }
        }
    }

    // === plain rules ===

    fn plain_rule(&self, buf: &mut Buf, name: &str, shape: &Shape) {
        buf.open(&format!("fn can_parse_{}(&self) -> bool {{", snake(name)));
        buf.line(&self.at_cond_kinds(&self.rule_first_kinds(name)));
        buf.close("}");

        buf.open(&format!("fn parse_{}(&mut self) -> SyntaxNode {{", snake(name)));
        if let Some(arms) = enum_arms(shape) {
            // Transparent dispatch: no wrapper node (the AST accessor is
            // an enum over the arms).
            for arm in &arms {
                buf.open(&format!("if self.can_parse_{}() {{", snake(arm)));
                buf.line(&format!("return self.parse_{}();", snake(arm)));
                buf.close("}");
            }
            buf.line(&format!("self.c.error_here(\"expected {name}\".to_owned());"));
            buf.line("SyntaxNode::from_children(SyntaxKind::ERROR, Vec::new())");
        } else {
            buf.line("let mut children = Vec::new();");
            self.emit_shape(buf, shape);
            buf.line(&format!(
                "SyntaxNode::from_children(SyntaxKind::{}, children)",
                kind_name(name)
            ));
        }
        buf.close("}");
    }

    fn emit_shape(&self, buf: &mut Buf, shape: &Shape) {
        match &shape.kind {
            ShapeKind::Seq(parts) => {
                for part in parts {
                    self.emit_shape(buf, part);
                }
            }
            ShapeKind::Alt(arms) => {
                for (i, arm) in arms.iter().enumerate() {
                    if i == 0 {
                        buf.open(&format!("if {} {{", self.at_cond(arm)));
                    } else {
                        buf.else_open(&format!("}} else if {} {{", self.at_cond(arm)));
                    }
                    self.emit_shape(buf, arm);
                }
                buf.else_open("} else {");
                buf.line("self.c.error_here(\"expected one of the alternatives\".to_owned());");
                buf.close("}");
            }
            ShapeKind::Opt(inner) => {
                buf.open(&format!("if {} {{", self.at_cond(inner)));
                self.emit_shape(buf, inner);
                buf.close("}");
            }
            ShapeKind::Rep(inner) => {
                buf.open(&format!("while {} {{", self.at_cond(inner)));
                self.emit_shape(buf, inner);
                buf.close("}");
            }
            ShapeKind::Label { shape: inner, .. } => self.emit_shape(buf, inner),
            ShapeKind::Lit(text) => {
                buf.line(&format!(
                    "self.c.expect_into({}, &mut children);",
                    self.literal_kind(text)
                ));
            }
            ShapeKind::TokenRef(name) => {
                let token = self.lang.tokens.get(name).expect("checked: token exists");
                buf.line(&format!(
                    "self.c.expect_into(SyntaxKind::{}, &mut children);",
                    kind_name(&token.name)
                ));
            }
            ShapeKind::NodeRef(name) => {
                buf.line(&format!(
                    "children.push(SyntaxElement::Node(Box::new(self.parse_{}())));",
                    snake(name)
                ));
            }
            ShapeKind::Sep { item, sep } => {
                self.emit_shape(buf, item);
                buf.open(&format!("while self.c.at({}) {{", self.literal_kind(sep)));
                buf.line("self.c.bump_into(&mut children);");
                buf.line("// Trailing separator is allowed.");
                buf.open(&format!("if {} {{", self.at_cond(item)));
                self.emit_shape(buf, item);
                buf.else_open("} else {");
                buf.line("break;");
                buf.close("}");
                buf.close("}");
            }
        }
    }

    // === praat rules ===

    fn praat_rule(&self, buf: &mut Buf, name: &str, praat: &Praat) {
        let sn = snake(name);
        buf.open(&format!("fn can_parse_{sn}(&self) -> bool {{"));
        buf.line(&self.at_cond_kinds(&self.rule_first_kinds(name)));
        buf.close("}");

        buf.open(&format!("fn parse_{sn}(&mut self) -> SyntaxNode {{"));
        buf.line(&format!("self.parse_{sn}_bp(0)"));
        buf.close("}");

        let rows: Vec<RowKind> = praat
            .rows
            .iter()
            .map(|r| classify_row(r).expect("checked: praat rows classify"))
            .collect();

        buf.open(&format!("fn parse_{sn}_bp(&mut self, min_bp: u16) -> SyntaxNode {{"));
        buf.line(&format!("let mut lhs = self.parse_{sn}_atom();"));
        buf.open("loop {");
        for row in &rows {
            match row {
                RowKind::Prefix { .. } => {} // atom position
                RowKind::Infix { lbp, toks, rbp } => {
                    let kinds: Vec<String> = toks.iter().map(|t| self.literal_kind(t)).collect();
                    buf.open(&format!(
                        "if self.c.at_any(&[{}]) && {rbp} > min_bp {{",
                        kinds.join(", ")
                    ));
                    buf.line("let mut children = vec![SyntaxElement::Node(Box::new(lhs))];");
                    buf.line("self.c.bump_into(&mut children);");
                    buf.line(&format!(
                        "children.push(SyntaxElement::Node(Box::new(self.parse_{sn}_bp({lbp}))));"
                    ));
                    buf.line(&format!(
                        "lhs = SyntaxNode::from_children(SyntaxKind::{}, children);",
                        kind_name(&format!("{name}Infix"))
                    ));
                    buf.line("continue;");
                    buf.close("}");
                }
                RowKind::Postfix { lbp, tail } => {
                    let lead: Vec<String> = match tail.first() {
                        Some(TailPart::Toks(toks)) => {
                            toks.iter().map(|t| self.literal_kind(t)).collect()
                        }
                        _ => vec![],
                    };
                    buf.open(&format!(
                        "if self.c.at_any(&[{}]) && {lbp} > min_bp {{",
                        lead.join(", ")
                    ));
                    buf.line("let mut children = vec![SyntaxElement::Node(Box::new(lhs))];");
                    for (i, part) in tail.iter().enumerate() {
                        match part {
                            TailPart::Toks(_) if i == 0 => {
                                buf.line("self.c.bump_into(&mut children);");
                            }
                            TailPart::Toks(toks) => self.emit_tok_group(buf, toks),
                            TailPart::Node(rule) => buf.line(&format!(
                                "children.push(SyntaxElement::Node(Box::new(self.parse_{}())));",
                                snake(rule)
                            )),
                        }
                    }
                    buf.line(&format!(
                        "lhs = SyntaxNode::from_children(SyntaxKind::{}, children);",
                        kind_name(&format!("{name}Postfix"))
                    ));
                    buf.line("continue;");
                    buf.close("}");
                }
                RowKind::Mixfix { lbp, head, inner, rbp } => {
                    let kinds: Vec<String> = head.iter().map(|t| self.literal_kind(t)).collect();
                    buf.open(&format!(
                        "if self.c.at_any(&[{}]) && {rbp} > min_bp {{",
                        kinds.join(", ")
                    ));
                    buf.line("let mut children = vec![SyntaxElement::Node(Box::new(lhs))];");
                    buf.line("self.c.bump_into(&mut children);");
                    for (bp, toks) in inner {
                        buf.line(&format!(
                            "children.push(SyntaxElement::Node(Box::new(self.parse_{sn}_bp({bp}))));"
                        ));
                        self.emit_tok_group(buf, toks);
                    }
                    buf.line(&format!(
                        "children.push(SyntaxElement::Node(Box::new(self.parse_{sn}_bp({lbp}))));"
                    ));
                    buf.line(&format!(
                        "lhs = SyntaxNode::from_children(SyntaxKind::{}, children);",
                        kind_name(&format!("{name}Mixfix"))
                    ));
                    buf.line("continue;");
                    buf.close("}");
                }
            }
        }
        buf.line("break;");
        buf.close("}");
        buf.line("lhs");
        buf.close("}");

        buf.open(&format!("fn parse_{sn}_atom(&mut self) -> SyntaxNode {{"));
        for row in &rows {
            if let RowKind::Prefix { toks, rbp } = row {
                let kinds: Vec<String> = toks.iter().map(|t| self.literal_kind(t)).collect();
                buf.open(&format!("if self.c.at_any(&[{}]) {{", kinds.join(", ")));
                buf.line("let mut children = Vec::new();");
                buf.line("self.c.bump_into(&mut children);");
                buf.line(&format!(
                    "children.push(SyntaxElement::Node(Box::new(self.parse_{sn}_bp({rbp}))));"
                ));
                buf.line(&format!(
                    "return SyntaxNode::from_children(SyntaxKind::{}, children);",
                    kind_name(&format!("{name}Prefix"))
                ));
                buf.close("}");
            }
        }
        for (atom, _) in &praat.simple {
            buf.open(&format!("if self.can_parse_{}() {{", snake(atom)));
            buf.line(&format!("return self.parse_{}();", snake(atom)));
            buf.close("}");
        }
        buf.line(&format!("self.c.error_here(\"expected {name}\".to_owned());"));
        buf.line("// Consume one token so enclosing loops make progress.");
        buf.open("if self.c.eof() {");
        buf.line("SyntaxNode::from_children(SyntaxKind::ERROR, Vec::new())");
        buf.else_open("} else {");
        buf.line("let mut children = Vec::new();");
        buf.line("self.c.bump_into(&mut children);");
        buf.line("SyntaxNode::from_children(SyntaxKind::ERROR, children)");
        buf.close("}");
        buf.close("}");
    }
}

/// A rule whose body is `A | B | C` (plain node refs only) parses
/// transparently — no wrapper node.
pub fn enum_arms(shape: &Shape) -> Option<Vec<String>> {
    let ShapeKind::Alt(arms) = &shape.kind else { return None };
    arms.iter()
        .map(|arm| match &arm.kind {
            ShapeKind::NodeRef(n) => Some(n.clone()),
            _ => None,
        })
        .collect()
}
