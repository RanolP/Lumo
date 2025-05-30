use lumo_core::{Spanned, TypeNode, WithId};
use winnow::{Parser, combinator::alt};

use super::{Input, Result, path};

pub fn ty(i: &mut Input) -> Result<WithId<Spanned<TypeNode>>> {
    alt((path_ty,)).parse_next(i)
}

pub fn path_ty(i: &mut Input) -> Result<WithId<Spanned<TypeNode>>> {
    path.map(|p| p.map_deep(TypeNode::Path)).parse_next(i)
}
