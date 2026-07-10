//! Recursive-descent parser for the `.langue` format.
//!
//! Two entry points: [`parse_syn_file`] for kind-suffixed files and
//! [`parse_manifest`] for the suffix-less manifest (D-27/D-33). A rule body
//! ends at EOF or where the next item starts (`Name =`, `token`, `trivia`,
//! `extern`) — the ungrammar convention.

use super::ast::*;
use super::lexer::{self, Token, TokenKind};
use crate::diag::Diagnostic;
use langue_rt::Span;

/// Words that can never be shape refs.
const RESERVED: &[&str] = &["token", "trivia", "extern", "praat", "sep", "simple", "operators"];

pub fn parse_syn_file(file: &str, text: &str) -> (File, Vec<Diagnostic>) {
    let (tokens, mut diags) = lexer::lex(file, text);
    let mut p = Parser { file, tokens, pos: 0, diags: Vec::new() };
    let ast = p.parse_syn();
    diags.append(&mut p.diags);
    (ast, diags)
}

pub fn parse_manifest(file: &str, text: &str) -> (File, Vec<Diagnostic>) {
    let (tokens, mut diags) = lexer::lex(file, text);
    let mut p = Parser { file, tokens, pos: 0, diags: Vec::new() };
    let ast = p.parse_manifest_file();
    diags.append(&mut p.diags);
    (ast, diags)
}

struct Parser<'f> {
    file: &'f str,
    tokens: Vec<Token>,
    pos: usize,
    diags: Vec<Diagnostic>,
}

