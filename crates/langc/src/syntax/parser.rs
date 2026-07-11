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

pub fn parse_elab_file(file: &str, text: &str) -> (File, Vec<Diagnostic>) {
    let (tokens, mut diags) = lexer::lex(file, text);
    let mut p = Parser { file, tokens, pos: 0, diags: Vec::new() };
    let ast = p.parse_elab();
    diags.append(&mut p.diags);
    (ast, diags)
}

pub fn parse_type_file(file: &str, text: &str) -> (File, Vec<Diagnostic>) {
    let (tokens, mut diags) = lexer::lex(file, text);
    let mut p = Parser { file, tokens, pos: 0, diags: Vec::new() };
    let ast = p.parse_type();
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
                TokenKind::Name(n) if name_is_node_ref(&n) => {
                    end = self.cur().span;
                    self.pos += 1;
                    elems.push(OpElem::Node(n));
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

    // === elab files (D-35) ===

    fn at_sym(&self, s: &str) -> bool {
        matches!(self.cur().kind, TokenKind::Sym(sym) if sym == s)
    }

    fn expect_sym(&mut self, s: &str) -> bool {
        if self.at_sym(s) {
            self.pos += 1;
            true
        } else {
            self.error_here(format!("expected `{s}`, found {}", self.cur().kind.describe()));
            false
        }
    }

    /// Skip to the next plausible elab item start after an error.
    fn recover_to_elab_item(&mut self) {
        while !self.at_eof() {
            if self.at_name("from") || self.at_name("between") || self.at_name("extern") {
                return;
            }
            self.pos += 1;
        }
    }

    fn parse_elab(&mut self) -> File {
        let mut items = Vec::new();
        while !self.at_eof() {
            let before = self.pos;
            if let Some(item) = self.parse_elab_item() {
                items.push(item);
            }
            if self.pos == before {
                self.error_here(format!(
                    "expected `from`, `between`, or `extern`, found {}",
                    self.cur().kind.describe()
                ));
                self.pos += 1;
                self.recover_to_elab_item();
            }
        }
        File { items }
    }

    fn parse_elab_item(&mut self) -> Option<Item> {
        if self.at_name("from") {
            return self.parse_from_block();
        }
        if self.at_name("between") {
            return self.parse_between_block();
        }
        if self.at_name("extern") {
            return self.parse_extern_elab();
        }
        None
    }

    fn parse_from_block(&mut self) -> Option<Item> {
        let start = self.bump().span; // from
        let (from, _) = self.expect_name("a source language");
        if !self.at_name("to") {
            self.error_here(format!(
                "expected `to`, found {}",
                self.cur().kind.describe()
            ));
            self.recover_to_elab_item();
            return None;
        }
        self.bump();
        let (to, to_span) = self.expect_name("a target language");
        let span = start.cover(to_span);
        self.expect_punct('{');
        let mut rules = Vec::new();
        while !self.at_punct('}') && !self.at_eof() {
            let before = self.pos;
            let Some(pattern) = self.parse_pat() else {
                self.recover_rule_body();
                continue;
            };
            if !self.expect_sym("==>") {
                self.recover_rule_body();
                continue;
            }
            let Some(construction) = self.parse_con() else {
                self.recover_rule_body();
                continue;
            };
            let span = pattern.span().cover(construction.span());
            rules.push(ElabRule { pattern, construction, span });
            if self.pos == before {
                self.pos += 1; // safety: always make progress
            }
        }
        self.expect_punct('}');
        Some(Item::ElabBlock(ElabBlock { from, to, span, rules }))
    }

    fn parse_between_block(&mut self) -> Option<Item> {
        let start = self.bump().span; // between
        let (lang, lang_span) = self.expect_name("a language name");
        let span = start.cover(lang_span);
        self.expect_punct('{');
        let mut relations = Vec::new();
        while !self.at_punct('}') && !self.at_eof() {
            let before = self.pos;
            let Some(lhs) = self.parse_pat() else {
                self.recover_rule_body();
                continue;
            };
            if !self.expect_sym("===") {
                self.recover_rule_body();
                continue;
            }
            let Some(rhs) = self.parse_con() else {
                self.recover_rule_body();
                continue;
            };
            let span = lhs.span().cover(rhs.span());
            relations.push(Relation { lhs, rhs, span });
            if self.pos == before {
                self.pos += 1;
            }
        }
        self.expect_punct('}');
        Some(Item::BetweenBlock(BetweenBlock { lang, span, relations }))
    }

    /// Skip a broken rule: conservatively drop the rest of the enclosing
    /// block (to its closing `}`), so the item loop always progresses.
    fn recover_rule_body(&mut self) {
        let mut depth = 0usize;
        while !self.at_eof() {
            match &self.cur().kind {
                TokenKind::Punct('{') => depth += 1,
                TokenKind::Punct('}') => {
                    if depth == 0 {
                        return;
                    }
                    depth -= 1;
                }
                _ => {}
            }
            self.pos += 1;
        }
    }

    fn parse_extern_elab(&mut self) -> Option<Item> {
        let start = self.cur().span;
        self.bump(); // extern
        if self.at_name("pass") {
            self.bump();
            let (name, name_span) = self.expect_name("a pass name");
            return Some(Item::ExternPass(ExternPass { name, span: start.cover(name_span) }));
        }
        if self.at_name("rule") {
            self.bump();
            let (name, _) = self.expect_name("a rule name");
            if !self.at_name("from") {
                self.error_here(format!(
                    "expected `from`, found {}",
                    self.cur().kind.describe()
                ));
                self.recover_to_elab_item();
                return None;
            }
            self.bump();
            let (from, _) = self.expect_name("a source language");
            if !self.at_name("to") {
                self.error_here(format!(
                    "expected `to`, found {}",
                    self.cur().kind.describe()
                ));
                self.recover_to_elab_item();
                return None;
            }
            self.bump();
            let (to, to_span) = self.expect_name("a target language");
            return Some(Item::ExternRule(ExternRule {
                name,
                from,
                to,
                span: start.cover(to_span),
            }));
        }
        self.error_here(format!(
            "expected `rule` or `pass` after `extern`, found {}",
            self.cur().kind.describe()
        ));
        self.recover_to_elab_item();
        None
    }

    // === type files (D-16/D-17/D-23) ===

    fn parse_type(&mut self) -> File {
        let mut items = Vec::new();
        while !self.at_eof() {
            let before = self.pos;
            if let Some(item) = self.parse_type_item() {
                items.push(item);
            }
            if self.pos == before {
                self.error_here(format!(
                    "expected `context`, a judgment declaration, or a rule, found {}",
                    self.cur().kind.describe()
                ));
                self.pos += 1;
                self.recover_to_type_item();
            }
        }
        File { items }
    }

    fn recover_to_type_item(&mut self) {
        while !self.at_eof() {
            if self.at_name("context") || matches!(self.cur().kind, TokenKind::Name(_)) {
                return;
            }
            self.pos += 1;
        }
    }

    fn parse_type_item(&mut self) -> Option<Item> {
        if self.at_name("context") {
            return self.parse_context_decl();
        }
        if !matches!(self.cur().kind, TokenKind::Name(_)) {
            return None;
        }
        if self.at_judgment_decl() {
            return self.parse_judgment_decl();
        }
        self.parse_judgment_rule()
    }

    /// `context Γ = [Ident: TypeV]`
    fn parse_context_decl(&mut self) -> Option<Item> {
        let start = self.bump().span; // context
        let (name, name_span) = self.expect_name("a context name");
        self.expect_punct('=');
        self.expect_punct('[');
        let (key_sort, _) = self.expect_name("a key sort");
        self.expect_punct(':');
        let (value_sort, _) = self.expect_name("a value sort");
        let end = self.cur().span;
        self.expect_punct(']');
        Some(Item::ContextDecl(ContextDecl {
            name,
            name_span,
            key_sort,
            value_sort,
            span: start.cover(end),
        }))
    }

    fn at_arrow(&self) -> bool {
        matches!(self.cur().kind, TokenKind::Sym("->" | "<-"))
    }

    /// Lookahead: a declaration is `Name Name (arrow Name)+ (with Name
    /// (, Name)*)?` followed by another item start (a Name or EOF) —
    /// a rule always continues with `:=` instead (D-17).
    fn at_judgment_decl(&self) -> bool {
        let name_at = |j: usize| matches!(self.nth(j).kind, TokenKind::Name(_));
        if !name_at(0) || !name_at(1) {
            return false;
        }
        let mut j = 2;
        let mut arrows = 0;
        while matches!(self.nth(j).kind, TokenKind::Sym("->" | "<-")) {
            if !name_at(j + 1) {
                return false;
            }
            arrows += 1;
            j += 2;
        }
        if arrows == 0 {
            return false;
        }
        if matches!(&self.nth(j).kind, TokenKind::Name(n) if n == "with") {
            if !name_at(j + 1) {
                return false;
            }
            j += 2;
            while self.nth(j).kind == TokenKind::Punct(',') {
                if !name_at(j + 1) {
                    return false;
                }
                j += 2;
            }
        }
        matches!(self.nth(j).kind, TokenKind::Name(_) | TokenKind::Eof)
    }

    /// `infer_C MIR -> TypeC with Γ`
    fn parse_judgment_decl(&mut self) -> Option<Item> {
        let (name, name_span) = self.expect_name("a judgment name");
        let mut params = vec![{
            let (p, s) = self.expect_name("a sort");
            (p, s)
        }];
        let mut end = params[0].1;
        while self.at_arrow() {
            self.bump();
            let (p, s) = self.expect_name("a sort");
            end = s;
            params.push((p, s));
        }
        let mut contexts = Vec::new();
        if self.at_name("with") {
            self.bump();
            loop {
                let (c, s) = self.expect_name("a context name");
                end = s;
                contexts.push((c, s));
                if !self.eat_punct(',') {
                    break;
                }
            }
        }
        Some(Item::JudgmentDecl(JudgmentDecl {
            name,
            name_span,
            params,
            contexts,
            span: name_span.cover(end),
        }))
    }

    /// `head := goal, goal, …`
    fn parse_judgment_rule(&mut self) -> Option<Item> {
        let (judgment, judgment_span) = self.expect_name("a judgment name");
        let mut params = Vec::new();
        while !self.at_sym(":=") && !self.at_eof() {
            if self.at_arrow() {
                self.bump(); // arrows are separators (D-17)
                continue;
            }
            let before = self.pos;
            let Some(param) = self.parse_term_expr() else {
                self.recover_to_type_item();
                return None;
            };
            params.push(param);
            if self.pos == before {
                self.pos += 1;
            }
        }
        if !self.expect_sym(":=") {
            return None;
        }
        let mut body = Vec::new();
        loop {
            let Some(goal) = self.parse_body_goal() else {
                self.recover_to_type_item();
                break;
            };
            body.push(goal);
            if !self.eat_punct(',') {
                break;
            }
        }
        let end = body.last().map(goal_span).unwrap_or(judgment_span);
        Some(Item::JudgmentRule(JudgmentRule {
            judgment,
            judgment_span,
            params,
            body,
            span: judgment_span.cover(end),
        }))
    }

    /// A goal: a bare call (`check_C $a $b with Γ+{a: b}`), a
    /// parenthesized call, or a unification `a = b`.
    fn parse_body_goal(&mut self) -> Option<BodyGoal> {
        // Bare call: a Name directly followed by a term start that is
        // not `{` (which would open the Name's own field block).
        if matches!(self.cur().kind, TokenKind::Name(_))
            && matches!(
                self.nth(1).kind,
                TokenKind::Var(_) | TokenKind::Str(_) | TokenKind::Punct('(')
            )
        {
            return Some(BodyGoal::Call(self.parse_bare_call(false)?));
        }
        let lhs = self.parse_term_expr()?;
        if self.eat_punct('=') {
            let rhs = self.parse_term_expr()?;
            return Some(BodyGoal::Unify(lhs, rhs));
        }
        match lhs {
            TermExpr::Call(call) => Some(BodyGoal::Call(call)),
            other => {
                self.diags.push(Diagnostic::error(
                    self.file,
                    other.span(),
                    "a goal must be a judgment call or a unification `a = b`",
                ));
                None
            }
        }
    }

    /// A judgment call after its name. Unbounded (top-level) calls
    /// only take `$var`/`'lit'`/`(…)` arguments — a bare Name would be
    /// ambiguous with the next item's start; the parenthesized form
    /// (`bounded`) allows node-pattern arguments too.
    fn parse_bare_call(&mut self, bounded: bool) -> Option<CallGoal> {
        let (judgment, judgment_span) = self.expect_name("a judgment name");
        let mut args = Vec::new();
        let mut end = judgment_span;
        loop {
            if self.at_arrow() {
                self.bump();
                continue;
            }
            if !self.at_term_start(bounded) {
                break;
            }
            let arg = self.parse_term_expr()?;
            end = arg.span();
            args.push(arg);
        }
        let extends = self.parse_ctx_exts(&mut end)?;
        Some(CallGoal { judgment, judgment_span, args, extends, span: judgment_span.cover(end) })
    }

    fn at_term_start(&self, allow_name: bool) -> bool {
        match &self.cur().kind {
            TokenKind::Var(_) | TokenKind::Str(_) | TokenKind::Punct('(') => true,
            // `with` ends the argument list either way.
            TokenKind::Name(n) => allow_name && n != "with",
            _ => false,
        }
    }

    /// `with Γ+{a: b} (, Δ+{c: d})*` — empty when there is no `with`.
    fn parse_ctx_exts(&mut self, end: &mut Span) -> Option<Vec<CtxExt>> {
        let mut extends = Vec::new();
        if !self.at_name("with") {
            return Some(extends);
        }
        self.bump();
        loop {
            let (ctx, ctx_span) = self.expect_name("a context name");
            self.expect_punct('+');
            self.expect_punct('{');
            let key = self.parse_term_expr()?;
            self.expect_punct(':');
            let value = self.parse_term_expr()?;
            *end = self.cur().span;
            self.expect_punct('}');
            extends.push(CtxExt { ctx, ctx_span, key, value });
            if !self.eat_punct(',') {
                break;
            }
        }
        Some(extends)
    }

    fn parse_term_expr(&mut self) -> Option<TermExpr> {
        let span = self.cur().span;
        match self.cur().kind.clone() {
            TokenKind::Var(name) => {
                self.pos += 1;
                Some(TermExpr::Var { name, span })
            }
            TokenKind::Str(text) => {
                self.pos += 1;
                Some(TermExpr::Lit { text, span })
            }
            // `(check_V $e <- $t)` — a parenthesized judgment call.
            TokenKind::Punct('(') => {
                self.pos += 1;
                let call = self.parse_bare_call(true)?;
                let end = self.cur().span;
                self.expect_punct(')');
                Some(TermExpr::Call(CallGoal { span: span.cover(end), ..call }))
            }
            TokenKind::Name(name) => {
                self.pos += 1;
                // `Γ.$name` — a context read (D-16).
                if self.eat_punct('.') {
                    let key = self.parse_term_expr()?;
                    let end = key.span();
                    return Some(TermExpr::CtxRead {
                        ctx: name,
                        key: Box::new(key),
                        span: span.cover(end),
                    });
                }
                let mut fields = Vec::new();
                let mut end = span;
                if self.eat_punct('{') {
                    while !self.at_punct('}') && !self.at_eof() {
                        let (field, _) = self.expect_name("a field label");
                        if field.is_empty() {
                            self.recover_rule_body();
                            break;
                        }
                        self.expect_punct(':');
                        let value = self.parse_term_expr()?;
                        fields.push((field, value));
                        if !self.eat_punct(',') {
                            break;
                        }
                    }
                    end = self.cur().span;
                    self.expect_punct('}');
                }
                Some(TermExpr::Node { name, fields, span: span.cover(end) })
            }
            other => {
                self.error_here(format!("expected a term, found {}", other.describe()));
                None
            }
        }
    }

    // === patterns and constructions (D-35) ===

    fn parse_pat(&mut self) -> Option<Pat> {
        let span = self.cur().span;
        match self.cur().kind.clone() {
            TokenKind::Var(name) => {
                self.pos += 1;
                Some(Pat::Var { name, span })
            }
            TokenKind::Str(text) => {
                self.pos += 1;
                Some(Pat::Lit { text, span })
            }
            TokenKind::Punct('[') => {
                let (name, _, end) = self.parse_list_capture()?;
                Some(Pat::ListVar { name, span: span.cover(end) })
            }
            TokenKind::Name(_) => {
                let (lang, name, mut span) = self.parse_node_head()?;
                let mut fields = Vec::new();
                if self.eat_punct('{') {
                    while !self.at_punct('}') && !self.at_eof() {
                        let (field, _) = self.expect_name("a field label");
                        if field.is_empty() {
                            self.recover_rule_body();
                            break;
                        }
                        self.expect_punct(':');
                        let pat = self.parse_pat()?;
                        fields.push((field, pat));
                        if !self.eat_punct(',') {
                            break;
                        }
                    }
                    let end = self.cur().span;
                    self.expect_punct('}');
                    span = span.cover(end);
                }
                Some(Pat::Node { lang, name, fields, span })
            }
            other => {
                self.error_here(format!("expected a pattern, found {}", other.describe()));
                None
            }
        }
    }

    fn parse_con(&mut self) -> Option<Con> {
        let span = self.cur().span;
        match self.cur().kind.clone() {
            TokenKind::Var(name) => {
                self.pos += 1;
                // `$x to Lang`
                if self.at_name("to") {
                    self.bump();
                    let (lang, lang_span) = self.expect_name("a target language");
                    return Some(Con::VarTo { name, lang, span: span.cover(lang_span) });
                }
                // `$e[$b := $a]`
                if self.at_punct('[') {
                    self.pos += 1;
                    let var = self.expect_var("the bound metavariable")?;
                    self.expect_sym(":=");
                    let replacement = self.expect_var("the replacement metavariable")?;
                    let end = self.cur().span;
                    self.expect_punct(']');
                    return Some(Con::Subst {
                        target: name,
                        var,
                        replacement,
                        span: span.cover(end),
                    });
                }
                Some(Con::Var { name, span })
            }
            TokenKind::Str(text) => {
                self.pos += 1;
                Some(Con::Lit { text, span })
            }
            TokenKind::Punct('[') => {
                let (name, lang, end) = self.parse_list_capture()?;
                let Some(lang) = lang else {
                    self.diags.push(Diagnostic::error(
                        self.file,
                        span.cover(end),
                        "a list in a construction must recurse: `[$x* to Lang]`",
                    ));
                    return None;
                };
                Some(Con::ListVarTo { name, lang, span: span.cover(end) })
            }
            TokenKind::Name(_) => {
                let (lang, name, mut span) = self.parse_node_head()?;
                let mut fields = Vec::new();
                if self.eat_punct('{') {
                    while !self.at_punct('}') && !self.at_eof() {
                        let (field, _) = self.expect_name("a field label");
                        if field.is_empty() {
                            self.recover_rule_body();
                            break;
                        }
                        self.expect_punct(':');
                        let con = self.parse_con()?;
                        fields.push((field, con));
                        if !self.eat_punct(',') {
                            break;
                        }
                    }
                    let end = self.cur().span;
                    self.expect_punct('}');
                    span = span.cover(end);
                }
                Some(Con::Node { lang, name, fields, span })
            }
            other => {
                self.error_here(format!(
                    "expected a construction, found {}",
                    other.describe()
                ));
                None
            }
        }
    }

    /// `(Lang ::)? Name` — the head of a node pattern/construction.
    fn parse_node_head(&mut self) -> Option<(Option<String>, String, Span)> {
        let (first, first_span) = self.expect_name("a node name");
        if first.is_empty() {
            return None;
        }
        if self.at_sym("::") {
            self.pos += 1;
            let (name, name_span) = self.expect_name("a node name");
            if name.is_empty() {
                return None;
            }
            Some((Some(first), name, first_span.cover(name_span)))
        } else {
            Some((None, first, first_span))
        }
    }

    /// `[$x* (to Lang)? ]` minus the leading `[` decision — returns
    /// `(var, lang, closing span)`. The caller sits on `[`.
    fn parse_list_capture(&mut self) -> Option<(String, Option<String>, Span)> {
        self.expect_punct('[');
        let name = self.expect_var("a list metavariable")?;
        self.expect_punct('*');
        let mut lang = None;
        if self.at_name("to") {
            self.bump();
            let (l, _) = self.expect_name("a target language");
            if l.is_empty() {
                return None;
            }
            lang = Some(l);
        }
        let end = self.cur().span;
        self.expect_punct(']');
        Some((name, lang, end))
    }

    fn expect_var(&mut self, what: &str) -> Option<String> {
        if let TokenKind::Var(v) = &self.cur().kind {
            let v = v.clone();
            self.pos += 1;
            Some(v)
        } else {
            self.error_here(format!("expected {what}, found {}", self.cur().kind.describe()));
            None
        }
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

fn goal_span(goal: &BodyGoal) -> Span {
    match goal {
        BodyGoal::Unify(a, b) => a.span().cover(b.span()),
        BodyGoal::Call(c) => c.span,
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

    fn ty(text: &str) -> File {
        let (ast, diags) = parse_type_file("t.type.langue", text);
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        ast
    }

    #[test]
    fn type_file_decls_and_rules() {
        let ast = ty("\
context Γ = [Ident: TypeV]
infer_V MIR -> TypeV with Γ
infer_V NumV -> $t := $t = NamedTypeV { name: 'Number' }
infer_V VarV { name: $n } -> $t := $t = Γ.$n
infer_V ThunkV { body: $b } -> $t := $t = UTypeV { inner: (infer_C $b) }
check_C $a $b := check_C $a $b with Γ+{$a: $b}
");
        assert_eq!(ast.items.len(), 6);
        let Item::ContextDecl(c) = &ast.items[0] else { panic!("{:?}", ast.items[0]) };
        assert_eq!((c.name.as_str(), c.key_sort.as_str(), c.value_sort.as_str()),
            ("Γ", "Ident", "TypeV"));
        let Item::JudgmentDecl(d) = &ast.items[1] else { panic!("{:?}", ast.items[1]) };
        assert_eq!(d.params.len(), 2);
        assert_eq!(d.contexts[0].0, "Γ");
        let Item::JudgmentRule(r) = &ast.items[2] else { panic!("{:?}", ast.items[2]) };
        assert_eq!(r.params.len(), 2);
        assert!(matches!(&r.body[0], BodyGoal::Unify(_, _)));
        let Item::JudgmentRule(r) = &ast.items[3] else { panic!("{:?}", ast.items[3]) };
        let BodyGoal::Unify(_, rhs) = &r.body[0] else { panic!() };
        assert!(matches!(rhs, TermExpr::CtxRead { ctx, .. } if ctx == "Γ"));
        let Item::JudgmentRule(r) = &ast.items[4] else { panic!("{:?}", ast.items[4]) };
        let BodyGoal::Unify(_, rhs) = &r.body[0] else { panic!() };
        let TermExpr::Node { fields, .. } = rhs else { panic!() };
        assert!(matches!(&fields[0].1, TermExpr::Call(c) if c.judgment == "infer_C"));
        let Item::JudgmentRule(r) = &ast.items[5] else { panic!("{:?}", ast.items[5]) };
        let BodyGoal::Call(call) = &r.body[0] else { panic!() };
        assert_eq!(call.args.len(), 2);
        assert_eq!(call.extends.len(), 1);
        assert_eq!(call.extends[0].ctx, "Γ");
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

    // === elab files (D-35) ===

    fn elab(text: &str) -> File {
        let (ast, diags) = parse_elab_file("t.elab.langue", text);
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        ast
    }

    /// The locked D-35 from-block example, verbatim.
    #[test]
    fn from_block_locked_example() {
        let ast = elab(
            "from Lumo to MIR {\n  FnDecl { name: $n, param_list: ParamList { params: [$p*] }, body: $b }\n    ==> Lambda { params: [$p* to MIR], body: $b to MIR }\n}",
        );
        let Item::ElabBlock(b) = &ast.items[0] else { panic!() };
        assert_eq!((b.from.as_str(), b.to.as_str()), ("Lumo", "MIR"));
        assert_eq!(b.rules.len(), 1);
        let rule = &b.rules[0];
        let Pat::Node { lang, name, fields, .. } = &rule.pattern else { panic!() };
        assert_eq!(lang, &None);
        assert_eq!(name, "FnDecl");
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].0, "name");
        assert!(matches!(&fields[0].1, Pat::Var { name, .. } if name == "n"));
        let Pat::Node { name, fields: inner, .. } = &fields[1].1 else { panic!() };
        assert_eq!(name, "ParamList");
        assert!(matches!(&inner[0].1, Pat::ListVar { name, .. } if name == "p"));
        let Con::Node { name, fields, .. } = &rule.construction else { panic!() };
        assert_eq!(name, "Lambda");
        assert!(matches!(
            &fields[0].1,
            Con::ListVarTo { name, lang, .. } if name == "p" && lang == "MIR"
        ));
        assert!(matches!(
            &fields[1].1,
            Con::VarTo { name, lang, .. } if name == "b" && lang == "MIR"
        ));
    }

    /// The locked D-14/D-35 between example, verbatim.
    #[test]
    fn between_block_locked_example() {
        let ast = elab(
            "between MIR {\n  Apply { fn: Lambda { param: $b, body: $e }, arg: $a } === $e[$b := $a]\n}",
        );
        let Item::BetweenBlock(b) = &ast.items[0] else { panic!() };
        assert_eq!(b.lang, "MIR");
        assert_eq!(b.relations.len(), 1);
        let rel = &b.relations[0];
        let Pat::Node { name, fields, .. } = &rel.lhs else { panic!() };
        assert_eq!(name, "Apply");
        assert_eq!(fields[0].0, "fn");
        assert!(matches!(
            &rel.rhs,
            Con::Subst { target, var, replacement, .. }
                if target == "e" && var == "b" && replacement == "a"
        ));
    }

    #[test]
    fn qualified_names_literals_and_bare_nodes() {
        let ast = elab(
            "from Lumo to MIR {\n  Lumo::WildcardPattern ==> MIR::VarV { name: 'x' }\n  NumberExpr { value: $v } ==> NumV { value: $v }\n}",
        );
        let Item::ElabBlock(b) = &ast.items[0] else { panic!() };
        assert_eq!(b.rules.len(), 2);
        let Pat::Node { lang, name, fields, .. } = &b.rules[0].pattern else { panic!() };
        assert_eq!(lang.as_deref(), Some("Lumo"));
        assert_eq!(name, "WildcardPattern");
        assert!(fields.is_empty());
        let Con::Node { lang, fields, .. } = &b.rules[0].construction else { panic!() };
        assert_eq!(lang.as_deref(), Some("MIR"));
        assert!(matches!(&fields[0].1, Con::Lit { text, .. } if text == "x"));
    }

    #[test]
    fn extern_rule_and_pass() {
        let ast = elab(
            "extern rule member_classify from Lumo to MIR\nextern pass scc_fix\nextern pass use_require",
        );
        assert_eq!(ast.items.len(), 3);
        let Item::ExternRule(r) = &ast.items[0] else { panic!() };
        assert_eq!(r.name, "member_classify");
        assert_eq!((r.from.as_str(), r.to.as_str()), ("Lumo", "MIR"));
        let Item::ExternPass(p) = &ast.items[1] else { panic!() };
        assert_eq!(p.name, "scc_fix");
    }

    #[test]
    fn literal_pattern_and_trailing_comma() {
        let ast = elab(
            "from Lumo to MIR {\n  Attribute { name: 'extern', } ==> VarV { name: $n, }\n}",
        );
        let Item::ElabBlock(b) = &ast.items[0] else { panic!() };
        let Pat::Node { fields, .. } = &b.rules[0].pattern else { panic!() };
        assert!(matches!(&fields[0].1, Pat::Lit { text, .. } if text == "extern"));
    }

    #[test]
    fn broken_rule_recovers_to_next_item() {
        let (ast, diags) = parse_elab_file(
            "t.elab.langue",
            "from Lumo to MIR {\n  FnDecl { ==> Bad\n  }\n}\nextern pass scc_fix",
        );
        assert!(!diags.is_empty());
        assert!(ast
            .items
            .iter()
            .any(|i| matches!(i, Item::ExternPass(p) if p.name == "scc_fix")));
    }

    #[test]
    fn list_construction_without_to_is_error() {
        let (_, diags) = parse_elab_file(
            "t.elab.langue",
            "from Lumo to MIR {\n  ParamList { params: [$p*] } ==> ValueArgs { args: [$p*] }\n}",
        );
        assert!(
            diags.iter().any(|d| d.message.contains("must recurse")),
            "{diags:?}"
        );
    }
}
