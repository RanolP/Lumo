use lumo_lexer::{lex_lossless, Keyword, LosslessTokenKind as LexKind};
use lumo_span::Span;

use crate::syntax_kind::SyntaxKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LosslessToken {
    pub kind: SyntaxKind,
    pub span: Span,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxElement {
    Node(Box<SyntaxNode>),
    Token(LosslessToken),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxNode {
    pub kind: SyntaxKind,
    pub span: Span,
    pub children: Vec<SyntaxElement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutput {
    pub root: SyntaxNode,
    pub errors: Vec<ParseError>,
}

pub fn parse(source: &str) -> ParseOutput {
    let lexed = lex_lossless(source);
    let mut p = Parser {
        tokens: lexed.tokens,
        index: 0,
        errors: Vec::new(),
    };
    let root = p.parse_file();
    ParseOutput {
        root,
        errors: p.errors,
    }
}

pub fn node_text(node: &SyntaxNode) -> String {
    let mut out = String::new();
    write_node_text(node, &mut out);
    out
}

fn write_node_text(node: &SyntaxNode, out: &mut String) {
    for child in &node.children {
        match child {
            SyntaxElement::Node(n) => write_node_text(n, out),
            SyntaxElement::Token(t) => out.push_str(&t.text),
        }
    }
}

fn lexer_kind_to_syntax_kind(kind: &LexKind, text: &str) -> SyntaxKind {
    match kind {
        LexKind::Ident => SyntaxKind::IDENT,
        LexKind::StringLit => SyntaxKind::STRING_LIT,
        LexKind::NumberLit => SyntaxKind::NUMBER_LIT,
        LexKind::Whitespace => SyntaxKind::WHITESPACE,
        LexKind::Newline => SyntaxKind::NEWLINE,
        LexKind::Unknown => SyntaxKind::UNKNOWN,
        LexKind::Keyword(kw) => match kw {
            Keyword::Data => SyntaxKind::DATA_KW,
            Keyword::Fn => SyntaxKind::FN_KW,
            Keyword::Extern => SyntaxKind::EXTERN_KW,
            Keyword::Let => SyntaxKind::LET_KW,
            Keyword::In => SyntaxKind::IN_KW,
            Keyword::Thunk => SyntaxKind::THUNK_KW,
            Keyword::Force => SyntaxKind::FORCE_KW,
            Keyword::Match => SyntaxKind::MATCH_KW,
            Keyword::Cap => SyntaxKind::CAP_KW,
            Keyword::Handle => SyntaxKind::HANDLE_KW,
            Keyword::Bundle => SyntaxKind::BUNDLE_KW,
            Keyword::Use => SyntaxKind::USE_KW,
            Keyword::Impl => SyntaxKind::IMPL_KW,
            Keyword::If => SyntaxKind::IF_KW,
            Keyword::Else => SyntaxKind::ELSE_KW,
            _ => SyntaxKind::UNKNOWN,
        },
        LexKind::Symbol(_) => SyntaxKind::from_symbol(text).unwrap_or(SyntaxKind::UNKNOWN),
    }
}

struct Parser {
    tokens: Vec<lumo_lexer::LosslessToken>,
    index: usize,
    errors: Vec<ParseError>,
}

// ── Primitive helpers ──────────────────────────────────────────────────────────

impl Parser {
    fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn current(&self) -> Option<&lumo_lexer::LosslessToken> {
        self.tokens.get(self.index)
    }

    fn bump(&mut self) -> Option<LosslessToken> {
        let token = self.tokens.get(self.index).cloned();
        if token.is_some() {
            self.index += 1;
        }
        token.map(lexer_token_to_lst)
    }

    fn current_span(&self) -> Span {
        self.current()
            .map(|t| t.span)
            .or_else(|| {
                self.index.checked_sub(1).and_then(|idx| {
                    self.tokens
                        .get(idx)
                        .map(|t| Span::new(t.span.end, t.span.end))
                })
            })
            .unwrap_or(Span::new(0, 0))
    }

    fn error_here(&mut self, message: &str) {
        let span = self.current_span();
        self.errors.push(ParseError {
            span,
            message: message.to_owned(),
        });
    }

    // ── Trivia ────────────────────────────────────────────────────────────────

    fn is_trivia_lex(kind: &LexKind) -> bool {
        matches!(kind, LexKind::Whitespace | LexKind::Newline)
    }

    fn at_trivia(&self) -> bool {
        self.current()
            .map(|t| Self::is_trivia_lex(&t.kind))
            .unwrap_or(false)
    }

    fn skip_trivia_into(&mut self, children: &mut Vec<SyntaxElement>) {
        while self.at_trivia() {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }
    }

    // ── Non-trivia peek ───────────────────────────────────────────────────────

    /// Returns the nth non-trivia token (0 = first), without consuming.
    fn peek_non_trivia_token(&self, n: usize) -> Option<&lumo_lexer::LosslessToken> {
        let mut count = 0;
        let mut i = self.index;
        while i < self.tokens.len() {
            let tok = &self.tokens[i];
            if !Self::is_trivia_lex(&tok.kind) {
                if count == n {
                    return Some(tok);
                }
                count += 1;
            }
            i += 1;
        }
        None
    }

    fn at_non_trivia_keyword(&self, kw: Keyword) -> bool {
        matches!(
            self.peek_non_trivia_token(0).map(|t| &t.kind),
            Some(LexKind::Keyword(actual)) if *actual == kw
        )
    }

    fn at_non_trivia_symbol(&self, text: &str) -> bool {
        self.peek_non_trivia_token(0)
            .map(|t| t.text.as_str())
            == Some(text)
    }

    fn at_non_trivia_ident(&self) -> bool {
        matches!(
            self.peek_non_trivia_token(0).map(|t| &t.kind),
            Some(LexKind::Ident)
        )
    }

    fn at_non_trivia_ident_text(&self, text: &str) -> bool {
        matches!(
            self.peek_non_trivia_token(0),
            Some(tok) if matches!(tok.kind, LexKind::Ident) && tok.text == text
        )
    }

    fn peek_next_non_trivia_symbol(&self) -> Option<&str> {
        self.peek_non_trivia_token(1).map(|t| t.text.as_str())
    }

    // ── "At" predicates (current token, no trivia skip) ─────────────────────

    fn at_keyword(&self, keyword: Keyword) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(LexKind::Keyword(actual)) if *actual == keyword
        )
    }

    fn at_ident(&self) -> bool {
        matches!(self.current().map(|t| &t.kind), Some(LexKind::Ident))
    }

    fn at_ident_or_keyword(&self) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(LexKind::Ident) | Some(LexKind::Keyword(_))
        )
    }

    fn at_string_lit(&self) -> bool {
        matches!(self.current().map(|t| &t.kind), Some(LexKind::StringLit))
    }

    fn at_number_lit(&self) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(LexKind::NumberLit)
        )
    }

    fn at_symbol_text(&self, text: &str) -> bool {
        self.current().map(|t| t.text.as_str()) == Some(text)
    }

    fn at_trivia_or_unknown(&self) -> bool {
        self.current()
            .map(|t| matches!(t.kind, LexKind::Whitespace | LexKind::Newline | LexKind::Unknown))
            .unwrap_or(false)
    }

    // ── Infix-op peek (skips trivia) ─────────────────────────────────────────

    /// Returns `(op_syntax_kind, l_bp, r_bp)` for the next non-trivia infix binary op.
    fn peek_infix_op_non_trivia(&self) -> Option<(SyntaxKind, u8, u8)> {
        let tok = self.peek_non_trivia_token(0)?;
        let text = tok.text.as_str();
        let (sk, l, r) = match text {
            "||" => (SyntaxKind::PIPE_PIPE, 3u8, 4u8),
            "&&" => (SyntaxKind::AMP_AMP, 5, 6),
            "==" => (SyntaxKind::EQ_EQ, 7, 8),
            "!=" => (SyntaxKind::BANG_EQ, 7, 8),
            "<" => (SyntaxKind::LT, 9, 10),
            "<=" => (SyntaxKind::LT_EQ, 9, 10),
            ">" => (SyntaxKind::GT, 9, 10),
            ">=" => (SyntaxKind::GT_EQ, 9, 10),
            "+" => (SyntaxKind::PLUS, 11, 12),
            "-" => (SyntaxKind::MINUS, 11, 12),
            "*" => (SyntaxKind::STAR, 13, 14),
            "/" => (SyntaxKind::SLASH, 13, 14),
            "%" => (SyntaxKind::PERCENT, 13, 14),
            _ => return None,
        };
        // Only numeric operators — not idents/keywords
        if !matches!(tok.kind, LexKind::Symbol(_)) {
            return None;
        }
        Some((sk, l, r))
    }
}

