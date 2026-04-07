/// Parser for `.langue` grammar files.
///
/// Grammar of the `.langue` format:
/// ```text
/// grammar     = (token_def | rule)*
/// token_def   = '@token' IDENT+
/// rule        = IDENT '=' rule_body
/// rule_body   = alternatives | sequence
/// alternatives = '|' element ('|' element)*
/// sequence    = element+
/// element     = (IDENT ':')? atom postfix*
/// atom        = QUOTED_STRING | IDENT | '(' sequence ')'
/// postfix     = '?' | '*'
/// ```
use crate::grammar::{Alternative, Element, Grammar, NodeRef, Rule, RuleBody, TokenRef};

#[derive(Debug)]
pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "at offset {}: {}", self.offset, self.message)
    }
}

pub fn parse(input: &str) -> Result<Grammar, ParseError> {
    let mut p = Parser {
        input,
        pos: 0,
        token_defs: Vec::new(),
    };
    p.skip_ws();
    let mut rules = Vec::new();
    while !p.eof() {
        if p.at_str("//") {
            p.skip_line_comment();
            continue;
        }
        if p.at_str("@token") {
            p.parse_token_def()?;
            continue;
        }
        rules.push(p.parse_rule()?);
    }
    Ok(Grammar {
        token_defs: p.token_defs,
        rules,
    })
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
    token_defs: Vec<String>,
}

impl<'a> Parser<'a> {
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn at_str(&self, s: &str) -> bool {
        self.remaining().starts_with(s)
    }

    fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.advance(c.len_utf8());
            } else if self.at_str("//") {
                self.skip_line_comment();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(c) = self.peek() {
            self.advance(c.len_utf8());
            if c == '\n' {
                break;
            }
        }
    }

    fn err(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            offset: self.pos,
            message: message.into(),
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), ParseError> {
        match self.peek() {
            Some(c) if c == expected => {
                self.advance(c.len_utf8());
                Ok(())
            }
            Some(c) => Err(self.err(format!("expected '{expected}', got '{c}'"))),
            None => Err(self.err(format!("expected '{expected}', got EOF"))),
        }
    }

    /// Parse an identifier: `[A-Za-z_][A-Za-z0-9_]*`
    fn parse_ident(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        match self.peek() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                self.advance(c.len_utf8());
            }
            _ => return Err(self.err("expected identifier")),
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance(c.len_utf8());
            } else {
                break;
            }
        }
        Ok(self.input[start..self.pos].to_owned())
    }

    /// Parse a quoted string: `'...'`
    fn parse_quoted(&mut self) -> Result<String, ParseError> {
        self.expect_char('\'')?;
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == '\'' {
                let text = self.input[start..self.pos].to_owned();
                self.advance(1); // closing '
                return Ok(text);
            }
            self.advance(c.len_utf8());
        }
        Err(self.err("unterminated quoted string"))
    }

    // --- Top-level ---

    fn parse_token_def(&mut self) -> Result<(), ParseError> {
        // consume "@token"
        self.advance(6);
        self.skip_ws();
        if self.eof() || !self.peek().map_or(false, |c| c.is_ascii_alphabetic()) {
            return Err(self.err("expected token name after @token"));
        }
        while !self.eof() {
            // Stop if we hit a directive or rule start
            if self.at_str("@") || self.at_str("//") {
                break;
            }
            if self.peek_is_rule_start() {
                break;
            }
            if let Some(c) = self.peek() {
                if c.is_ascii_alphabetic() || c == '_' {
                    let name = self.parse_ident()?;
                    self.token_defs.push(name);
                    self.skip_ws();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn parse_rule(&mut self) -> Result<Rule, ParseError> {
        let name = self.parse_ident()?;
        self.skip_ws();
        self.expect_char('=')?;
        self.skip_ws();
        let body = self.parse_rule_body()?;
        Ok(Rule { name, body })
    }

    fn parse_rule_body(&mut self) -> Result<RuleBody, ParseError> {
        // If starts with '|', it's alternatives
        if self.peek() == Some('|') {
            // Check it's not '||'
            let rest = self.remaining();
            if rest.len() > 1 && rest.as_bytes()[1] == b'|' {
                // '||' is a symbol token, not an alternative separator
                return self.parse_sequence_body();
            }
            return self.parse_alternatives_body();
        }

        // Peek ahead: parse as sequence, then check if we have top-level '|'
        // (This handles `A | B` without leading `|`)
        self.parse_sequence_body()
    }

    fn parse_alternatives_body(&mut self) -> Result<RuleBody, ParseError> {
        let mut alts = Vec::new();
        while self.peek() == Some('|') {
            // Check it's not '||'
            let rest = self.remaining();
            if rest.len() > 1 && rest.as_bytes()[1] == b'|' {
                break;
            }
            self.advance(1); // consume '|'
            self.skip_ws();

            // Each alternative is either a quoted string (for token-only alts like BinaryOp)
            // or a single identifier (node name)
            if self.peek() == Some('\'') {
                let text = self.parse_quoted()?;
                let _token_ref = TokenRef::from_literal(&text);
                alts.push(Alternative {
                    name: format!("'{text}'"),
                });
                // Store as a rule with a single token element? No — alternatives
                // are references to other node names. For token-only alts (like BinaryOp),
                // we store the literal as the alt name and handle in codegen.
                self.skip_ws();
                continue;
            }

            let alt_name = self.parse_ident()?;
            alts.push(Alternative { name: alt_name });
            self.skip_ws();
        }
        Ok(RuleBody::Alternatives(alts))
    }

    fn parse_sequence_body(&mut self) -> Result<RuleBody, ParseError> {
        let elements = self.parse_elements()?;
        Ok(RuleBody::Sequence(elements))
    }

    fn parse_elements(&mut self) -> Result<Vec<Element>, ParseError> {
        let mut elements = Vec::new();
        while !self.eof() {
            // Stop at rule boundary: newline followed by an identifier + '='
            // or at closing paren, or at '|'
            match self.peek() {
                None => break,
                Some(')') => break,
                Some('|') => {
                    // Check for '||' (a symbol literal inside quotes wouldn't reach here)
                    let rest = self.remaining();
                    if rest.len() > 1 && rest.as_bytes()[1] == b'|' {
                        // Not an alt separator — but also not valid in raw position.
                        // This shouldn't happen in well-formed input. Break.
                        break;
                    }
                    break;
                }
                _ => {}
            }

            // Check if this looks like the start of a new rule (Ident =)
            if self.peek_is_rule_start() {
                break;
            }

            // Check for @token directive
            if self.at_str("@") {
                break;
            }

            let elem = self.parse_element()?;
            elements.push(elem);
            self.skip_ws();
        }
        Ok(elements)
    }

    fn peek_is_rule_start(&self) -> bool {
        // Check if remaining text looks like `Ident =` (but not `Ident '==' ...`)
        let rem = self.remaining();
        let mut chars = rem.chars();
        // Must start with uppercase letter (convention: rule names are PascalCase)
        match chars.next() {
            Some(c) if c.is_ascii_uppercase() => {}
            _ => return false,
        }
        // Consume rest of ident
        let mut ident_end = c_len(rem.as_bytes()[0]);
        for c in chars {
            if c.is_ascii_alphanumeric() || c == '_' {
                ident_end += c.len_utf8();
            } else {
                break;
            }
        }
        // Skip whitespace
        let after_ident = &rem[ident_end..];
        let trimmed = after_ident.trim_start();
        // Check for '=' but not '==' or '=>'
        if trimmed.starts_with('=') && !trimmed.starts_with("==") && !trimmed.starts_with("=>") {
            return true;
        }
        false
    }

    fn parse_element(&mut self) -> Result<Element, ParseError> {
        // Check for labeled element: `name:atom`
        // We need lookahead: if we see `Ident ':'` (not `':='`), it's a label.
        if self.peek_is_label() {
            let label = self.parse_ident()?;
            self.expect_char(':')?;
            self.skip_ws();
            let inner = self.parse_atom()?;
            let inner = self.parse_postfix(inner)?;
            return Ok(Element::Labeled(label, Box::new(inner)));
        }

        let atom = self.parse_atom()?;
        self.parse_postfix(atom)
    }

    fn peek_is_label(&self) -> bool {
        let rem = self.remaining();
        // Must start with lowercase (labels are lowercase, rules/tokens are PascalCase)
        match rem.chars().next() {
            Some(c) if c.is_ascii_lowercase() || c == '_' => {}
            _ => return false,
        }
        // Find end of ident
        let mut end = 0;
        for c in rem.chars() {
            if c.is_ascii_alphanumeric() || c == '_' {
                end += c.len_utf8();
            } else {
                break;
            }
        }
        // Check for ':' but not ':='
        let after = &rem[end..];
        let trimmed = after.trim_start();
        trimmed.starts_with(':') && !trimmed.starts_with(":=")
    }

    fn parse_atom(&mut self) -> Result<Element, ParseError> {
        match self.peek() {
            Some('\'') => {
                let text = self.parse_quoted()?;
                let token_ref = TokenRef::from_literal(&text);
                Ok(Element::Token(token_ref))
            }
            Some('(') => {
                self.advance(1); // '('
                self.skip_ws();
                let elements = self.parse_elements()?;
                self.skip_ws();
                self.expect_char(')')?;
                Ok(Element::Group(elements))
            }
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                let name = self.parse_ident()?;
                // Resolve: @token name → token, otherwise → node
                if self.token_defs.iter().any(|t| t == &name) {
                    Ok(Element::Token(TokenRef::Named(name)))
                } else {
                    Ok(Element::Node(NodeRef { name }))
                }
            }
            Some(c) => Err(self.err(format!("unexpected character '{c}'"))),
            None => Err(self.err("unexpected end of input")),
        }
    }

    fn parse_postfix(&mut self, mut elem: Element) -> Result<Element, ParseError> {
        loop {
            match self.peek() {
                Some('?') => {
                    self.advance(1);
                    elem = Element::Optional(Box::new(elem));
                }
                Some('*') => {
                    self.advance(1);
                    elem = Element::Repeated(Box::new(elem));
                }
                _ => break,
            }
        }
        Ok(elem)
    }
}

fn c_len(byte: u8) -> usize {
    if byte < 0x80 {
        1
    } else if byte < 0xE0 {
        2
    } else if byte < 0xF0 {
        3
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_token_def() {
        let g = parse("@token Ident StringLit NumberLit").unwrap();
        assert_eq!(g.token_defs, vec!["Ident", "StringLit", "NumberLit"]);
        assert!(g.rules.is_empty());
    }

    #[test]
    fn parse_simple_rule() {
        let g = parse("@token Ident\nFile = Item*").unwrap();
        assert_eq!(g.rules.len(), 1);
        assert_eq!(g.rules[0].name, "File");
        match &g.rules[0].body {
            RuleBody::Sequence(elems) => {
                assert_eq!(elems.len(), 1);
                match &elems[0] {
                    Element::Repeated(inner) => match inner.as_ref() {
                        Element::Node(n) => assert_eq!(n.name, "Item"),
                        other => panic!("expected Node, got {other:?}"),
                    },
                    other => panic!("expected Repeated, got {other:?}"),
                }
            }
            other => panic!("expected Sequence, got {other:?}"),
        }
    }

    #[test]
    fn parse_alternatives() {
        let input = r#"
Item =
  | DataDecl
  | FnDecl
  | UseDecl
"#;
        let g = parse(input).unwrap();
        assert_eq!(g.rules.len(), 1);
        match &g.rules[0].body {
            RuleBody::Alternatives(alts) => {
                assert_eq!(alts.len(), 3);
                assert_eq!(alts[0].name, "DataDecl");
                assert_eq!(alts[1].name, "FnDecl");
                assert_eq!(alts[2].name, "UseDecl");
            }
            other => panic!("expected Alternatives, got {other:?}"),
        }
    }

    #[test]
    fn parse_labeled_and_keywords() {
        let input = "@token Ident\nDataDecl = 'data' name:Ident '{' '}'";
        let g = parse(input).unwrap();
        assert_eq!(g.rules.len(), 1);
        match &g.rules[0].body {
            RuleBody::Sequence(elems) => {
                assert_eq!(elems.len(), 4);
                // 'data'
                match &elems[0] {
                    Element::Token(TokenRef::Keyword(k)) => assert_eq!(k, "data"),
                    other => panic!("expected Keyword, got {other:?}"),
                }
                // name:Ident
                match &elems[1] {
                    Element::Labeled(label, inner) => {
                        assert_eq!(label, "name");
                        match inner.as_ref() {
                            Element::Token(TokenRef::Named(n)) => assert_eq!(n, "Ident"),
                            other => panic!("expected Named token, got {other:?}"),
                        }
                    }
                    other => panic!("expected Labeled, got {other:?}"),
                }
                // '{'
                match &elems[2] {
                    Element::Token(TokenRef::Symbol(s)) => assert_eq!(s, "{"),
                    other => panic!("expected Symbol, got {other:?}"),
                }
            }
            other => panic!("expected Sequence, got {other:?}"),
        }
    }

    #[test]
    fn parse_optional_and_repeated() {
        let input = "@token Ident\nFoo = GenericParams? Variant*";
        let g = parse(input).unwrap();
        match &g.rules[0].body {
            RuleBody::Sequence(elems) => {
                assert_eq!(elems.len(), 2);
                match &elems[0] {
                    Element::Optional(inner) => match inner.as_ref() {
                        Element::Node(n) => assert_eq!(n.name, "GenericParams"),
                        other => panic!("expected Node, got {other:?}"),
                    },
                    other => panic!("expected Optional, got {other:?}"),
                }
                match &elems[1] {
                    Element::Repeated(inner) => match inner.as_ref() {
                        Element::Node(n) => assert_eq!(n.name, "Variant"),
                        other => panic!("expected Node, got {other:?}"),
                    },
                    other => panic!("expected Repeated, got {other:?}"),
                }
            }
            other => panic!("expected Sequence, got {other:?}"),
        }
    }

    #[test]
    fn parse_group() {
        let input = "@token Ident\nFoo = ('.' name:Ident)?";
        let g = parse(input).unwrap();
        match &g.rules[0].body {
            RuleBody::Sequence(elems) => {
                assert_eq!(elems.len(), 1);
                match &elems[0] {
                    Element::Optional(inner) => match inner.as_ref() {
                        Element::Group(items) => {
                            assert_eq!(items.len(), 2);
                        }
                        other => panic!("expected Group, got {other:?}"),
                    },
                    other => panic!("expected Optional, got {other:?}"),
                }
            }
            other => panic!("expected Sequence, got {other:?}"),
        }
    }

    #[test]
    fn parse_multiple_rules() {
        let input = r#"
@token Ident StringLit

File = Item*

Item =
  | DataDecl
  | FnDecl

DataDecl = 'data' name:Ident '{' variants:Variant* '}'

Variant = '.' name:Ident
"#;
        let g = parse(input).unwrap();
        assert_eq!(g.token_defs, vec!["Ident", "StringLit"]);
        assert_eq!(g.rules.len(), 4);
        assert_eq!(g.rules[0].name, "File");
        assert_eq!(g.rules[1].name, "Item");
        assert_eq!(g.rules[2].name, "DataDecl");
        assert_eq!(g.rules[3].name, "Variant");
    }

    #[test]
    fn parse_comments() {
        let input = r#"
// This is a comment
@token Ident
// Another comment
File = Item*
"#;
        let g = parse(input).unwrap();
        assert_eq!(g.rules.len(), 1);
    }

    #[test]
    fn parse_token_only_alternatives() {
        let input = r#"
BinaryOp =
  | '+'
  | '-'
  | '*'
"#;
        let g = parse(input).unwrap();
        match &g.rules[0].body {
            RuleBody::Alternatives(alts) => {
                assert_eq!(alts.len(), 3);
                assert_eq!(alts[0].name, "'+'");
                assert_eq!(alts[1].name, "'-'");
                assert_eq!(alts[2].name, "'*'");
            }
            other => panic!("expected Alternatives, got {other:?}"),
        }
    }
}
