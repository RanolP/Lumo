use lumo_core::{SimpleTypeRef, TypeNode};

use crate::{InferError, Scope};

pub fn transform_syntax_type(
    scope: &mut Scope,
    node: &TypeNode,
) -> Result<SimpleTypeRef, InferError> {
    match node {
        TypeNode::Path(path_node) => {
            let idents: Vec<_> = path_node
                .0
                .iter()
                .map(|ident| ident.0.content.clone())
                .collect();
            let path = idents.join(".");
            scope.get_ref(&path).ok_or(InferError::new(
                "cannot transform path type syntax".to_owned(),
            ))
        }
        TypeNode::Tuple(type_nodes) => Err(InferError::new(
            "cannot transform tuple type syntax".to_owned(),
        )),
    }
}