// ── Top-level ──────────────────────────────────────────────────────────────────

impl Parser {
    fn parse_file(&mut self) -> SyntaxNode {
        let mut children = Vec::new();

        while !self.eof() {
            self.skip_trivia_into(&mut children);
            if self.eof() {
                break;
            }

            // Collect leading attributes into a temporary list
            let mut attr_children: Vec<SyntaxElement> = Vec::new();
            while self.at_symbol_text("#") {
                let attr = self.parse_attribute();
                attr_children.push(SyntaxElement::Node(Box::new(attr)));
                self.skip_trivia_into(&mut attr_children);
            }

            if self.eof() {
                // dangling attributes at EOF
                children.extend(attr_children);
                break;
            }

            let item = self.parse_item_with_attrs(attr_children);
            children.push(SyntaxElement::Node(Box::new(item)));
        }

        node_from_children(SyntaxKind::FILE, children)
    }

    fn parse_item_with_attrs(&mut self, attr_children: Vec<SyntaxElement>) -> SyntaxNode {
        if self.at_keyword(Keyword::Fn) {
            self.parse_fn_decl(attr_children)
        } else if self.at_keyword(Keyword::Data) {
            self.parse_data_decl(attr_children)
        } else if self.at_keyword(Keyword::Cap) {
            self.parse_cap_decl(attr_children)
        } else if self.at_keyword(Keyword::Extern) {
            self.parse_extern_decl(attr_children)
        } else if self.at_keyword(Keyword::Use) {
            self.parse_use_decl(attr_children)
        } else if self.at_keyword(Keyword::Impl) {
            self.parse_impl_decl(attr_children)
        } else {
            self.error_here(
                "expected top-level declaration (`fn`, `data`, `cap`, `extern`, `use`, `impl`)",
            );
            let mut children = attr_children;
            children.push(SyntaxElement::Node(Box::new(self.parse_error_node())));
            node_from_children(SyntaxKind::ERROR, children)
        }
    }
}

// ── Attributes ────────────────────────────────────────────────────────────────

impl Parser {
    fn parse_attribute(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '#'

        self.skip_trivia_into(&mut children);

        if self.at_symbol_text("[") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '['
        } else {
            self.error_here("expected `[` after `#`");
            return node_from_children(SyntaxKind::ATTRIBUTE, children);
        }

        self.skip_trivia_into(&mut children);

        // Accept ident or keyword (e.g. `#[extern(...)]` uses the keyword `extern`)
        if self.at_ident_or_keyword() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        } else {
            self.error_here("expected attribute name");
        }

        self.skip_trivia_into(&mut children);

        if self.at_symbol_text("(") {
            let args = self.parse_attribute_args();
            children.push(SyntaxElement::Node(Box::new(args)));
            self.skip_trivia_into(&mut children);
        }

        if self.at_symbol_text("]") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ']'
        } else {
            self.error_here("expected `]` to close attribute");
        }

        node_from_children(SyntaxKind::ATTRIBUTE, children)
    }

    fn parse_attribute_args(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '('

        loop {
            self.skip_trivia_into(&mut children);
            if self.eof() || self.at_symbol_text(")") {
                break;
            }

            // AttributeArgItem: ArgIdent (just ident) or Arg (ident '=' expr)
            let item = if self.at_ident()
                && self.peek_next_non_trivia_symbol() == Some("=")
            {
                self.parse_attribute_arg()
            } else if self.at_ident() {
                self.parse_attribute_arg_ident()
            } else {
                break;
            };
            children.push(SyntaxElement::Node(Box::new(item)));

            self.skip_trivia_into(&mut children);
            if self.at_symbol_text(",") {
                children.push(SyntaxElement::Token(self.bump().unwrap()));
            }
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text(")") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ')'
        } else {
            self.error_here("expected `)` in attribute args");
        }

        node_from_children(SyntaxKind::ATTRIBUTE_ARGS, children)
    }

    fn parse_attribute_arg_ident(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // ident
        node_from_children(SyntaxKind::ATTRIBUTE_ARG_IDENT, children)
    }

    fn parse_attribute_arg(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // key:Ident
        self.skip_trivia_into(&mut children);
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '='
        let value = self.parse_expr_bp(0);
        children.push(SyntaxElement::Node(Box::new(value)));
        node_from_children(SyntaxKind::ATTRIBUTE_ARG, children)
    }
}

