use lumo_core::{
    DestructuringBodyNode, DestructuringTagNode, FunctionParameterPatternNode, IdentifierNode,
    PathNode, PatternNode, SimplePatternNode, Spanned, TokenKind, WithId,
};
use winnow::{
    Parser,
    combinator::{alt, cut_err, empty, preceded, repeat, separated, seq, terminated},
};

use crate::{with_node_id, with_span, without_block};

use super::{Input, Result, token};

pub fn identifier(i: &mut Input) -> Result<WithId<Spanned<IdentifierNode>>> {
    with_node_id!(token(TokenKind::IdentifierIdentifier).map(|t| t.map(IdentifierNode)))
        .parse_next(i)
}

pub fn path(i: &mut Input) -> Result<WithId<Spanned<PathNode>>> {
    with_node_id!(with_span!(seq!(PathNode(separated(
        1..,
        identifier,
        token(TokenKind::PunctuationFullStop)
    )))))
    .parse_next(i)
}

pub fn pat_fn(i: &mut Input) -> Result<WithId<Spanned<FunctionParameterPatternNode>>> {
    alt((
        identifier.map(|ident| ident.map_deep(FunctionParameterPatternNode::Bind)),
        pat_simple.map(|node| node.map_deep(FunctionParameterPatternNode::SimplePattern)),
    ))
    .parse_next(i)
}

pub fn pat(i: &mut Input) -> Result<WithId<Spanned<PatternNode>>> {
    alt((
        with_span!(
            preceded(token(TokenKind::KeywordLet), identifier)
                .map(|ident| ident.map(PatternNode::NameBind))
        )
        .map(|e| e.transpose()),
        pat_simple.map(|node| node.map_deep(PatternNode::SimplePattern)),
    ))
    .parse_next(i)
}

pub fn pat_simple(i: &mut Input) -> Result<WithId<Spanned<SimplePatternNode>>> {
    alt((
        with_node_id!(
            token(TokenKind::IdentifierUnderscore)
                .map(|token| token.map(SimplePatternNode::Discard))
        ),
        with_node_id!(with_span!(seq!(SimplePatternNode::TaggedDestructuring(
            destructuring_tag,
            destructuring_body
        )))),
    ))
    .parse_next(i)
}

pub fn destructuring_tag(i: &mut Input) -> Result<WithId<Spanned<DestructuringTagNode>>> {
    with_node_id!(with_span!(alt((preceded(
        token(TokenKind::PunctuationFullStop),
        identifier
    )
    .map(DestructuringTagNode::Inferred),))))
    .parse_next(i)
}

pub fn destructuring_body(i: &mut Input) -> Result<WithId<Spanned<DestructuringBodyNode>>> {
    with_node_id!(with_span!(alt((
        preceded(
            token(TokenKind::PunctuationLeftParenthesis),
            without_block!(cut_err(terminated(
                separated(0.., pat, token(TokenKind::PunctuationComma))
                    .map(DestructuringBodyNode::Positional),
                token(TokenKind::PunctuationRightParenthesis)
            )))
        ),
        empty.value(DestructuringBodyNode::None),
    ))))
    .parse_next(i)
}
