use std::collections::{HashMap, HashSet};

use lumo_core::{RepresentationalType, SimpleType, SimpleTypeRef, VariableState};

use crate::Scope;

#[derive(Clone, Hash, PartialEq, Eq)]
struct PolarVariable(VariableState, bool);

pub fn coalesce_type(scope: &Scope, ty: SimpleTypeRef) -> RepresentationalType {
    let mut recursive = HashMap::<PolarVariable, usize>::new();
    let mut counter = 0;

    visit(
        &mut recursive,
        &mut counter,
        scope,
        HashSet::new(),
        ty,
        true,
    )
}

fn visit(
    recursive: &mut HashMap<PolarVariable, usize>,
    counter: &mut usize,
    scope: &Scope,
    wip: HashSet<PolarVariable>,
    ty: SimpleTypeRef,
    polar: bool,
) -> RepresentationalType {
    match scope.get(ty).expect("must be valid type ref") {
        SimpleType::Primitive(name) => RepresentationalType::Primitive(name.clone()),
        SimpleType::VariantTag { root, variant } => RepresentationalType::VariantTag {
            root: root.clone(),
            variant: variant.clone(),
        },
        SimpleType::Function(args, ret) => {
            let mut arg_types = Vec::new();
            for ty in args {
                arg_types.push(visit(
                    recursive,
                    counter,
                    scope,
                    wip.clone(),
                    ty.clone(),
                    !polar,
                ));
            }
            let ret_type = visit(recursive, counter, scope, wip, ret.clone(), polar);
            RepresentationalType::Function(arg_types, Box::new(ret_type))
        }
        SimpleType::Tuple(types) => {
            let mut res_types = Vec::new();
            for ty in types {
                res_types.push(visit(
                    recursive,
                    counter,
                    scope,
                    wip.clone(),
                    ty.clone(),
                    polar,
                ));
            }
            RepresentationalType::Tuple(res_types)
        }
        SimpleType::Variable(vs) => {
            let pol = PolarVariable(vs.clone(), polar);
            if wip.contains(&pol) {
                RepresentationalType::Variable(*recursive.entry(pol).or_insert_with(move || {
                    *counter += 1;
                    *counter
                }))
            } else {
                let bounds = if polar {
                    &vs.lower_bounds
                } else {
                    &vs.upper_bounds
                };
                let mut bound_types = Vec::new();
                let mut new_wip = HashSet::new();
                new_wip.clone_from(&wip);
                new_wip.insert(pol.clone());
                for ty in bounds {
                    bound_types.push(visit(
                        recursive,
                        counter,
                        scope,
                        new_wip.clone(),
                        ty.clone(),
                        polar,
                    ));
                }
                let res = if polar {
                    bound_types
                        .into_iter()
                        .reduce(|l, r| RepresentationalType::Union(Box::new(l), Box::new(r)))
                } else {
                    bound_types
                        .into_iter()
                        .reduce(|l, r| RepresentationalType::Inter(Box::new(l), Box::new(r)))
                };
                let res = match res {
                    Some(v) => RepresentationalType::Union(
                        Box::new(RepresentationalType::Variable({
                            *counter += 1;
                            *counter
                        })),
                        Box::new(v),
                    ),
                    None => RepresentationalType::Variable({
                        *counter += 1;
                        *counter
                    }),
                };
                match recursive.get(&pol) {
                    Some(v) => RepresentationalType::Recursive(*v, Box::new(res)),
                    None => res,
                }
            }
        }
    }
}