// ── Types ─────────────────────────────────────────────────────────────────────

impl Parser {
    fn parse_type_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        self.skip_trivia_into(&mut children);

        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        } else {
            self.error_here("expected type name");
            return node_from_children(SyntaxKind::TYPE_EXPR, children);
        }

        self.skip_trivia_into(&mut children);

        if self.at_symbol_text("[") {
            let args = self.parse_generic_args();
            children.push(SyntaxElement::Node(Box::new(args)));
        }

        node_from_children(SyntaxKind::TYPE_EXPR, children)
    }

    fn parse_generic_args(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '['

        loop {
            self.skip_trivia_into(&mut children);
            if self.eof() || self.at_symbol_text("]") {
                break;
            }
            let ty = self.parse_type_expr();
            children.push(SyntaxElement::Node(Box::new(ty)));
            self.skip_trivia_into(&mut children);
            if self.at_symbol_text(",") {
                children.push(SyntaxElement::Token(self.bump().unwrap()));
            }
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("]") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ']'
        } else {
            self.error_here("expected `]` in generic args");
        }

        node_from_children(SyntaxKind::GENERIC_ARGS, children)
    }

    fn parse_generic_params(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '['

        loop {
            self.skip_trivia_into(&mut children);
            if self.eof() || self.at_symbol_text("]") {
                break;
            }
            if self.at_ident() {
                let param = self.parse_generic_param();
                children.push(SyntaxElement::Node(Box::new(param)));
                self.skip_trivia_into(&mut children);
                if self.at_symbol_text(",") {
                    children.push(SyntaxElement::Token(self.bump().unwrap()));
                }
            } else {
                break;
            }
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("]") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ']'
        } else {
            self.error_here("expected `]` in generic params");
        }

        node_from_children(SyntaxKind::GENERIC_PARAMS, children)
    }

    fn parse_generic_param(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // name:Ident

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text(":") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ':'
            let constraint = self.parse_type_expr();
            children.push(SyntaxElement::Node(Box::new(constraint)));
        }

        node_from_children(SyntaxKind::GENERIC_PARAM, children)
    }
}

// ── Parameters ────────────────────────────────────────────────────────────────

impl Parser {
    fn parse_param_list(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '('

        loop {
            self.skip_trivia_into(&mut children);
            if self.eof() || self.at_symbol_text(")") {
                break;
            }
            // A param starts with an ident
            if self.at_ident() {
                let param = self.parse_param();
                children.push(SyntaxElement::Node(Box::new(param)));
                self.skip_trivia_into(&mut children);
                if self.at_symbol_text(",") {
                    children.push(SyntaxElement::Token(self.bump().unwrap()));
                }
            } else {
                break;
            }
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text(")") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ')'
        } else {
            self.error_here("expected `)` in param list");
        }

        node_from_children(SyntaxKind::PARAM_LIST, children)
    }

    fn parse_param(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // name:Ident

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text(":") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ':'
            let ty = self.parse_type_expr();
            children.push(SyntaxElement::Node(Box::new(ty)));
        } else {
            self.error_here("expected `:` in param");
        }

        node_from_children(SyntaxKind::PARAM, children)
    }

    fn parse_impl_param_list(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '('

        loop {
            self.skip_trivia_into(&mut children);
            if self.eof() || self.at_symbol_text(")") {
                break;
            }
            if self.at_ident() {
                // `self` without type annotation is allowed in impl methods
                let next_sym = self.peek_next_non_trivia_symbol();
                if next_sym == Some(":") {
                    let param = self.parse_param();
                    children.push(SyntaxElement::Node(Box::new(param)));
                } else {
                    // bare `self` — emit as PARAM with just the name
                    let mut param_children = Vec::new();
                    param_children.push(SyntaxElement::Token(self.bump().unwrap()));
                    children.push(SyntaxElement::Node(Box::new(
                        node_from_children(SyntaxKind::PARAM, param_children),
                    )));
                }
                self.skip_trivia_into(&mut children);
                if self.at_symbol_text(",") {
                    children.push(SyntaxElement::Token(self.bump().unwrap()));
                }
            } else {
                break;
            }
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text(")") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ')'
        } else {
            self.error_here("expected `)` in param list");
        }

        node_from_children(SyntaxKind::PARAM_LIST, children)
    }
}

// ── Cap annotation ────────────────────────────────────────────────────────────

impl Parser {
    fn parse_cap_annotation(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '/'
        let cap_sig = self.parse_cap_sig();
        children.push(SyntaxElement::Node(Box::new(cap_sig)));
        node_from_children(SyntaxKind::CAP_ANNOTATION, children)
    }

    fn parse_cap_sig(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        self.skip_trivia_into(&mut children);

        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        } else {
            self.error_here("expected capability name");
            return node_from_children(SyntaxKind::CAP_SIG, children);
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("[") {
            let args = self.parse_generic_args();
            children.push(SyntaxElement::Node(Box::new(args)));
        }

        node_from_children(SyntaxKind::CAP_SIG, children)
    }
}

// ── Function declarations ─────────────────────────────────────────────────────

impl Parser {
    fn parse_fn_decl(&mut self, mut children: Vec<SyntaxElement>) -> SyntaxNode {
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'fn'
        self.skip_trivia_into(&mut children);

        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        } else {
            self.error_here("expected function name");
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("[") {
            let gp = self.parse_generic_params();
            children.push(SyntaxElement::Node(Box::new(gp)));
            self.skip_trivia_into(&mut children);
        }

        if self.at_symbol_text("(") {
            let pl = self.parse_param_list();
            children.push(SyntaxElement::Node(Box::new(pl)));
        } else {
            self.error_here("expected `(` in fn declaration");
        }

        self.skip_trivia_into(&mut children);

        // optional return type
        if self.at_symbol_text(":") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ':'
            let ret_ty = self.parse_type_expr();
            children.push(SyntaxElement::Node(Box::new(ret_ty)));
            self.skip_trivia_into(&mut children);
        }

        // optional cap annotation '/ CapSig'
        if self.at_symbol_text("/") && !self.at_non_trivia_symbol("=") {
            // Make sure it's a cap annotation and not a division (here / is right after the sig)
            let cap_ann = self.parse_cap_annotation();
            children.push(SyntaxElement::Node(Box::new(cap_ann)));
            self.skip_trivia_into(&mut children);
        }

