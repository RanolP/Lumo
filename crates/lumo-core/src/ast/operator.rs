use crate::Spanned;

use super::{ExpressionNode, IdentifierNode, WithId};

#[derive(Clone, Debug)]
pub enum PrefixOperatorKind {
    Not,
    Negate,
}

#[derive(Clone, Debug)]
pub enum InfixOperatorKind {
    Add,
    Multiply,
}

#[derive(Clone, Debug)]
pub enum PostfixOperatorKind {
    FieldAccess(WithId<Spanned<IdentifierNode>>),
    FunctionCall(Vec<WithId<Spanned<ExpressionNode>>>),
    Index(Vec<WithId<Spanned<ExpressionNode>>>),
}

#[derive(Clone, Debug)]
pub struct PrefixOperatorNode {
    pub kind: WithId<Spanned<PrefixOperatorKind>>,
    pub expr: WithId<Spanned<Box<ExpressionNode>>>,
}

#[derive(Clone, Debug)]
pub struct InfixOperatorNode {
    pub lhs: WithId<Spanned<Box<ExpressionNode>>>,
    pub kind: WithId<Spanned<InfixOperatorKind>>,
    pub rhs: WithId<Spanned<Box<ExpressionNode>>>,
}

#[derive(Clone, Debug)]
pub struct PostfixOperatorNode {
    pub expr: WithId<Spanned<Box<ExpressionNode>>>,
    pub kind: WithId<Spanned<PostfixOperatorKind>>,
}
