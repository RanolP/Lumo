use crate::Spanned;

use super::{ExpressionNode, IdentifierNode, SimplePatternNode, TypeNode, WithId};

#[derive(Clone, Debug)]
pub enum ItemNode {
    DeclEnumNode(DeclEnumNode),
    DeclFunctionNode(DeclFunctionNode),
}

#[derive(Clone, Debug)]
pub struct DeclEnumNode {
    pub name: WithId<Spanned<IdentifierNode>>,
    pub branches: Vec<WithId<Spanned<EnumBranchNode>>>,
}

#[derive(Clone, Debug)]
pub struct EnumBranchNode {
    pub name: WithId<Spanned<IdentifierNode>>,
    pub fields: Option<WithId<Spanned<FieldsNode>>>,
}

#[derive(Clone, Debug)]
pub enum FieldsNode {
    Unnamed(Vec<WithId<Spanned<TypeNode>>>),
    Named(Vec<(WithId<Spanned<IdentifierNode>>, WithId<Spanned<TypeNode>>)>),
}

#[derive(Clone, Debug)]
pub struct DeclFunctionNode {
    pub name: WithId<Spanned<IdentifierNode>>,
    pub parameters: Vec<WithId<Spanned<FunctionParameterNode>>>,
    pub return_type: Option<WithId<Spanned<TypeNode>>>,
    pub body: Option<WithId<Spanned<ExpressionNode>>>,
}

#[derive(Clone, Debug)]
pub struct FunctionParameterNode {
    pub pattern: WithId<Spanned<FunctionParameterPatternNode>>,
    pub ty: Option<WithId<Spanned<TypeNode>>>,
}

#[derive(Clone, Debug)]
pub enum FunctionParameterPatternNode {
    Bind(IdentifierNode),
    MutBind(IdentifierNode),
    SimplePattern(SimplePatternNode),
}
