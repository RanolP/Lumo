use std::fmt::{Debug, Display};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TokenKind {
    // ====== Spaces
    SpaceHorizontal,
    SpaceVertical,
    Eof,

    // Identifier
    IdentifierIdentifier,
    IdentifierUnderscore,

    // Keywords
    KeywordEnum,
    KeywordFn,
    KeywordLet,
    KeywordMatch,
    KeywordMut,
    KeywordStruct,

    // Single Punctuation
    PunctuationExclamationMark,
    PunctuationNumberSign,
    PunctuationDollarSign,
    PunctuationPercentSign,
    PunctuationAmpersand,
    PunctuationAsterisk,
    PunctuationPlusSign,
    PunctuationComma,
    PunctuationHyphenMinus,
    PunctuationFullStop,
    PunctuationSolidus,
    PunctuationColon,
    PunctuationSemicolon,
    PunctuationLessThanSign,
    PunctuationEqualsSign,
    PunctuationGreaterThanSign,
    PunctuationQuestionMark,
    PunctuationCommercialAt,
    PunctuationReverseSolidus,
    PunctuationCircumflexAccent,
    PunctuationVerticalLine,
    PunctuationTilde,
    PunctuationLeftParenthesis,
    PunctuationLeftSquareBracket,
    PunctuationLeftCurlyBracket,
    PunctuationRightParenthesis,
    PunctuationRightSquareBracket,
    PunctuationRightCurlyBracket,

    // Combination of Punctuations
    PunctuationsFatArrow,
}

#[derive(Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub content: String,
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Token({:?}, {:?})", self.kind, self.content))
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}({:?})", self.kind, self.content))
    }
}
