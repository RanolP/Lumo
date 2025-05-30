use lumo_core::{
    DeclEnumNode, DeclFunctionNode, EnumBranchNode, ExpressionNode, FieldsNode,
    FunctionParameterNode, ItemNode, Spanned, Token, TokenKind, WithId,
};
use winnow::{
    ModalResult, Parser,
    combinator::{alt, cut_err, empty, opt, preceded, repeat, separated, seq, terminated},
    error::{ContextError, ParseError, StrContext},
};

mod base;
mod expr;
mod fragment;
mod operator;
mod ty;

pub(super) use base::*;
pub(super) use expr::*;
pub(super) use fragment::*;
pub(super) use operator::*;
pub(super) use ty::*;

use crate::{with_node_id, with_span, without_block};

pub fn parse(
    input: &[Spanned<Token>],
) -> std::result::Result<Vec<WithId<Spanned<ItemNode>>>, ParseError<Input, ContextError>> {
    program.parse(Input {
        state: State {
            node_id: 0,
            is_block: false,
            min_binding_power: 0,
        },
        input,
    })
}

pub fn program(i: &mut Input) -> ModalResult<Vec<WithId<Spanned<ItemNode>>>> {
    repeat(
        0..,
        alt((
            alt((
                token(TokenKind::SpaceVertical),
                token(TokenKind::Eof),
                token(TokenKind::PunctuationSemicolon),
            ))
            .value(Option::<WithId<Spanned<ItemNode>>>::None),
            item.map(|v| Some(v)).context(StrContext::Label("item")),
        )),
    )
    .map(|v: Vec<_>| v.into_iter().flatten().collect::<Vec<_>>())
    .parse_next(i)
}
pub fn item(i: &mut Input) -> ModalResult<WithId<Spanned<ItemNode>>> {
    alt((
        decl_enum.map(|node| node.map_deep(ItemNode::DeclEnumNode)),
        decl_function.map(|node| node.map_deep(ItemNode::DeclFunctionNode)),
    ))
    .parse_next(i)
}

pub fn decl_enum(i: &mut Input) -> ModalResult<WithId<Spanned<DeclEnumNode>>> {
    with_node_id!(with_span!(preceded(
        token(TokenKind::KeywordEnum).context(StrContext::Label("enum")),
        cut_err(seq!(DeclEnumNode {
            name: identifier.context(StrContext::Label("name")),
            _: token(TokenKind::PunctuationLeftCurlyBracket),
            branches: separated(
                0..,
                enum_branch.context(StrContext::Label("branch")),
                alt((
                    token(TokenKind::PunctuationComma),
                    token(TokenKind::PunctuationSemicolon),
                    token(TokenKind::SpaceVertical),
                ))
            ),
            _: opt(alt((
                token(TokenKind::PunctuationComma),
                token(TokenKind::PunctuationSemicolon)
            ))),
            _: token(TokenKind::PunctuationRightCurlyBracket),
        }))
        .context(StrContext::Label("body"))
    )))
    .parse_next(i)
}

pub fn enum_branch(input: &mut Input) -> ModalResult<WithId<Spanned<EnumBranchNode>>> {
    with_node_id!(with_span!(seq!(EnumBranchNode {
        name: identifier,
        fields: opt(fields),
    })))
    .parse_next(input)
}

pub fn fields(i: &mut Input) -> ModalResult<WithId<Spanned<FieldsNode>>> {
    with_node_id!(with_span!(alt((
        preceded(
            token(TokenKind::PunctuationLeftParenthesis),
            cut_err(terminated(
                terminated(
                    separated(
                        0..,
                        ty,
                        alt((
                            token(TokenKind::PunctuationComma),
                            token(TokenKind::SpaceVertical)
                        ))
                    ),
                    opt(token(TokenKind::PunctuationComma))
                ),
                token(TokenKind::PunctuationRightParenthesis),
            ))
        )
        .map(FieldsNode::Unnamed),
        preceded(
            token(TokenKind::PunctuationLeftCurlyBracket),
            cut_err(terminated(
                terminated(
                    separated(
                        0..,
                        seq!(
                            identifier,
                            _: token(TokenKind::PunctuationColon),
                            ty,
                        ),
                        alt((
                            token(TokenKind::PunctuationComma),
                            token(TokenKind::SpaceVertical)
                        ))
                    ),
                    opt(token(TokenKind::PunctuationComma))
                ),
                token(TokenKind::PunctuationRightCurlyBracket),
            ))
        )
        .map(FieldsNode::Named),
    ))))
    .parse_next(i)
}

pub fn decl_function(i: &mut Input) -> ModalResult<WithId<Spanned<DeclFunctionNode>>> {
    with_node_id!(with_span!(preceded(
        token(TokenKind::KeywordFn).context(StrContext::Label("function")),
        cut_err(seq!(DeclFunctionNode {
            name: identifier.context(StrContext::Label("name")),
            _: token(TokenKind::PunctuationLeftParenthesis),
            parameters: separated(
                0..,
                function_parameter,
                token(TokenKind::PunctuationComma),
            ),
            _: opt(token(TokenKind::PunctuationComma)),
            _: cut_err(token(TokenKind::PunctuationRightParenthesis)).context(StrContext::Label("parameter")),
            return_type: opt(preceded(
                token(TokenKind::PunctuationColon),
                ty
            )),
            body: alt((
                expr_block.map(|expr| Some(expr.map_deep(ExpressionNode::Block))),
                preceded(
                    token(TokenKind::PunctuationsFatArrow), without_block!(expr).map(Some)),
                empty.value(None),
            )).context(StrContext::Label("body")),
        }))
    )))
    .context(StrContext::Label("function declaration"))
    .parse_next(i)
}

pub fn function_parameter(i: &mut Input) -> ModalResult<WithId<Spanned<FunctionParameterNode>>> {
    with_node_id!(with_span!(seq!(FunctionParameterNode {
        pattern: pat_fn,
        ty: opt(preceded(token(TokenKind::PunctuationColon), ty)),
    })))
    .context(StrContext::Label("function parameter"))
    .parse_next(i)
}
