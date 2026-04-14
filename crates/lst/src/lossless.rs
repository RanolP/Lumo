use lumo_lexer::{lex_lossless, Keyword, LosslessToken, LosslessTokenKind, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    File,
    DataDecl,
    FnDecl,
    ExternDecl,
    LetExpr,
    ProduceExpr,
    ThunkExpr,
    ForceExpr,
    MatchExpr,
    CapDecl,
    IdentExpr,
    StringExpr,
    CallExpr,
    PerformExpr,
    HandleExpr,
    BundleExpr,
    NumberExpr,
    UseDecl,
    ImplDecl,
    IfElseExpr,
    Error,
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

struct Parser {
    tokens: Vec<LosslessToken>,
    index: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    fn parse_file(&mut self) -> SyntaxNode {
        let mut children = Vec::new();

        while !self.eof() {
            if self.at_symbol_text("#") {
                self.consume_attribute_tokens(&mut children);
                continue;
            }
            if self.at_keyword(Keyword::Cap) {
                let node = self.parse_cap_decl();
                children.push(SyntaxElement::Node(Box::new(node)));
                continue;
            }
            if self.at_keyword(Keyword::Data) {
                let node = self.parse_data_decl();
                children.push(SyntaxElement::Node(Box::new(node)));
                continue;
            }
            if self.at_keyword(Keyword::Fn) {
                let node = self.parse_fn_decl();
                children.push(SyntaxElement::Node(Box::new(node)));
                continue;
            }
            if self.at_keyword(Keyword::Extern) {
                let node = self.parse_extern_decl();
                children.push(SyntaxElement::Node(Box::new(node)));
                continue;
            }
            if self.at_keyword(Keyword::Use) {
                let node = self.parse_use_decl();
                children.push(SyntaxElement::Node(Box::new(node)));
                continue;
            }
            if self.at_keyword(Keyword::Impl) {
                let node = self.parse_impl_decl();
                children.push(SyntaxElement::Node(Box::new(node)));
                continue;
            }

            if self.at_trivia_or_unknown() {
                children.push(SyntaxElement::Token(self.bump().unwrap()));
                continue;
            }

            self.error_here("expected top-level `data`, `fn`, or `extern`");
            children.push(SyntaxElement::Node(Box::new(self.parse_error_node())));
        }

        node_from_children(SyntaxKind::File, children)
    }

    fn parse_cap_decl(&mut self) -> SyntaxNode {
        let mut children = Vec::new();

        children.push(SyntaxElement::Token(self.bump().unwrap())); // cap

        while !self.eof() {
            if self.at_symbol_text("{") {
                break;
            }
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if !self.eof() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // {
        } else {
            self.error_here("expected `{` in cap declaration");
            return node_from_children(SyntaxKind::CapDecl, children);
        }

        while !self.eof() && !self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if !self.eof() && self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        } else {
            self.error_here("expected `}` in cap declaration");
        }

        node_from_children(SyntaxKind::CapDecl, children)
    }

    fn parse_data_decl(&mut self) -> SyntaxNode {
        let mut children = Vec::new();

        children.push(SyntaxElement::Token(self.bump().unwrap())); // data

        while !self.eof() {
            if self.at_symbol_text("{") {
                break;
            }
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if !self.eof() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // {
        } else {
            self.error_here("expected `{` in data declaration");
            return node_from_children(SyntaxKind::DataDecl, children);
        }

        while !self.eof() && !self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if !self.eof() && self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        } else {
            self.error_here("expected `}` in data declaration");
        }

        node_from_children(SyntaxKind::DataDecl, children)
    }

    fn parse_fn_decl(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // fn

        // Consume signature tokens until the body `{`.
        // Cap annotations like `/ {}` contain balanced braces that must be skipped.
        // When we see `/ {`, the `{` opens a cap annotation — track brace depth.
        let mut sig_depth = 0usize;
        let mut after_slash = false;
        while !self.eof() {
            if self.at_symbol_text("{") {
                if sig_depth == 0 && !after_slash {
                    break; // this `{` starts the body block
                }
                sig_depth += 1;
            } else if self.at_symbol_text("}") {
                if sig_depth > 0 {
                    sig_depth -= 1;
                }
            }
            let tok = self.bump().unwrap();
            // Track whether we just saw `/` (skipping whitespace/newline)
            let is_trivia = matches!(
                tok.kind,
                LosslessTokenKind::Whitespace | LosslessTokenKind::Newline
            );
            if !is_trivia {
                after_slash = tok.text == "/";
            }
            children.push(SyntaxElement::Token(tok));
        }

        if !self.eof() && self.at_symbol_text("{") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // {
        } else {
            self.error_here("expected `{` in fn declaration");
            return node_from_children(SyntaxKind::FnDecl, children);
        }

        // Consume tokens until matching }
        let mut depth = 1usize;
        while !self.eof() && depth > 0 {
            if self.at_symbol_text("{") {
                depth += 1;
            } else if self.at_symbol_text("}") {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        } else {
            self.error_here("expected `}` in fn declaration");
        }

        node_from_children(SyntaxKind::FnDecl, children)
    }

    fn parse_extern_decl(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // extern

        while !self.eof() && !self.at_symbol_text(";") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if self.at_symbol_text(";") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        node_from_children(SyntaxKind::ExternDecl, children)
    }

    fn parse_use_decl(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // use

        while !self.eof() && !self.at_symbol_text(";") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if self.at_symbol_text(";") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        node_from_children(SyntaxKind::UseDecl, children)
    }

    fn parse_impl_decl(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // impl

        // Consume everything until `{`
        while !self.eof() {
            if self.at_symbol_text("{") {
                break;
            }
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if !self.eof() {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // {
        } else {
            self.error_here("expected `{` in impl declaration");
            return node_from_children(SyntaxKind::ImplDecl, children);
        }

        // Consume tokens until matching }
        let mut depth = 1usize;
        while !self.eof() && depth > 0 {
            if self.at_symbol_text("{") {
                depth += 1;
            } else if self.at_symbol_text("}") {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        } else {
            self.error_here("expected `}` in impl declaration");
        }

        node_from_children(SyntaxKind::ImplDecl, children)
    }

    fn consume_attribute_tokens(&mut self, children: &mut Vec<SyntaxElement>) {
        while self.at_symbol_text("#") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // #
            if self.at_symbol_text("[") {
                children.push(SyntaxElement::Token(self.bump().unwrap())); // [
            } else {
                self.error_here("expected `[` after `#`");
                return;
            }

            let mut depth = 1usize;
            while !self.eof() && depth > 0 {
                if self.at_symbol_text("[") {
                    depth += 1;
                } else if self.at_symbol_text("]") {
                    depth = depth.saturating_sub(1);
                }
                children.push(SyntaxElement::Token(self.bump().unwrap()));
            }
        }
    }

    fn parse_expr(&mut self) -> SyntaxNode {
        let mut leading = Vec::new();
        while self.at_trivia() {
            leading.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        let mut node = if self.at_keyword(Keyword::Let) {
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
        } else if self.at_ident() {
            self.parse_ident_or_call_expr()
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

        let mut children = leading;
        children.extend(node.children);
        node.children = children;
        node.span = span_from_children(&node.children);
        node
    }

    fn parse_let_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // let

        while !self.eof() && !self.at_symbol_text("=") {
            if self.at_keyword(Keyword::Data)
                || self.at_keyword(Keyword::Cap)
                || self.at_keyword(Keyword::Fn)
                || self.at_keyword(Keyword::Extern)
                || self.at_keyword(Keyword::Use)
            {
                break;
            }
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if self.at_symbol_text("=") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
            let value = self.parse_expr();
            children.push(SyntaxElement::Node(Box::new(value)));
        } else {
            self.error_here("expected `=` in let expression");
        }

        node_from_children(SyntaxKind::LetExpr, children)
    }

    fn parse_thunk_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // thunk
        let payload = self.parse_expr();
        children.push(SyntaxElement::Node(Box::new(payload)));
        node_from_children(SyntaxKind::ThunkExpr, children)
    }

    fn parse_force_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // force
        let payload = self.parse_expr();
        children.push(SyntaxElement::Node(Box::new(payload)));
        node_from_children(SyntaxKind::ForceExpr, children)
    }

    fn parse_match_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // match
        children.push(SyntaxElement::Node(Box::new(self.parse_expr()))); // scrutinee

        while !self.eof() && self.at_trivia() {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if !self.at_symbol_text("{") {
            self.error_here("expected `{` in match expression");
            return node_from_children(SyntaxKind::MatchExpr, children);
        }
        children.push(SyntaxElement::Token(self.bump().unwrap())); // {

        while !self.eof() && !self.at_symbol_text("}") {
            while !self.eof() && !self.at_symbol_text("=>") {
                if self.at_symbol_text("}") {
                    break;
                }
                children.push(SyntaxElement::Token(self.bump().unwrap()));
            }
            if self.at_symbol_text("=>") {
                children.push(SyntaxElement::Token(self.bump().unwrap()));
                children.push(SyntaxElement::Node(Box::new(self.parse_expr())));
            } else {
                break;
            }
            if self.at_symbol_text(",") {
                children.push(SyntaxElement::Token(self.bump().unwrap()));
            }
        }

        if self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        } else {
            self.error_here("expected `}` in match expression");
        }

        node_from_children(SyntaxKind::MatchExpr, children)
    }

    fn parse_if_else_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // if

        // Collect condition tokens until `{`
        while !self.eof() && !self.at_symbol_text("{") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        // Parse then-body `{ ... }`
        if self.at_symbol_text("{") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // {
            let mut depth = 1usize;
            while !self.eof() && depth > 0 {
                if self.at_symbol_text("{") {
                    depth += 1;
                } else if self.at_symbol_text("}") {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                children.push(SyntaxElement::Token(self.bump().unwrap()));
            }
            if self.at_symbol_text("}") {
                children.push(SyntaxElement::Token(self.bump().unwrap())); // }
            }
        }

        // Skip trivia before potential `else`
        while self.at_trivia() {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        // Check for `else`
        if self.at_keyword(Keyword::Else) {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // else

            while self.at_trivia() {
                children.push(SyntaxElement::Token(self.bump().unwrap()));
            }

            if self.at_keyword(Keyword::If) {
                // else if — recurse
                let nested = self.parse_if_else_expr();
                children.push(SyntaxElement::Node(Box::new(nested)));
            } else if self.at_symbol_text("{") {
                // else { ... }
                children.push(SyntaxElement::Token(self.bump().unwrap())); // {
                let mut depth = 1usize;
                while !self.eof() && depth > 0 {
                    if self.at_symbol_text("{") {
                        depth += 1;
                    } else if self.at_symbol_text("}") {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    children.push(SyntaxElement::Token(self.bump().unwrap()));
                }
                if self.at_symbol_text("}") {
                    children.push(SyntaxElement::Token(self.bump().unwrap())); // }
                }
            }
        }

        node_from_children(SyntaxKind::IfElseExpr, children)
    }

    fn parse_handle_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // handle

        // parse operation name (ident or tokens until "with")
        let inner = self.parse_expr();
        children.push(SyntaxElement::Node(Box::new(inner)));

        // skip trivia
        while !self.eof() && self.at_trivia() {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        // expect "with" (contextual keyword)
        if self.at_ident_text("with") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        } else {
            self.error_here("expected `with` in handle expression");
            return node_from_children(SyntaxKind::HandleExpr, children);
        }

        // parse handler expression
        let handler = self.parse_expr();
        children.push(SyntaxElement::Node(Box::new(handler)));

        // skip trivia
        while !self.eof() && self.at_trivia() {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        // expect "in"
        if self.at_keyword(Keyword::In) {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        } else {
            self.error_here("expected `in` in handle expression");
            return node_from_children(SyntaxKind::HandleExpr, children);
        }

        // parse body expression
        let body = self.parse_expr();
        children.push(SyntaxElement::Node(Box::new(body)));

        node_from_children(SyntaxKind::HandleExpr, children)
    }

    fn parse_bundle_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // bundle

        while !self.eof() && self.at_trivia() {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if !self.at_symbol_text("{") {
            self.error_here("expected `{` after `bundle`");
            return node_from_children(SyntaxKind::BundleExpr, children);
        }
        children.push(SyntaxElement::Token(self.bump().unwrap())); // {

        // Consume tokens until matching }
        let mut depth = 1usize;
        while !self.eof() && depth > 0 {
            if self.at_symbol_text("{") {
                depth += 1;
            } else if self.at_symbol_text("}") {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if self.at_symbol_text("}") {
            children.push(SyntaxElement::Token(self.bump().unwrap())); // }
        } else {
            self.error_here("expected `}` in bundle expression");
        }

        node_from_children(SyntaxKind::BundleExpr, children)
    }

    fn at_ident_text(&self, text: &str) -> bool {
        matches!(
            self.current().map(|t| (&t.kind, t.text.as_str())),
            Some((LosslessTokenKind::Ident, actual)) if actual == text
        )
    }

    fn parse_ident_or_call_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // ident

        let mut has_postfix = false;
        loop {
            if self.at_symbol_text(".") {
                has_postfix = true;
                children.push(SyntaxElement::Token(self.bump().unwrap())); // .
                if self.at_ident() {
                    children.push(SyntaxElement::Token(self.bump().unwrap())); // member
                } else {
                    self.error_here("expected member name after `.`");
                    break;
                }
                continue;
            }

            if self.at_symbol_text("(") {
                has_postfix = true;
                children.push(SyntaxElement::Token(self.bump().unwrap())); // (
                while !self.eof() && !self.at_symbol_text(")") {
                    children.push(SyntaxElement::Node(Box::new(self.parse_expr())));
                    if self.at_symbol_text(",") {
                        children.push(SyntaxElement::Token(self.bump().unwrap()));
                    } else {
                        break;
                    }
                }

                if self.at_symbol_text(")") {
                    children.push(SyntaxElement::Token(self.bump().unwrap()));
                } else {
                    self.error_here("expected `)` in call expression");
                }
                continue;
            }

            break;
        }

        node_from_children(
            if has_postfix {
                SyntaxKind::CallExpr
            } else {
                SyntaxKind::IdentExpr
            },
            children,
        )
    }

    fn parse_string_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // string literal
        node_from_children(SyntaxKind::StringExpr, children)
    }

    fn parse_number_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // number literal
        node_from_children(SyntaxKind::NumberExpr, children)
    }

    fn parse_error_node(&mut self) -> SyntaxNode {
        if self.eof() {
            return SyntaxNode {
                kind: SyntaxKind::Error,
                span: Span::new(0, 0),
                children: Vec::new(),
            };
        }

        let token = self.bump().unwrap();
        node_from_children(SyntaxKind::Error, vec![SyntaxElement::Token(token)])
    }

    fn at_trivia(&self) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(LosslessTokenKind::Whitespace) | Some(LosslessTokenKind::Newline)
        )
    }

    fn at_trivia_or_unknown(&self) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(LosslessTokenKind::Whitespace)
                | Some(LosslessTokenKind::Newline)
                | Some(LosslessTokenKind::Unknown)
        )
    }

    fn at_keyword(&self, keyword: Keyword) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(LosslessTokenKind::Keyword(actual)) if *actual == keyword
        )
    }

    fn at_ident(&self) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(LosslessTokenKind::Ident)
        )
    }

    fn at_string_lit(&self) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(LosslessTokenKind::StringLit)
        )
    }

    fn at_number_lit(&self) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(LosslessTokenKind::NumberLit)
        )
    }

    fn at_symbol_text(&self, text: &str) -> bool {
        self.current().map(|t| t.text.as_str()) == Some(text)
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

    fn current(&self) -> Option<&LosslessToken> {
        self.tokens.get(self.index)
    }

    fn bump(&mut self) -> Option<LosslessToken> {
        let token = self.tokens.get(self.index).cloned();
        if token.is_some() {
            self.index += 1;
        }
        token
    }

    fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn error_here(&mut self, message: &str) {
        let span = self.current_span();
        self.errors.push(ParseError {
            span,
            message: message.to_owned(),
        });
    }
}

fn span_from_children(children: &[SyntaxElement]) -> Span {
    let mut start = None;
    let mut end = None;

    for child in children {
        let span = match child {
            SyntaxElement::Node(n) => n.span,
            SyntaxElement::Token(t) => t.span,
        };
        start = Some(
            start
                .map(|s: usize| s.min(span.start))
                .unwrap_or(span.start),
        );
        end = Some(end.map(|e: usize| e.max(span.end)).unwrap_or(span.end));
    }

    Span::new(start.unwrap_or(0), end.unwrap_or(0))
}

fn node_from_children(kind: SyntaxKind, children: Vec<SyntaxElement>) -> SyntaxNode {
    let mut start = None;
    let mut end = None;

    for child in &children {
        let span = match child {
            SyntaxElement::Node(n) => n.span,
            SyntaxElement::Token(t) => t.span,
        };
        start = Some(
            start
                .map(|s: usize| s.min(span.start))
                .unwrap_or(span.start),
        );
        end = Some(end.map(|e: usize| e.max(span.end)).unwrap_or(span.end));
    }

    SyntaxNode {
        kind,
        span: Span::new(start.unwrap_or(0), end.unwrap_or(0)),
        children,
    }
}
