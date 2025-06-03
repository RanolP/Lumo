use crate::Spanned;

use super::{ExpressionNode, IdentifierNode, PatternNode, TypeNode, WithId};

#[derive(Clone, Debug)]
pub enum ItemNode {
    DeclEnumNode(DeclEnumNode),
    DeclFunctionNode(DeclFunctionNode),
}

impl ItemNode {
    pub fn representative_name(&self) -> String {
        match self {
            ItemNode::DeclEnumNode(decl_enum_node) => (**decl_enum_node.name).0.content.clone(),
            ItemNode::DeclFunctionNode(decl_function_node) => {
                (**decl_function_node.name).0.content.clone()
            }
        }
    }
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
    SimplePattern(PatternNode),
}
