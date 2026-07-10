//! Typed AST accessor emitter (legacy `lst/src/ast.rs` shape): one
//! zero-cost wrapper per struct rule with label accessors, one enum per
//! all-node-ref alternative rule, and per praat rule an enum over row
//! placements plus its simple atoms.

use crate::project::fields::{self, FieldTarget};
use crate::project::model::Language;
use crate::project::praat::{classify_row, RowKind, TailPart};
use crate::syntax::ast::{Praat, RuleBody, Shape};

use super::naming::{kind_name, snake};
use super::parser::enum_arms;
use super::syntax_kind::praat_kinds;
use super::Buf;

pub fn generate(lang: &Language) -> String {
    let mut buf = Buf::new();
    buf.line("#![allow(dead_code)]");
    buf.blank();
    buf.line("use super::lossless::{SyntaxNode, Token};");
    buf.line("use super::syntax_kind::SyntaxKind;");
    buf.blank();
    buf.open("pub trait AstNode<'a>: Sized {");
    buf.line("fn cast(node: &'a SyntaxNode) -> Option<Self>;");
    buf.line("fn syntax(&self) -> &'a SyntaxNode;");
    buf.close("}");

    for (name, rule) in &lang.rules {
        buf.blank();
        match &rule.body {
            RuleBody::Plain(shape) => match enum_arms(shape) {
                Some(arms) => {
                    let arms: Vec<(String, String)> =
                        arms.into_iter().map(|a| (a.clone(), a)).collect();
                    emit_enum(&mut buf, name, &arms);
                }
                None => emit_struct(&mut buf, lang, name, &kind_name(name), Some(shape)),
            },
            RuleBody::Praat(praat) => emit_praat(&mut buf, name, praat),
        }
    }
    buf.finish()
}

fn emit_cast_by_kind(buf: &mut Buf, ty: &str, kind: &str) {
    buf.open(&format!("impl<'a> AstNode<'a> for {ty}<'a> {{"));
    buf.open("fn cast(node: &'a SyntaxNode) -> Option<Self> {");
    buf.line(&format!("(node.kind == SyntaxKind::{kind}).then(|| Self(node))"));
    buf.close("}");
    buf.open("fn syntax(&self) -> &'a SyntaxNode {");
    buf.line("self.0");
    buf.close("}");
    buf.close("}");
}

/// A struct rule (or a praat row struct when `shape` is `None`).
fn emit_struct(buf: &mut Buf, lang: &Language, name: &str, kind: &str, shape: Option<&Shape>) {
    buf.line(&format!("pub struct {name}<'a>(pub &'a SyntaxNode);"));
    buf.blank();
    emit_cast_by_kind(buf, name, kind);

    let accessors: Vec<Accessor> = shape
        .map(|s| fields::struct_fields(lang, s))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|f| {
            let class = match &f.target {
                FieldTarget::Node(rule) => Class::Node(rule.clone()),
                FieldTarget::Token(token) => Class::Token(kind_name(token)),
                FieldTarget::LitToken(text) => {
                    Class::Token(kind_name(&lang.literal_token(text)?.name))
                }
            };
            Some(Accessor { name: f.label, class, many: f.many, skip: f.skip })
        })
        .collect();
    if accessors.is_empty() {
        return;
    }
    buf.blank();
    buf.open(&format!("impl<'a> {name}<'a> {{"));
    for (i, acc) in accessors.iter().enumerate() {
        if i > 0 {
            buf.blank();
        }
        emit_accessor(buf, acc);
    }
    buf.close("}");
}

fn emit_accessor(buf: &mut Buf, acc: &Accessor) {
    match (&acc.class, acc.many) {
        (Class::Token(kind), false) => {
            buf.open(&format!(
                "pub fn {}(&self) -> Option<&'a Token> {{",
                acc.name
            ));
            buf.line(&format!(
                "self.0.child_tokens().filter(|t| t.kind == SyntaxKind::{kind}).nth({})",
                acc.skip
            ));
            buf.close("}");
        }
        (Class::Token(kind), true) => {
            buf.open(&format!(
                "pub fn {}(&self) -> impl Iterator<Item = &'a Token> + 'a {{",
                acc.name
            ));
            buf.line(&format!(
                "self.0.child_tokens().filter(|t| t.kind == SyntaxKind::{kind}).skip({})",
                acc.skip
            ));
            buf.close("}");
        }
        (Class::Node(rule), false) => {
            buf.open(&format!(
                "pub fn {}(&self) -> Option<{rule}<'a>> {{",
                acc.name
            ));
            buf.line(&format!(
                "self.0.child_nodes().filter_map({rule}::cast).nth({})",
                acc.skip
            ));
            buf.close("}");
        }
        (Class::Node(rule), true) => {
            buf.open(&format!(
                "pub fn {}(&self) -> impl Iterator<Item = {rule}<'a>> + 'a {{",
                acc.name
            ));
            buf.line(&format!(
                "self.0.child_nodes().filter_map({rule}::cast).skip({})",
                acc.skip
            ));
            buf.close("}");
        }
    }
}