        // body
        let body = self.parse_fn_body();
        children.push(SyntaxElement::Node(Box::new(body)));

        node_from_children(SyntaxKind::FN_DECL, children)
    }

    fn parse_fn_body(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        self.skip_trivia_into(&mut children);

        if self.at_symbol_text("=") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '='
            let expr = self.parse_expr_bp(0);
            children.push(SyntaxElement::Node(Box::new(expr)));
            node_from_children(SyntaxKind::EXPR_BODY, children)
        } else if self.at_symbol_text("{") {
            // Prepend trivia then parse block (block starts with '{')
            // We already consumed trivia into children — but parse_block_expr
            // expects to start at `{`. Need to handle trivia differently here.
            // Actually: parse_block_expr will push `{` itself. The trivia in
            // `children` are just leading trivia. Flatten into the block node.
            let mut block = self.parse_block_expr();
            // Prepend the leading trivia to the block's children
            let mut new_block_children = children;
            new_block_children.extend(block.children);
            block.children = new_block_children;
            block.span = span_from_children(&block.children);
            block
        } else {
            self.error_here("expected `{` or `=` in fn body");
            node_from_children(SyntaxKind::ERROR, children)
        }
    }
}

// ── Data declarations ─────────────────────────────────────────────────────────

impl Parser {
    fn parse_data_decl(&mut self, mut children: Vec<SyntaxElement>) -> SyntaxNode {
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'data'
        self.skip_trivia_into(&mut children);

        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        } else {
            self.error_here("expected data type name");
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("[") {
            let gp = self.parse_generic_params();
            children.push(SyntaxElement::Node(Box::new(gp)));
            self.skip_trivia_into(&mut children);
        }

        if self.at_symbol_text("{") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '{'
        } else {
            self.error_here("expected `{` in data declaration");
            return node_from_children(SyntaxKind::DATA_DECL, children);
        }

        loop {
            self.skip_trivia_into(&mut children);
            if self.eof() || self.at_symbol_text("}") {
                break;
            }
            // variants start with '#' (attribute) or '.'
            if self.at_symbol_text("#") || self.at_symbol_text(".") {
                let variant = self.parse_variant();
                children.push(SyntaxElement::Node(Box::new(variant)));
                self.skip_trivia_into(&mut children);
                if self.at_symbol_text(",") {
                    children.push(SyntaxElement::Token(self.bump().unwrap()));
                }
            } else {
                self.error_here("expected variant (`.name`) in data declaration");
                children.push(SyntaxElement::Node(Box::new(self.parse_error_node())));
                break;
            }
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '}'
        } else {
            self.error_here("expected `}` in data declaration");
        }

        node_from_children(SyntaxKind::DATA_DECL, children)
    }

    fn parse_variant(&mut self) -> SyntaxNode {
        let mut children = Vec::new();

        // optional attributes
        while self.at_symbol_text("#") {
            let attr = self.parse_attribute();
            children.push(SyntaxElement::Node(Box::new(attr)));
            self.skip_trivia_into(&mut children);
        }

        if self.at_symbol_text(".") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '.'
        } else {
            self.error_here("expected `.` in variant");
        }

        self.skip_trivia_into(&mut children);
        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        } else {
            self.error_here("expected variant name");
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("(") {
            let fields = self.parse_variant_fields();
            children.push(SyntaxElement::Node(Box::new(fields)));
        }

        node_from_children(SyntaxKind::VARIANT, children)
    }

    fn parse_variant_fields(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '('

        loop {
            self.skip_trivia_into(&mut children);
            if self.eof() || self.at_symbol_text(")") {
                break;
            }
            if self.at_ident() {
                let ty = self.parse_type_expr();
                children.push(SyntaxElement::Node(Box::new(ty)));
                self.skip_trivia_into(&mut children);
                if self.at_symbol_text(",") {
                    children.push(SyntaxElement::Token(self.bump().unwrap()));
                }
            } else {
                break;
            }
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text(")") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ')'
        } else {
            self.error_here("expected `)` in variant fields");
        }

        node_from_children(SyntaxKind::VARIANT_FIELDS, children)
    }
}

// ── Cap declarations ──────────────────────────────────────────────────────────

impl Parser {
    fn parse_cap_decl(&mut self, mut children: Vec<SyntaxElement>) -> SyntaxNode {
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'cap'
        self.skip_trivia_into(&mut children);

        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        } else {
            self.error_here("expected capability name");
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("{") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '{'
        } else {
            self.error_here("expected `{` in cap declaration");
            return node_from_children(SyntaxKind::CAP_DECL, children);
        }

        loop {
            self.skip_trivia_into(&mut children);
            if self.eof() || self.at_symbol_text("}") {
                break;
            }
            if self.at_keyword(Keyword::Fn) {
                let op = self.parse_operation_decl();
                children.push(SyntaxElement::Node(Box::new(op)));
            } else {
                self.error_here("expected `fn` in cap declaration");
                children.push(SyntaxElement::Node(Box::new(self.parse_error_node())));
                break;
            }
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '}'
        } else {
            self.error_here("expected `}` in cap declaration");
        }

        node_from_children(SyntaxKind::CAP_DECL, children)
    }

    fn parse_operation_decl(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'fn'
        self.skip_trivia_into(&mut children);

        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("(") {
            let pl = self.parse_param_list();
            children.push(SyntaxElement::Node(Box::new(pl)));
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text(":") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ':'
            let ret_ty = self.parse_type_expr();
            children.push(SyntaxElement::Node(Box::new(ret_ty)));
        }

        node_from_children(SyntaxKind::OPERATION_DECL, children)
    }
}

// ── Extern declarations ───────────────────────────────────────────────────────

impl Parser {
    fn parse_extern_decl(&mut self, mut children: Vec<SyntaxElement>) -> SyntaxNode {
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'extern'
        self.skip_trivia_into(&mut children);

        // 'type' is not a lexer Keyword — it arrives as an Ident token
        if self.at_non_trivia_ident_text("type") {
            return self.parse_extern_type_decl(children);
        }

        if self.at_keyword(Keyword::Fn) {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // 'fn'
        } else {
            self.error_here("expected `fn` or `type` after `extern`");
        }

        self.skip_trivia_into(&mut children);
        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("(") {
            let pl = self.parse_param_list();
            children.push(SyntaxElement::Node(Box::new(pl)));
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text(":") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ':'
            let ret_ty = self.parse_type_expr();
            children.push(SyntaxElement::Node(Box::new(ret_ty)));
            self.skip_trivia_into(&mut children);
        }

        if self.at_symbol_text("/") {
            let cap_ann = self.parse_cap_annotation();
            children.push(SyntaxElement::Node(Box::new(cap_ann)));
            self.skip_trivia_into(&mut children);
        }

        if self.at_symbol_text(";") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ';'
        }

