use crate::{
    BundleEntry, CapDecl, DataDecl, ExternFnDecl, ExternTypeDecl, Expr, File, FnDecl,
    ImplDecl, ImplMethodDecl, Item, MatchArm, OperationDecl, Param, UseDecl, VariantDecl,
};
use lumo_lexer::{Keyword, Symbol, Token, TokenKind};
use lumo_span::Span;
use lumo_types::{CapRef, Pattern, Spanned, TypeExpr};

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
    };
    let items = parser.parse_file();
    if parser.errors.is_empty() {
        let content_hash = crate::hash_file(&items);
        Ok(File {
            items,
            content_hash,
            errors: vec![],
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
                // Skip a token to avoid infinite loop
                if !self.at_end() {
                    self.advance();
                }
            }
        }
        items
    }

    // -----------------------------------------------------------------------
    // Items
    // -----------------------------------------------------------------------

    fn parse_item(&mut self) -> Option<Item> {
        // Check for #[inline(always)]
        let inline = self.try_parse_inline_hint();

        match self.peek()? {
            TokenKind::Keyword(Keyword::Extern) => self.parse_extern_item(inline),
            TokenKind::Keyword(Keyword::Data) => {
                Some(Item::Data(self.parse_data_decl()?))
            }
            TokenKind::Keyword(Keyword::Cap) => {
                Some(Item::Cap(self.parse_cap_decl()?))
            }
            TokenKind::Keyword(Keyword::Fn) => {
                Some(Item::Fn(self.parse_fn_decl()?))
            }
            TokenKind::Keyword(Keyword::Use) => {
                Some(Item::Use(self.parse_use_decl()?))
            }
            TokenKind::Keyword(Keyword::Impl) => {
                Some(Item::Impl(self.parse_impl_decl()?))
            }
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
        self.advance(); // #
        if !self.eat_sym(Symbol::LBracket) {
            self.pos = save;
            return false;
        }
        if !self.eat_ident("inline") {
            self.pos = save;
            return false;
        }
        if !self.eat_sym(Symbol::LParen) {
            self.pos = save;
            return false;
        }
        if !self.eat_ident("always") {
            self.pos = save;
            return false;
        }
        if !self.eat_sym(Symbol::RParen) {
            self.pos = save;
            return false;
        }
        if !self.eat_sym(Symbol::RBracket) {
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
                link_module: None,
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
            _ => {
                self.error("expected string literal after `as`".into());
                None
            }
        }
    }

    fn parse_data_decl(&mut self) -> Option<DataDecl> {
        let start = self.expect_kw(Keyword::Data).ok()?;
        let (name, _) = self.expect_ident().ok()?;
        let generics: Vec<String> = self.try_parse_generic_params()
            .into_iter().map(|g| g.name().to_owned()).collect();
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
        let end_span = if payload.is_empty() {
            name_span
        } else {
            self.peek_span()
        };
        Some(VariantDecl {
            name,
            payload,
            as_raw: None,
            span: Span::new(start.start, end_span.end),
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
        let body = self.parse_expr()?;
        let end = body.span();
        Some(FnDecl {
            name,
            generics,
            params,
            return_type,
            cap,
            body,
            inline: false,
            span: Span::new(start.start, end.end),
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
                // use a.b.{c, d}
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

        // Try to detect `Name =` pattern for named impl
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
        // Look for `Name =` (ident followed by `=`, not `:=` or `=>`)
        if self.pos + 1 >= self.tokens.len() {
            return None;
        }
        let is_ident = matches!(self.peek(), Some(TokenKind::Ident(_)));
        let followed_by_eq =
            matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind), Some(TokenKind::Symbol(Symbol::Equals)));
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
        let body = self.parse_expr()?;
        let end = body.span();
        Some(ImplMethodDecl {
            name,
            params,
            return_type,
            body,
            span: Span::new(start.start, end.end),
        })
    }

    // -----------------------------------------------------------------------
    // Shared
    // -----------------------------------------------------------------------

    fn try_parse_generic_params(&mut self) -> Vec<crate::GenericParam> {
        use crate::GenericParam;
        if !self.eat_sym(Symbol::LBracket) {
            return Vec::new();
        }
        let mut params = Vec::new();
        loop {
            let is_cap_row = self.peek() == Some(&TokenKind::Keyword(Keyword::Cap));
            if is_cap_row {
                self.advance();
            }
            if let Some(TokenKind::Ident(_)) = self.peek() {
                let (name, _) = self.expect_ident().ok().unwrap();
                params.push(if is_cap_row {
                    GenericParam::CapRow(name)
                } else {
                    GenericParam::Type(name)
                });
                self.eat_sym(Symbol::Comma);
            } else {
                break;
            }
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
        // Don't consume `:=`
        if matches!(self.tokens.get(self.pos), Some(t) if t.kind == TokenKind::Symbol(Symbol::ColonEquals)) {
            return None;
        }
        self.advance(); // consume `:`
        self.parse_type_expr()
    }

    fn try_parse_cap_annotation(&mut self) -> Option<CapRef> {
        use lumo_types::CapEntry;
        if !self.eat_sym(Symbol::Slash) {
            return None;
        }
        self.expect_sym(Symbol::LBrace).ok()?;
        if self.eat_sym(Symbol::RBrace) {
            return Some(vec![]);
        }
        let mut entries: Vec<CapEntry> = Vec::new();
        loop {
            if self.eat_sym(Symbol::DotDot) {
                // `..name` → Spread; bare `..` → Infer
                if matches!(self.peek(), Some(TokenKind::Ident(_))) {
                    let (var, _) = self.expect_ident().ok()?;
                    entries.push(CapEntry::Spread(var));
                } else {
                    entries.push(CapEntry::Infer);
                }
            } else {
                let (name, _) = self.expect_ident().ok()?;
                let type_args = if self.peek_ident_text("for") {
                    self.advance(); // consume "for"
                    let (ty, _) = self.expect_ident().ok()?;
                    vec![TypeExpr::Named(ty)]
                } else {
                    vec![]
                };
                entries.push(CapEntry::Cap(TypeExpr::Cap { name, type_args }));
            }
            if !self.eat_sym(Symbol::Comma) {
                break;
            }
            if self.peek() == Some(&TokenKind::Symbol(Symbol::RBrace)) {
                break; // trailing comma
            }
        }
        self.expect_sym(Symbol::RBrace).ok()?;
        Some(entries)
    }

    fn parse_type_expr(&mut self) -> Option<Spanned<TypeExpr>> {
        // `fn(T, U): R` or `fn(T): R / { IO }` — function type in value position
        if self.peek() == Some(&TokenKind::Keyword(Keyword::Fn)) {
            let start = self.advance().span; // consume `fn`
            self.expect_sym(Symbol::LParen).ok()?;
            let mut params = Vec::new();
            while self.peek() != Some(&TokenKind::Symbol(Symbol::RParen)) && !self.at_end() {
                if let Some(ty) = self.parse_type_expr() {
                    params.push(ty.value);
                }
                self.eat_sym(Symbol::Comma);
            }
            self.expect_sym(Symbol::RParen).ok()?;
            self.expect_sym(Symbol::Colon).ok()?;
            let ret = self.parse_type_expr()?;
            let cap = self.try_parse_cap_annotation().unwrap_or_default();
            return Some(Spanned {
                span: Span::new(start.start, ret.span.end),
                value: TypeExpr::Fn { params, ret: Box::new(ret.value), cap },
            });
        }
        // Handle `produce`/`thunk` prefixes
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
    // Expressions
    // -----------------------------------------------------------------------

    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_expr_inner()
    }

    fn parse_expr_inner(&mut self) -> Option<Expr> {
        let (expr, allow_postfix) = self.parse_expr_primary_flagged()?;
        self.parse_postfix(expr, allow_postfix)
    }

    fn parse_postfix(&mut self, expr: Expr, allow_postfix: bool) -> Option<Expr> {
        if !allow_postfix {
            return Some(expr);
        }
        let mut expr = expr;

        // Postfix: member access (.) and call (())
        loop {
            if self.eat_sym(Symbol::Dot) {
                let (member, member_span) = self.expect_ident().ok()?;
                let span = Span::new(expr.span().start, member_span.end);
                expr = Expr::Member {
                    object: Box::new(expr),
                    member,
                    span,
                };
            } else if self.peek() == Some(&TokenKind::Symbol(Symbol::LParen)) {
                let args = self.parse_call_args()?;
                let end = self.peek_span();
                let span = Span::new(expr.span().start, end.start);
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                    span,
                };
            } else {
                break;
            }
        }

        Some(expr)
    }

    fn parse_call_args(&mut self) -> Option<Vec<Expr>> {
        self.expect_sym(Symbol::LParen).ok()?;
        let mut args = Vec::new();
        while self.peek() != Some(&TokenKind::Symbol(Symbol::RParen)) && !self.at_end() {
            if let Some(arg) = self.parse_expr() {
                args.push(arg);
            }
            self.eat_sym(Symbol::Comma);
        }
        self.expect_sym(Symbol::RParen).ok()?;
        Some(args)
    }

    /// Parse a primary expression and return whether postfix is allowed.
    fn parse_expr_primary_flagged(&mut self) -> Option<(Expr, bool)> {
        match self.peek()? {
            TokenKind::Keyword(Keyword::Fn) => self.parse_lambda_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Produce) => self.parse_produce_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Thunk) => self.parse_thunk_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Force) => self.parse_force_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Let) => self.parse_let_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Match) => self.parse_match_expr().map(|e| (e, true)),
            TokenKind::Keyword(Keyword::Perform) => self.parse_perform_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Handle) => self.parse_handle_expr().map(|e| (e, false)),
            TokenKind::Keyword(Keyword::Bundle) => self.parse_bundle_expr().map(|e| (e, true)),
            TokenKind::Ident(_) => {
                let (name, span) = self.expect_ident().ok()?;
                Some((Expr::Ident { name, span }, true))
            }
            TokenKind::StringLit(_) => {
                let tok = self.advance();
                if let TokenKind::StringLit(s) = &tok.kind {
                    Some((Expr::String {
                        value: strip_string_quotes(s),
                        span: tok.span,
                    }, false))
                } else {
                    None
                }
            }
            TokenKind::NumberLit(_) => {
                let tok = self.advance();
                if let TokenKind::NumberLit(s) = &tok.kind {
                    Some((Expr::Number {
                        value: s.clone(),
                        span: tok.span,
                    }, false))
                } else {
                    None
                }
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
        let end = inner.span();
        Some(Expr::Produce {
            expr: Box::new(inner),
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_thunk_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Thunk).ok()?;
        let inner = self.parse_expr()?;
        let end = inner.span();
        Some(Expr::Thunk {
            expr: Box::new(inner),
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_lambda_expr(&mut self) -> Option<Expr> {
        let start = self.advance().span; // consume `fn`
        self.expect_sym(Symbol::LParen).ok()?;
        let mut params = Vec::new();
        while self.peek() != Some(&TokenKind::Symbol(Symbol::RParen)) && !self.at_end() {
            let (name, _) = self.expect_ident().ok()?;
            let ty = if self.eat_sym(Symbol::Colon) {
                self.parse_type_expr()
            } else {
                None
            };
            params.push((name, ty));
            self.eat_sym(Symbol::Comma);
        }
        self.expect_sym(Symbol::RParen).ok()?;
        self.expect_sym(Symbol::LBrace).ok()?;
        let body = self.parse_expr()?;
        let end = self.expect_sym(Symbol::RBrace).ok()?;
        Some(Expr::Lambda {
            params,
            body: Box::new(body),
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_force_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Force).ok()?;
        let (inner, allow_postfix) = self.parse_expr_primary_flagged()?;
        let expr = self.parse_postfix(inner, allow_postfix)?;
        let end_span = expr.span();
        Some(Expr::Force {
            span: Span::new(start.start, end_span.end),
            expr: Box::new(expr),
        })
    }

    fn parse_let_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Let).ok()?;
        let (name, _) = self.expect_ident().ok()?;
        self.expect_sym(Symbol::Equals).ok()?;
        let value = self.parse_expr()?;
        self.expect_kw(Keyword::In).ok()?;
        let body = self.parse_expr()?;
        let end = body.span();
        Some(Expr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
            span: Span::new(start.start, end.end),
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
        let end = self.expect_sym(Symbol::RBrace).ok()?;
        Some(Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_match_arm(&mut self) -> Option<MatchArm> {
        let pattern = self.parse_pattern()?;
        let start = self.peek_span();
        self.expect_sym(Symbol::FatArrow).ok()?;
        let body = self.parse_expr()?;
        let end = body.span();
        // Consume arm-terminating semicolon
        self.eat_sym(Symbol::Semi);
        Some(MatchArm {
            pattern,
            body,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_perform_expr(&mut self) -> Option<Expr> {
        let start = self.expect_kw(Keyword::Perform).ok()?;
        let (cap, cap_span) = self.expect_ident().ok()?;
        Some(Expr::Perform {
            cap,
            span: Span::new(start.start, cap_span.end),
        })
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
            self.error("expected `with` after handle capability".into());
            return None;
        }
        let handler = self.parse_expr()?;
        self.expect_kw(Keyword::In).ok()?;
        let body = self.parse_expr()?;
        let end = body.span();
        Some(Expr::Handle {
            cap,
            type_args,
            handler: Box::new(handler),
            body: Box::new(body),
            span: Span::new(start.start, end.end),
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
        let end = self.expect_sym(Symbol::RBrace).ok()?;
        Some(Expr::Bundle {
            entries,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_bundle_entry(&mut self) -> Option<BundleEntry> {
        let start = self.expect_kw(Keyword::Fn).ok()?;
        let (name, _) = self.expect_ident().ok()?;
        let params = self.parse_param_list()?;
        self.expect_sym(Symbol::ColonEquals).ok()?;
        let body = self.parse_expr()?;
        let end = body.span();
        Some(BundleEntry {
            name,
            params,
            body,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_paren_or_ann(&mut self) -> Option<Expr> {
        let start = self.expect_sym(Symbol::LParen).ok()?;
        let inner = self.parse_expr()?;
        if self.eat_sym(Symbol::Colon) {
            // Annotation: (expr : Type)
            let ty = self.parse_type_expr()?;
            let end = self.expect_sym(Symbol::RParen).ok()?;
            Some(Expr::Ann {
                expr: Box::new(inner),
                ty,
                span: Span::new(start.start, end.end),
            })
        } else {
            // Grouping: (expr)
            self.expect_sym(Symbol::RParen).ok()?;
            Some(inner)
        }
    }

    fn parse_error_expr(&mut self) -> Option<Expr> {
        let start = self.expect_sym(Symbol::Lt).ok()?;
        if !self.eat_ident("error") {
            self.error("expected `error` in `<error>`".into());
            return None;
        }
        let end = self.expect_sym(Symbol::Gt).ok()?;
        Some(Expr::Error {
            span: Span::new(start.start, end.end),
        })
    }

    // -----------------------------------------------------------------------
    // Patterns
    // -----------------------------------------------------------------------

    fn parse_pattern(&mut self) -> Option<Pattern> {
        match self.peek()? {
            TokenKind::Symbol(Symbol::Dot) => {
                self.advance(); // .
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
    fn parse_extern_type() {
        let file = parse("extern type String").unwrap();
        assert_eq!(file.items.len(), 1);
        match &file.items[0] {
            Item::ExternType(ext) => {
                assert_eq!(ext.name, "String");
                assert_eq!(ext.extern_name, None);
            }
            _ => panic!("expected ExternType"),
        }
    }

    #[test]
    fn parse_extern_type_as() {
        let file = parse("extern type Number as \"number\"").unwrap();
        match &file.items[0] {
            Item::ExternType(ext) => {
                assert_eq!(ext.name, "Number");
                assert_eq!(ext.extern_name, Some("number".into()));
            }
            _ => panic!("expected ExternType"),
        }
    }

    #[test]
    fn parse_data_decl() {
        let file = parse("data Bool { .true, .false }").unwrap();
        match &file.items[0] {
            Item::Data(d) => {
                assert_eq!(d.name, "Bool");
                assert_eq!(d.variants.len(), 2);
                assert_eq!(d.variants[0].name, "true");
                assert_eq!(d.variants[1].name, "false");
            }
            _ => panic!("expected Data"),
        }
    }

    #[test]
    fn parse_data_generic() {
        let file = parse("data List[A] { .nil, .cons(A, List[A]) }").unwrap();
        match &file.items[0] {
            Item::Data(d) => {
                assert_eq!(d.name, "List");
                assert_eq!(d.generics, vec!["A"]);
                assert_eq!(d.variants[1].name, "cons");
                assert_eq!(d.variants[1].payload.len(), 2);
            }
            _ => panic!("expected Data"),
        }
    }

    #[test]
    fn parse_fn_simple() {
        let file = parse("fn id(x: Bool): produce Bool := produce x").unwrap();
        match &file.items[0] {
            Item::Fn(f) => {
                assert_eq!(f.name, "id");
                assert_eq!(f.params.len(), 1);
                assert_eq!(f.params[0].name, "x");
                assert!(matches!(&f.body, Expr::Produce { .. }));
            }
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn parse_use_simple() {
        let file = parse("use std.io;").unwrap();
        match &file.items[0] {
            Item::Use(u) => {
                assert_eq!(u.path, vec!["std", "io"]);
                assert_eq!(u.names, None);
            }
            _ => panic!("expected Use"),
        }
    }

    #[test]
    fn parse_use_tree() {
        let file = parse("use std.io.{print, read};").unwrap();
        match &file.items[0] {
            Item::Use(u) => {
                assert_eq!(u.path, vec!["std", "io"]);
                assert_eq!(u.names, Some(vec!["print".into(), "read".into()]));
            }
            _ => panic!("expected Use"),
        }
    }

    #[test]
    fn parse_let_expr() {
        let file = parse("fn f() := let x = 42 in produce x").unwrap();
        match &file.items[0] {
            Item::Fn(f) => match &f.body {
                Expr::Let {
                    name, value, body, ..
                } => {
                    assert_eq!(name, "x");
                    assert!(matches!(value.as_ref(), Expr::Number { value: v, .. } if v == "42"));
                    assert!(matches!(body.as_ref(), Expr::Produce { .. }));
                }
                other => panic!("expected Let, got: {other:?}"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn parse_match_expr() {
        let src = "fn f(b: Bool) := match b { .true => produce 1; .false => produce 0; }";
        let file = parse(src).unwrap();
        match &file.items[0] {
            Item::Fn(f) => match &f.body {
                Expr::Match { arms, .. } => {
                    assert_eq!(arms.len(), 2);
                    assert_eq!(
                        arms[0].pattern,
                        Pattern::Ctor {
                            name: "true".into(),
                            args: vec![]
                        }
                    );
                }
                other => panic!("expected Match, got: {other:?}"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn parse_desugared_op() {
        // Desugared form: perform Add.add(a, b)
        let src = "fn add(a: Number, b: Number) := (perform Add).add(a, b)";
        let file = parse(src).unwrap();
        match &file.items[0] {
            Item::Fn(f) => match &f.body {
                Expr::Call { callee, args, .. } => {
                    assert_eq!(args.len(), 2);
                    match callee.as_ref() {
                        Expr::Member { object, member, .. } => {
                            assert_eq!(member, "add");
                            assert!(matches!(object.as_ref(), Expr::Perform { cap, .. } if cap == "Add"));
                        }
                        other => panic!("expected Member, got: {other:?}"),
                    }
                }
                other => panic!("expected Call, got: {other:?}"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn parse_handle_bundle() {
        let src = r#"fn f() := handle IO with bundle { fn log(msg: String) := produce msg } in produce 42"#;
        let file = parse(src).unwrap();
        match &file.items[0] {
            Item::Fn(f) => match &f.body {
                Expr::Handle {
                    cap, handler, body, ..
                } => {
                    assert_eq!(cap, "IO");
                    assert!(matches!(handler.as_ref(), Expr::Bundle { .. }));
                    assert!(matches!(body.as_ref(), Expr::Produce { .. }));
                }
                other => panic!("expected Handle, got: {other:?}"),
            },
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn parse_error_expr() {
        let src = "fn f() := <error>";
        let file = parse(src).unwrap();
        match &file.items[0] {
            Item::Fn(f) => assert!(matches!(&f.body, Expr::Error { .. })),
            _ => panic!("expected Fn"),
        }
    }

    #[test]
    fn parse_impl_decl() {
        let src = "impl Number: Add { fn add(self: Number, other: Number): produce Number := produce self }";
        let file = parse(src).unwrap();
        match &file.items[0] {
            Item::Impl(i) => {
                assert_eq!(i.name, None);
                assert_eq!(i.target_type.value, TypeExpr::Named("Number".into()));
                assert_eq!(
                    i.capability.as_ref().map(|c| &c.value),
                    Some(&TypeExpr::Named("Add".into()))
                );
                assert_eq!(i.methods.len(), 1);
                assert_eq!(i.methods[0].name, "add");
            }
            _ => panic!("expected Impl"),
        }
    }

    #[test]
    fn round_trip_basic() {
        let src = "fn id(x: Bool): produce Bool := produce x";
        let file = parse(src).unwrap();
        let printed = crate::print::print_file(&file);
        let reparsed = parse(&printed).unwrap();
        let reprinted = crate::print::print_file(&reparsed);
        assert_eq!(printed, reprinted);
    }

    #[test]
    fn round_trip_data() {
        let src = "data List[A] { .nil, .cons(A, List[A]) }";
        let file = parse(src).unwrap();
        let printed = crate::print::print_file(&file);
        let reparsed = parse(&printed).unwrap();
        let reprinted = crate::print::print_file(&reparsed);
        assert_eq!(printed, reprinted);
    }

    #[test]
    fn round_trip_complex() {
        let src = r#"extern type String

extern type Number as "number"

data Bool { .true, .false }

fn not(b: Bool): produce Bool := match b { .true => produce Bool.false; .false => produce Bool.true; }

use std.io;

impl Number: Add { fn add(self: Number, other: Number): produce Number := (perform Add).add(self, other) }"#;
        let file = parse(src).unwrap();
        let printed = crate::print::print_file(&file);
        let reparsed = parse(&printed).unwrap();
        let reprinted = crate::print::print_file(&reparsed);
        assert_eq!(printed, reprinted);
    }
}
