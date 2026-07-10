use crate::diag::Diagnostic;
use langue_rt::Span;

/// One `.langue` token. Whitespace and `//` comments are skipped — the
/// `.langue` format itself is not lossless.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TokenKind {
    /// Possibly-dotted name: `FnDecl`, `keyword.fn`, `check_V`.
    Name(String),
    /// `'fn'` — unescaped contents.
    Str(String),
    /// `/[0-9]+/` — raw pattern between the slashes (`\/` unescaped).
    Regex(String),
    /// Binding power: `100`.
    Num(u16),
    /// One of `= | ? * + ( ) { } [ ] , : @`.
    Punct(char),
    /// `==>` `===` `::` `:=` — multi-char symbols (elab files).
    Sym(&'static str),
    /// `$x` — a metavariable (elab files).
    Var(String),
    Eof,
}

impl TokenKind {
    pub fn describe(&self) -> String {
        match self {
            TokenKind::Name(n) => format!("name `{n}`"),
            TokenKind::Str(s) => format!("literal `'{s}'`"),
            TokenKind::Regex(_) => "regex".to_owned(),
            TokenKind::Num(n) => format!("number `{n}`"),
            TokenKind::Punct(c) => format!("`{c}`"),
            TokenKind::Sym(s) => format!("`{s}`"),
            TokenKind::Var(v) => format!("metavariable `${v}`"),
            TokenKind::Eof => "end of file".to_owned(),
        }
    }
}

/// Lex a whole `.langue` file. Always returns an EOF-terminated token list;
/// unlexable bytes are reported and skipped.
pub fn lex(file: &str, text: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut diags = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let start = i;
        let b = bytes[i];
        match b {
            b' ' | b'\t' | b'\r' | b'\n' => {
                i += 1;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'=' if bytes[i..].starts_with(b"==>") || bytes[i..].starts_with(b"===") => {
                let sym = if bytes[i..].starts_with(b"==>") { "==>" } else { "===" };
                i += 3;
                tokens.push(Token {
                    kind: TokenKind::Sym(sym),
                    span: Span::new(start as u32, i as u32),
                });
            }
            b':' if matches!(bytes.get(i + 1), Some(b':' | b'=')) => {
                let sym = if bytes[i + 1] == b':' { "::" } else { ":=" };
                i += 2;
                tokens.push(Token {
                    kind: TokenKind::Sym(sym),
                    span: Span::new(start as u32, i as u32),
                });
            }
            b'$' => {
                i += 1;
                if i < bytes.len() && is_name_start(bytes[i]) {
                    let name_start = i;
                    while i < bytes.len() && is_name_continue(bytes[i]) {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::Var(text[name_start..i].to_owned()),
                        span: Span::new(start as u32, i as u32),
                    });
                } else {
                    diags.push(Diagnostic::error(
                        file,
                        Span::new(start as u32, i as u32),
                        "expected a name after `$`",
                    ));
                }
            }
            b'=' | b'|' | b'?' | b'*' | b'+' | b'(' | b')' | b'{' | b'}' | b'[' | b']'
            | b',' | b':' | b'@' => {
                i += 1;
                tokens.push(Token {
                    kind: TokenKind::Punct(b as char),
                    span: Span::new(start as u32, i as u32),
                });
            }
            b'\'' => {
                i += 1;
                let mut value = String::new();
                let mut closed = false;
                while i < bytes.len() {
                    match bytes[i] {
                        b'\\' if i + 1 < bytes.len() => {
                            value.push(bytes[i + 1] as char);
                            i += 2;
                        }
                        b'\'' => {
                            i += 1;
                            closed = true;
                            break;
                        }
                        b'\n' => break,
                        _ => {
                            // Copy the full UTF-8 character, not just one byte.
                            let ch_len = utf8_len(bytes[i]);
                            value.push_str(&text[i..i + ch_len]);
                            i += ch_len;
                        }
                    }
                }
                if !closed {
                    diags.push(Diagnostic::error(
                        file,
                        Span::new(start as u32, i as u32),
                        "unterminated string literal",
                    ));
                }
                tokens.push(Token {
                    kind: TokenKind::Str(value),
                    span: Span::new(start as u32, i as u32),
                });
            }
            b'/' => {
                i += 1;
                let mut value = String::new();
                let mut closed = false;
                while i < bytes.len() {
                    match bytes[i] {
                        b'\\' if bytes.get(i + 1) == Some(&b'/') => {
                            value.push('/');
                            i += 2;
                        }
                        b'\\' if i + 1 < bytes.len() => {
                            value.push('\\');
                            value.push(bytes[i + 1] as char);
                            i += 2;
                        }
                        b'/' => {
                            i += 1;
                            closed = true;
                            break;
                        }
                        b'\n' => break,
                        _ => {
                            let ch_len = utf8_len(bytes[i]);
                            value.push_str(&text[i..i + ch_len]);
                            i += ch_len;
                        }
                    }
                }
                if !closed {
                    diags.push(Diagnostic::error(
                        file,
                        Span::new(start as u32, i as u32),
                        "unterminated regex literal",
                    ));
                }
                tokens.push(Token {
                    kind: TokenKind::Regex(value),
                    span: Span::new(start as u32, i as u32),
                });
            }
            b'0'..=b'9' => {
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                let digits = &text[start..i];
                let value = digits.parse::<u16>().unwrap_or_else(|_| {
                    diags.push(Diagnostic::error(
                        file,
                        Span::new(start as u32, i as u32),
                        format!("number `{digits}` does not fit a binding power (max 65535)"),
                    ));
                    0
                });
                tokens.push(Token {
                    kind: TokenKind::Num(value),
                    span: Span::new(start as u32, i as u32),
                });
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                i += 1;
                loop {
                    while i < bytes.len() && is_name_continue(bytes[i]) {
                        i += 1;
                    }
                    // A dot continues the name only when a name char follows:
                    // `keyword.fn` is one name, `Param.` is a name then `.`.
                    if i + 1 < bytes.len()
                        && bytes[i] == b'.'
                        && is_name_start(bytes[i + 1])
                    {
                        i += 2;
                    } else {
                        break;
                    }
                }
                tokens.push(Token {
                    kind: TokenKind::Name(text[start..i].to_owned()),
                    span: Span::new(start as u32, i as u32),
                });
            }
            _ => {
                let ch_len = utf8_len(b);
                i += ch_len;
                diags.push(Diagnostic::error(
                    file,
                    Span::new(start as u32, i as u32),
                    format!("unexpected character `{}`", &text[start..i]),
                ));
            }
        }
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(text.len() as u32, text.len() as u32),
    });
    (tokens, diags)
}

