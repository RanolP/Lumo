use std::{collections::HashSet, iter::once};

use lumo_core::{
    DestructuringBodyNode, ExpressionNode, FunctionCallArgumentNode, FunctionParameterPatternNode,
    ItemNode, PatternNode, PostfixOperatorKind, SimpleType, SimpleTypeRef,
};

use crate::type_checker::{environment::Scope, error::InferError, syntax::transform_syntax_type};

pub fn infer_item(scope: &mut Scope, term: &ItemNode) -> Result<SimpleTypeRef, InferError> {
    match term {
        ItemNode::DeclEnumNode(decl_enum_node) => Ok(scope
            .get_ref(&decl_enum_node.name.0.content)
            .expect("enum declared from scan")),
        ItemNode::DeclFunctionNode(decl_function_node) => {
            let ty_ref = scope
                .get_ref(&decl_function_node.name.0.content)
                .expect("function declared from scan");
            let Some(SimpleType::Function(arg_types, ret)) = scope.get(ty_ref.clone()).cloned()
            else {
                panic!("Function is not typed as function");
            };

            let mut params: Vec<SimpleTypeRef> = Vec::new();
            for param in &decl_function_node.parameters {
                match &**param.pattern {
                    FunctionParameterPatternNode::Bind(name) => {
                        let ty = scope.assign(&name.0.content, SimpleType::variable());
                        if let Some(rhs) = &param.ty {
                            let rhs = transform_syntax_type(scope, &rhs)?;
                            constrain(scope, ty.clone(), rhs, HashSet::new())?;
                        }
                        params.push(ty);
                    }
                    FunctionParameterPatternNode::MutBind(identifier_node) => {
                        return Err(InferError::new(format!(
                            "mut bind pattern is not supported yet"
                        )));
                    }
                    FunctionParameterPatternNode::SimplePattern(simple_pattern_node) => {
                        params.push(infer_pat(scope, simple_pattern_node.clone())?);
                    }
                }
            }

            if let Some(body) = &decl_function_node.body {
                let body = infer_expr(scope, &body)?;
                constrain(scope, body, ret, HashSet::new())?;
            }

            Ok(ty_ref)
        }
    }
}

pub fn infer_pat(scope: &mut Scope, pat: PatternNode) -> Result<SimpleTypeRef, InferError> {
    match pat {
        PatternNode::NameBind(name) => Ok(scope.assign(&name.0.content, SimpleType::variable())),
        PatternNode::Discard(..) => Ok(scope.put(SimpleType::variable())),
        PatternNode::TaggedDestructuring(tag, body) => match &**body {
            DestructuringBodyNode::None => Ok(scope.put(SimpleType::variable())),
            DestructuringBodyNode::Positional(items) => {
                // TODO: utilize tag
                let result = scope.put(SimpleType::variable());
                for item in items {
                    infer_pat(scope, PatternNode::clone(&item))?;
                }
                Ok(result)
            }
            DestructuringBodyNode::Named(items) => Err(InferError::new(format!(
                "named destructuring pattern is not supported yet"
            ))),
        },
    }
}

