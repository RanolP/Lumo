use std::collections::HashMap;

use crate::WithId;

pub struct Environment {
    scopes: Vec<Scope>,
    id_scope_map: HashMap<usize, usize>,
}

impl Environment {
    fn scope_entry(&mut self, parent: &dyn WithId, node: &dyn WithId) -> &mut Scope {
        todo!()
    }
    fn scope_mut(&mut self, node: &dyn WithId) -> Option<&mut Scope> {
        self.id_scope_map
            .get(&node.id())
            .and_then(|id| self.scopes.get_mut(*id))
    }
}

pub struct Scope {}
