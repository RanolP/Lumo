use crate::Spanned;

use super::{
    IdentifierNode, InfixOperatorNode, PathNode, PatternNode, PostfixOperatorNode,
    PrefixOperatorNode, WithId,
};

#[derive(Clone, Debug)]
pub enum ExpressionNode {
    FunctionCall(FunctionCallNode),
    Match(MatchNode),
    PrefixOperator(PrefixOperatorNode),
    InfixOperator(InfixOperatorNode),
    PostfixOperator(PostfixOperatorNode),
    Name(NameNode),
    Block(BlockNode),
    EnumVariant(EnumVariantNode),
}

#[derive(Clone, Debug)]
pub struct FunctionCallNode {
    pub f: Box<ExpressionNode>,
    pub args: Vec<FunctionCallArgumentNode>,
}

#[derive(Clone, Debug)]
pub enum FunctionCallArgumentNode {
    Expr(ExpressionNode),
    MutName(IdentifierNode),
}

#[derive(Clone, Debug)]
pub struct MatchNode {
    pub expr: Box<WithId<Spanned<ExpressionNode>>>,
    pub arms: Vec<WithId<Spanned<MatchArmNode>>>,
}

#[derive(Clone, Debug)]
pub struct MatchArmNode {
    pub pat: WithId<Spanned<PatternNode>>,
    pub body: WithId<Spanned<ExpressionNode>>,
}

#[derive(Clone, Debug)]
pub struct NameNode(pub IdentifierNode);

#[derive(Clone, Debug)]
pub struct BlockNode(pub Vec<WithId<Spanned<ExpressionNode>>>);

#[derive(Clone, Debug)]
pub struct EnumVariantNode {
    pub tag: WithId<Spanned<EnumTagNode>>,
    pub body: WithId<Spanned<CompoundExprBodyNode>>,
}

#[derive(Clone, Debug)]
pub enum EnumTagNode {
    /// ```
    /// let .some(x) = option;
    /// ```
    Inferred(WithId<Spanned<IdentifierNode>>),
    /// ```
    /// let std.option.Option.some(x) = option;
    /// ```
    Full(WithId<Spanned<PathNode>>),
}
#[derive(Clone, Debug)]
pub enum CompoundExprBodyNode {
    None,
    Positional(Vec<WithId<Spanned<PatternNode>>>),
    Named(
        Vec<(
            WithId<Spanned<IdentifierNode>>,
            WithId<Spanned<PatternNode>>,
        )>,
    ),
}
