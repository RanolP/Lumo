use std::{
    collections::{HashMap, HashSet},
    fmt::format,
};

use lumo_core::{
    DeclEnumNode, DeclFunctionNode, FieldsNode, ItemNode, SimpleType, SimpleTypeRef, Spanned,
    WithId,
};

use crate::{
    InferError, Scope,
    type_checker::{infer::constrain, syntax::transform_syntax_type},
};

pub fn scan(items: &[WithId<Spanned<ItemNode>>]) -> Result<Scope, InferError> {
    let mut enum_items = HashMap::<String, (DeclEnumNode, HashMap<String, SimpleTypeRef>)>::new();
    let mut fn_items = HashMap::<String, Vec<&DeclFunctionNode>>::new();

    let mut scope = Scope::new();

    for item in items {
        match &***item {
            ItemNode::DeclEnumNode(decl_enum_node) => {
                let name = decl_enum_node.name.0.content.clone();
                if enum_items.contains_key(&name) || fn_items.contains_key(&name) {
                    return Err(InferError::new(format!(
                        "Item `{name}` declared multiple times"
                    )));
                }
                let enum_ty = scope.assign(&decl_enum_node.name.0.content, SimpleType::variable());
                let mut tag_types = HashMap::new();
                for variant in &decl_enum_node.branches {
                    let tag = scope.put(SimpleType::VariantTag {
                        root: decl_enum_node.name.clone(),
                        variant: variant.name.clone(),
                    });
                    constrain(&mut scope, tag.clone(), enum_ty.clone(), HashSet::new())?;
                    tag_types.insert(variant.name.0.content.clone(), tag);
                }
                enum_items.insert(name, (decl_enum_node.clone(), tag_types));
            }
            ItemNode::DeclFunctionNode(decl_function_node) => {
                let name = decl_function_node.name.0.content.clone();
                if enum_items.contains_key(&name) {
                    return Err(InferError::new(format!(
                        "Item `{name}` declared multiple times"
                    )));
                }
                let fn_item = fn_items.entry(name).or_default();
                fn_item.push(decl_function_node);
            }
        }
    }

    for (_, (item, tag_types)) in enum_items {
        for variant in item.branches {
            let tag_type = tag_types
                .get(&variant.name.0.content)
                .expect("variant type must exists");
            let full_name = format!("{}.{}", item.name.0.content, variant.name.0.content);

            match &variant.fields {
                None => {
                    scope.assign_alias(&full_name, tag_type.clone());
                }
                Some(fields) => match &***fields {
                    FieldsNode::Unnamed(items) => {
                        let mut arg_types = Vec::new();
                        for arg in items {
                            arg_types.push(transform_syntax_type(&mut scope, &arg)?);
                        }
                        scope.assign(
                            &full_name,
                            SimpleType::Function(arg_types, tag_type.clone()),
                        );
                    }
                    FieldsNode::Named(items) => {
                        return Err(InferError::new(
                            "enum variant with named fields is not supported yet".to_owned(),
                        ));
                    }
                },
            }
        }
    }

    for (name, items) in fn_items {
        let core_definitions: Vec<_> = items
            .iter()
            .filter(|item| item.parameters.iter().all(|arg| arg.ty.is_some()))
            .collect();
        match core_definitions.len() {
            0 => {
                return Err(InferError::new(format!(
                    "Function `{name}` has no core definition"
                )));
            }
            1 => {
                let core_definition = core_definitions[0];
                let ty = scope.assign(&core_definition.name.0.content, SimpleType::variable());
                // TODO constrain to core definition
            }
            2.. => {
                return Err(InferError::new(format!(
                    "Function `{name}` has multiple core definition"
                )));
            }
        }
    }

    Ok(scope)
}