        node_from_children(SyntaxKind::EXTERN_FN_DECL, children)
    }

    fn parse_extern_type_decl(&mut self, mut children: Vec<SyntaxElement>) -> SyntaxNode {
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'type' (as ident)
        self.skip_trivia_into(&mut children);

        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        } else {
            self.error_here("expected type name after `extern type`");
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text(";") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ';'
        }

        node_from_children(SyntaxKind::EXTERN_TYPE_DECL, children)
    }
}

// ── Use declarations ──────────────────────────────────────────────────────────

impl Parser {
    fn parse_use_decl(&mut self, _attr_children: Vec<SyntaxElement>) -> SyntaxNode {
        let mut children = _attr_children;
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'use'
        self.skip_trivia_into(&mut children);

        let use_path = self.parse_use_path();
        children.push(SyntaxElement::Node(Box::new(use_path)));
        self.skip_trivia_into(&mut children);

        if self.at_symbol_text(";") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ';'
        } else {
            self.error_here("expected `;` in use declaration");
        }

        node_from_children(SyntaxKind::USE_DECL, children)
    }

    fn parse_use_path(&mut self) -> SyntaxNode {
        let mut children = Vec::new();

        // segments: Ident* separated by '.'
        while self.at_non_trivia_ident() {
            self.skip_trivia_into(&mut children);
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ident segment
            // optional '.' before next ident or use_tree
            self.skip_trivia_into(&mut children);
            if self.at_symbol_text(".") {
                // peek: if next after '.' is '{', it's UseTree; if ident, continue path
                let next = self.peek_non_trivia_token(1).map(|t| t.text.as_str());
                if next == Some("{") {
                    let use_tree = self.parse_use_tree();
                    children.push(SyntaxElement::Node(Box::new(use_tree)));
                    break;
                } else {
                    children.push(SyntaxElement::Token(self.bump().unwrap())); // '.'
                }
            } else {
                break;
            }
        }

        node_from_children(SyntaxKind::USE_PATH, children)
    }

    fn parse_use_tree(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '.'
        self.skip_trivia_into(&mut children);
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '{'

        loop {
            self.skip_trivia_into(&mut children);
            if self.eof() || self.at_symbol_text("}") {
                break;
            }
            if self.at_ident() {
                children.push(SyntaxElement::Token(self.bump().unwrap()));
                self.skip_trivia_into(&mut children);
                if self.at_symbol_text(",") {
                    children.push(SyntaxElement::Token(self.bump().unwrap()));
                }
            } else {
                break;
            }
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '}'
        } else {
            self.error_here("expected `}` in use tree");
        }

        node_from_children(SyntaxKind::USE_TREE, children)
    }
}

// ── Impl declarations ─────────────────────────────────────────────────────────

impl Parser {
    fn parse_impl_decl(&mut self, mut children: Vec<SyntaxElement>) -> SyntaxNode {
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'impl'
        self.skip_trivia_into(&mut children);

        // Optional generic params
        if self.at_symbol_text("[") {
            let gp = self.parse_generic_params();
            children.push(SyntaxElement::Node(Box::new(gp)));
            self.skip_trivia_into(&mut children);
        }

        // Optional name: ident followed by '=' (not '==')
        if self.at_non_trivia_ident()
            && self.peek_next_non_trivia_symbol() == Some("=")
        {
            self.skip_trivia_into(&mut children);
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name:Ident
            self.skip_trivia_into(&mut children);
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '='
            self.skip_trivia_into(&mut children);
        }

        // Target TypeExpr (stop at ':' or '{')
        if self.at_non_trivia_ident() {
            let target = self.parse_type_expr();
            children.push(SyntaxElement::Node(Box::new(target)));
            self.skip_trivia_into(&mut children);
        } else {
            self.error_here("expected target type in impl declaration");
        }

        // Optional cap: ': TypeExpr'
        if self.at_symbol_text(":") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ':'
            let cap_ty = self.parse_type_expr();
            children.push(SyntaxElement::Node(Box::new(cap_ty)));
            self.skip_trivia_into(&mut children);
        }

        if self.at_symbol_text("{") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '{'
        } else {
            self.error_here("expected `{` in impl declaration");
            return node_from_children(SyntaxKind::IMPL_DECL, children);
        }

        loop {
            self.skip_trivia_into(&mut children);
            if self.eof() || self.at_symbol_text("}") {
                break;
            }
            if self.at_keyword(Keyword::Fn) {
                let method = self.parse_impl_method();
                children.push(SyntaxElement::Node(Box::new(method)));
                self.skip_trivia_into(&mut children);
                if self.at_symbol_text(";") || self.at_symbol_text(",") {
                    children.push(SyntaxElement::Token(self.bump().unwrap()));
                }
            } else {
                self.error_here("expected `fn` in impl block");
                children.push(SyntaxElement::Node(Box::new(self.parse_error_node())));
                break;
            }
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '}'
        } else {
            self.error_here("expected `}` in impl declaration");
        }

        node_from_children(SyntaxKind::IMPL_DECL, children)
    }

    fn parse_impl_method(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'fn'
        self.skip_trivia_into(&mut children);

        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("(") {
            let pl = self.parse_impl_param_list();
            children.push(SyntaxElement::Node(Box::new(pl)));
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text(":") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ':'
            let ret_ty = self.parse_type_expr();
            children.push(SyntaxElement::Node(Box::new(ret_ty)));
            self.skip_trivia_into(&mut children);
        }

        let body = self.parse_fn_body();
        children.push(SyntaxElement::Node(Box::new(body)));

        node_from_children(SyntaxKind::IMPL_METHOD, children)
    }
}

// ── Expressions ───────────────────────────────────────────────────────────────

