use crate::{Spanned, Token};

use super::WithId;

#[derive(Clone, Debug)]
pub struct IdentifierNode(pub Token);

#[derive(Clone, Debug)]
pub struct PathNode(pub Vec<WithId<Spanned<IdentifierNode>>>);

#[derive(Clone, Debug)]
pub enum TypeNode {
    Path(PathNode),
    Tuple(Vec<TypeNode>),
}

#[derive(Clone, Debug)]
pub enum PatternNode {
    NameBind(Spanned<IdentifierNode>),
    SimplePattern(SimplePatternNode),
}

#[derive(Clone, Debug)]
pub enum SimplePatternNode {
    Discard(Token),
    TaggedDestructuring(
        WithId<Spanned<DestructuringTagNode>>,
        WithId<Spanned<DestructuringBodyNode>>,
    ),
}

#[derive(Clone, Debug)]
pub enum DestructuringTagNode {
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
pub enum DestructuringBodyNode {
    None,
    Positional(Vec<WithId<Spanned<PatternNode>>>),
    Named(
        Vec<(
            WithId<Spanned<IdentifierNode>>,
            WithId<Spanned<PatternNode>>,
        )>,
    ),
}
