use crate::{
    BundleEntry, CapDecl, DataDecl, ExternFnDecl, ExternTypeDecl, Expr, File, FnDecl, ImplDecl,
    ImplMethodDecl, Item, MatchArm, OperationDecl, Param, UseDecl, VariantDecl,
};
use lumo_lexer::{Keyword, Symbol, Token, TokenKind};
use lumo_span::Span;
use lumo_types::{CapRef, ContentHash, ExprId, Pattern, Spanned, TypeExpr};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

pub fn parse(source: &str) -> Result<File, Vec<ParseError>> {
    let lex_output = lumo_lexer::lex(source);
    let mut parser = Parser {
        tokens: lex_output.tokens,
        pos: 0,
        errors: Vec::new(),
        spans: Vec::new(),
    };
    let items = parser.parse_file();
    if parser.errors.is_empty() {
        Ok(File {
            items,
            content_hash: ContentHash(0), // Recomputed by caller if needed
            spans: parser.spans,
        })
    } else {
        Err(parser.errors)
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
    spans: Vec<Span>,
}

impl Parser {
    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos).map(|t| &t.kind)
    }

    fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span)
            .unwrap_or_else(|| {
                self.tokens
                    .last()
                    .map(|t| Span::new(t.span.end, t.span.end))
                    .unwrap_or(Span::new(0, 0))
            })
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        self.pos += 1;
        tok
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn alloc(&mut self, span: Span) -> ExprId {
        let id = ExprId(self.spans.len() as u32);
        self.spans.push(span);
        id
    }

    fn expect_kw(&mut self, kw: Keyword) -> Result<Span, ()> {
        if self.peek() == Some(&TokenKind::Keyword(kw)) {
            Ok(self.advance().span)
        } else {
            self.error(format!("expected keyword `{kw:?}`"));
            Err(())
        }
    }

    fn expect_sym(&mut self, sym: Symbol) -> Result<Span, ()> {
        if self.peek() == Some(&TokenKind::Symbol(sym)) {
            Ok(self.advance().span)
        } else {
            self.error(format!("expected symbol `{sym:?}`"));
            Err(())
        }
    }

    fn expect_ident(&mut self) -> Result<(String, Span), ()> {
        match self.peek() {
            Some(TokenKind::Ident(_)) => {
                let tok = self.advance();
                if let TokenKind::Ident(name) = &tok.kind {
                    Ok((name.clone(), tok.span))
                } else {
                    unreachable!()
                }
            }
            _ => {
                self.error("expected identifier".into());
                Err(())
            }
        }
    }

    fn peek_ident_text(&self, text: &str) -> bool {
        matches!(self.peek(), Some(TokenKind::Ident(name)) if name == text)
    }

    fn eat_sym(&mut self, sym: Symbol) -> bool {
        if self.peek() == Some(&TokenKind::Symbol(sym)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn eat_kw(&mut self, kw: Keyword) -> bool {
        if self.peek() == Some(&TokenKind::Keyword(kw)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn eat_ident(&mut self, name: &str) -> bool {
        if matches!(self.peek(), Some(TokenKind::Ident(n)) if n == name) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn error(&mut self, message: String) {
        let span = self.peek_span();
        self.errors.push(ParseError { span, message });
    }

    // -----------------------------------------------------------------------
    // File
    // -----------------------------------------------------------------------

    fn parse_file(&mut self) -> Vec<Item> {
        let mut items = Vec::new();
        while !self.at_end() {
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else {
                if !self.at_end() {
                    self.advance();
                }
            }
        }
        items
    }

    // -----------------------------------------------------------------------
    // Items (shared with HIR parser, minor differences)
    // -----------------------------------------------------------------------

    fn parse_item(&mut self) -> Option<Item> {
        let inline = self.try_parse_inline_hint();
        match self.peek()? {
            TokenKind::Keyword(Keyword::Extern) => self.parse_extern_item(inline),
            TokenKind::Keyword(Keyword::Data) => Some(Item::Data(self.parse_data_decl()?)),
            TokenKind::Keyword(Keyword::Cap) => Some(Item::Cap(self.parse_cap_decl()?)),
            TokenKind::Keyword(Keyword::Fn) => Some(Item::Fn(self.parse_fn_decl()?)),
            TokenKind::Keyword(Keyword::Use) => Some(Item::Use(self.parse_use_decl()?)),
            TokenKind::Keyword(Keyword::Impl) => Some(Item::Impl(self.parse_impl_decl()?)),
            _ => {
                self.error("expected item declaration".into());
                None
            }
        }
    }

    fn try_parse_inline_hint(&mut self) -> bool {
        if self.peek() != Some(&TokenKind::Symbol(Symbol::Hash)) {
            return false;
        }
        let save = self.pos;
        self.advance();
        if !self.eat_sym(Symbol::LBracket)
            || !self.eat_ident("inline")
            || !self.eat_sym(Symbol::LParen)
            || !self.eat_ident("always")
            || !self.eat_sym(Symbol::RParen)
            || !self.eat_sym(Symbol::RBracket)
        {
            self.pos = save;
            return false;
        }
        true
    }

    fn parse_extern_item(&mut self, inline: bool) -> Option<Item> {
        let start = self.expect_kw(Keyword::Extern).ok()?;
        if self.eat_ident("type") {
            let (name, name_span) = self.expect_ident().ok()?;
            let extern_name = self.try_parse_extern_as();
            Some(Item::ExternType(ExternTypeDecl {
                name,
                extern_name,
                span: Span::new(start.start, name_span.end),
            }))
        } else if self.eat_kw(Keyword::Fn) {
            let (name, _) = self.expect_ident().ok()?;
            let params = self.parse_param_list()?;
            let return_type = self.try_parse_return_ann();
            let cap = self.try_parse_cap_annotation();
            let extern_name = self.try_parse_extern_as();
            let end = self.peek_span();
            Some(Item::ExternFn(ExternFnDecl {
                name,
                extern_name,
                inline,
                params,
                return_type,
                cap,
                span: Span::new(start.start, end.start),
            }))
        } else {
            self.error("expected `type` or `fn` after `extern`".into());
            None
        }
    }

    fn try_parse_extern_as(&mut self) -> Option<String> {
        if !self.eat_ident("as") {
            return None;
        }
        match self.peek() {
            Some(TokenKind::StringLit(_)) => {
                let tok = self.advance();
                if let TokenKind::StringLit(s) = &tok.kind {
                    Some(strip_string_quotes(s))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_data_decl(&mut self) -> Option<DataDecl> {
        let start = self.expect_kw(Keyword::Data).ok()?;
        let (name, _) = self.expect_ident().ok()?;
        let generics = self.try_parse_generic_params();
        self.expect_sym(Symbol::LBrace).ok()?;
        let mut variants = Vec::new();
        while self.peek() == Some(&TokenKind::Symbol(Symbol::Dot)) {
            if let Some(v) = self.parse_variant() {
                variants.push(v);
            }
            self.eat_sym(Symbol::Comma);
        }
        let end = self.expect_sym(Symbol::RBrace).ok()?;
        Some(DataDecl {
            name,
            generics,
            variants,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_variant(&mut self) -> Option<VariantDecl> {
        let start = self.expect_sym(Symbol::Dot).ok()?;
        let (name, name_span) = self.expect_ident().ok()?;
        let mut payload = Vec::new();
        if self.eat_sym(Symbol::LParen) {
            while self.peek() != Some(&TokenKind::Symbol(Symbol::RParen)) && !self.at_end() {
                if let Some(ty) = self.parse_type_expr() {
                    payload.push(ty);
                }
                self.eat_sym(Symbol::Comma);
            }
            self.expect_sym(Symbol::RParen).ok()?;
        }
        Some(VariantDecl {
            name,
            payload,
            span: Span::new(start.start, name_span.end),
        })
    }

    fn parse_cap_decl(&mut self) -> Option<CapDecl> {
        let start = self.expect_kw(Keyword::Cap).ok()?;
        let (name, _) = self.expect_ident().ok()?;
        self.expect_sym(Symbol::LBrace).ok()?;
        let mut operations = Vec::new();
        while self.peek() == Some(&TokenKind::Keyword(Keyword::Fn)) {
            if let Some(op) = self.parse_operation_decl() {
                operations.push(op);
            }
        }
        let end = self.expect_sym(Symbol::RBrace).ok()?;
        Some(CapDecl {
            name,
            operations,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_operation_decl(&mut self) -> Option<OperationDecl> {
        let start = self.expect_kw(Keyword::Fn).ok()?;
        let (name, _) = self.expect_ident().ok()?;
        let params = self.parse_param_list()?;
        let return_type = self.try_parse_return_ann();
        let end = self.peek_span();
        Some(OperationDecl {
            name,
            params,
            return_type,
            span: Span::new(start.start, end.start),
        })
    }

    fn parse_fn_decl(&mut self) -> Option<FnDecl> {
        let start = self.expect_kw(Keyword::Fn).ok()?;
        let (name, _) = self.expect_ident().ok()?;
        let generics = self.try_parse_generic_params();
        let params = self.parse_param_list()?;
        let return_type = self.try_parse_return_ann();
        let cap = self.try_parse_cap_annotation();
        self.expect_sym(Symbol::ColonEquals).ok()?;
        let value = self.parse_expr()?;
        let end_span = self.peek_span();
        Some(FnDecl {
            name,
            generics,
            params,
            return_type,
            cap,
            value,
            span: Span::new(start.start, end_span.start),
        })
    }

    fn parse_use_decl(&mut self) -> Option<UseDecl> {
        let start = self.expect_kw(Keyword::Use).ok()?;
        let mut path = Vec::new();
        let (first, _) = self.expect_ident().ok()?;
        path.push(first);
        let mut names = None;
        while self.eat_sym(Symbol::Dot) {
            if self.eat_sym(Symbol::LBrace) {
                let mut n = Vec::new();
                while let Some(TokenKind::Ident(_)) = self.peek() {
                    let (name, _) = self.expect_ident().ok()?;
                    n.push(name);
                    self.eat_sym(Symbol::Comma);
                }
                self.expect_sym(Symbol::RBrace).ok()?;
                names = Some(n);
                break;
            } else if let Some(TokenKind::Ident(_)) = self.peek() {
                let (seg, _) = self.expect_ident().ok()?;
                path.push(seg);
            } else {
                break;
            }
        }
        let end = self.expect_sym(Symbol::Semi).ok()?;
        Some(UseDecl {
            path,
            names,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_impl_decl(&mut self) -> Option<ImplDecl> {
        let start = self.expect_kw(Keyword::Impl).ok()?;
        let generics = self.try_parse_generic_params();
        let name = self.try_parse_impl_name();
        let target_type = self.parse_type_expr()?;
        let capability = if self.eat_sym(Symbol::Colon) {
            Some(self.parse_type_expr()?)
        } else {
            None
        };
        self.expect_sym(Symbol::LBrace).ok()?;
        let mut methods = Vec::new();
        while self.peek() == Some(&TokenKind::Keyword(Keyword::Fn)) {
            if let Some(m) = self.parse_impl_method() {
                methods.push(m);
            }
        }
        let end = self.expect_sym(Symbol::RBrace).ok()?;
        Some(ImplDecl {
            name,
            generics,
            target_type,
            capability,
            methods,
            span: Span::new(start.start, end.end),
        })
    }

    fn try_parse_impl_name(&mut self) -> Option<String> {
        if self.pos + 1 >= self.tokens.len() {
            return None;
        }
        let is_ident = matches!(self.peek(), Some(TokenKind::Ident(_)));
        let followed_by_eq = matches!(
            self.tokens.get(self.pos + 1).map(|t| &t.kind),
            Some(TokenKind::Symbol(Symbol::Equals))
        );
        if is_ident && followed_by_eq {
            let (name, _) = self.expect_ident().ok()?;
            self.advance(); // consume `=`
            Some(name)
        } else {
            None
        }
    }

    fn parse_impl_method(&mut self) -> Option<ImplMethodDecl> {
        let start = self.expect_kw(Keyword::Fn).ok()?;
        let (name, _) = self.expect_ident().ok()?;
        let params = self.parse_param_list()?;
        let return_type = self.try_parse_return_ann();
        self.expect_sym(Symbol::ColonEquals).ok()?;
        let value = self.parse_expr()?;
        let end_span = self.peek_span();
        Some(ImplMethodDecl {
            name,
            params,
            return_type,
            value,
            span: Span::new(start.start, end_span.start),
        })
    }

    // -----------------------------------------------------------------------
    // Shared
    // -----------------------------------------------------------------------

    fn try_parse_generic_params(&mut self) -> Vec<String> {
        if !self.eat_sym(Symbol::LBracket) {
            return Vec::new();
        }
        let mut params = Vec::new();
        while let Some(TokenKind::Ident(_)) = self.peek() {
            let (name, _) = self.expect_ident().ok().unwrap();
            params.push(name);
            self.eat_sym(Symbol::Comma);
        }
        let _ = self.expect_sym(Symbol::RBracket);
        params
    }

    fn parse_param_list(&mut self) -> Option<Vec<Param>> {
        self.expect_sym(Symbol::LParen).ok()?;
        let mut params = Vec::new();
        while self.peek() != Some(&TokenKind::Symbol(Symbol::RParen)) && !self.at_end() {
            if let Some(param) = self.parse_param() {
                params.push(param);
            }
            self.eat_sym(Symbol::Comma);
        }
        self.expect_sym(Symbol::RParen).ok()?;
        Some(params)
    }

    fn parse_param(&mut self) -> Option<Param> {
        let (name, start) = self.expect_ident().ok()?;
        self.expect_sym(Symbol::Colon).ok()?;
        let ty = self.parse_type_expr()?;
        let end = ty.span;
        Some(Param {
            name,
            ty,
            span: Span::new(start.start, end.end),
        })
    }

    fn try_parse_return_ann(&mut self) -> Option<Spanned<TypeExpr>> {
        if self.peek() != Some(&TokenKind::Symbol(Symbol::Colon)) {
            return None;
        }
        if matches!(
            self.tokens.get(self.pos),
            Some(t) if t.kind == TokenKind::Symbol(Symbol::ColonEquals)
        ) {
            return None;
        }
        self.advance();
        self.parse_type_expr()
    }

    fn try_parse_cap_annotation(&mut self) -> Option<CapRef> {
        if !self.eat_sym(Symbol::Slash) {
            return None;
        }
        self.expect_sym(Symbol::LBrace).ok()?;
        if self.eat_sym(Symbol::RBrace) {
            return Some(CapRef::Pure);
        }
        let is_infer = self.eat_sym(Symbol::DotDot);
        if is_infer {
            if self.eat_sym(Symbol::RBrace) {
                return Some(CapRef::Infer(vec![]));
            }
            self.eat_sym(Symbol::Comma);
        }
        let mut entries = Vec::new();
        loop {
            let (name, _) = self.expect_ident().ok()?;
            let type_args = if self.peek_ident_text("for") {
                self.advance(); // consume "for"
                let (ty, _) = self.expect_ident().ok()?;
                vec![TypeExpr::Named(ty)]
            } else {
                vec![]
            };
            entries.push(TypeExpr::Cap { name, type_args });
            if !self.eat_sym(Symbol::Comma) {
                break;
            }
        }
        self.expect_sym(Symbol::RBrace).ok()?;
        if is_infer {
            Some(CapRef::Infer(entries))
        } else {
            Some(CapRef::Named(entries))
        }
    }

    fn parse_type_expr(&mut self) -> Option<Spanned<TypeExpr>> {
        if self.peek() == Some(&TokenKind::Keyword(Keyword::Produce)) {
            let start = self.advance().span;
            let inner = self.parse_type_expr()?;
            return Some(Spanned {
                span: Span::new(start.start, inner.span.end),
                value: TypeExpr::Produce(Box::new(inner.value)),
            });
        }
        if self.peek() == Some(&TokenKind::Keyword(Keyword::Thunk)) {
            let start = self.advance().span;
            let inner = self.parse_type_expr()?;
            return Some(Spanned {
                span: Span::new(start.start, inner.span.end),
                value: TypeExpr::Thunk(Box::new(inner.value)),
            });
        }
        let (name, span) = self.expect_ident().ok()?;
        if self.eat_sym(Symbol::LBracket) {
            let mut args = Vec::new();
            while self.peek() != Some(&TokenKind::Symbol(Symbol::RBracket)) && !self.at_end() {
                if let Some(ty) = self.parse_type_expr() {
                    args.push(ty.value);
                }
                self.eat_sym(Symbol::Comma);
            }
            let end = self.expect_sym(Symbol::RBracket).ok()?;
            Some(Spanned {
                span: Span::new(span.start, end.end),
                value: TypeExpr::App { head: name, args },
            })
        } else {
            Some(Spanned {
                span,
                value: TypeExpr::Named(name),
            })
        }
    }

    // -----------------------------------------------------------------------
    // Expressions — LIR-specific
    // -----------------------------------------------------------------------

    fn parse_expr(&mut self) -> Option<Expr> {
        let (expr, allow_postfix) = self.parse_expr_primary()?;
        self.parse_postfix(expr, allow_postfix)
    }

    fn parse_postfix(&mut self, expr: Expr, allow_postfix: bool) -> Option<Expr> {
        if !allow_postfix {
            return Some(expr);
        }
        let mut expr = expr;
        loop {
            if self.eat_sym(Symbol::Dot) {
                let (field, field_span) = self.expect_ident().ok()?;
                let expr_span = self.spans[expr.id().0 as usize];
                let id = self.alloc(Span::new(expr_span.start, field_span.end));
                expr = Expr::Member {
                    id,
                    object: Box::new(expr),
                    field,
                };
            } else if self.peek() == Some(&TokenKind::Symbol(Symbol::LParen)) {
                // Single-arg apply: callee(arg)
                self.advance(); // (
                let arg = self.parse_expr()?;
                self.expect_sym(Symbol::RParen).ok()?;
                let id = self.alloc(self.peek_span());
                expr = Expr::Apply {
                    id,
                    callee: Box::new(expr),
                    arg: Box::new(arg),
                };
            } else {
                break;
            }
        }
        Some(expr)
    }

    fn parse_expr_primary(&mut self) -> Option<(Expr, bool)> {
        match self.peek()? {
            TokenKind::Keyword(Keyword::Produce) => self.parse_produce_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Thunk) => self.parse_thunk_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Force) => self.parse_force_expr().map(|e| (e, true)),
            TokenKind::Keyword(Keyword::Lambda) => self.parse_lambda_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Roll) => self.parse_roll_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Unroll) => self.parse_unroll_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Ctor) => self.parse_ctor_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Let) => self.parse_let_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Match) => self.parse_match_expr().map(|e| (e, true)),
            TokenKind::Keyword(Keyword::Perform) => self.parse_perform_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Handle) => self.parse_handle_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Bundle) => self.parse_bundle_expr().map(|e| (e, true)),
            TokenKind::Ident(_) => {
                let (name, span) = self.expect_ident().ok()?;
                let id = self.alloc(span);
                Some((Expr::Ident { id, name }, true))
            }
            TokenKind::StringLit(_) => {
                let tok = self.advance();
                let span = tok.span;
                let s = if let TokenKind::StringLit(s) = &tok.kind {
                    strip_string_quotes(s)
                } else {
                    return None;
                };
                let id = self.alloc(span);
                Some((Expr::String { id, value: s }, false))
            }
            TokenKind::NumberLit(_) => {
                let tok = self.advance();
                let span = tok.span;
                let s = if let TokenKind::NumberLit(s) = &tok.kind {
                    s.clone()
                } else {
                    return None;
                };
                let id = self.alloc(span);
                Some((Expr::Number { id, value: s }, false))
            }
            TokenKind::Symbol(Symbol::LParen) => self.parse_paren_or_ann().map(|e| (e, true)),
            TokenKind::Symbol(Symbol::Lt) => self.parse_error_expr().map(|e| (e, false)),
            _ => {
                self.error("expected expression".into());
                None
            }
        }
    }

    fn parse_produce_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Produce).ok()?;
        let inner = self.parse_expr()?;
        let id = self.alloc(start);
        Some(Expr::Produce {
            id,
            expr: Box::new(inner),
        })
    }

    fn parse_thunk_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Thunk).ok()?;
        let inner = self.parse_expr()?;
        let id = self.alloc(start);
        Some(Expr::Thunk {
            id,
            expr: Box::new(inner),
        })
    }

    fn parse_force_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Force).ok()?;
        let (inner, allow_postfix) = self.parse_expr_primary()?;
        let inner = self.parse_postfix(inner, allow_postfix)?;
        let id = self.alloc(start);
        Some(Expr::Force {
            id,
            expr: Box::new(inner),
        })
    }

    fn parse_lambda_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Lambda).ok()?;
        let (param, _) = self.expect_ident().ok()?;
        self.expect_sym(Symbol::Dot).ok()?;
        let body = self.parse_expr()?;
        let id = self.alloc(start);
        Some(Expr::Lambda {
            id,
            param,
            body: Box::new(body),
        })
    }

    fn parse_roll_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Roll).ok()?;
        let (inner, allow_postfix) = self.parse_expr_primary()?;
        let inner = self.parse_postfix(inner, allow_postfix)?;
        let id = self.alloc(start);
        Some(Expr::Roll {
            id,
            expr: Box::new(inner),
        })
    }

    fn parse_unroll_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Unroll).ok()?;
        let (inner, allow_postfix) = self.parse_expr_primary()?;
        let inner = self.parse_postfix(inner, allow_postfix)?;
        let id = self.alloc(start);
        Some(Expr::Unroll {
            id,
            expr: Box::new(inner),
        })
    }

    fn parse_ctor_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Ctor).ok()?;
        let (name, _) = self.expect_ident().ok()?;
        // Allow dotted name: Type.Variant
        let name = if self.eat_sym(Symbol::Dot) {
            let (variant, _) = self.expect_ident().ok()?;
            format!("{name}.{variant}")
        } else {
            name
        };
        // Check for args: ctor Type.Variant(arg1, arg2)
        if self.eat_sym(Symbol::LParen) {
            let mut args = Vec::new();
            while self.peek() != Some(&TokenKind::Symbol(Symbol::RParen)) && !self.at_end() {
                if let Some(arg) = self.parse_expr() {
                    args.push(arg);
                }
                self.eat_sym(Symbol::Comma);
            }
            self.expect_sym(Symbol::RParen).ok()?;
            let id = self.alloc(start);
            Some(Expr::Ctor {
                id,
                name,
                called: true,
                args,
            })
        } else {
            let id = self.alloc(start);
            Some(Expr::Ctor {
                id,
                name,
                called: false,
                args: vec![],
            })
        }
    }

    fn parse_let_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Let).ok()?;
        let (name, _) = self.expect_ident().ok()?;
        self.expect_sym(Symbol::Equals).ok()?;
        let value = self.parse_expr()?;
        self.expect_kw(Keyword::In).ok()?;
        let body = self.parse_expr()?;
        let id = self.alloc(start);
        Some(Expr::Let {
            id,
            name,
            value: Box::new(value),
            body: Box::new(body),
        })
    }

    fn parse_match_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Match).ok()?;
        let scrutinee = self.parse_expr()?;
        self.expect_sym(Symbol::LBrace).ok()?;
        let mut arms = Vec::new();
        while self.peek() != Some(&TokenKind::Symbol(Symbol::RBrace)) && !self.at_end() {
            if let Some(arm) = self.parse_match_arm() {
                arms.push(arm);
            } else {
                break;
            }
        }
        self.expect_sym(Symbol::RBrace).ok()?;
        let id = self.alloc(start);
        Some(Expr::Match {
            id,
            scrutinee: Box::new(scrutinee),
            arms,
        })
    }

    fn parse_match_arm(&mut self) -> Option<MatchArm> {
        let pattern = self.parse_pattern()?;
        let start = self.peek_span();
        self.expect_sym(Symbol::FatArrow).ok()?;
        let body = self.parse_expr()?;
        let end = self.peek_span();
        self.eat_sym(Symbol::Semi);
        Some(MatchArm {
            pattern,
            body,
            span: Span::new(start.start, end.start),
        })
    }

    fn parse_perform_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Perform).ok()?;
        let (cap, _) = self.expect_ident().ok()?;
        let id = self.alloc(start);
        Some(Expr::Perform { id, cap, type_args: vec![] })
    }

    fn parse_handle_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Handle).ok()?;
        let (cap, _) = self.expect_ident().ok()?;
        let type_args = if self.peek_ident_text("for") {
            self.advance(); // consume "for"
            let (ty, _) = self.expect_ident().ok()?;
            vec![ty]
        } else {
            vec![]
        };
        if !self.eat_ident("with") {
            self.error("expected `with`".into());
            return None;
        }
        let handler = self.parse_expr()?;
        self.expect_kw(Keyword::In).ok()?;
        let body = self.parse_expr()?;
        let id = self.alloc(start);
        Some(Expr::Handle {
            id,
            cap,
            type_args,
            handler: Box::new(handler),
            body: Box::new(body),
        })
    }

    fn parse_bundle_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Bundle).ok()?;
        self.expect_sym(Symbol::LBrace).ok()?;
        let mut entries = Vec::new();
        while self.peek() == Some(&TokenKind::Keyword(Keyword::Fn)) {
            if let Some(entry) = self.parse_bundle_entry() {
                entries.push(entry);
            }
        }
        self.expect_sym(Symbol::RBrace).ok()?;
        let id = self.alloc(start);
        Some(Expr::Bundle { id, entries })
    }

    fn parse_bundle_entry(&mut self) -> Option<BundleEntry> {
        let start = self.expect_kw(Keyword::Fn).ok()?;
        let (name, _) = self.expect_ident().ok()?;
        let params = self.parse_param_list()?;
        self.expect_sym(Symbol::ColonEquals).ok()?;
        let body = self.parse_expr()?;
        let end = self.peek_span();
        Some(BundleEntry {
            name,
            params,
            body,
            span: Span::new(start.start, end.start),
        })
    }

    fn parse_paren_or_ann(&mut self) -> Option<Expr> {
        self.advance(); // (
        let inner = self.parse_expr()?;
        if self.eat_sym(Symbol::Colon) {
            let ty = self.parse_type_expr()?;
            self.expect_sym(Symbol::RParen).ok()?;
            let id = self.alloc(self.peek_span());
            Some(Expr::Ann {
                id,
                expr: Box::new(inner),
                ty: ty.value,
            })
        } else {
            self.expect_sym(Symbol::RParen).ok()?;
            Some(inner)
        }
    }

    fn parse_error_expr(&mut self) -> Option<Expr> {
        let start = self.expect_sym(Symbol::Lt).ok()?;
        if !self.eat_ident("error") {
            self.error("expected `error`".into());
            return None;
        }
        self.expect_sym(Symbol::Gt).ok()?;
        let id = self.alloc(start);
        Some(Expr::Error { id })
    }

    // -----------------------------------------------------------------------
    // Patterns
    // -----------------------------------------------------------------------

    fn parse_pattern(&mut self) -> Option<Pattern> {
        match self.peek()? {
            TokenKind::Symbol(Symbol::Dot) => {
                self.advance();
                let (name, _) = self.expect_ident().ok()?;
                let mut args = Vec::new();
                if self.eat_sym(Symbol::LParen) {
                    while self.peek() != Some(&TokenKind::Symbol(Symbol::RParen)) && !self.at_end()
                    {
                        if let Some(pat) = self.parse_pattern() {
                            args.push(pat);
                        }
                        self.eat_sym(Symbol::Comma);
                    }
                    self.expect_sym(Symbol::RParen).ok()?;
                }
                Some(Pattern::Ctor { name, args })
            }
            TokenKind::Ident(n) if n == "_" => {
                self.advance();
                Some(Pattern::Wildcard)
            }
            TokenKind::Ident(_) => {
                let (name, _) = self.expect_ident().ok()?;
                if name == "_" {
                    Some(Pattern::Wildcard)
                } else {
                    Some(Pattern::Bind(name))
                }
            }
            _ => {
                self.error("expected pattern".into());
                None
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn strip_string_quotes(s: &str) -> String {
    let inner = s
        .strip_prefix('"')
        .unwrap_or(s)
        .strip_suffix('"')
        .unwrap_or(s);
    unescape_string(inner)
}

fn unescape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(c) => {
                    out.push('\\');
                    out.push(c);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lambda_chain() {
        let src = "fn f(a: Number, b: Number) := thunk lambda a. lambda b. produce a";
        let file = parse(src).unwrap();
        match &file.items[0] {
            Item::Fn(f) => {
                assert_eq!(f.name, "f");
                match &f.value {
                    Expr::Thunk { expr, .. } => match expr.as_ref() {
                        Expr::Lambda { param, body, .. } => {
                            assert_eq!(param, "a");
                            match body.as_ref() {
                                Expr::Lambda { param: p2, .. } => assert_eq!(p2, "b"),
                                other => panic!("expected Lambda, got: {other:?}"),
                            }
                        }
                        other => panic!("expected Lambda, got: {other:?}"),
                    },
                    other => panic!("expected Thunk, got: {other:?}"),
                }
            }
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn parse_apply_chain() {
        // (force f)(a)(b)
        let src = "fn g() := (force f)(a)(b)";
        let file = parse(src).unwrap();
        match &file.items[0] {
            Item::Fn(f) => match &f.value {
                Expr::Apply { callee, arg, .. } => {
                    assert!(matches!(arg.as_ref(), Expr::Ident { name, .. } if name == "b"));
                    match callee.as_ref() {
                        Expr::Apply {
                            callee: inner_callee,
                            arg: inner_arg,
                            ..
                        } => {
                            assert!(
                                matches!(inner_arg.as_ref(), Expr::Ident { name, .. } if name == "a")
                            );
                            assert!(matches!(inner_callee.as_ref(), Expr::Force { .. }));
                        }
                        other => panic!("expected Apply, got: {other:?}"),
                    }
                }
                other => panic!("expected Apply, got: {other:?}"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn parse_ctor() {
        let src = "fn g() := roll (ctor Bool.true)";
        let file = parse(src).unwrap();
        match &file.items[0] {
            Item::Fn(f) => match &f.value {
                Expr::Roll { expr, .. } => match expr.as_ref() {
                    Expr::Ctor {
                        name,
                        called,
                        args,
                        ..
                    } => {
                        assert_eq!(name, "Bool.true");
                        assert!(!called);
                        assert!(args.is_empty());
                    }
                    other => panic!("expected Ctor, got: {other:?}"),
                },
                other => panic!("expected Roll, got: {other:?}"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn parse_ctor_with_args() {
        let src = "fn g() := roll (ctor List.cons(x, xs))";
        let file = parse(src).unwrap();
        match &file.items[0] {
            Item::Fn(f) => match &f.value {
                Expr::Roll { expr, .. } => match expr.as_ref() {
                    Expr::Ctor {
                        name,
                        called,
                        args,
                        ..
                    } => {
                        assert_eq!(name, "List.cons");
                        assert!(called);
                        assert_eq!(args.len(), 2);
                    }
                    other => panic!("expected Ctor, got: {other:?}"),
                },
                other => panic!("expected Roll, got: {other:?}"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn parse_unroll_match() {
        let src = "fn g(b: Bool) := match unroll b { .true => produce 1; .false => produce 0; }";
        let file = parse(src).unwrap();
        match &file.items[0] {
            Item::Fn(f) => match &f.value {
                Expr::Match {
                    scrutinee, arms, ..
                } => {
                    assert!(matches!(scrutinee.as_ref(), Expr::Unroll { .. }));
                    assert_eq!(arms.len(), 2);
                }
                other => panic!("expected Match, got: {other:?}"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn round_trip_basic() {
        let src = "fn f(x: Bool) := thunk lambda x. produce x";
        let file = parse(src).unwrap();
        let printed = crate::print::print_file(&file);
        let reparsed = parse(&printed).unwrap();
        let reprinted = crate::print::print_file(&reparsed);
        assert_eq!(printed, reprinted);
    }

    #[test]
    fn round_trip_ctor() {
        let src = "data Bool { .true, .false }\n\nfn g() := roll (ctor Bool.true)";
        let file = parse(src).unwrap();
        let printed = crate::print::print_file(&file);
        let reparsed = parse(&printed).unwrap();
        let reprinted = crate::print::print_file(&reparsed);
        assert_eq!(printed, reprinted);
    }
}
