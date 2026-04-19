use std::collections::{HashMap, HashSet};

use lumo_lir as lir;

#[derive(Debug, Clone)]
pub struct ResolutionMap {
    impls: HashMap<(String, Vec<String>), ImplResolution>,
    ambiguous: HashSet<(String, Vec<String>)>,
}

#[derive(Debug, Clone)]
pub struct ImplResolution {
    pub impl_const: String,
    pub methods: HashMap<String, MethodInfo>,
}

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub params: Vec<lir::Param>,
    pub body: lir::Expr,
}

impl ResolutionMap {
    pub fn get(&self, key: &(String, Vec<String>)) -> Option<&ImplResolution> {
        if self.ambiguous.contains(key) {
            return None;
        }
        self.impls.get(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &(String, Vec<String>)> {
        self.impls.keys()
    }
}

pub fn build_resolution_map(file: &lir::File) -> ResolutionMap {
    let cap_names: HashSet<String> = file
        .items
        .iter()
        .filter_map(|item| match item {
            lir::Item::Cap(c) => Some(c.name.clone()),
            _ => None,
        })
        .collect();

    let mut impls: HashMap<(String, Vec<String>), ImplResolution> = HashMap::new();
    let mut ambiguous: HashSet<(String, Vec<String>)> = HashSet::new();

    for item in &file.items {
        let lir::Item::Impl(impl_decl) = item else {
            continue;
        };
        let target = impl_decl.target_type.value.display();

        let (key, const_name) =
            if impl_decl.capability.is_none() && cap_names.contains(&target) {
                // Platform default: `impl Cap { ... }`
                ((target.clone(), vec![target.clone()]), target.clone())
            } else if let Some(cap_ty) = &impl_decl.capability {
                let cap = cap_ty.value.display();
                if !cap_names.contains(&cap) {
                    continue;
                }
                let const_name = impl_decl
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("__impl_{target}_{cap}"));
                ((cap, vec![target]), const_name)
            } else {
                // Inherent impl `impl T { ... }` where T is not a cap — skip.
                continue;
            };

        let methods: HashMap<String, MethodInfo> = impl_decl
            .methods
            .iter()
            .map(|m| {
                (
                    m.name.clone(),
                    MethodInfo {
                        params: m.params.clone(),
                        body: m.value.clone(),
                    },
                )
            })
            .collect();

        if impls.contains_key(&key) {
            ambiguous.insert(key.clone());
        } else {
            impls.insert(
                key,
                ImplResolution {
                    impl_const: const_name,
                    methods,
                },
            );
        }
    }

    ResolutionMap { impls, ambiguous }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hir, lexer::lex, parser::parse};
    use lumo_lir as lir;

    fn lower(src: &str) -> lir::File {
        let lexed = lex(src);
        let parsed = parse(&lexed.tokens, &lexed.errors);
        let h = hir::lower(&parsed.file);
        lir::lower(&h)
    }

    #[test]
    fn typeclass_default_impl_is_indexed() {
        let src = r#"
            cap Add { fn add(a: Number, b: Number): Number }
            impl Number: Add { fn add(a: Number, b: Number): Number { a } }
        "#;
        let file = lower(src);
        let map = build_resolution_map(&file);
        let key = ("Add".to_owned(), vec!["Number".to_owned()]);
        let res = map.get(&key).expect("expected resolution");
        assert_eq!(res.impl_const, "__impl_Number_Add");
        assert!(res.methods.contains_key("add"));
    }

    #[test]
    fn platform_default_impl_is_indexed() {
        let src = r#"
            cap Logger { fn log(msg: String): Number }
            impl Logger { fn log(msg: String): Number { 0 } }
        "#;
        let file = lower(src);
        let map = build_resolution_map(&file);
        let key = ("Logger".to_owned(), vec!["Logger".to_owned()]);
        let res = map.get(&key).expect("expected resolution");
        assert_eq!(res.impl_const, "Logger");
    }

    #[test]
    fn ambiguous_impls_are_excluded() {
        let src = r#"
            cap Add { fn add(a: Number, b: Number): Number }
            impl Number: Add { fn add(a: Number, b: Number): Number { a } }
            impl Number: Add { fn add(a: Number, b: Number): Number { b } }
        "#;
        let file = lower(src);
        let map = build_resolution_map(&file);
        let key = ("Add".to_owned(), vec!["Number".to_owned()]);
        assert!(map.get(&key).is_none(), "ambiguous binding must not resolve");
    }
}
