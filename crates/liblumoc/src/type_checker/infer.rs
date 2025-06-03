use std::{collections::HashSet, iter::once};

use lumo_core::{
    ExpressionNode, FunctionCallArgumentNode, FunctionParameterPatternNode, ItemNode, SimpleType,
    SimpleTypeRef,
};

use crate::type_checker::{environment::Scope, error::InferError};

pub fn infer_item(scope: &mut Scope, term: &ItemNode) -> Result<SimpleTypeRef, InferError> {
    match term {
        ItemNode::DeclEnumNode(decl_enum_node) => Ok(scope.put(SimpleType::Todo)),
        ItemNode::DeclFunctionNode(decl_function_node) => {
            let mut params = Vec::new();
            for param in &decl_function_node.parameters {
                match &**param.pattern {
                    FunctionParameterPatternNode::Bind(name) => {
                        params.push(scope.assign(name.clone(), SimpleType::variable()));
                    }
                    FunctionParameterPatternNode::MutBind(identifier_node) => {
                        return Err(InferError::new(format!(
                            "mut bind pattern is not supported yet"
                        )));
                    }
                    FunctionParameterPatternNode::SimplePattern(simple_pattern_node) => {
                        match simple_pattern_node {
                            lumo_core::SimplePatternNode::Discard(_) => {
                                params.push(scope.put(SimpleType::variable()));
                            }
                            lumo_core::SimplePatternNode::TaggedDestructuring(_, _) => {
                                return Err(InferError::new(format!(
                                    "destructuring pattern is not supported yet"
                                )));
                            }
                        }
                    }
                }
            }

            let ret = scope.put(SimpleType::variable());
            if let Some(body) = &decl_function_node.body {
                let body = infer_expr(scope, &body)?;
                constrain(scope, ret.clone(), body, HashSet::new())?;
            }

            Ok(scope.put(SimpleType::Function(params, ret)))
        }
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
        ExpressionNode::PrefixOperator(prefix_operator_node) => todo!(),
        ExpressionNode::InfixOperator(infix_operator_node) => todo!(),
        ExpressionNode::PostfixOperator(postfix_operator_node) => todo!(),
        ExpressionNode::Name(name_node) => scope.by_name(&name_node.0).ok_or(InferError::new(
            format!("There is no `{}` in scope", &name_node.0.0.content),
        )),
        ExpressionNode::Block(block_node) => {
            let mut result = SimpleTypeRef::UNIT;
            for ty in &block_node.0 {
                result = infer_expr(scope, ty)?;
            }
            Ok(result)
        }
        ExpressionNode::EnumVariant(enum_variant_node) => todo!(),
    }
}

fn constrain(
    scope: &mut Scope,
    lhs: SimpleTypeRef,
    rhs: SimpleTypeRef,
    mut cache: HashSet<(SimpleTypeRef, SimpleTypeRef)>,
) -> Result<(), InferError> {
    if cache.contains(&(lhs.clone(), rhs.clone())) {
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