impl Parser<'_> {
    fn cur(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn nth(&self, n: usize) -> &Token {
        &self.tokens[(self.pos + n).min(self.tokens.len() - 1)]
    }

    fn at_eof(&self) -> bool {
        matches!(self.cur().kind, TokenKind::Eof)
    }

    fn bump(&mut self) -> Token {
        let t = self.cur().clone();
        if !self.at_eof() {
            self.pos += 1;
        }
        t
    }

    fn at_punct(&self, c: char) -> bool {
        self.cur().kind == TokenKind::Punct(c)
    }

    fn eat_punct(&mut self, c: char) -> bool {
        if self.at_punct(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_punct(&mut self, c: char) {
        if !self.eat_punct(c) {
            self.error_here(format!("expected `{c}`, found {}", self.cur().kind.describe()));
        }
    }

    fn at_name(&self, text: &str) -> bool {
        matches!(&self.cur().kind, TokenKind::Name(n) if n == text)
    }

    /// `Name` followed by `=` — the start of the next item.
    fn at_item_start(&self) -> bool {
        matches!(self.cur().kind, TokenKind::Name(_))
            && self.nth(1).kind == TokenKind::Punct('=')
    }

    fn expect_name(&mut self, what: &str) -> (String, Span) {
        if let TokenKind::Name(n) = &self.cur().kind {
            let n = n.clone();
            let span = self.cur().span;
            self.pos += 1;
            (n, span)
        } else {
            self.error_here(format!("expected {what}, found {}", self.cur().kind.describe()));
            (String::new(), self.cur().span)
        }
    }

    fn error_here(&mut self, message: String) {
        self.diags.push(Diagnostic::error(self.file, self.cur().span, message));
    }

    /// Skip to the next plausible item start after an error.
    fn recover_to_item(&mut self) {
        while !self.at_eof() {
            if self.at_item_start() || self.at_name("token") || self.at_name("trivia") || self.at_name("extern") {
                return;
            }
            self.pos += 1;
        }
    }

    // === syn files ===

    fn parse_syn(&mut self) -> File {
        let mut items = Vec::new();
        while !self.at_eof() {
            let before = self.pos;
            if let Some(item) = self.parse_syn_item() {
                items.push(item);
            }
            if self.pos == before {
                // Didn't consume anything: report and skip forward.
                self.error_here(format!("expected an item, found {}", self.cur().kind.describe()));
                self.pos += 1;
                self.recover_to_item();
            }
        }
        File { items }
    }

    fn parse_syn_item(&mut self) -> Option<Item> {
        if self.at_name("token") || self.at_name("trivia") {
            return self.parse_token_decl();
        }
        if self.at_name("extern") {
            return self.parse_extern_recover();
        }
        if self.at_item_start() {
            return self.parse_rule();
        }
        None
    }

    fn parse_token_decl(&mut self) -> Option<Item> {
        let is_trivia = self.at_name("trivia");
        self.bump();
        let (name, name_span) = self.expect_name("a token name");
        self.expect_punct('=');
        let pattern_span = self.cur().span;
        let pattern = match self.cur().kind.clone() {
            TokenKind::Str(s) => {
                self.pos += 1;
                TokenPattern::Literal(s)
            }
            TokenKind::Regex(r) => {
                self.pos += 1;
                TokenPattern::Regex(r)
            }
            other => {
                self.error_here(format!(
                    "expected a string or regex pattern, found {}",
                    other.describe()
                ));
                self.recover_to_item();
                return None;
            }
        };
        Some(Item::Token(TokenDecl { name, name_span, pattern, pattern_span, is_trivia }))
    }

    fn parse_extern_recover(&mut self) -> Option<Item> {
        let start = self.cur().span;
        self.bump(); // extern
        if !self.at_name("recover") {
            self.error_here(format!(
                "expected `recover` after `extern`, found {}",
                self.cur().kind.describe()
            ));
            self.recover_to_item();
            return None;
        }
        self.bump();
        let (rule, rule_span) = self.expect_name("a rule name");
        Some(Item::ExternRecover(ExternRecover { rule, span: start.cover(rule_span) }))
    }

    fn parse_rule(&mut self) -> Option<Item> {
        let (name, name_span) = self.expect_name("a rule name");
        self.expect_punct('=');
        let body = if self.at_name("praat") {
            RuleBody::Praat(self.parse_praat()?)
        } else {
            RuleBody::Plain(self.parse_alt())
        };
        Some(Item::Rule(RuleDecl { name, name_span, body }))
    }

    // === shapes ===

    fn parse_alt(&mut self) -> Shape {
        self.eat_punct('|'); // optional leading bar
        let first = self.parse_seq();
        if !self.at_punct('|') {
            return first;
        }
        let mut span = first.span;
        let mut arms = vec![first];
        while self.eat_punct('|') {
            let arm = self.parse_seq();
            span = span.cover(arm.span);
            arms.push(arm);
        }
        Shape::new(ShapeKind::Alt(arms), span)
    }

    fn parse_seq(&mut self) -> Shape {
        let mut parts = Vec::new();
        while self.can_start_atom() {
            parts.push(self.parse_labeled());
        }
        match parts.len() {
            0 => {
                self.error_here(format!(
                    "expected a shape, found {}",
                    self.cur().kind.describe()
                ));
                Shape::new(ShapeKind::Seq(Vec::new()), self.cur().span)
            }
            1 => parts.pop().unwrap(),
            _ => {
                let span = parts[0].span.cover(parts[parts.len() - 1].span);
                Shape::new(ShapeKind::Seq(parts), span)
            }
        }
    }

    fn can_start_atom(&self) -> bool {
        match &self.cur().kind {
            TokenKind::Str(_) => true,
            TokenKind::Punct('(') => true,
            TokenKind::Name(n) => {
                if self.nth(1).kind == TokenKind::Punct('=') {
                    return false; // next rule starts here
                }
                if n == "sep" {
                    return self.nth(1).kind == TokenKind::Punct('(');
                }
                !RESERVED.contains(&n.as_str())
            }
            _ => false,
        }
    }

    fn parse_labeled(&mut self) -> Shape {
        if let TokenKind::Name(n) = &self.cur().kind {
            if self.nth(1).kind == TokenKind::Punct(':') {
                let label = n.clone();
                let start = self.cur().span;
                self.pos += 2; // name ':'
                let inner = self.parse_postfix();
                let span = start.cover(inner.span);
                return Shape::new(
                    ShapeKind::Label { label, shape: Box::new(inner) },
                    span,
                );
            }
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Shape {
        let mut shape = self.parse_atom();
        loop {
            if self.at_punct('?') {
                let span = shape.span.cover(self.bump().span);
                shape = Shape::new(ShapeKind::Opt(Box::new(shape)), span);
            } else if self.at_punct('*') {
                let span = shape.span.cover(self.bump().span);
                shape = Shape::new(ShapeKind::Rep(Box::new(shape)), span);
            } else {
                break;
            }
        }
        shape
    }

    fn parse_atom(&mut self) -> Shape {
        let span = self.cur().span;
        match self.cur().kind.clone() {
            TokenKind::Str(s) => {
                self.pos += 1;
                Shape::new(ShapeKind::Lit(s), span)
            }
            TokenKind::Name(n) if n == "sep" => {
                self.pos += 1;
                self.expect_punct('(');
                let item = self.parse_alt();
                self.expect_punct(',');
                let sep = match self.cur().kind.clone() {
                    TokenKind::Str(s) => {
                        self.pos += 1;
                        s
                    }
                    other => {
                        self.error_here(format!(
                            "expected a separator literal, found {}",
                            other.describe()
                        ));
                        String::new()
                    }
                };
                let end = self.cur().span;
                self.expect_punct(')');
                Shape::new(ShapeKind::Sep { item: Box::new(item), sep }, span.cover(end))
            }
            TokenKind::Name(n) => {
                self.pos += 1;
                if name_is_node_ref(&n) {
                    Shape::new(ShapeKind::NodeRef(n), span)
                } else {
                    Shape::new(ShapeKind::TokenRef(n), span)
                }
            }
            TokenKind::Punct('(') => {
                self.pos += 1;
                let inner = self.parse_alt();
                let end = self.cur().span;
                self.expect_punct(')');
                Shape::new(inner.kind, span.cover(end))
            }
            other => {
                self.error_here(format!("expected a shape, found {}", other.describe()));
                self.pos += 1;
                Shape::new(ShapeKind::Seq(Vec::new()), span)
            }
        }
    }

    // === praat ===

    fn parse_praat(&mut self) -> Option<Praat> {
        self.bump(); // praat
        self.expect_punct('{');

        if !self.at_name("simple") {
            self.error_here(format!(
                "expected `simple = …` in praat block, found {}",
                self.cur().kind.describe()
            ));
            self.recover_to_item();
            return None;
        }
        self.bump();
        self.expect_punct('=');
        let mut simple = Vec::new();
        loop {
            let (name, span) = self.expect_name("an atom rule name");
            if name.is_empty() {
                self.recover_to_item();
                return None;
            }
            simple.push((name, span));
            if !self.eat_punct('|') {
                break;
            }
        }

        if !self.at_name("operators") {
            self.error_here(format!(
                "expected `operators {{ … }}` in praat block, found {}",
                self.cur().kind.describe()
            ));
            self.recover_to_item();
            return None;
        }
        self.bump();
        self.expect_punct('{');
        let mut rows = Vec::new();
        while !self.at_punct('}') && !self.at_eof() {
            if let Some(row) = self.parse_op_row() {
                rows.push(row);
            } else {
                break;
            }
            if !self.eat_punct(',') {
                break;
            }
        }
        self.expect_punct('}'); // operators
        self.expect_punct('}'); // praat
        Some(Praat { simple, rows })
    }

    fn parse_op_row(&mut self) -> Option<OpRow> {
        let start = self.cur().span;
        let mut elems = Vec::new();
        let mut end = start;
        loop {
            match self.cur().kind.clone() {
                TokenKind::Punct('@') => {
                    self.pos += 1;
                    match self.cur().kind {
                        TokenKind::Num(bp) => {
                            end = self.cur().span;
                            self.pos += 1;
                            elems.push(OpElem::Operand(bp));
                        }
                        _ => {
                            self.error_here(format!(
                                "expected a binding power after `@`, found {}",
                                self.cur().kind.describe()
                            ));
                            return None;
                        }
                    }
                }
                TokenKind::Str(s) => {
                    end = self.cur().span;
                    self.pos += 1;
                    let mut toks = vec![s];
                    while self.at_punct('|') {
                        self.pos += 1;
                        match self.cur().kind.clone() {
                            TokenKind::Str(s2) => {
                                end = self.cur().span;
                                self.pos += 1;
                                toks.push(s2);
                            }
                            other => {
                                self.error_here(format!(
                                    "expected a token literal after `|`, found {}",
                                    other.describe()
                                ));
                                return None;
                            }
                        }
                    }
                    elems.push(OpElem::Toks(toks));
                }
                _ => break,
            }
        }
        if elems.is_empty() {
            self.error_here(format!(
                "expected an operator row, found {}",
                self.cur().kind.describe()
            ));
            return None;
        }
        Some(OpRow { elems, span: start.cover(end) })
    }

    // === manifest (D-33) ===

    fn parse_manifest_file(&mut self) -> File {
        let mut items = Vec::new();
        while !self.at_eof() {
            let before = self.pos;
            if self.at_item_start() {
                if let Some(p) = self.parse_pipeline() {
                    items.push(Item::Pipeline(p));
                }
            }
            if self.pos == before {
                self.error_here(format!(
                    "expected `name = <pipeline>`, found {}",
                    self.cur().kind.describe()
                ));
                self.pos += 1;
                self.recover_to_item();
            }
        }
        File { items }
    }

    fn parse_pipeline(&mut self) -> Option<Pipeline> {
        let (name, name_span) = self.expect_name("a pipeline name");
        self.expect_punct('=');
        let mut stages = Vec::new();
        loop {
            if let Some(stage) = self.parse_stage() {
                stages.push(stage);
            } else {
                return None;
            }
            if !self.eat_punct('|') {
                break;
            }
        }
        Some(Pipeline { name, name_span, stages })
    }

    fn parse_stage(&mut self) -> Option<Stage> {
        let start = self.cur().span;
        let (head, _) = self.expect_name("a stage");
        if head.is_empty() {
            return None;
        }
        let kind = match head.as_str() {
            "parse" => {
                let (lang, lang_span) = self.expect_name("a language name");
                return Some(Stage { kind: StageKind::Parse { lang }, span: start.cover(lang_span) });
            }
            "elab" => {
                let (from, _) = self.expect_name("a source language");
                if !self.at_name("to") {
                    self.error_here(format!(
                        "expected `to` in elab stage, found {}",
                        self.cur().kind.describe()
                    ));
                    return None;
                }
                self.bump();
                let (to, to_span) = self.expect_name("a target language");
                StageKind::Elab { from, to }.spanned(start.cover(to_span))
            }
            _ => {
                let (lang, lang_span) = self.expect_name("a language name");
                StageKind::Judgment { judgment: head, lang }.spanned(start.cover(lang_span))
            }
        };
        Some(kind)
    }
}

impl StageKind {
    fn spanned(self, span: Span) -> Stage {
        Stage { kind: self, span }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn syn(text: &str) -> File {
        let (ast, diags) = parse_syn_file("t.syn.langue", text);
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        ast
    }

    #[test]
    fn token_and_trivia_decls() {
        let ast = syn("token keyword.fn = 'fn'\ntoken ident = /[a-z]+/\ntrivia ws = /[ \\t]+/");
        assert_eq!(ast.items.len(), 3);
        let Item::Token(t) = &ast.items[0] else { panic!() };
        assert_eq!(t.name, "keyword.fn");
        assert_eq!(t.pattern, TokenPattern::Literal("fn".into()));
        assert!(!t.is_trivia);
        let Item::Token(t) = &ast.items[2] else { panic!() };
        assert!(t.is_trivia);
        assert_eq!(t.pattern, TokenPattern::Regex("[ \\t]+".into()));
    }

    #[test]
    fn plain_rule_with_labels_sep_opt() {
        let ast = syn("FnDecl = 'fn' name:ident '(' params:sep(Param, ',')? ')'");
        let Item::Rule(r) = &ast.items[0] else { panic!() };
        assert_eq!(r.name, "FnDecl");
        let RuleBody::Plain(shape) = &r.body else { panic!() };
        let ShapeKind::Seq(parts) = &shape.kind else { panic!() };
        assert_eq!(parts.len(), 5);
        assert!(matches!(&parts[0].kind, ShapeKind::Lit(l) if l == "fn"));
        let ShapeKind::Label { label, shape } = &parts[1].kind else { panic!() };
        assert_eq!(label, "name");
        assert!(matches!(&shape.kind, ShapeKind::TokenRef(t) if t == "ident"));
        let ShapeKind::Label { label, shape } = &parts[3].kind else { panic!() };
        assert_eq!(label, "params");
        let ShapeKind::Opt(inner) = &shape.kind else { panic!() };
        let ShapeKind::Sep { item, sep } = &inner.kind else { panic!() };
        assert!(matches!(&item.kind, ShapeKind::NodeRef(n) if n == "Param"));
        assert_eq!(sep, ",");
    }

    #[test]
    fn alt_rule_with_leading_bars_ends_at_next_rule() {
        let ast = syn("Item =\n  | FnDecl\n  | LetDecl\nFnDecl = 'fn'");
        assert_eq!(ast.items.len(), 2);
        let Item::Rule(r) = &ast.items[0] else { panic!() };
        let RuleBody::Plain(shape) = &r.body else { panic!() };
        let ShapeKind::Alt(arms) = &shape.kind else { panic!() };
        assert_eq!(arms.len(), 2);
    }

    #[test]
    fn group_with_rep_and_alt() {
        let ast = syn("Bound = head:Ty ('+' tail:Ty)*");
        let Item::Rule(r) = &ast.items[0] else { panic!() };
        let RuleBody::Plain(shape) = &r.body else { panic!() };
        let ShapeKind::Seq(parts) = &shape.kind else { panic!() };
        let ShapeKind::Rep(inner) = &parts[1].kind else { panic!() };
        assert!(matches!(&inner.kind, ShapeKind::Seq(_)));
    }

    #[test]
    fn praat_block() {
        let ast = syn(
            "Expr = praat {\n  simple = Lit | Ident | ParenExpr\n  operators {\n    '-' | '!' @100,\n    @89 '**' @90,\n    @80 '*' | '/' @79,\n    @40 '?' @0 ':' @39,\n  }\n}",
        );
        let Item::Rule(r) = &ast.items[0] else { panic!() };
        let RuleBody::Praat(p) = &r.body else { panic!() };
        assert_eq!(
            p.simple.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
            vec!["Lit", "Ident", "ParenExpr"]
        );
        assert_eq!(p.rows.len(), 4);
        assert_eq!(
            p.rows[0].elems,
            vec![OpElem::Toks(vec!["-".into(), "!".into()]), OpElem::Operand(100)]
        );
        assert_eq!(
            p.rows[1].elems,
            vec![OpElem::Operand(89), OpElem::Toks(vec!["**".into()]), OpElem::Operand(90)]
        );
        assert_eq!(
            p.rows[3].elems,
            vec![
                OpElem::Operand(40),
                OpElem::Toks(vec!["?".into()]),
                OpElem::Operand(0),
                OpElem::Toks(vec![":".into()]),
                OpElem::Operand(39),
            ]
        );
    }

    #[test]
    fn extern_recover() {
        let ast = syn("extern recover Expr");
        let Item::ExternRecover(e) = &ast.items[0] else { panic!() };
        assert_eq!(e.rule, "Expr");
    }

    #[test]
    fn manifest_dictated_form() {
        let (ast, diags) = parse_manifest(
            "lumo.langue",
            "main = parse Lumo | elab Lumo to MIR | elab MIR to LIR | check_V LIR",
        );
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        let Item::Pipeline(p) = &ast.items[0] else { panic!() };
        assert_eq!(p.name, "main");
        assert_eq!(p.stages.len(), 4);
        assert!(matches!(&p.stages[0].kind, StageKind::Parse { lang } if lang == "Lumo"));
        assert!(
            matches!(&p.stages[1].kind, StageKind::Elab { from, to } if from == "Lumo" && to == "MIR")
        );
        assert!(
            matches!(&p.stages[3].kind, StageKind::Judgment { judgment, lang } if judgment == "check_V" && lang == "LIR")
        );
        assert_eq!(p.root_language(), Some("Lumo"));
    }

    #[test]
    fn rule_body_stops_before_token_decl() {
        let ast = syn("Foo = Bar\ntoken x = /y/");
        assert_eq!(ast.items.len(), 2);
        let Item::Rule(r) = &ast.items[0] else { panic!() };
        let RuleBody::Plain(shape) = &r.body else { panic!() };
        assert!(matches!(&shape.kind, ShapeKind::NodeRef(n) if n == "Bar"));
    }

    #[test]
    fn broken_input_reports_and_recovers() {
        let (ast, diags) =
            parse_syn_file("t.syn.langue", "token = 'x'\nGood = 'ok'");
        assert!(!diags.is_empty());
        assert!(ast.items.iter().any(
            |i| matches!(i, Item::Rule(r) if r.name == "Good")
        ));
    }
}
