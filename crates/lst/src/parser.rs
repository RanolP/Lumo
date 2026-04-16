use lumo_lexer::{Keyword, LexError, Span, Symbol, Token, TokenKind};

/// Binding power for prefix unary operators (-, !)
const PREFIX_BP: u8 = 15;

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
    ExternType(ExternTypeDecl),
    ExternFn(ExternFnDecl),
    Data(DataDecl),
    Cap(CapDecl),
    Fn(FnDecl),
    Use(UseDecl),
    Impl(ImplDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: Option<Expr>,
    pub args: Vec<AttributeArg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeArg {
    pub key: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternTypeDecl {
    pub attrs: Vec<Attribute>,
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternFnDecl {
    pub attrs: Vec<Attribute>,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeSig>,
    pub cap: Option<CapSig>,
    pub span: Span,
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
pub struct CapDecl {
    pub name: String,
    pub operations: Vec<OperationDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeSig>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeSig>,
    pub cap: Option<CapSig>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseDecl {
    pub path: Vec<String>,
    pub names: Option<Vec<String>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplDecl {
    /// None = unnamed (author's impl), Some = named impl
    pub name: Option<String>,
    pub generics: Vec<GenericParam>,
    pub target_type: TypeSig,
    /// None = inherent impl, Some = capability impl
    pub capability: Option<TypeSig>,
    pub methods: Vec<ImplMethod>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeSig>,
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
pub struct CapSig {
    pub repr: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    EqEq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    AndAnd,
    OrOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Ident {
        name: String,
        span: Span,
    },
    String {
        value: String,
        span: Span,
    },
    Member {
        object: Box<Expr>,
        member: String,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
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
    Let {
        name: String,
        value: Box<Expr>,
        span: Span,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Perform {
        cap: String,
        span: Span,
    },
    Handle {
        cap: String,
        handler: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    },
    Bundle {
        entries: Vec<BundleEntry>,
        span: Span,
    },
    Number {
        value: String,
        span: Span,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Assign {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    },
    Ann {
        expr: Box<Expr>,
        ty: TypeSig,
        span: Span,
    },
    Block {
        stmts: Vec<BlockStmt>,
        result: Box<Expr>,
        span: Span,
    },
    IfElse {
        condition: Box<Expr>,
        then_body: Box<Expr>,
        else_body: Option<Box<Expr>>,
        span: Span,
    },
    Error {
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockStmt {
    Let {
        name: String,
        value: Expr,
        span: Span,
    },
    Expr {
        expr: Expr,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleEntry {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Expr,
    pub span: Span,
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
        let attrs = p.parse_attributes();

        if p.at_keyword(Keyword::Extern) {
            if let Some(item) = p.parse_extern_item(attrs) {
                items.push(item);
            }
            continue;
        }
        if p.at_keyword(Keyword::Cap) {
            if !attrs.is_empty() {
                p.error_here("attributes are only supported on `extern` items");
            }
            items.push(Item::Cap(p.parse_cap_decl()));
            continue;
        }
        if p.at_keyword(Keyword::Data) {
            if !attrs.is_empty() {
                p.error_here("attributes are only supported on `extern` items");
            }
            items.push(Item::Data(p.parse_data_decl()));
            continue;
        }
        if p.at_keyword(Keyword::Fn) {
            if !attrs.is_empty() {
                p.error_here("attributes are only supported on `extern` items");
            }
            items.push(Item::Fn(p.parse_fn_decl()));
            continue;
        }
        if p.at_keyword(Keyword::Use) {
            if !attrs.is_empty() {
                p.error_here("attributes are only supported on `extern` items");
            }
            items.push(Item::Use(p.parse_use_decl()));
            continue;
        }
        if p.at_keyword(Keyword::Impl) {
            if !attrs.is_empty() {
                p.error_here("attributes are only supported on `extern` items");
            }
            items.push(Item::Impl(p.parse_impl_decl()));
            continue;
        }

        if !attrs.is_empty() {
            p.error_here("attribute must be followed by an item declaration");
            continue;
        }

        p.error_here("expected top-level `data`, `cap`, `fn`, `impl`, or `extern`");
        p.bump();
    }

    ParseOutput {
        file: File { items },
        errors: p.errors,
    }
}

pub fn parse_lossless(lossless: &crate::lossless::ParseOutput) -> ParseOutput {
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
    node: &crate::lossless::SyntaxNode,
    tokens: &mut Vec<Token>,
    lex_errors: &mut Vec<LexError>,
) {
    for child in &node.children {
        match child {
            crate::lossless::SyntaxElement::Node(n) => {
                collect_lossless_tokens(n, tokens, lex_errors);
            }
            crate::lossless::SyntaxElement::Token(t) => match &t.kind {
                lumo_lexer::LosslessTokenKind::Keyword(kw) => tokens.push(Token {
                    kind: TokenKind::Keyword(*kw),
                    span: t.span,
                }),
                lumo_lexer::LosslessTokenKind::Ident => tokens.push(Token {
                    kind: TokenKind::Ident(t.text.clone()),
                    span: t.span,
                }),
                lumo_lexer::LosslessTokenKind::StringLit => tokens.push(Token {
                    kind: TokenKind::StringLit(t.text.clone()),
                    span: t.span,
                }),
                lumo_lexer::LosslessTokenKind::NumberLit => tokens.push(Token {
                    kind: TokenKind::NumberLit(t.text.clone()),
                    span: t.span,
                }),
                lumo_lexer::LosslessTokenKind::Symbol(sym) => tokens.push(Token {
                    kind: TokenKind::Symbol(*sym),
                    span: t.span,
                }),
                lumo_lexer::LosslessTokenKind::Whitespace
                | lumo_lexer::LosslessTokenKind::Newline => {}
                lumo_lexer::LosslessTokenKind::Unknown => {
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
    fn parse_attributes(&mut self) -> Vec<Attribute> {
        let mut attrs = Vec::new();
        while self.at_symbol(Symbol::Hash) {
            attrs.push(self.parse_attribute());
        }
        attrs
    }

    fn parse_attribute(&mut self) -> Attribute {
        let hash = self.expect_symbol(Symbol::Hash);
        self.expect_symbol(Symbol::LBracket);
        let name = self.expect_word();
        let mut value = None;
        let mut args = Vec::new();
        if self.at_symbol(Symbol::Equals) {
            self.bump();
            value = Some(self.parse_attribute_expr_until(
                |p| p.at_symbol(Symbol::RBracket),
                "expected attribute value expression",
            ));
        } else if self.at_symbol(Symbol::LParen) {
            self.bump();
            while !self.eof() && !self.at_symbol(Symbol::RParen) {
                let key_start = self.current_span();
                let key = self.expect_ident();
                self.expect_symbol(Symbol::Equals);
                let value = self.parse_attribute_expr_until(
                    |p| p.at_symbol(Symbol::Comma) || p.at_symbol(Symbol::RParen),
                    "expected attribute argument expression",
                );
                let value_end = self
                    .tokens
                    .get(self.index)
                    .map(|t| t.span.start)
                    .unwrap_or(key_start.end);
                args.push(AttributeArg {
                    key,
                    value,
                    span: Span::new(key_start.start, value_end.max(key_start.start)),
                });
                if self.at_symbol(Symbol::Comma) {
                    self.bump();
                } else {
                    break;
                }
            }
            self.expect_symbol(Symbol::RParen);
        }
        let end = self.expect_symbol(Symbol::RBracket);
        Attribute {
            name,
            value,
            args,
            span: Span::new(hash.start, end.end),
        }
    }

    fn parse_attribute_expr_until<F>(&mut self, stop: F, missing_message: &str) -> Expr
    where
        F: Fn(&Self) -> bool,
    {
        let start_span = self.current_span();
        let start = self.index;

        let mut paren_depth = 0_usize;
        let mut bracket_depth = 0_usize;
        let mut brace_depth = 0_usize;

        while !self.eof() {
            if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 && stop(self) {
                break;
            }
            let token = match self.current() {
                Some(token) => token,
                None => break,
            };
            match token.kind {
                TokenKind::Symbol(Symbol::LParen) => paren_depth += 1,
                TokenKind::Symbol(Symbol::RParen) => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                }
                TokenKind::Symbol(Symbol::LBracket) => bracket_depth += 1,
                TokenKind::Symbol(Symbol::RBracket) => {
                    if bracket_depth > 0 {
                        bracket_depth -= 1;
                    }
                }
                TokenKind::Symbol(Symbol::LBrace) => brace_depth += 1,
                TokenKind::Symbol(Symbol::RBrace) => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                    }
                }
                _ => {}
            }
            self.bump();
        }

        if start == self.index {
            self.error_here(missing_message);
            return Expr::Error { span: start_span };
        }

        let tokens = &self.tokens[start..self.index];
        let mut sub = Parser {
            tokens,
            index: 0,
            errors: Vec::new(),
        };
        let expr = sub.parse_expr();
        if !sub.eof() {
            sub.error_here("unexpected tokens in attribute expression");
            while !sub.eof() {
                sub.bump();
            }
        }
        self.errors.extend(sub.errors);
        expr
    }

    fn parse_extern_item(&mut self, attrs: Vec<Attribute>) -> Option<Item> {
        let start = self.expect_keyword(Keyword::Extern);
        if self.at_ident_text("type") {
            return Some(Item::ExternType(self.parse_extern_type_decl(attrs, start)));
        }
        if self.at_keyword(Keyword::Fn) {
            return Some(Item::ExternFn(self.parse_extern_fn_decl(attrs, start)));
        }
        self.error_here("expected `type` or `fn` after `extern`");
        while !self.eof() && !self.at_symbol(Symbol::Semi) {
            if self.at_keyword(Keyword::Data)
                || self.at_keyword(Keyword::Cap)
                || self.at_keyword(Keyword::Fn)
                || self.at_keyword(Keyword::Extern)
                || self.at_keyword(Keyword::Use)
            {
                break;
            }
            self.bump();
        }
        if self.at_symbol(Symbol::Semi) {
            self.bump();
        }
        None
    }

    fn parse_extern_type_decl(&mut self, attrs: Vec<Attribute>, start: Span) -> ExternTypeDecl {
        self.expect_ident_text("type");
        let name = self.expect_ident();
        let end = self.expect_symbol(Symbol::Semi);
        ExternTypeDecl {
            attrs,
            name,
            span: Span::new(start.start, end.end),
        }
    }

    fn parse_extern_fn_decl(&mut self, attrs: Vec<Attribute>, start: Span) -> ExternFnDecl {
        self.expect_keyword(Keyword::Fn);
        let name = self.expect_ident();
        let params = if self.at_symbol(Symbol::LParen) {
            self.parse_params()
        } else {
            self.error_here("expected parameter list `(...)` after extern fn name");
            Vec::new()
        };

        let mut return_type = None;
        if self.at_symbol(Symbol::Colon) {
            self.bump();
            let (repr, span) = self.collect_signature_until(|p| {
                p.at_symbol(Symbol::Slash) || p.at_symbol(Symbol::Semi)
            });
            if let Some(span) = span {
                return_type = Some(TypeSig { repr, span });
            } else {
                self.error_here("expected return type after `:`");
            }
        }
        let mut cap = None;
        if self.at_symbol(Symbol::Slash) {
            self.bump();
            let (repr, span) = self.collect_signature_until(|p| p.at_symbol(Symbol::Semi));
            if let Some(span) = span {
                cap = Some(CapSig { repr, span });
            } else {
                self.error_here("expected capability after `/`");
            }
        }

        let end = if self.at_symbol(Symbol::Semi) {
            self.bump().map(|t| t.span).unwrap_or(start)
        } else if self.eof()
            || self.at_keyword(Keyword::Data)
            || self.at_keyword(Keyword::Cap)
            || self.at_keyword(Keyword::Fn)
            || self.at_keyword(Keyword::Extern)
            || self.at_keyword(Keyword::Use)
            || self.at_symbol(Symbol::Hash)
        {
            self.tokens
                .get(self.index.saturating_sub(1))
                .map(|t| t.span)
                .unwrap_or(start)
        } else {
            self.error_here("expected `;` after extern fn declaration");
            self.current_span()
        };

        ExternFnDecl {
            attrs,
            name,
            params,
            return_type,
            cap,
            span: Span::new(start.start, end.end),
        }
    }

    fn parse_cap_decl(&mut self) -> CapDecl {
        let start = self.expect_keyword(Keyword::Cap);
        let name = self.expect_ident();
        self.expect_symbol(Symbol::LBrace);

        let mut operations = Vec::new();
        while !self.eof() && !self.at_symbol(Symbol::RBrace) {
            if self.at_keyword(Keyword::Fn) {
                operations.push(self.parse_operation_decl());
            } else {
                self.error_here("expected `fn` in cap body");
                self.bump();
                continue;
            }
            if self.at_symbol(Symbol::Comma) || self.at_symbol(Symbol::Semi) {
                self.bump();
            }
        }

        let end = self.expect_symbol(Symbol::RBrace);
        CapDecl {
            name,
            operations,
            span: Span::new(start.start, end.end),
        }
    }

    fn parse_operation_decl(&mut self) -> OperationDecl {
        let start = self.expect_keyword(Keyword::Fn);
        let name = self.expect_ident();
        let params = if self.at_symbol(Symbol::LParen) {
            self.parse_params()
        } else {
            self.error_here("expected parameter list `(...)` after operation name");
            Vec::new()
        };
        let mut return_type = None;
        let mut end = self
            .tokens
            .get(self.index.saturating_sub(1))
            .map(|t| t.span)
            .unwrap_or(start);
        if self.at_symbol(Symbol::Colon) {
            self.bump();
            let (repr, span) = self.collect_signature_until(|p| {
                p.at_symbol(Symbol::Comma)
                    || p.at_symbol(Symbol::Semi)
                    || p.at_symbol(Symbol::RBrace)
                    || p.at_keyword(Keyword::Fn)
            });
            if let Some(span) = span {
                end = span;
                return_type = Some(TypeSig { repr, span });
            } else {
                self.error_here("expected return type after `:`");
            }
        }
        OperationDecl {
            name,
            params,
            return_type,
            span: Span::new(start.start, end.end),
        }
    }

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
                p.at_symbol(Symbol::Slash)
                    || p.at_symbol(Symbol::LBrace)
                    || p.at_symbol(Symbol::Equals)
            });
            if let Some(span) = span {
                return_type = Some(TypeSig { repr, span });
            } else {
                self.error_here("expected return type after `:`");
            }
        }
        let mut cap = None;
        if self.at_symbol(Symbol::Slash) {
            self.bump();
            let (repr, span) = self.collect_balanced_signature_until_lbrace();
            if let Some(span) = span {
                cap = Some(CapSig { repr, span });
            } else {
                self.error_here("expected capability after `/`");
            }
        }

        let body = if self.at_symbol(Symbol::Equals) {
            self.bump();
            self.parse_expr()
        } else {
            self.parse_block()
        };
        let body_span = expr_span(&body);
        FnDecl {
            name,
            generics,
            params,
            return_type,
            cap,
            body,
            span: Span::new(start.start, body_span.end),
        }
    }

    // Grammar: 'use' ident ('.' ident)* ['.' '{' ident (',' ident)* '}'] ';'
    fn parse_use_decl(&mut self) -> UseDecl {
        let start = self.expect_keyword(Keyword::Use);
        let mut path = Vec::new();

        // Parse first path segment
        if self.at_ident() {
            path.push(ident_text(&self.bump().unwrap()).unwrap_or_default().to_owned());
        } else {
            self.error_here("expected module path after `use`");
            return UseDecl {
                path,
                names: None,
                span: start,
            };
        }

        // Parse remaining '.' separated segments, or '.' '{' destructure '}'
        let mut names = None;
        while self.at_symbol(Symbol::Dot) {
            self.bump(); // .

            // Check for destructuring: { a, b, self }
            if self.at_symbol(Symbol::LBrace) {
                self.bump(); // {
                let mut name_list = Vec::new();
                while !self.eof() && !self.at_symbol(Symbol::RBrace) {
                    if self.at_ident() {
                        name_list.push(
                            ident_text(&self.bump().unwrap())
                                .unwrap_or_default()
                                .to_owned(),
                        );
                    } else {
                        self.error_here("expected name in use destructuring");
                        break;
                    }
                    if self.at_symbol(Symbol::Comma) {
                        self.bump();
                    } else {
                        break;
                    }
                }
                if self.at_symbol(Symbol::RBrace) {
                    self.bump();
                } else {
                    self.error_here("expected `}` in use destructuring");
                }
                names = Some(name_list);
                break;
            }

            // Regular path segment
            if self.at_ident() {
                path.push(ident_text(&self.bump().unwrap()).unwrap_or_default().to_owned());
            } else {
                self.error_here("expected identifier after `.` in use path");
                break;
            }
        }

        let end = self.current_span();
        if self.at_symbol(Symbol::Semi) {
            self.bump();
        }

        UseDecl {
            path,
            names,
            span: Span::new(start.start, end.end),
        }
    }

    // Grammar:
    //   impl[generics] Name = TargetType : Cap { methods }   -- named cap impl
    //   impl[generics] TargetType : Cap { methods }           -- unnamed cap impl
    //   impl TargetType { methods }                           -- inherent impl
    //   impl Name = TargetType { methods }                    -- named inherent impl
    fn parse_impl_decl(&mut self) -> ImplDecl {
        let start = self.expect_keyword(Keyword::Impl);

        // Optional generics: impl[T: Clone]
        let generics = if self.at_symbol(Symbol::LBracket) {
            self.parse_generics()
        } else {
            Vec::new()
        };

        // Determine if named (ident followed by `=`) or unnamed.
        // We need lookahead: if current is ident and next is `=`, it's named.
        let name = if self.at_ident() && self.peek_is_symbol(Symbol::Equals) {
            let n = self.expect_ident();
            self.expect_symbol(Symbol::Equals);
            Some(n)
        } else {
            None
        };

        // Parse target type (until `:` or `{`)
        let (target_repr, target_span) = self.collect_signature_until(|p| {
            p.at_symbol(Symbol::Colon) || p.at_symbol(Symbol::LBrace)
        });
        let target_type = if let Some(span) = target_span {
            TypeSig {
                repr: target_repr,
                span,
            }
        } else {
            self.error_here("expected target type in impl declaration");
            TypeSig {
                repr: "<missing>".to_owned(),
                span: self.current_span(),
            }
        };

        // Optional capability: `: Clone`
        let capability = if self.at_symbol(Symbol::Colon) {
            self.bump();
            let (cap_repr, cap_span) =
                self.collect_signature_until(|p| p.at_symbol(Symbol::LBrace));
            if let Some(span) = cap_span {
                Some(TypeSig {
                    repr: cap_repr,
                    span,
                })
            } else {
                self.error_here("expected capability type after `:`");
                None
            }
        } else {
            None
        };

        // Parse methods block: { fn name(params): RetType = body ... }
        self.expect_symbol(Symbol::LBrace);
        let mut methods = Vec::new();

        while !self.eof() && !self.at_symbol(Symbol::RBrace) {
            let method_start = self.current_span();
            if !self.at_keyword(Keyword::Fn) {
                self.error_here("expected `fn` in impl method");
                self.bump();
                continue;
            }
            self.bump(); // fn

            let method_name = self.expect_ident();
            let params = if self.at_symbol(Symbol::LParen) {
                self.parse_impl_params()
            } else {
                Vec::new()
            };

            let mut return_type = None;
            if self.at_symbol(Symbol::Colon) {
                self.bump();
                let (repr, span) = self.collect_signature_until(|p| {
                    p.at_symbol(Symbol::LBrace) || p.at_symbol(Symbol::Equals)
                });
                if let Some(span) = span {
                    return_type = Some(TypeSig { repr, span });
                } else {
                    self.error_here("expected return type after `:`");
                }
            }

            let body = if self.at_symbol(Symbol::Equals) {
                self.bump();
                self.parse_expr()
            } else {
                self.parse_block()
            };
            let body_end = expr_span(&body);

            methods.push(ImplMethod {
                name: method_name,
                params,
                return_type,
                body,
                span: Span::new(method_start.start, body_end.end),
            });

            if self.at_symbol(Symbol::Semi) || self.at_symbol(Symbol::Comma) {
                self.bump();
            }
        }

        let end = self.expect_symbol(Symbol::RBrace);
        ImplDecl {
            name,
            generics,
            target_type,
            capability,
            methods,
            span: Span::new(start.start, end.end),
        }
    }

    /// Parse params for impl methods. `self` is allowed without a type annotation.
    fn parse_impl_params(&mut self) -> Vec<Param> {
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

            if name == "self" && !self.at_symbol(Symbol::Colon) {
                // `self` without type annotation — type injected during HIR lowering
                out.push(Param {
                    name,
                    ty: TypeSig {
                        repr: "Self".to_owned(),
                        span: name_token.span,
                    },
                    span: name_token.span,
                });
            } else {
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
            }

            if self.at_symbol(Symbol::Comma) {
                self.bump();
            }
        }

        self.expect_symbol(Symbol::RParen);
        out
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

    /// Collect tokens for a capability annotation (after `/`) until the next
    /// unbalanced `{` that starts the function body block.  This handles the
    /// common `/ {}` and `/ { E1, E2 }` forms: the `{ ... }` is consumed as
    /// part of the capability signature, and the *next* `{` begins the body.
    fn collect_balanced_signature_until_lbrace(&mut self) -> (String, Option<Span>) {
        let mut parts = Vec::new();
        let mut start = None;
        let mut end = None;
        let mut depth: usize = 0;

        while !self.eof() {
            if self.at_symbol(Symbol::Equals) && depth == 0 {
                // `=` at depth 0 starts an expression body, stop.
                break;
            }
            if self.at_symbol(Symbol::LBrace) {
                if depth == 0 && !parts.is_empty() {
                    // We already have tokens and we're at depth 0:
                    // this `{` is the body block, stop.
                    break;
                }
                depth += 1;
                let t = self.bump().unwrap();
                start.get_or_insert(t.span.start);
                end = Some(t.span.end);
                parts.push(token_text(t));
                continue;
            }
            if self.at_symbol(Symbol::RBrace) {
                if depth == 0 {
                    break; // unexpected
                }
                depth -= 1;
                let t = self.bump().unwrap();
                end = Some(t.span.end);
                parts.push(token_text(t));
                if depth == 0 {
                    // Finished a balanced `{ ... }`.  If the next token is `{` or `=`,
                    // that starts the body — stop here.
                    if self.at_symbol(Symbol::LBrace) || self.at_symbol(Symbol::Equals) {
                        break;
                    }
                }
                continue;
            }
            // Non-brace tokens
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
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Expr {
        // Keyword expressions consume everything rightward — return immediately
        if self.at_keyword(Keyword::Let) {
            let start = self.expect_keyword(Keyword::Let);
            let name = self.expect_ident();
            self.expect_symbol(Symbol::Equals);
            let value = self.parse_expr();
            let end = expr_span(&value);
            return Expr::Let {
                name,
                value: Box::new(value),
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

        if self.at_keyword(Keyword::If) {
            return self.parse_if_expr();
        }

        if self.at_keyword(Keyword::Handle) {
            let start = self.expect_keyword(Keyword::Handle);
            let (cap, _) = self.collect_signature_until(|p| {
                p.at_ident_text("with")
            });
            if self.at_ident_text("with") {
                self.bump();
            } else {
                self.error_here("expected `with` after handle cap name");
            }
            let handler = self.parse_expr();
            self.expect_keyword(Keyword::In);
            let body = self.parse_expr();
            let end = expr_span(&body);
            return Expr::Handle {
                cap,
                handler: Box::new(handler),
                body: Box::new(body),
                span: Span::new(start.start, end.end),
            };
        }

        // Primary expression (including prefix unary)
        let mut expr = if self.at_symbol(Symbol::Minus) || self.at_symbol(Symbol::Bang) {
            // Prefix unary operators
            let op_token = self.bump().expect("checked at_symbol").clone();
            let op = if op_token.kind == TokenKind::Symbol(Symbol::Minus) {
                UnaryOp::Neg
            } else {
                UnaryOp::Not
            };
            let operand = self.parse_expr_bp(PREFIX_BP);
            let end = expr_span(&operand);
            Expr::Unary {
                op,
                expr: Box::new(operand),
                span: Span::new(op_token.span.start, end.end),
            }
        } else if self.at_keyword(Keyword::Bundle) {
            self.parse_bundle_expr()
        } else if self.at_symbol(Symbol::LBrace) {
            self.parse_block()
        } else if self.at_ident() {
            let token = self.bump().expect("checked at_ident").clone();
            Expr::Ident {
                name: ident_text(&token).unwrap_or_default().to_owned(),
                span: token.span,
            }
        } else if self.at_string_lit() {
            let token = self.bump().expect("checked at_string_lit").clone();
            Expr::String {
                value: decode_string_lit(token_text(&token).as_str()),
                span: token.span,
            }
        } else if self.at_number_lit() {
            let token = self.bump().expect("checked at_number_lit").clone();
            Expr::Number {
                value: token_text(&token),
                span: token.span,
            }
        } else if self.at_symbol(Symbol::LParen) {
            let start = self.expect_symbol(Symbol::LParen);
            let inner = self.parse_expr();
            if self.at_symbol(Symbol::Colon) {
                self.bump();
                let (repr, ty_span) = self.collect_signature_until(|p| {
                    p.at_symbol(Symbol::RParen)
                });
                let end = self.expect_symbol(Symbol::RParen);
                if let Some(ty_span) = ty_span {
                    Expr::Ann {
                        expr: Box::new(inner),
                        ty: TypeSig { repr, span: ty_span },
                        span: Span::new(start.start, end.end),
                    }
                } else {
                    self.error_here("expected type after `:`");
                    Expr::Error { span: Span::new(start.start, end.end) }
                }
            } else {
                self.expect_symbol(Symbol::RParen);
                inner
            }
        } else {
            let span = self.current_span();
            self.error_here("expected expression");
            if !(self.at_keyword(Keyword::Data)
                || self.at_keyword(Keyword::Cap)
                || self.at_keyword(Keyword::Fn)
                || self.at_keyword(Keyword::Extern)
                || self.at_keyword(Keyword::Use)
                || self.at_keyword(Keyword::Impl))
                && !self.eof()
            {
                self.bump();
            }
            return Expr::Error { span };
        };

        // Infix/postfix loop (Pratt)
        loop {
            // Postfix: .member (highest precedence)
            if self.at_symbol(Symbol::Dot) {
                let start = expr_span(&expr).start;
                self.bump();
                let member = self.expect_ident();
                let end = self
                    .tokens
                    .get(self.index.saturating_sub(1))
                    .map(|t| t.span.end)
                    .unwrap_or(start);
                expr = Expr::Member {
                    object: Box::new(expr),
                    member,
                    span: Span::new(start, end),
                };
                continue;
            }

            // Postfix: (args) (highest precedence)
            if self.at_symbol(Symbol::LParen) {
                let start = expr_span(&expr).start;
                self.bump();
                let mut args = Vec::new();
                while !self.eof() && !self.at_symbol(Symbol::RParen) {
                    args.push(self.parse_expr());
                    if self.at_symbol(Symbol::Comma) {
                        self.bump();
                    } else {
                        break;
                    }
                }
                let end = self.expect_symbol(Symbol::RParen).end;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                    span: Span::new(start, end),
                };
                continue;
            }

            // Assignment: ident = value ; body (right-assoc, lowest bp)
            if self.at_symbol(Symbol::Equals) {
                if let Expr::Ident { ref name, .. } = expr {
                    let (l_bp, _) = (1_u8, 2_u8); // right-assoc: l_bp=1, r_bp=2 (but we use 0 for rhs)
                    if l_bp < min_bp {
                        break;
                    }
                    let name = name.clone();
                    let start = expr_span(&expr).start;
                    self.bump(); // consume =
                    let value = self.parse_expr();
                    if self.at_symbol(Symbol::Semi) {
                        self.bump(); // consume ;
                    }
                    let body = self.parse_expr();
                    let end = expr_span(&body);
                    expr = Expr::Assign {
                        name,
                        value: Box::new(value),
                        body: Box::new(body),
                        span: Span::new(start, end.end),
                    };
                    continue;
                }
                break;
            }

            // Infix binary operators
            if let Some((op, l_bp, r_bp)) = self.peek_infix_op() {
                if l_bp < min_bp {
                    break;
                }
                self.bump(); // consume operator token
                let right = self.parse_expr_bp(r_bp);
                let start = expr_span(&expr).start;
                let end = expr_span(&right).end;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                    span: Span::new(start, end),
                };
                continue;
            }

            break;
        }

        expr
    }

    fn at_infix_operator(&self) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(TokenKind::Symbol(
                Symbol::Plus
                    | Symbol::Minus
                    | Symbol::Star
                    | Symbol::Slash
                    | Symbol::Percent
                    | Symbol::EqEq
                    | Symbol::BangEq
                    | Symbol::Lt
                    | Symbol::LtEq
                    | Symbol::Gt
                    | Symbol::GtEq
                    | Symbol::AmpAmp
                    | Symbol::PipePipe
            ))
        )
    }

    fn peek_infix_op(&self) -> Option<(BinaryOp, u8, u8)> {
        let sym = match self.current().map(|t| &t.kind) {
            Some(TokenKind::Symbol(s)) => *s,
            _ => return None,
        };
        let (op, l_bp, r_bp) = match sym {
            Symbol::PipePipe => (BinaryOp::OrOr, 3, 4),
            Symbol::AmpAmp => (BinaryOp::AndAnd, 5, 6),
            Symbol::EqEq => (BinaryOp::EqEq, 7, 8),
            Symbol::BangEq => (BinaryOp::NotEq, 7, 8),
            Symbol::Lt => (BinaryOp::Lt, 9, 10),
            Symbol::LtEq => (BinaryOp::LtEq, 9, 10),
            Symbol::Gt => (BinaryOp::Gt, 9, 10),
            Symbol::GtEq => (BinaryOp::GtEq, 9, 10),
            Symbol::Plus => (BinaryOp::Add, 11, 12),
            Symbol::Minus => (BinaryOp::Sub, 11, 12),
            Symbol::Star => (BinaryOp::Mul, 13, 14),
            Symbol::Slash => (BinaryOp::Div, 13, 14),
            Symbol::Percent => (BinaryOp::Mod, 13, 14),
            _ => return None,
        };
        Some((op, l_bp, r_bp))
    }

    fn parse_block(&mut self) -> Expr {
        let start = self.expect_symbol(Symbol::LBrace);
        let mut stmts = Vec::new();

        loop {
            if self.eof() || self.at_symbol(Symbol::RBrace) {
                // Empty block or trailing — error, but produce Error result
                self.error_here("expected expression in block");
                let end = self.expect_symbol(Symbol::RBrace);
                return Expr::Block {
                    stmts,
                    result: Box::new(Expr::Error {
                        span: Span::new(end.start, end.end),
                    }),
                    span: Span::new(start.start, end.end),
                };
            }

            // Try `let name = expr ;`
            if self.at_keyword(Keyword::Let) {
                let let_start = self.expect_keyword(Keyword::Let);
                let name = self.expect_ident();
                self.expect_symbol(Symbol::Equals);
                let value = self.parse_expr();
                let value_span = expr_span(&value);
                if self.at_symbol(Symbol::Semi) {
                    self.bump();
                    stmts.push(BlockStmt::Let {
                        name,
                        value,
                        span: Span::new(let_start.start, value_span.end),
                    });
                    continue;
                }
                // No semicolon — treat `let x = expr` as the block result
                self.error_here("expected `;` after let binding in block");
                let end = self.expect_symbol(Symbol::RBrace);
                return Expr::Block {
                    stmts,
                    result: Box::new(Expr::Let {
                        name,
                        value: Box::new(value),
                        span: Span::new(let_start.start, value_span.end),
                    }),
                    span: Span::new(start.start, end.end),
                };
            }

            // Parse an expression
            let expr = self.parse_expr();
            let expr_span_val = expr_span(&expr);

            if self.at_symbol(Symbol::Semi) {
                // Expression statement
                self.bump();
                stmts.push(BlockStmt::Expr {
                    expr,
                    span: expr_span_val,
                });
                continue;
            }

            // No semicolon — this is the result expression
            let end = self.expect_symbol(Symbol::RBrace);
            return Expr::Block {
                stmts,
                result: Box::new(expr),
                span: Span::new(start.start, end.end),
            };
        }
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

    fn parse_if_expr(&mut self) -> Expr {
        let start = self.expect_keyword(Keyword::If);
        let condition = self.parse_expr_bp(0);
        let then_body = self.parse_block();
        let then_end = expr_span(&then_body);

        let (else_body, end) = if self.at_keyword(Keyword::Else) {
            self.bump();
            if self.at_keyword(Keyword::If) {
                let e = self.parse_if_expr();
                let s = expr_span(&e);
                (Some(Box::new(e)), s)
            } else {
                let e = self.parse_block();
                let s = expr_span(&e);
                (Some(Box::new(e)), s)
            }
        } else {
            (None, then_end)
        };

        Expr::IfElse {
            condition: Box::new(condition),
            then_body: Box::new(then_body),
            else_body,
            span: Span::new(start.start, end.end),
        }
    }

    fn parse_bundle_expr(&mut self) -> Expr {
        let start = self.expect_keyword(Keyword::Bundle);
        self.expect_symbol(Symbol::LBrace);
        let mut entries = Vec::new();

        while !self.eof() && !self.at_symbol(Symbol::RBrace) {
            let entry_start = self.current_span();
            if !self.at_keyword(Keyword::Fn) {
                self.error_here("expected `fn` in bundle entry");
                self.bump();
                continue;
            }
            self.bump(); // fn

            let name = self.expect_ident();
            let params = if self.at_symbol(Symbol::LParen) {
                self.parse_params()
            } else {
                Vec::new()
            };

            let body = if self.at_symbol(Symbol::Equals) {
                self.bump();
                self.parse_expr()
            } else {
                self.parse_block()
            };
            let body_end = expr_span(&body);

            entries.push(BundleEntry {
                name,
                params,
                body,
                span: Span::new(entry_start.start, body_end.end),
            });

            if self.at_symbol(Symbol::Semi) || self.at_symbol(Symbol::Comma) {
                self.bump();
            }
        }

        let end = self.expect_symbol(Symbol::RBrace);
        Expr::Bundle {
            entries,
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

    fn peek_is_symbol(&self, sym: Symbol) -> bool {
        matches!(
            self.tokens.get(self.index + 1).map(|t| &t.kind),
            Some(TokenKind::Symbol(actual)) if *actual == sym
        )
    }

    fn at_ident(&self) -> bool {
        matches!(self.current().map(|t| &t.kind), Some(TokenKind::Ident(_)))
    }

    fn at_string_lit(&self) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(TokenKind::StringLit(_))
        )
    }

    fn at_number_lit(&self) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(TokenKind::NumberLit(_))
        )
    }

    fn at_ident_text(&self, expected: &str) -> bool {
        matches!(
            self.current().map(|t| &t.kind),
            Some(TokenKind::Ident(actual)) if actual == expected
        )
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

    fn expect_word(&mut self) -> String {
        if let Some(token) = self.current() {
            match &token.kind {
                TokenKind::Ident(_) | TokenKind::Keyword(_) => {
                    let text = token_text(token);
                    self.bump();
                    return text;
                }
                _ => {}
            }
        }
        self.error_here("expected identifier");
        "<missing>".to_owned()
    }

    fn expect_ident_text(&mut self, expected: &str) {
        if self.at_ident_text(expected) {
            self.bump();
            return;
        }
        self.error_here(&format!("expected `{expected}`"));
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
        Expr::String { span, .. } => *span,
        Expr::Member { span, .. } => *span,
        Expr::Call { span, .. } => *span,
        Expr::Thunk { span, .. } => *span,
        Expr::Force { span, .. } => *span,
        Expr::Let { span, .. } => *span,
        Expr::Match { span, .. } => *span,
        Expr::Perform { span, .. } => *span,
        Expr::Handle { span, .. } => *span,
        Expr::Bundle { span, .. } => *span,
        Expr::Number { span, .. } => *span,
        Expr::Binary { span, .. } => *span,
        Expr::Unary { span, .. } => *span,
        Expr::Assign { span, .. } => *span,
        Expr::Ann { span, .. } => *span,
        Expr::Block { span, .. } => *span,
        Expr::IfElse { span, .. } => *span,
        Expr::Error { span } => *span,
    }
}

fn decode_string_lit(text: &str) -> String {
    let inner = text
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(text);
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn token_text(token: &Token) -> String {
    match &token.kind {
        TokenKind::Keyword(Keyword::Data) => "data".to_owned(),
        TokenKind::Keyword(Keyword::Fn) => "fn".to_owned(),
        TokenKind::Keyword(Keyword::Extern) => "extern".to_owned(),
        TokenKind::Keyword(Keyword::Let) => "let".to_owned(),
        TokenKind::Keyword(Keyword::In) => "in".to_owned(),
        TokenKind::Keyword(Keyword::Produce) => "produce".to_owned(),
        TokenKind::Keyword(Keyword::Thunk) => "thunk".to_owned(),
        TokenKind::Keyword(Keyword::Force) => "force".to_owned(),
        TokenKind::Keyword(Keyword::Match) => "match".to_owned(),
        TokenKind::Keyword(Keyword::Cap) => "cap".to_owned(),
        TokenKind::Keyword(Keyword::Perform) => "perform".to_owned(),
        TokenKind::Keyword(Keyword::Handle) => "handle".to_owned(),
        TokenKind::Keyword(Keyword::Bundle) => "bundle".to_owned(),
        TokenKind::Keyword(Keyword::Use) => "use".to_owned(),
        TokenKind::Keyword(Keyword::Impl) => "impl".to_owned(),
        TokenKind::Keyword(Keyword::If) => "if".to_owned(),
        TokenKind::Keyword(Keyword::Else) => "else".to_owned(),
        TokenKind::Keyword(Keyword::Lambda) => "lambda".to_owned(),
        TokenKind::Keyword(Keyword::Roll) => "roll".to_owned(),
        TokenKind::Keyword(Keyword::Unroll) => "unroll".to_owned(),
        TokenKind::Keyword(Keyword::Ctor) => "ctor".to_owned(),
        TokenKind::Ident(s) => s.clone(),
        TokenKind::StringLit(s) => s.clone(),
        TokenKind::Symbol(Symbol::Hash) => "#".to_owned(),
        TokenKind::Symbol(Symbol::LBracket) => "[".to_owned(),
        TokenKind::Symbol(Symbol::RBracket) => "]".to_owned(),
        TokenKind::Symbol(Symbol::LParen) => "(".to_owned(),
        TokenKind::Symbol(Symbol::RParen) => ")".to_owned(),
        TokenKind::Symbol(Symbol::LBrace) => "{".to_owned(),
        TokenKind::Symbol(Symbol::RBrace) => "}".to_owned(),
        TokenKind::Symbol(Symbol::Semi) => ";".to_owned(),
        TokenKind::Symbol(Symbol::Colon) => ":".to_owned(),
        TokenKind::Symbol(Symbol::Comma) => ",".to_owned(),
        TokenKind::Symbol(Symbol::Equals) => "=".to_owned(),
        TokenKind::Symbol(Symbol::ColonEquals) => ":=".to_owned(),
        TokenKind::Symbol(Symbol::Slash) => "/".to_owned(),
        TokenKind::Symbol(Symbol::Star) => "*".to_owned(),
        TokenKind::Symbol(Symbol::FatArrow) => "=>".to_owned(),
        TokenKind::Symbol(Symbol::Dot) => ".".to_owned(),
        TokenKind::Symbol(Symbol::DotDot) => "..".to_owned(),
        TokenKind::Symbol(Symbol::Plus) => "+".to_owned(),
        TokenKind::Symbol(Symbol::Minus) => "-".to_owned(),
        TokenKind::Symbol(Symbol::Percent) => "%".to_owned(),
        TokenKind::Symbol(Symbol::Bang) => "!".to_owned(),
        TokenKind::Symbol(Symbol::Lt) => "<".to_owned(),
        TokenKind::Symbol(Symbol::Gt) => ">".to_owned(),
        TokenKind::Symbol(Symbol::LtEq) => "<=".to_owned(),
        TokenKind::Symbol(Symbol::GtEq) => ">=".to_owned(),
        TokenKind::Symbol(Symbol::EqEq) => "==".to_owned(),
        TokenKind::Symbol(Symbol::BangEq) => "!=".to_owned(),
        TokenKind::Symbol(Symbol::AmpAmp) => "&&".to_owned(),
        TokenKind::Symbol(Symbol::PipePipe) => "||".to_owned(),
        TokenKind::NumberLit(s) => s.clone(),
    }
}