pub fn infer_expr(scope: &mut Scope, term: &ExpressionNode) -> Result<SimpleTypeRef, InferError> {
    match term {
        ExpressionNode::FunctionCall(function_call_node) => {
            let res = scope.put(SimpleType::variable());

            let mut arg_types = Vec::new();
            for arg in &function_call_node.args {
                match arg {
                    FunctionCallArgumentNode::Expr(expression_node) => {
                        arg_types.push(infer_expr(scope, expression_node)?)
                    }
                    FunctionCallArgumentNode::MutName(identifier_node) => {
                        return Err(InferError::new(
                            "mut name in argument position is not supported yet".to_owned(),
                        ));
                    }
                };
            }

            let fn_from_expr_type = infer_expr(scope, &function_call_node.f)?;
            let fn_from_argument_type = scope.put(SimpleType::Function(arg_types, res.clone()));

            constrain(
                scope,
                fn_from_expr_type,
                fn_from_argument_type,
                HashSet::new(),
            )?;
            Ok(res)
        }
        ExpressionNode::Match(match_node) => {
            let expr_ty = infer_expr(scope, &match_node.expr)?;
            let ret = scope.put(SimpleType::variable());
            // todo
            for arm in &match_node.arms {}

            Ok(ret)
        }
        ExpressionNode::PrefixOperator(prefix_operator_node) => Err(InferError::new(
            "prefix operator is not implemented yet".to_owned(),
        )),
        ExpressionNode::InfixOperator(infix_operator_node) => Err(InferError::new(
            "infix operator is not implemented yet".to_owned(),
        )),
        ExpressionNode::PostfixOperator(postfix_operator_node) => {
            let expr = infer_expr(scope, &postfix_operator_node.expr)?;
            match &**postfix_operator_node.kind {
                PostfixOperatorKind::FieldAccess(_) => {
                    return Err(InferError::new(
                        "field access is not implemented yet".to_owned(),
                    ));
                }
                PostfixOperatorKind::FunctionCall(items) => {
                    let ret = scope.put(SimpleType::variable());
                    let mut arg_types = Vec::new();
                    for arg in items {
                        arg_types.push(infer_expr(scope, arg)?);
                    }
                    let fn_ty = scope.put(SimpleType::Function(arg_types, ret.clone()));
                    constrain(scope, expr, fn_ty, HashSet::new())?;
                    Ok(ret)
                }
                PostfixOperatorKind::Index(items) => {
                    return Err(InferError::new(
                        "index operator is not implemented yet".to_owned(),
                    ));
                }
            }
        }
        ExpressionNode::Name(name_node) => {
            scope
                .get_ref(&name_node.0.0.content)
                .ok_or(InferError::new(format!(
                    "There is no `{}` in scope",
                    &name_node.0.0.content
                )))
        }
        ExpressionNode::Block(block_node) => {
            let mut result = SimpleTypeRef::UNIT;
            for ty in &block_node.0 {
                result = infer_expr(scope, ty)?;
            }
            Ok(result)
        }
        ExpressionNode::EnumVariant(enum_variant_node) => {
            Ok(scope.put(SimpleType::variable()))
            // return Err(InferError::new(
            //     "enum variant is not implemented yet".to_owned(),
            // ));
        }
    }
}

pub fn constrain(
    scope: &mut Scope,
    lhs: SimpleTypeRef,
    rhs: SimpleTypeRef,
    mut cache: HashSet<(SimpleTypeRef, SimpleTypeRef)>,
) -> Result<(), InferError> {
    if cache.contains(&(lhs.clone(), rhs.clone())) {
        return Ok(());
    }
    if lhs == rhs {
        return Ok(());
    }
    cache.insert((lhs.clone(), rhs.clone()));
    let [Some(mut lhs_ty), Some(mut rhs_ty)] = scope.get_disjoint_mut([&lhs, &rhs]) else {
        return Ok(());
    };

    let to_constrain: Vec<(SimpleTypeRef, SimpleTypeRef)> = match (&mut lhs_ty, &mut rhs_ty) {
        (SimpleType::Primitive(lhs), SimpleType::Primitive(rhs)) if lhs == rhs => return Ok(()),
        (SimpleType::Function(lhs_args, lhs_ret), SimpleType::Function(rhs_args, rhs_ret))
            if lhs_args.len() == rhs_args.len() =>
        {
            (0..lhs_args.len())
                .map(|i| (lhs_args[i].clone(), rhs_args[i].clone()))
                .chain(once((lhs_ret.clone(), rhs_ret.clone())))
                .collect()
        }
        (SimpleType::Variable(state), _) => {
            state.upper_bounds.insert(0, rhs.clone());
            state
                .lower_bounds
                .iter()
                .map(|ty| (ty.clone(), rhs.clone()))
                .collect()
        }
        (_, SimpleType::Variable(state)) => {
            state.lower_bounds.insert(0, lhs.clone());
            state
                .upper_bounds
                .iter()
                .map(|ty| (lhs.clone(), ty.clone()))
                .collect()
        }
        _ => {
            return Err(InferError::new(format!(
                "Cannot constraint {:?} <: {:?}",
                lhs_ty, rhs_ty,
            )));
        }
    };
    for (lhs, rhs) in to_constrain {
        constrain(scope, lhs, rhs, cache.clone())?;
    }
    Ok(())
}
