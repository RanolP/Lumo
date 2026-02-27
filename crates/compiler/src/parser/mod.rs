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
    pub generics: Vec<GenericParam>,
    pub variants: Vec<VariantDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantDecl {
    pub name: String,
    pub payload: Vec<TypeSig>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeSig>,
    pub effect: Option<EffectSig>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericParam {
    pub name: String,
    pub constraint: Option<TypeSig>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: TypeSig,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSig {
    pub repr: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectSig {
    pub repr: String,
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
    Thunk {
        expr: Box<Expr>,
        span: Span,
    },
    Force {
        expr: Box<Expr>,
        span: Span,
    },
    LetIn {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Apply {
        owner: String,
        member: String,
        args: Vec<Expr>,
        span: Span,
    },
    Error {
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: String,
    pub body: Expr,
    pub span: Span,
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
        let generics = if self.at_symbol(Symbol::LBracket) {
            self.parse_generics()
        } else {
            Vec::new()
        };

        self.expect_symbol(Symbol::LBrace);

        let mut variants = Vec::new();
        while !self.eof() && !self.at_symbol(Symbol::RBrace) {
            if !self.at_symbol(Symbol::Dot) {
                self.error_here("expected variant name prefixed with `.`");
                self.bump();
                continue;
            }
            variants.push(self.parse_variant_decl());
            if self.at_symbol(Symbol::Comma) {
                self.bump();
            }
        }

        let end = self.expect_symbol(Symbol::RBrace);
        DataDecl {
            name,
            generics,
            variants,
            span: Span::new(start.start, end.end),
        }
    }

    fn parse_variant_decl(&mut self) -> VariantDecl {
        let dot = self.expect_symbol(Symbol::Dot);
        let name_token = if self.at_ident() {
            self.bump().cloned()
        } else {
            None
        };
        let Some(name_token) = name_token else {
            self.error_here("expected variant name after `.`");
            return VariantDecl {
                name: "<missing>".to_owned(),
                payload: Vec::new(),
                span: dot,
            };
        };
        let name = ident_text(&name_token).unwrap_or_default().to_owned();
        let mut payload = Vec::new();
        let mut end = name_token.span.end;

        if self.at_symbol(Symbol::LParen) {
            self.bump();
            while !self.eof() && !self.at_symbol(Symbol::RParen) {
                let (repr, span) = self.collect_signature_until(|p| {
                    p.at_symbol(Symbol::Comma) || p.at_symbol(Symbol::RParen)
                });
                if let Some(span) = span {
                    end = span.end.max(end);
                    payload.push(TypeSig { repr, span });
                } else {
                    self.error_here("expected variant payload type");
                    break;
                }
                if self.at_symbol(Symbol::Comma) {
                    self.bump();
                }
            }
            let rparen = self.expect_symbol(Symbol::RParen);
            end = end.max(rparen.end);
        }

        VariantDecl {
            name,
            payload,
            span: Span::new(dot.start, end),
        }
    }

    fn parse_fn_decl(&mut self) -> FnDecl {
        let start = self.expect_keyword(Keyword::Fn);
        let name = self.expect_ident();
        let generics = if self.at_symbol(Symbol::LBracket) {
            self.parse_generics()
        } else {
            Vec::new()
        };
        let params = if self.at_symbol(Symbol::LParen) {
            self.parse_params()
        } else {
            self.error_here("expected parameter list `(...)` after fn name");
            Vec::new()
        };
        let mut return_type = None;
        if self.at_symbol(Symbol::Colon) {
            self.bump();
            let (repr, span) = self.collect_signature_until(|p| {
                p.at_symbol(Symbol::Slash) || p.at_symbol(Symbol::ColonEquals)
            });
            if let Some(span) = span {
                return_type = Some(TypeSig { repr, span });
            } else {
                self.error_here("expected return type after `:`");
            }
        }
        let mut effect = None;
        if self.at_symbol(Symbol::Slash) {
            self.bump();
            let (repr, span) = self.collect_signature_until(|p| p.at_symbol(Symbol::ColonEquals));
            if let Some(span) = span {
                effect = Some(EffectSig { repr, span });
            } else {
                self.error_here("expected effect after `/`");
            }
        }

        while !self.eof() && !self.at_symbol(Symbol::ColonEquals) {
            if self.at_keyword(Keyword::Data) || self.at_keyword(Keyword::Fn) {
                break;
            }
            self.bump();
        }
        self.expect_symbol(Symbol::ColonEquals);

        let body = self.parse_expr();
        let body_span = expr_span(&body);
        FnDecl {
            name,
            generics,
            params,
            return_type,
            effect,
            body,
            span: Span::new(start.start, body_span.end),
        }
    }

    fn parse_generics(&mut self) -> Vec<GenericParam> {
        let mut out = Vec::new();
        self.expect_symbol(Symbol::LBracket);

        while !self.eof() && !self.at_symbol(Symbol::RBracket) {
            if !self.at_ident() {
                self.error_here("expected generic parameter name");
                self.bump();
                continue;
            }

            let name_token = self.bump().expect("checked at_ident").clone();
            let name = ident_text(&name_token).unwrap_or_default().to_owned();
            let mut end = name_token.span.end;
            let mut constraint = None;

            if self.at_symbol(Symbol::Colon) {
                self.bump();
                let (repr, span) = self.collect_signature_until(|p| {
                    p.at_symbol(Symbol::Comma) || p.at_symbol(Symbol::RBracket)
                });
                if let Some(span) = span {
                    end = span.end;
                    constraint = Some(TypeSig { repr, span });
                } else {
                    self.error_here("expected generic constraint");
                }
            }

            out.push(GenericParam {
                name,
                constraint,
                span: Span::new(name_token.span.start, end),
            });

            if self.at_symbol(Symbol::Comma) {
                self.bump();
            }
        }

        self.expect_symbol(Symbol::RBracket);
        out
    }

    fn parse_params(&mut self) -> Vec<Param> {
        let mut out = Vec::new();
        self.expect_symbol(Symbol::LParen);

        while !self.eof() && !self.at_symbol(Symbol::RParen) {
            if !self.at_ident() {
                self.error_here("expected parameter name");
                self.bump();
                continue;
            }

            let name_token = self.bump().expect("checked at_ident").clone();
            let name = ident_text(&name_token).unwrap_or_default().to_owned();
            self.expect_symbol(Symbol::Colon);
            let (repr, span) = self.collect_signature_until(|p| {
                p.at_symbol(Symbol::Comma) || p.at_symbol(Symbol::RParen)
            });
            let ty = if let Some(span) = span {
                TypeSig { repr, span }
            } else {
                self.error_here("expected parameter type");
                TypeSig {
                    repr: "<missing>".to_owned(),
                    span: self.current_span(),
                }
            };
            let end = ty.span.end.max(name_token.span.end);
            out.push(Param {
                name,
                ty,
                span: Span::new(name_token.span.start, end),
            });

            if self.at_symbol(Symbol::Comma) {
                self.bump();
            }
        }

        self.expect_symbol(Symbol::RParen);
        out
    }

    fn collect_signature_until<F>(&mut self, stop: F) -> (String, Option<Span>)
    where
        F: Fn(&Self) -> bool,
    {
        let mut parts = Vec::new();
        let mut start = None;
        let mut end = None;

        while !self.eof() && !stop(self) {
            let token = match self.bump() {
                Some(t) => t,
                None => break,
            };
            start.get_or_insert(token.span.start);
            end = Some(token.span.end);
            parts.push(token_text(token));
        }

        let repr = parts.join(" ");
        let span = start.zip(end).map(|(s, e)| Span::new(s, e));
        (repr, span)
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

        if self.at_keyword(Keyword::Thunk) {
            let start = self.expect_keyword(Keyword::Thunk);
            let expr = self.parse_expr();
            let end = expr_span(&expr);
            return Expr::Thunk {
                expr: Box::new(expr),
                span: Span::new(start.start, end.end),
            };
        }

        if self.at_keyword(Keyword::Force) {
            let start = self.expect_keyword(Keyword::Force);
            let expr = self.parse_expr();
            let end = expr_span(&expr);
            return Expr::Force {
                expr: Box::new(expr),
                span: Span::new(start.start, end.end),
            };
        }

        if self.at_keyword(Keyword::Match) {
            return self.parse_match_expr();
        }

        if self.at_ident() {
            let token = self.bump().unwrap().clone();
            let name = ident_text(&token).unwrap_or_default().to_owned();
            if self.at_symbol(Symbol::Dot) {
                self.bump();
                let member = self.expect_ident();
                self.expect_symbol(Symbol::LParen);
                let mut args = Vec::new();
                while !self.eof() && !self.at_symbol(Symbol::RParen) {
                    args.push(self.parse_expr());
                    if self.at_symbol(Symbol::Comma) {
                        self.bump();
                    } else {
                        break;
                    }
                }
                let end = self.expect_symbol(Symbol::RParen);
                return Expr::Apply {
                    owner: name,
                    member,
                    args,
                    span: Span::new(token.span.start, end.end),
                };
            }
            return Expr::Ident {
                name,
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

    fn parse_match_expr(&mut self) -> Expr {
        let start = self.expect_keyword(Keyword::Match);
        let scrutinee = self.parse_expr();
        self.expect_symbol(Symbol::LBrace);

        let mut arms = Vec::new();
        while !self.eof() && !self.at_symbol(Symbol::RBrace) {
            let (pattern, pattern_span) = self.collect_signature_until(|p| {
                p.at_symbol(Symbol::FatArrow) || p.at_symbol(Symbol::RBrace)
            });
            let pattern_span = match pattern_span {
                Some(span) => span,
                None => {
                    self.error_here("expected match pattern");
                    break;
                }
            };

            self.expect_symbol(Symbol::FatArrow);
            let body = self.parse_expr();
            let body_span = expr_span(&body);
            arms.push(MatchArm {
                pattern,
                body,
                span: Span::new(pattern_span.start, body_span.end),
            });

            if self.at_symbol(Symbol::Comma) {
                self.bump();
            }
        }

        let end = self.expect_symbol(Symbol::RBrace);
        Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
            span: Span::new(start.start, end.end),
        }
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
        Expr::Thunk { span, .. } => *span,
        Expr::Force { span, .. } => *span,
        Expr::LetIn { span, .. } => *span,
        Expr::Match { span, .. } => *span,
        Expr::Apply { span, .. } => *span,
        Expr::Error { span } => *span,
    }
}

fn token_text(token: &Token) -> String {
    match &token.kind {
        TokenKind::Keyword(Keyword::Data) => "data".to_owned(),
        TokenKind::Keyword(Keyword::Fn) => "fn".to_owned(),
        TokenKind::Keyword(Keyword::Let) => "let".to_owned(),
        TokenKind::Keyword(Keyword::In) => "in".to_owned(),
        TokenKind::Keyword(Keyword::Produce) => "produce".to_owned(),
        TokenKind::Keyword(Keyword::Thunk) => "thunk".to_owned(),
        TokenKind::Keyword(Keyword::Force) => "force".to_owned(),
        TokenKind::Keyword(Keyword::Match) => "match".to_owned(),
        TokenKind::Ident(s) => s.clone(),
        TokenKind::Symbol(Symbol::LBracket) => "[".to_owned(),
        TokenKind::Symbol(Symbol::RBracket) => "]".to_owned(),
        TokenKind::Symbol(Symbol::LParen) => "(".to_owned(),
        TokenKind::Symbol(Symbol::RParen) => ")".to_owned(),
        TokenKind::Symbol(Symbol::LBrace) => "{".to_owned(),
        TokenKind::Symbol(Symbol::RBrace) => "}".to_owned(),
        TokenKind::Symbol(Symbol::Colon) => ":".to_owned(),
        TokenKind::Symbol(Symbol::Comma) => ",".to_owned(),
        TokenKind::Symbol(Symbol::Equals) => "=".to_owned(),
        TokenKind::Symbol(Symbol::ColonEquals) => ":=".to_owned(),
        TokenKind::Symbol(Symbol::Slash) => "/".to_owned(),
        TokenKind::Symbol(Symbol::Star) => "*".to_owned(),
        TokenKind::Symbol(Symbol::FatArrow) => "=>".to_owned(),
        TokenKind::Symbol(Symbol::Dot) => ".".to_owned(),
    }
}
