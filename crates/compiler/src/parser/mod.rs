use crate::lexer::{Keyword, LexError, Span, Symbol, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Data(DataDecl),
    Fn(FnDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDecl {
    pub name: String,
    pub variants: Vec<VariantDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantDecl {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
    pub name: String,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Ident {
        name: String,
        span: Span,
    },
    Produce {
        expr: Box<Expr>,
        span: Span,
    },
    LetIn {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    },
    Error {
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutput {
    pub file: File,
    pub errors: Vec<ParseError>,
}

pub fn parse(tokens: &[Token], lex_errors: &[LexError]) -> ParseOutput {
    let mut p = Parser {
        tokens,
        index: 0,
        errors: lex_errors
            .iter()
            .map(|e| ParseError {
                span: e.span,
                message: e.message.clone(),
            })
            .collect(),
    };

    let mut items = Vec::new();
    while !p.eof() {
        if p.at_keyword(Keyword::Data) {
            items.push(Item::Data(p.parse_data_decl()));
            continue;
        }
        if p.at_keyword(Keyword::Fn) {
            items.push(Item::Fn(p.parse_fn_decl()));
            continue;
        }

        p.error_here("expected top-level `data` or `fn`");
        p.bump();
    }

    ParseOutput {
        file: File { items },
        errors: p.errors,
    }
}

pub fn parse_lossless(lossless: &crate::lst::lossless::ParseOutput) -> ParseOutput {
    let mut tokens = Vec::new();
    let mut lex_errors = Vec::new();
    collect_lossless_tokens(&lossless.root, &mut tokens, &mut lex_errors);

    let mut out = parse(&tokens, &lex_errors);
    out.errors
        .extend(lossless.errors.iter().map(|e| ParseError {
            span: e.span,
            message: e.message.clone(),
        }));
    out
}

fn collect_lossless_tokens(
    node: &crate::lst::lossless::SyntaxNode,
    tokens: &mut Vec<Token>,
    lex_errors: &mut Vec<LexError>,
) {
    for child in &node.children {
        match child {
            crate::lst::lossless::SyntaxElement::Node(n) => {
                collect_lossless_tokens(n, tokens, lex_errors);
            }
            crate::lst::lossless::SyntaxElement::Token(t) => match &t.kind {
                crate::lexer::LosslessTokenKind::Keyword(kw) => tokens.push(Token {
                    kind: TokenKind::Keyword(*kw),
                    span: t.span,
                }),
                crate::lexer::LosslessTokenKind::Ident => tokens.push(Token {
                    kind: TokenKind::Ident(t.text.clone()),
                    span: t.span,
                }),
                crate::lexer::LosslessTokenKind::Symbol(sym) => tokens.push(Token {
                    kind: TokenKind::Symbol(*sym),
                    span: t.span,
                }),
                crate::lexer::LosslessTokenKind::Whitespace
                | crate::lexer::LosslessTokenKind::Newline => {}
                crate::lexer::LosslessTokenKind::Unknown => {
                    let message = match t.text.chars().next() {
                        Some(ch) if t.text.chars().count() == 1 => {
                            format!("unexpected character: {ch:?}")
                        }
                        _ => format!("unexpected character: {:?}", t.text),
                    };
                    lex_errors.push(LexError {
                        span: t.span,
                        message,
                    });
                }
            },
        }
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    fn parse_data_decl(&mut self) -> DataDecl {
        let start = self.expect_keyword(Keyword::Data);
        let name = self.expect_ident();

        while !self.eof() && !self.at_symbol(Symbol::LBrace) {
            self.bump();
        }
        self.expect_symbol(Symbol::LBrace);

        let mut variants = Vec::new();
        while !self.eof() && !self.at_symbol(Symbol::RBrace) {
            if self.at_ident() {
                let token = self.bump().unwrap();
                let name = ident_text(token).unwrap_or_default().to_owned();
                variants.push(VariantDecl {
                    name,
                    span: token.span,
                });
                while !self.eof()
                    && !self.at_symbol(Symbol::Comma)
                    && !self.at_symbol(Symbol::RBrace)
                {
                    self.bump();
                }
                if self.at_symbol(Symbol::Comma) {
                    self.bump();
                }
            } else {
                self.error_here("expected variant name");
                self.bump();
            }
        }

        let end = self.expect_symbol(Symbol::RBrace);
        DataDecl {
            name,
            variants,
            span: Span::new(start.start, end.end),
        }
    }

    fn parse_fn_decl(&mut self) -> FnDecl {
        let start = self.expect_keyword(Keyword::Fn);
        let name = self.expect_ident();

        while !self.eof() && !self.at_symbol(Symbol::ColonEquals) {
            self.bump();
        }
        self.expect_symbol(Symbol::ColonEquals);

        let body = self.parse_expr();
        let body_span = expr_span(&body);
        FnDecl {
            name,
            body,
            span: Span::new(start.start, body_span.end),
        }
    }

    fn parse_expr(&mut self) -> Expr {
        if self.at_keyword(Keyword::Let) {
            let start = self.expect_keyword(Keyword::Let);
            let name = self.expect_ident();
            self.expect_symbol(Symbol::Equals);
            let value = self.parse_expr();
            self.expect_keyword(Keyword::In);
            let body = self.parse_expr();
            let end = expr_span(&body);
            return Expr::LetIn {
                name,
                value: Box::new(value),
                body: Box::new(body),
                span: Span::new(start.start, end.end),
            };
        }

        if self.at_keyword(Keyword::Produce) {
            let start = self.expect_keyword(Keyword::Produce);
            let expr = self.parse_expr();
            let end = expr_span(&expr);
            return Expr::Produce {
                expr: Box::new(expr),
                span: Span::new(start.start, end.end),
            };
        }

        if self.at_ident() {
            let token = self.bump().unwrap();
            return Expr::Ident {
                name: ident_text(token).unwrap_or_default().to_owned(),
                span: token.span,
            };
        }

        let span = self.current_span();
        self.error_here("expected expression");
        if !(self.at_keyword(Keyword::Data) || self.at_keyword(Keyword::Fn)) && !self.eof() {
            self.bump();
        }
        Expr::Error { span }
    }

    fn at_keyword(&self, kw: Keyword) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(TokenKind::Keyword(actual)) if *actual == kw
        )
    }

    fn at_symbol(&self, sym: Symbol) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(TokenKind::Symbol(actual)) if *actual == sym
        )
    }

    fn at_ident(&self) -> bool {
        matches!(self.current().map(|t| &t.kind), Some(TokenKind::Ident(_)))
    }

    fn expect_keyword(&mut self, kw: Keyword) -> Span {
        if self.at_keyword(kw) {
            return self.bump().unwrap().span;
        }
        self.error_here(&format!("expected keyword {kw:?}"));
        self.current_span()
    }

    fn expect_symbol(&mut self, sym: Symbol) -> Span {
        if self.at_symbol(sym) {
            return self.bump().unwrap().span;
        }
        self.error_here(&format!("expected symbol {sym:?}"));
        self.current_span()
    }

    fn expect_ident(&mut self) -> String {
        if self.at_ident() {
            let token = self.bump().unwrap();
            return ident_text(token).unwrap_or_default().to_owned();
        }
        self.error_here("expected identifier");
        "<missing>".to_owned()
    }

    fn error_here(&mut self, message: &str) {
        self.errors.push(ParseError {
            span: self.current_span(),
            message: message.to_owned(),
        });
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn current_span(&self) -> Span {
        self.current().map(|t| t.span).unwrap_or(Span::new(0, 0))
    }

    fn bump(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.index);
        if token.is_some() {
            self.index += 1;
        }
        token
    }

    fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }
}

fn ident_text(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Ident(s) => Some(s),
        _ => None,
    }
}

fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::Ident { span, .. } => *span,
        Expr::Produce { span, .. } => *span,
        Expr::LetIn { span, .. } => *span,
        Expr::Error { span } => *span,
    }
}
