use lumo_core::{
    BlockNode, CompoundExprBodyNode, EnumTagNode, EnumVariantNode, ExpressionNode, MatchArmNode,
    MatchNode, NameNode, Spanned, TokenKind, WithId,
};
use winnow::{
    ModalResult, Parser,
    combinator::{
        alt, cut_err, delimited, empty, opt, preceded, repeat, separated, seq, terminated,
    },
    error::StrContext,
};

use crate::{
    parser::{expr, identifier, pat, token},
    with_block, with_node_id, with_span, without_block,
};

use super::Input;

pub fn basic_expr(i: &mut Input) -> ModalResult<WithId<Spanned<ExpressionNode>>> {
    alt((
        enum_variant.map(|expr| expr.map_deep(ExpressionNode::EnumVariant)),
        expr_match.map(|expr| expr.map_deep(ExpressionNode::Match)),
        expr_block.map(|expr| expr.map_deep(ExpressionNode::Block)),
        expr_name.map(|expr| expr.map_deep(ExpressionNode::Name)),
    ))
    .parse_next(i)
}

pub fn expr_name(i: &mut Input) -> ModalResult<WithId<Spanned<NameNode>>> {
    (identifier.map(|t| t.map_deep(NameNode))).parse_next(i)
}

pub fn expr_block(i: &mut Input) -> ModalResult<WithId<Spanned<BlockNode>>> {
    with_node_id!(with_span!(|i: &mut Input| -> ModalResult<BlockNode> {
        let result: Vec<_> = preceded(
            token(TokenKind::PunctuationLeftCurlyBracket),
            with_block!(terminated(
                terminated(
                    separated(
                        0..,
                        opt(expr),
                        alt((
                            token(TokenKind::PunctuationSemicolon),
                            token(TokenKind::SpaceVertical)
                        ))
                    ),
                    opt(token(TokenKind::PunctuationSemicolon)),
                ),
                cut_err(token(TokenKind::PunctuationRightCurlyBracket)),
            )),
        )
        .parse_next(i)?;
        Ok(BlockNode(result.into_iter().flatten().collect()))
    }))
    .context(StrContext::Label("block"))
    .parse_next(i)
}

pub fn expr_match(i: &mut Input) -> ModalResult<WithId<Spanned<MatchNode>>> {
    with_node_id!(with_span!(seq!(MatchNode {
        _: token(TokenKind::KeywordMatch),
        expr: cut_err(expr).map(Box::new),
        arms: cut_err(without_block!(delimited(
            token(TokenKind::PunctuationLeftCurlyBracket),
            with_block!(repeat(0.., alt((
                token(TokenKind::SpaceVertical).value(None),
                terminated(match_arm, opt(token(TokenKind::PunctuationComma))).map(Some)
            )))).map(|arms: Vec<_>| arms.into_iter().flatten().collect()),
            token(TokenKind::PunctuationRightCurlyBracket),
        ))),
    })))
    .parse_next(i)
}

pub fn match_arm(i: &mut Input) -> ModalResult<WithId<Spanned<MatchArmNode>>> {
    with_node_id!(with_span!(seq!(MatchArmNode {
        pat: pat,
        _: token(TokenKind::PunctuationsFatArrow),
        body: expr,
    })))
    .parse_next(i)
}

pub fn enum_variant(i: &mut Input) -> ModalResult<WithId<Spanned<EnumVariantNode>>> {
    with_node_id!(with_span!(seq!(EnumVariantNode {
        tag: alt((with_node_id!(with_span!(
            preceded(token(TokenKind::PunctuationFullStop), identifier).map(EnumTagNode::Inferred)
        )),)),
        body: alt((with_node_id!(with_span!(
            empty.value(CompoundExprBodyNode::None)
        )),)),
    })))
    .parse_next(i)
}