impl Parser {
    /// Entry point for expression parsing (min_bp=0).
    fn parse_expr_bp(&mut self, min_bp: u8) -> SyntaxNode {
        const PREFIX_BP: u8 = 15;

        // Leading trivia become part of the atom node
        let mut lhs = self.parse_expr_atom(PREFIX_BP);

        loop {
            // Postfix: member `.ident`
            if self.at_non_trivia_symbol(".") {
                let mut children = vec![SyntaxElement::Node(Box::new(lhs))];
                self.skip_trivia_into(&mut children);
                children.push(SyntaxElement::Token(self.bump().unwrap())); // '.'
                self.skip_trivia_into(&mut children);
                if self.at_ident() {
                    children.push(SyntaxElement::Token(self.bump().unwrap())); // member
                } else {
                    self.error_here("expected member name after `.`");
                }
                lhs = node_from_children(SyntaxKind::MEMBER_EXPR, children);
                continue;
            }

            // Postfix: call `(`
            if self.at_non_trivia_symbol("(") {
                let mut children = vec![SyntaxElement::Node(Box::new(lhs))];
                self.skip_trivia_into(&mut children);
                children.push(SyntaxElement::Token(self.bump().unwrap())); // '('

                loop {
                    self.skip_trivia_into(&mut children);
                    if self.eof() || self.at_symbol_text(")") {
                        break;
                    }
                    let arg = self.parse_expr_bp(0);
                    children.push(SyntaxElement::Node(Box::new(arg)));
                    self.skip_trivia_into(&mut children);
                    if self.at_symbol_text(",") {
                        children.push(SyntaxElement::Token(self.bump().unwrap()));
                    } else {
                        break;
                    }
                }

                self.skip_trivia_into(&mut children);
                if self.at_symbol_text(")") {
                    children.push(SyntaxElement::Token(self.bump().unwrap())); // ')'
                } else {
                    self.error_here("expected `)` in call expression");
                }

                lhs = node_from_children(SyntaxKind::CALL_EXPR, children);
                continue;
            }

            // Assignment: ident = value ; body  (only at lowest bp level)
            if lhs.kind == SyntaxKind::IDENT_EXPR
                && self.at_non_trivia_symbol("=")
                && min_bp == 0
            {
                let mut children = vec![SyntaxElement::Node(Box::new(lhs))];
                self.skip_trivia_into(&mut children);
                children.push(SyntaxElement::Token(self.bump().unwrap())); // '='
                let value = self.parse_expr_bp(0);
                children.push(SyntaxElement::Node(Box::new(value)));
                self.skip_trivia_into(&mut children);
                if self.at_symbol_text(";") {
                    children.push(SyntaxElement::Token(self.bump().unwrap())); // ';'
                } else {
                    self.error_here("expected `;` in assign expression");
                }
                let body = self.parse_expr_bp(0);
                children.push(SyntaxElement::Node(Box::new(body)));
                lhs = node_from_children(SyntaxKind::ASSIGN_EXPR, children);
                break;
            }

            // Infix binary ops
            if let Some((_op_sk, l_bp, r_bp)) = self.peek_infix_op_non_trivia() {
                if l_bp < min_bp {
                    break;
                }
                let mut children = vec![SyntaxElement::Node(Box::new(lhs))];
                self.skip_trivia_into(&mut children);
                let op_tok = self.bump().unwrap(); // operator token
                let op_node = node_from_children(
                    SyntaxKind::BINARY_OP,
                    vec![SyntaxElement::Token(op_tok)],
                );
                children.push(SyntaxElement::Node(Box::new(op_node)));
                let rhs = self.parse_expr_bp(r_bp);
                children.push(SyntaxElement::Node(Box::new(rhs)));
                lhs = node_from_children(SyntaxKind::BINARY_EXPR, children);
                continue;
            }

            break;
        }

        lhs
    }

    fn parse_expr_atom(&mut self, prefix_bp: u8) -> SyntaxNode {
        let mut leading = Vec::new();
        self.skip_trivia_into(&mut leading);

        let node = if self.at_keyword(Keyword::Let) {
            self.parse_let_expr()
        } else if self.at_keyword(Keyword::Thunk) {
            self.parse_thunk_expr()
        } else if self.at_keyword(Keyword::Force) {
            self.parse_force_expr()
        } else if self.at_keyword(Keyword::Match) {
            self.parse_match_expr()
        } else if self.at_keyword(Keyword::If) {
            self.parse_if_else_expr()
        } else if self.at_keyword(Keyword::Handle) {
            self.parse_handle_expr()
        } else if self.at_keyword(Keyword::Bundle) {
            self.parse_bundle_expr()
        } else if self.at_symbol_text("{") {
            self.parse_block_expr()
        } else if self.at_symbol_text("(") {
            self.parse_paren_or_annotation_expr()
        } else if self.at_symbol_text("-") || self.at_symbol_text("!") {
            self.parse_unary_expr(prefix_bp)
        } else if self.at_ident() {
            self.parse_ident_expr()
        } else if self.at_string_lit() {
            self.parse_string_expr()
        } else if self.at_number_lit() {
            self.parse_number_expr()
        } else {
            self.error_here("expected expression");
            self.parse_error_node()
        };

        if leading.is_empty() {
            return node;
        }
        // Prepend leading trivia into the node
        prepend_children_to_node(node, leading)
    }

