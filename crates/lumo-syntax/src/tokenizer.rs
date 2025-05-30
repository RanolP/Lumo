use std::iter::once;

use lumo_core::{Offset, Span, Spanned, Token, TokenKind};
use unicode_ident::{is_xid_continue, is_xid_start};
use winnow::{
    Parser, Result, Stateful,
    combinator::{alt, dispatch, empty, fail, repeat},
    error::{ContextError, ParseError},
    token::any,
};

type Input<'a> = Stateful<&'a str, Offset>;

pub fn tokenize(source: &str) -> Result<Vec<Spanned<Token>>, ParseError<Input, ContextError>> {
    token_list.parse(Input {
        state: Offset {
            offset: 0,
            line: 1,
            col: 0,
        },
        input: source,
    })
}

pub fn token_list(i: &mut Input) -> Result<Vec<Spanned<Token>>> {
    let mut result: Vec<_> = repeat(0.., token).parse_next(i)?;
    result.push(make_token(TokenKind::Eof, "", i));
    Ok(result)
}

fn token(i: &mut Input) -> Result<Spanned<Token>> {
    alt((
        space_horizontal,
        space_vertical,
        ident_keyword,
        punctuations,
    ))
    .parse_next(i)
}

fn space_horizontal(i: &mut Input) -> Result<Spanned<Token>> {
    let content: Vec<_> = repeat(
        1..,
        alt((
            alt((
                "\t",       // CHARACTER TABULATION
                " ",        // SPACE
                "\u{00AD}", // SOFT HYPHEN
                "\u{00A0}", // NO-BREAK SPACE
                "\u{1680}", // OGHAM SPACE MARK
                "\u{2000}", // EN QUAD
                "\u{2001}", // EM QUAD
                "\u{2002}", // EN SPACE
                "\u{2003}", // EM SPACE
                "\u{2004}", // THREE-PER-EM SPACE
                "\u{2005}", // FOUR-PER-EM SPACE
                "\u{2006}", // SIX-PER-EM SPACE
                "\u{2007}", // FIGURE SPACE
                "\u{2008}", // PUNCTUATION SPACE
                "\u{2009}", // THIN SPACE
                "\u{200A}", // HAIR SPACE
                "\u{200B}", // ZERO WIDTH SPACE
                "\u{200E}", // LEFT-TO-RIGHT MARK
                "\u{200F}", // RIGHT-TO-LEFT MARK
                "\u{202F}", // NARROW NO-BREAK SPACE
                "\u{205F}", // MEDIUM MATHEMATICAL SPACE
            )),
            alt((
                "\u{3000}", // IDEPGRAPHIC SPACE
                "\u{FEFF}", // ZERO WIDTH NO-BREAK SPACE
            )),
        )),
    )
    .parse_next(i)?;
    Ok(make_token(TokenKind::SpaceHorizontal, content.join(""), i))
}

fn space_vertical(i: &mut Input) -> Result<Spanned<Token>> {
    let content = alt((
        "\r\n",     // CRLF
        "\n",       // LINE FEED
        "\u{000B}", // LINE TABULATION
        "\u{000C}", // FORM FEED
        "\r",       // CARRIAGE RETURN
        "\u{0085}", // NEXT LINE
        "\u{2028}", // LINE SEPARATOR
        "\u{2029}", // PARAGRAPH SEPARATOR
    ))
    .parse_next(i)?;
    Ok(make_token(TokenKind::SpaceVertical, content, i))
}

fn ident_keyword(i: &mut Input) -> Result<Spanned<Token>> {
    let (start, cont): (_, Vec<_>) = (
        any.verify(|c| *c == '_' || is_xid_start(*c)),
        repeat(0.., any.verify(|c| is_xid_continue(*c))),
    )
        .parse_next(i)?;
    let content = once(start).chain(cont).collect::<String>();

    Ok(make_token(
        match content.as_ref() {
            "enum" => TokenKind::KeywordEnum,
            "fn" => TokenKind::KeywordFn,
            "let" => TokenKind::KeywordLet,
            "match" => TokenKind::KeywordMatch,
            "mut" => TokenKind::KeywordMut,
            "struct" => TokenKind::KeywordStruct,
            "_" => TokenKind::IdentifierUnderscore,
            _ => TokenKind::IdentifierIdentifier,
        },
        content,
        i,
    ))
}

fn punctuations(i: &mut Input) -> Result<Spanned<Token>> {
    let (kind, content) = dispatch! {any;
        '!' => empty.value(TokenKind::PunctuationExclamationMark),
        '#' => empty.value(TokenKind::PunctuationNumberSign),
        '$' => empty.value(TokenKind::PunctuationDollarSign),
        '%' => empty.value(TokenKind::PunctuationPercentSign),
        '&' => empty.value(TokenKind::PunctuationAmpersand),
        '*' => empty.value(TokenKind::PunctuationAsterisk),
        '+' => empty.value(TokenKind::PunctuationPlusSign),
        ',' => empty.value(TokenKind::PunctuationComma),
        '-' => empty.value(TokenKind::PunctuationHyphenMinus),
        '.' => empty.value(TokenKind::PunctuationFullStop),
        '/' => empty.value(TokenKind::PunctuationSolidus),
        ':' => empty.value(TokenKind::PunctuationColon),
        ';' => empty.value(TokenKind::PunctuationSemicolon),
        '<' => empty.value(TokenKind::PunctuationLessThanSign),
        '=' => alt((
            '>'.value(TokenKind::PunctuationsFatArrow),
            empty.value(TokenKind::PunctuationEqualsSign),
        )),
        '>' => empty.value(TokenKind::PunctuationGreaterThanSign),
        '?' => empty.value(TokenKind::PunctuationQuestionMark),
        '@' => empty.value(TokenKind::PunctuationCommercialAt),
        '\\' => empty.value(TokenKind::PunctuationReverseSolidus),
        '^' => empty.value(TokenKind::PunctuationCircumflexAccent),
        '|' => empty.value(TokenKind::PunctuationVerticalLine),
        '~' => empty.value(TokenKind::PunctuationTilde),
        '(' => empty.value(TokenKind::PunctuationLeftParenthesis),
        '[' => empty.value(TokenKind::PunctuationLeftSquareBracket),
        '{' => empty.value(TokenKind::PunctuationLeftCurlyBracket),
        ')' => empty.value(TokenKind::PunctuationRightParenthesis),
        ']' => empty.value(TokenKind::PunctuationRightSquareBracket),
        '}' => empty.value(TokenKind::PunctuationRightCurlyBracket),
        _ => fail,
    }
    .with_taken()
    .parse_next(i)?;
    Ok(make_token(kind, content, i))
}

fn make_token(kind: TokenKind, content: impl AsRef<str>, i: &mut Input) -> Spanned<Token> {
    let content = content.as_ref().to_owned();
    let begin = i.state.clone();
    i.state.offset += content.len();
    if kind == TokenKind::SpaceVertical {
        i.state.col = 0;
        i.state.line += 1;
    } else {
        i.state.col += content.len();
    }
    Spanned::new(Span::new(begin, i.state.clone()), Token { kind, content })
}