fn is_name_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_name_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn utf8_len(first_byte: u8) -> usize {
    match first_byte {
        0x00..=0x7f => 1,
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(text: &str) -> Vec<TokenKind> {
        let (tokens, diags) = lex("t.langue", text);
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        tokens.into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn lexes_token_decl() {
        assert_eq!(
            kinds("token keyword.fn = 'fn'"),
            vec![
                TokenKind::Name("token".into()),
                TokenKind::Name("keyword.fn".into()),
                TokenKind::Punct('='),
                TokenKind::Str("fn".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_regex_with_escaped_slash() {
        assert_eq!(
            kinds(r"token comment = /a\/b[0-9]+/"),
            vec![
                TokenKind::Name("token".into()),
                TokenKind::Name("comment".into()),
                TokenKind::Punct('='),
                TokenKind::Regex("a/b[0-9]+".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn regex_keeps_other_escapes_raw() {
        assert_eq!(
            kinds(r"/\d+\.\d+/"),
            vec![TokenKind::Regex(r"\d+\.\d+".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn line_comment_is_skipped() {
        assert_eq!(
            kinds("a // hidden = 'x'\nb"),
            vec![
                TokenKind::Name("a".into()),
                TokenKind::Name("b".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn dot_does_not_join_without_following_name() {
        let (tokens, diags) = lex("t", "Param.");
        assert_eq!(tokens[0].kind, TokenKind::Name("Param".into()));
        // The bare `.` is not a valid token.
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn praat_row_tokens() {
        assert_eq!(
            kinds("@89 '**' @90,"),
            vec![
                TokenKind::Punct('@'),
                TokenKind::Num(89),
                TokenKind::Str("**".into()),
                TokenKind::Punct('@'),
                TokenKind::Num(90),
                TokenKind::Punct(','),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn elab_symbols_and_metavars() {
        assert_eq!(
            kinds("$e[$b := $a] ==> Lumo::FnDecl === x:"),
            vec![
                TokenKind::Var("e".into()),
                TokenKind::Punct('['),
                TokenKind::Var("b".into()),
                TokenKind::Sym(":="),
                TokenKind::Var("a".into()),
                TokenKind::Punct(']'),
                TokenKind::Sym("==>"),
                TokenKind::Name("Lumo".into()),
                TokenKind::Sym("::"),
                TokenKind::Name("FnDecl".into()),
                TokenKind::Sym("==="),
                TokenKind::Name("x".into()),
                TokenKind::Punct(':'),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn bare_dollar_is_reported() {
        let (_, diags) = lex("t", "$ x");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("expected a name after `$`"));
    }

    #[test]
    fn spans_are_byte_ranges() {
        let (tokens, _) = lex("t", "ab 'c'");
        assert_eq!(tokens[0].span, Span::new(0, 2));
        assert_eq!(tokens[1].span, Span::new(3, 6));
    }
}