/// `arms` are `(variant name, inner type)` pairs.
fn emit_enum(buf: &mut Buf, name: &str, arms: &[(String, String)]) {
    buf.open(&format!("pub enum {name}<'a> {{"));
    for (variant, ty) in arms {
        buf.line(&format!("{variant}({ty}<'a>),"));
    }
    buf.close("}");
    buf.blank();
    buf.open(&format!("impl<'a> AstNode<'a> for {name}<'a> {{"));
    buf.open("fn cast(node: &'a SyntaxNode) -> Option<Self> {");
    buf.line("None");
    for (variant, ty) in arms {
        buf.line(&format!("    .or_else(|| {ty}::cast(node).map(Self::{variant}))"));
    }
    buf.close("}");
    buf.open("fn syntax(&self) -> &'a SyntaxNode {");
    buf.open("match self {");
    for (variant, _) in arms {
        buf.line(&format!("Self::{variant}(n) => n.syntax(),"));
    }
    buf.close("}");
    buf.close("}");
    buf.close("}");
}

fn emit_praat(buf: &mut Buf, name: &str, praat: &Praat) {
    let row_structs = praat_kinds(name, praat); // ExprPrefix, ExprInfix, …
    let arms: Vec<(String, String)> = row_structs
        .iter()
        .map(|ty| {
            // `ExprInfix` appears in the enum as the short `Infix`.
            (ty.strip_prefix(name).unwrap_or(ty).to_owned(), ty.clone())
        })
        .chain(praat.simple.iter().map(|(a, _)| (a.clone(), a.clone())))
        .collect();
    emit_enum(buf, name, &arms);

    let rows: Vec<RowKind> = praat
        .rows
        .iter()
        .map(|r| classify_row(r).expect("checked: praat rows classify"))
        .collect();

    for row_struct in &row_structs {
        buf.blank();
        buf.line(&format!("pub struct {row_struct}<'a>(pub &'a SyntaxNode);"));
        buf.blank();
        emit_cast_by_kind(buf, row_struct, &kind_name(row_struct));
        buf.blank();
        buf.open(&format!("impl<'a> {row_struct}<'a> {{"));
        buf.line("/// The operator token (first non-trivia token).");
        buf.open("pub fn op(&self) -> Option<&'a Token> {");
        buf.line("self.0.child_tokens().find(|t| !t.kind.is_trivia())");
        buf.close("}");
        buf.blank();
        buf.open(&format!(
            "pub fn operands(&self) -> impl Iterator<Item = {name}<'a>> + 'a {{"
        ));
        buf.line(&format!("self.0.child_nodes().filter_map({name}::cast)"));
        buf.close("}");
        if row_struct.ends_with("Infix") {
            buf.blank();
            buf.open(&format!("pub fn lhs(&self) -> Option<{name}<'a>> {{"));
            buf.line("self.operands().next()");
            buf.close("}");
            buf.blank();
            buf.open(&format!("pub fn rhs(&self) -> Option<{name}<'a>> {{"));
            buf.line("self.operands().nth(1)");
            buf.close("}");
        }
        if row_struct.ends_with("Prefix") || row_struct.ends_with("Postfix") {
            buf.blank();
            buf.open(&format!("pub fn expr(&self) -> Option<{name}<'a>> {{"));
            buf.line("self.operands().next()");
            buf.close("}");
        }
        if row_struct.ends_with("Postfix") {
            // One accessor per node payload used by any postfix row.
            let mut payloads: Vec<String> = Vec::new();
            for row in &rows {
                if let RowKind::Postfix { tail, .. } = row {
                    for part in tail {
                        if let TailPart::Node(rule) = part {
                            if !payloads.contains(rule) {
                                payloads.push(rule.clone());
                            }
                        }
                    }
                }
            }
            for payload in payloads {
                buf.blank();
                buf.open(&format!(
                    "pub fn {}(&self) -> Option<{payload}<'a>> {{",
                    snake(&payload)
                ));
                buf.line(&format!(
                    "self.0.child_nodes().filter_map({payload}::cast).next()"
                ));
                buf.close("}");
            }
        }
        buf.close("}");
    }
}

enum Class {
    /// Target is a rule; the value is its accessor type name.
    Node(String),
    /// Target is a token; the value is its SyntaxKind variant.
    Token(String),
}

struct Accessor {
    name: String,
    class: Class,
    many: bool,
    /// How many same-class single accessors precede this one — the
    /// `nth`/`skip` offset (`value:Expr … body:Expr` → 0 and 1).
    skip: usize,
}