    fn parse_let_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'let'
        self.skip_trivia_into(&mut children);

        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        } else {
            self.error_here("expected name in let expression");
        }

        self.skip_trivia_into(&mut children);

        // optional type annotation
        if self.at_symbol_text(":") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ':'
            let ty = self.parse_type_expr();
            children.push(SyntaxElement::Node(Box::new(ty)));
            self.skip_trivia_into(&mut children);
        }

        if self.at_symbol_text("=") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '='
            let value = self.parse_expr_bp(0);
            children.push(SyntaxElement::Node(Box::new(value)));
        } else {
            self.error_here("expected `=` in let expression");
        }

        node_from_children(SyntaxKind::LET_EXPR, children)
    }

    fn parse_thunk_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'thunk'
        let body = self.parse_expr_bp(0);
        children.push(SyntaxElement::Node(Box::new(body)));
        node_from_children(SyntaxKind::THUNK_EXPR, children)
    }

    fn parse_force_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'force'
        let expr = self.parse_expr_bp(0);
        children.push(SyntaxElement::Node(Box::new(expr)));
        node_from_children(SyntaxKind::FORCE_EXPR, children)
    }

    fn parse_match_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'match'

        let scrutinee = self.parse_expr_bp(0);
        children.push(SyntaxElement::Node(Box::new(scrutinee)));
        self.skip_trivia_into(&mut children);

        if self.at_symbol_text("{") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '{'
        } else {
            self.error_here("expected `{` in match expression");
            return node_from_children(SyntaxKind::MATCH_EXPR, children);
        }

        loop {
            self.skip_trivia_into(&mut children);
            if self.eof() || self.at_symbol_text("}") {
                break;
            }
            let arm = self.parse_match_arm();
            children.push(SyntaxElement::Node(Box::new(arm)));
            self.skip_trivia_into(&mut children);
            if self.at_symbol_text(",") {
                children.push(SyntaxElement::Token(self.bump().unwrap()));
            }
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '}'
        } else {
            self.error_here("expected `}` in match expression");
        }

        node_from_children(SyntaxKind::MATCH_EXPR, children)
    }

    fn parse_match_arm(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        let pattern = self.parse_pattern();
        children.push(SyntaxElement::Node(Box::new(pattern)));
        self.skip_trivia_into(&mut children);

        if self.at_symbol_text("=>") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '=>'
        } else {
            self.error_here("expected `=>` in match arm");
        }

        let body = self.parse_expr_bp(0);
        children.push(SyntaxElement::Node(Box::new(body)));

        node_from_children(SyntaxKind::MATCH_ARM, children)
    }

    fn parse_pattern(&mut self) -> SyntaxNode {
        let mut leading = Vec::new();
        self.skip_trivia_into(&mut leading);

        let node = if self.at_symbol_text(".") {
            self.parse_variant_pattern()
        } else if self.at_keyword(Keyword::Let) {
            self.parse_bind_pattern()
        } else if self.at_symbol_text("_") {
            let mut ch = Vec::new();
            ch.push(SyntaxElement::Token(self.bump().unwrap()));
            node_from_children(SyntaxKind::WILDCARD_PATTERN, ch)
        } else {
            self.error_here("expected pattern");
            self.parse_error_node()
        };

        if leading.is_empty() {
            node
        } else {
            prepend_children_to_node(node, leading)
        }
    }

    fn parse_variant_pattern(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '.'
        self.skip_trivia_into(&mut children);

        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("(") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '('
            loop {
                self.skip_trivia_into(&mut children);
                if self.eof() || self.at_symbol_text(")") {
                    break;
                }
                let field = self.parse_pattern();
                children.push(SyntaxElement::Node(Box::new(field)));
                self.skip_trivia_into(&mut children);
                if self.at_symbol_text(",") {
                    children.push(SyntaxElement::Token(self.bump().unwrap()));
                }
            }
            self.skip_trivia_into(&mut children);
            if self.at_symbol_text(")") {
                children.push(SyntaxElement::Token(self.bump().unwrap())); // ')'
            } else {
                self.error_here("expected `)` in variant pattern");
            }
        }

        node_from_children(SyntaxKind::VARIANT_PATTERN, children)
    }

    fn parse_bind_pattern(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'let'
        self.skip_trivia_into(&mut children);
        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        }
        node_from_children(SyntaxKind::BIND_PATTERN, children)
    }

    fn parse_if_else_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'if'

        let condition = self.parse_expr_bp(0);
        children.push(SyntaxElement::Node(Box::new(condition)));
        self.skip_trivia_into(&mut children);

        if self.at_symbol_text("{") {
            let then_block = self.parse_block_expr();
            children.push(SyntaxElement::Node(Box::new(then_block)));
        } else {
            self.error_here("expected `{` in if-else expression");
            return node_from_children(SyntaxKind::IF_ELSE_EXPR, children);
        }

        self.skip_trivia_into(&mut children);

        if self.at_keyword(Keyword::Else) {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // 'else'
            self.skip_trivia_into(&mut children);

            if self.at_keyword(Keyword::If) {
                let nested = self.parse_if_else_expr();
                children.push(SyntaxElement::Node(Box::new(nested)));
            } else if self.at_symbol_text("{") {
                let else_block = self.parse_block_expr();
                children.push(SyntaxElement::Node(Box::new(else_block)));
            } else {
                self.error_here("expected `{` or `if` after `else`");
            }
        }

        node_from_children(SyntaxKind::IF_ELSE_EXPR, children)
    }

    fn parse_handle_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'handle'

        // cap:Ident (the capability name, not a full expr)
        self.skip_trivia_into(&mut children);
        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // cap name
        } else {
            self.error_here("expected capability name in handle expression");
        }

        self.skip_trivia_into(&mut children);
        if self.at_non_trivia_ident_text("with") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // 'with'
        } else {
            self.error_here("expected `with` in handle expression");
            return node_from_children(SyntaxKind::HANDLE_EXPR, children);
        }

        let handler = self.parse_expr_bp(0);
        children.push(SyntaxElement::Node(Box::new(handler)));
        self.skip_trivia_into(&mut children);

        if self.at_keyword(Keyword::In) {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // 'in'
        } else {
            self.error_here("expected `in` in handle expression");
            return node_from_children(SyntaxKind::HANDLE_EXPR, children);
        }

        let body = self.parse_expr_bp(0);
        children.push(SyntaxElement::Node(Box::new(body)));

        node_from_children(SyntaxKind::HANDLE_EXPR, children)
    }

    fn parse_bundle_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'bundle'
        self.skip_trivia_into(&mut children);

        if self.at_symbol_text("{") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '{'
        } else {
            self.error_here("expected `{` after `bundle`");
            return node_from_children(SyntaxKind::BUNDLE_EXPR, children);
        }

        loop {
            self.skip_trivia_into(&mut children);
            if self.eof() || self.at_symbol_text("}") {
                break;
            }
            if self.at_keyword(Keyword::Fn) {
                let entry = self.parse_bundle_entry();
                children.push(SyntaxElement::Node(Box::new(entry)));
                self.skip_trivia_into(&mut children);
                if self.at_symbol_text(",") {
                    children.push(SyntaxElement::Token(self.bump().unwrap()));
                }
            } else {
                self.error_here("expected `fn` in bundle");
                children.push(SyntaxElement::Node(Box::new(self.parse_error_node())));
                break;
            }
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '}'
        } else {
            self.error_here("expected `}` in bundle expression");
        }

        node_from_children(SyntaxKind::BUNDLE_EXPR, children)
    }

    fn parse_bundle_entry(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // 'fn'
        self.skip_trivia_into(&mut children);

        if self.at_ident() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // name
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("(") {
            let pl = self.parse_param_list();
            children.push(SyntaxElement::Node(Box::new(pl)));
        }

        self.skip_trivia_into(&mut children);
        let body = self.parse_fn_body();
        children.push(SyntaxElement::Node(Box::new(body)));

        node_from_children(SyntaxKind::BUNDLE_ENTRY, children)
    }

    fn parse_block_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '{'

        loop {
            self.skip_trivia_into(&mut children);
            if self.eof() || self.at_symbol_text("}") {
                // Empty block — emit error result
                self.error_here("expected expression in block");
                break;
            }

            if self.at_keyword(Keyword::Let) {
                // LetStmt: 'let' name '=' expr ';'
                let mut stmt_children = Vec::new();
                stmt_children.push(SyntaxElement::Token(self.bump().unwrap())); // 'let'
                self.skip_trivia_into(&mut stmt_children);

                if self.at_ident() {
                    stmt_children.push(SyntaxElement::Token(self.bump().unwrap())); // name
                }

                self.skip_trivia_into(&mut stmt_children);
                if self.at_symbol_text("=") {
                    stmt_children.push(SyntaxElement::Token(self.bump().unwrap())); // '='
                    let value = self.parse_expr_bp(0);
                    stmt_children.push(SyntaxElement::Node(Box::new(value)));
                    self.skip_trivia_into(&mut stmt_children);
                }

                if self.at_symbol_text(";") {
                    stmt_children.push(SyntaxElement::Token(self.bump().unwrap())); // ';'
                    children.push(SyntaxElement::Node(Box::new(
                        node_from_children(SyntaxKind::LET_STMT, stmt_children),
                    )));
                    continue;
                } else {
                    // No ';' — treat as result (shouldn't happen in well-formed code)
                    self.error_here("expected `;` after let in block");
                    children.push(SyntaxElement::Node(Box::new(
                        node_from_children(SyntaxKind::LET_STMT, stmt_children),
                    )));
                    break;
                }
            }

            // Parse an expression
            let expr = self.parse_expr_bp(0);

            // Check for ';' past any trailing trivia (at_non_trivia_symbol peeks without consuming)
            if self.at_non_trivia_symbol(";") {
                // ExprStmt
                let mut stmt_children = vec![SyntaxElement::Node(Box::new(expr))];
                self.skip_trivia_into(&mut stmt_children);
                stmt_children.push(SyntaxElement::Token(self.bump().unwrap())); // ';'
                children.push(SyntaxElement::Node(Box::new(
                    node_from_children(SyntaxKind::EXPR_STMT, stmt_children),
                )));
                continue;
            } else {
                // Result expr
                children.push(SyntaxElement::Node(Box::new(expr)));
                break;
            }
        }

        self.skip_trivia_into(&mut children);
        if self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // '}'
        } else {
            self.error_here("expected `}` in block expression");
        }

        node_from_children(SyntaxKind::BLOCK_EXPR, children)
    }

    fn parse_paren_or_annotation_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // '('

        let inner = self.parse_expr_bp(0);

        // After inner expr, check for ':' (annotation) or ')' (paren)
        // Peek past trivia to see what's next
        if self.at_non_trivia_symbol(":") {
            // AnnotationExpr: '(' expr ':' ty ')'
            children.push(SyntaxElement::Node(Box::new(inner)));
            self.skip_trivia_into(&mut children);
            children.push(SyntaxElement::Token(self.bump().unwrap())); // ':'
            let ty = self.parse_type_expr();
            children.push(SyntaxElement::Node(Box::new(ty)));
            self.skip_trivia_into(&mut children);
            if self.at_symbol_text(")") {
                children.push(SyntaxElement::Token(self.bump().unwrap())); // ')'
            } else {
                self.error_here("expected `)` in annotation expression");
            }
            node_from_children(SyntaxKind::ANNOTATION_EXPR, children)
        } else {
            // ParenExpr: '(' expr ')'
            children.push(SyntaxElement::Node(Box::new(inner)));
            self.skip_trivia_into(&mut children);
            if self.at_symbol_text(")") {
                children.push(SyntaxElement::Token(self.bump().unwrap())); // ')'
            } else {
                self.error_here("expected `)` in parenthesized expression");
            }
            node_from_children(SyntaxKind::PAREN_EXPR, children)
        }
    }

    fn parse_unary_expr(&mut self, prefix_bp: u8) -> SyntaxNode {
        let mut children = Vec::new();
        let op_tok = self.bump().unwrap(); // '-' or '!'
        let op_node = node_from_children(
            SyntaxKind::UNARY_OP,
            vec![SyntaxElement::Token(op_tok)],
        );
        children.push(SyntaxElement::Node(Box::new(op_node)));
        let operand = self.parse_expr_bp(prefix_bp);
        children.push(SyntaxElement::Node(Box::new(operand)));
        node_from_children(SyntaxKind::UNARY_EXPR, children)
    }

    fn parse_ident_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // ident
        node_from_children(SyntaxKind::IDENT_EXPR, children)
    }

    fn parse_string_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // string
        node_from_children(SyntaxKind::STRING_EXPR, children)
    }

    fn parse_number_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // number
        node_from_children(SyntaxKind::NUMBER_EXPR, children)
    }

    fn parse_error_node(&mut self) -> SyntaxNode {
        if self.eof() {
            return SyntaxNode {
                kind: SyntaxKind::ERROR,
                span: Span::new(0, 0),
                children: Vec::new(),
            };
        }
        let token = self.bump().unwrap();
        node_from_children(SyntaxKind::ERROR, vec![SyntaxElement::Token(token)])
    }
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn lexer_token_to_lst(tok: lumo_lexer::LosslessToken) -> LosslessToken {
    let kind = lexer_kind_to_syntax_kind(&tok.kind, &tok.text);
    LosslessToken {
        kind,
        span: tok.span,
        text: tok.text,
    }
}

fn span_from_children(children: &[SyntaxElement]) -> Span {
    let mut start: Option<usize> = None;
    let mut end: Option<usize> = None;
    for child in children {
        let span = match child {
            SyntaxElement::Node(n) => n.span,
            SyntaxElement::Token(t) => t.span,
        };
        start = Some(start.map(|s| s.min(span.start)).unwrap_or(span.start));
        end = Some(end.map(|e| e.max(span.end)).unwrap_or(span.end));
    }
    Span::new(start.unwrap_or(0), end.unwrap_or(0))
}

fn node_from_children(kind: SyntaxKind, children: Vec<SyntaxElement>) -> SyntaxNode {
    let span = span_from_children(&children);
    SyntaxNode {
        kind,
        span,
        children,
    }
}

fn prepend_children_to_node(mut node: SyntaxNode, prefix: Vec<SyntaxElement>) -> SyntaxNode {
    let mut new_children = prefix;
    new_children.extend(node.children);
    node.children = new_children;
    node.span = span_from_children(&node.children);
    node
}
