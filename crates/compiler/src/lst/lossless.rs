use crate::lexer::{lex_lossless, Keyword, LosslessToken, LosslessTokenKind, Span};

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
    LetExpr,
    ProduceExpr,
    IdentExpr,
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

            if self.at_trivia_or_unknown() {
                children.push(SyntaxElement::Token(self.bump().unwrap()));
                continue;
            }

            self.error_here("expected top-level `data` or `fn`");
            children.push(SyntaxElement::Node(Box::new(self.parse_error_node())));
        }

        node_from_children(SyntaxKind::File, children)
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

        while !self.eof() && !self.at_symbol_text(":=") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if !self.eof() && self.at_symbol_text(":=") {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        } else {
            self.error_here("expected `:=` in fn declaration");
            return node_from_children(SyntaxKind::FnDecl, children);
        }

        let expr = self.parse_expr();
        children.push(SyntaxElement::Node(Box::new(expr)));

        node_from_children(SyntaxKind::FnDecl, children)
    }

    fn parse_expr(&mut self) -> SyntaxNode {
        let mut leading = Vec::new();
        while self.at_trivia() {
            leading.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        let mut node = if self.at_keyword(Keyword::Let) {
            self.parse_let_expr()
        } else if self.at_keyword(Keyword::Produce) {
            self.parse_produce_expr()
        } else if self.at_ident() {
            let token = self.bump().unwrap();
            node_from_children(SyntaxKind::IdentExpr, vec![SyntaxElement::Token(token)])
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
            if self.at_keyword(Keyword::Data) || self.at_keyword(Keyword::Fn) {
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
            return node_from_children(SyntaxKind::LetExpr, children);
        }

        while !self.eof() && !self.at_keyword(Keyword::In) {
            if self.at_keyword(Keyword::Data) || self.at_keyword(Keyword::Fn) {
                break;
            }
            children.push(SyntaxElement::Token(self.bump().unwrap()));
        }

        if self.at_keyword(Keyword::In) {
            children.push(SyntaxElement::Token(self.bump().unwrap()));
            let body = self.parse_expr();
            children.push(SyntaxElement::Node(Box::new(body)));
        } else {
            self.error_here("expected `in` in let expression");
        }

        node_from_children(SyntaxKind::LetExpr, children)
    }

    fn parse_produce_expr(&mut self) -> SyntaxNode {
        let mut children = Vec::new();
        children.push(SyntaxElement::Token(self.bump().unwrap())); // produce

        if self.eof() || self.at_keyword(Keyword::Data) || self.at_keyword(Keyword::Fn) {
            self.error_here("expected payload expression after `produce`");
            return node_from_children(SyntaxKind::ProduceExpr, children);
        }

        if self.at_ident() {
            children.push(SyntaxElement::Node(Box::new(node_from_children(
                SyntaxKind::IdentExpr,
                vec![SyntaxElement::Token(self.bump().unwrap())],
            ))));
        } else if self.at_keyword(Keyword::Let) {
            let let_node = self.parse_let_expr();
            children.push(SyntaxElement::Node(Box::new(let_node)));
        } else if self.at_trivia_or_unknown() {
            while !self.eof() && self.at_trivia_or_unknown() {
                children.push(SyntaxElement::Token(self.bump().unwrap()));
            }
            if self.at_ident() || self.at_keyword(Keyword::Let) || self.at_keyword(Keyword::Produce)
            {
                let nested = self.parse_expr();
                children.push(SyntaxElement::Node(Box::new(nested)));
            } else {
                self.error_here("expected payload expression after `produce`");
                children.push(SyntaxElement::Node(Box::new(SyntaxNode {
                    kind: SyntaxKind::Error,
                    span: self.current_span(),
                    children: Vec::new(),
                })));
            }
        } else {
            children.push(SyntaxElement::Node(Box::new(self.parse_error_node())));
        }

        node_from_children(SyntaxKind::ProduceExpr, children)
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

    fn at_symbol_text(&self, text: &str) -> bool {
        self.current().map(|t| t.text.as_str()) == Some(text)
    }

    fn current_span(&self) -> Span {
        self.current().map(|t| t.span).unwrap_or(Span::new(0, 0))
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
        let span = self
            .current()
            .map(|t| t.span)
            .unwrap_or_else(|| Span::new(0, 0));
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
