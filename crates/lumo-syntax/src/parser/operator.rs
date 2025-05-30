use lumo_core::{
    ExpressionNode, InfixOperatorKind, InfixOperatorNode, PostfixOperatorKind, PostfixOperatorNode,
    PrefixOperatorKind, PrefixOperatorNode, Spanned, TokenKind, WithId,
};
use winnow::{
    ModalResult, Parser,
    combinator::{alt, cut_err, preceded, separated, terminated},
    error::StrContext,
};
use winnow_pratt::{
    InfixOperatorSet, PostfixBindingPower, PostfixOperatorSet, PrattParser, PrefixOperatorSet,
};

use crate::{parser::identifier, with_binding_power, with_node_id, with_span, without_block};

use super::{Error, Input, basic_expr, token};

mod binding_powers {
    use winnow_pratt::{BindingPower, InfixBindingPower, PostfixBindingPower, PrefixBindingPower};

    pub const ADD_LEFT_ASSOC: InfixBindingPower = BindingPower::infix(1, 2);
    pub const MULT_LEFT_ASSOC: InfixBindingPower = BindingPower::infix(3, 4);
    pub const PREFIX: PrefixBindingPower = BindingPower::prefix(5);
    pub const POSTFIX: PostfixBindingPower = BindingPower::postfix(6);
}

type TExpr = WithId<Spanned<ExpressionNode>>;

struct OpParser<T>(T);

impl<'a> PrefixOperatorSet<Input<'a>, TExpr, Error>
    for OpParser<WithId<Spanned<PrefixOperatorKind>>>
{
    fn binding_power(&self) -> winnow_pratt::PrefixBindingPower {
        match **self.0 {
            PrefixOperatorKind::Not | PrefixOperatorKind::Negate => binding_powers::PREFIX,
        }
    }

    fn apply(self, input: &mut Input, expr: TExpr) -> TExpr {
        input.state.node_id += 1;
        let node_id = input.state.node_id;
        let merged_span = self.0.span().merge(expr.span());
        WithId::new(
            node_id,
            Spanned::new(
                merged_span,
                ExpressionNode::PrefixOperator(PrefixOperatorNode {
                    kind: self.0,
                    expr: expr.map_deep(Box::new),
                }),
            ),
        )
    }

    fn parser() -> impl Parser<Input<'a>, Self, Error> {
        with_node_id!(with_span!(alt((
            token(TokenKind::PunctuationExclamationMark).value(PrefixOperatorKind::Not),
            token(TokenKind::PunctuationHyphenMinus).value(PrefixOperatorKind::Negate),
        ))))
        .map(OpParser)
    }
}

impl<'a> InfixOperatorSet<Input<'a>, TExpr, Error>
    for OpParser<WithId<Spanned<InfixOperatorKind>>>
{
    fn binding_power(&self) -> winnow_pratt::InfixBindingPower {
        match **self.0 {
            InfixOperatorKind::Add => binding_powers::ADD_LEFT_ASSOC,
            InfixOperatorKind::Multiply => binding_powers::MULT_LEFT_ASSOC,
        }
    }

    fn apply(self, input: &mut Input<'a>, lhs: TExpr, rhs: TExpr) -> TExpr {
        input.state.node_id += 1;
        let node_id = input.state.node_id;
        let merged_span = lhs.span().merge(self.0.span()).merge(rhs.span());
        WithId::new(
            node_id,
            Spanned::new(
                merged_span,
                ExpressionNode::InfixOperator(InfixOperatorNode {
                    kind: self.0,
                    lhs: lhs.map_deep(Box::new),
                    rhs: rhs.map_deep(Box::new),
                }),
            ),
        )
    }

    fn parser() -> impl Parser<Input<'a>, Self, Error> {
        with_node_id!(with_span!(alt((
            token(TokenKind::PunctuationPlusSign).value(InfixOperatorKind::Add),
            token(TokenKind::PunctuationAsterisk).value(InfixOperatorKind::Multiply),
        ))))
        .map(OpParser)
    }
}

impl<'a> PostfixOperatorSet<Input<'a>, TExpr, Error>
    for OpParser<WithId<Spanned<PostfixOperatorKind>>>
{
    fn binding_power(&self) -> PostfixBindingPower {
        match &**self.0 {
            PostfixOperatorKind::FieldAccess(_)
            | PostfixOperatorKind::FunctionCall(_)
            | PostfixOperatorKind::Index(_) => binding_powers::POSTFIX,
        }
    }

    fn apply(self, input: &mut Input<'a>, expr: TExpr) -> TExpr {
        input.state.node_id += 1;
        let node_id = input.state.node_id;
        let merged_span = self.0.span().merge(expr.span());
        WithId::new(
            node_id,
            Spanned::new(
                merged_span,
                ExpressionNode::PostfixOperator(PostfixOperatorNode {
                    kind: self.0,
                    expr: expr.map_deep(Box::new),
                }),
            ),
        )
    }

    fn parser() -> impl Parser<Input<'a>, Self, Error> {
        with_node_id!(with_span!(alt((
            preceded(token(TokenKind::PunctuationFullStop), cut_err(identifier))
                .map(PostfixOperatorKind::FieldAccess),
            preceded(
                token(TokenKind::PunctuationLeftParenthesis),
                cut_err(terminated(
                    without_block!(with_binding_power!(
                        0,
                        separated(0.., expr, token(TokenKind::PunctuationComma))
                    )),
                    token(TokenKind::PunctuationRightParenthesis)
                ))
            )
            .map(PostfixOperatorKind::FunctionCall),
            preceded(
                token(TokenKind::PunctuationLeftSquareBracket),
                cut_err(terminated(
                    without_block!(with_binding_power!(
                        0,
                        separated(0.., expr, token(TokenKind::PunctuationComma))
                    )),
                    token(TokenKind::PunctuationRightSquareBracket)
                ))
            )
            .map(PostfixOperatorKind::Index)
        ))))
        .map(OpParser)
    }
}

pub fn expr(input: &mut Input) -> ModalResult<TExpr> {
    PrattParser::modal::<
        Input,
        TExpr,
        Error,
        OpParser<WithId<Spanned<PrefixOperatorKind>>>,
        OpParser<WithId<Spanned<InfixOperatorKind>>>,
        OpParser<WithId<Spanned<PostfixOperatorKind>>>,
        _,
    >(basic_expr)
    .rhs_ctx_label("expression")
    .rhs_ctx_expected_description("rhs")
    .prefix_ctx_label("expression")
    .prefix_ctx_expected_description("expression")
    .parse_next(input)
}
