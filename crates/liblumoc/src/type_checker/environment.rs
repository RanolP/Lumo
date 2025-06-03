use std::collections::HashMap;

use lumo_core::{IdentifierNode, SimpleType, SimpleTypeRef};

pub struct Scope {
    id: usize,
    name_map: HashMap<String, SimpleTypeRef>,
    type_map: HashMap<usize, SimpleType>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            // 0 -> Unit Type (`()`)
            id: 1,
            name_map: Default::default(),
            type_map: Default::default(),
        }
    }

    pub fn put(&mut self, ty: SimpleType) -> SimpleTypeRef {
        self.id += 1;
        self.type_map.insert(self.id, ty);
        SimpleTypeRef(self.id)
    }
    pub fn assign(&mut self, name: IdentifierNode, ty: SimpleType) -> SimpleTypeRef {
        let ty_ref = self.put(ty);
        self.name_map.insert(name.0.content, ty_ref.clone());
        ty_ref
    }
    pub fn get(&self, ty_ref: SimpleTypeRef) -> Option<&SimpleType> {
        self.type_map.get(&ty_ref.0)
    }
    pub fn get_mut(&mut self, ty_ref: SimpleTypeRef) -> Option<&mut SimpleType> {
        self.type_map.get_mut(&ty_ref.0)
    }
    pub fn get_disjoint_mut<const N: usize>(
        &mut self,
        ty_ref_arr: [&SimpleTypeRef; N],
    ) -> [Option<&mut SimpleType>; N] {
        self.type_map
            .get_disjoint_mut(ty_ref_arr.map(|ty_ref| &ty_ref.0))
    }
    pub fn by_name(&self, ident: &IdentifierNode) -> Option<SimpleTypeRef> {
        self.name_map.get(&ident.0.content).cloned()
    }
}
