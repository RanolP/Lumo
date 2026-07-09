//! LIR structural invariant validation.
//!
//! Checks that the LIR satisfies CBPV structural invariants:
//! - `Roll` wraps a `Ctor`
//! - `Match` scrutinee is wrapped in `Unroll`
//! - `Apply` has exactly one argument (single-arg)
//! - `Lambda` has exactly one parameter (single-param)
//! - `FnDecl` values are curried spines (`thunk lambda x. lambda y. ... body`)

use crate::{BundleEntry, Expr, File, FnDecl, ImplMethodDecl, Item, MatchArm};
use lumo_types::ExprId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirWarning {
    pub expr_id: Option<ExprId>,
    pub message: String,
}

/// Validate structural invariants of a LIR file.
/// Returns warnings for any violations found.
/// These are development-time checks, not user-facing errors.
pub fn validate(file: &File) -> Vec<LirWarning> {
    let mut warnings = Vec::new();
    for item in &file.items {
        match item {
            Item::Fn(f) => validate_fn(&mut warnings, f),
            Item::Impl(impl_decl) => {
                for method in &impl_decl.methods {
                    validate_impl_method(&mut warnings, method);
                }
            }
            _ => {}
        }
    }
    warnings
}

fn validate_fn(warnings: &mut Vec<LirWarning>, f: &FnDecl) {
    // FnDecl with params should have a value that is a curried spine:
    // thunk lambda p1. lambda p2. ... body
    if !f.params.is_empty() {
        match &f.value {
            Expr::Thunk { expr, .. } => {
                let mut current = expr.as_ref();
                let mut lambda_count = 0;
                while let Expr::Lambda { body, .. } = current {
                    lambda_count += 1;
                    current = body.as_ref();
                }
                if lambda_count != f.params.len() {
                    warnings.push(LirWarning {
                        expr_id: Some(f.value.id()),
                        message: format!(
                            "fn `{}` has {} params but lambda spine has {} lambdas",
                            f.name,
                            f.params.len(),
                            lambda_count,
                        ),
                    });
                }
            }
            _ => {
                warnings.push(LirWarning {
                    expr_id: Some(f.value.id()),
                    message: format!(
                        "fn `{}` with params should have `thunk lambda ...` as value",
                        f.name
                    ),
                });
            }
        }
    }

    validate_expr(warnings, &f.value);
}

fn validate_impl_method(warnings: &mut Vec<LirWarning>, m: &ImplMethodDecl) {
    if !m.params.is_empty() {
        match &m.value {
            Expr::Thunk { expr, .. } => {
                let mut current = expr.as_ref();
                let mut lambda_count = 0;
                while let Expr::Lambda { body, .. } = current {
                    lambda_count += 1;
                    current = body.as_ref();
                }
                if lambda_count != m.params.len() {
                    warnings.push(LirWarning {
                        expr_id: Some(m.value.id()),
                        message: format!(
                            "method `{}` has {} params but lambda spine has {} lambdas",
                            m.name,
                            m.params.len(),
                            lambda_count,
                        ),
                    });
                }
            }
            _ => {
                warnings.push(LirWarning {
                    expr_id: Some(m.value.id()),
                    message: format!(
                        "method `{}` with params should have `thunk lambda ...` as value",
                        m.name
                    ),
                });
            }
        }
    }

    validate_expr(warnings, &m.value);
}

fn validate_expr(warnings: &mut Vec<LirWarning>, expr: &Expr) {
    match expr {
        // Roll should wrap a Ctor
        Expr::Roll { id, expr: inner, .. } => {
            if !matches!(inner.as_ref(), Expr::Ctor { .. }) {
                warnings.push(LirWarning {
                    expr_id: Some(*id),
                    message: format!(
                        "roll should wrap a ctor, got {:?}",
                        expr_kind(inner)
                    ),
                });
            }
            validate_expr(warnings, inner);
        }

        // Match scrutinee should be Unroll
        Expr::Match { id, scrutinee, arms, .. } => {
            if !matches!(scrutinee.as_ref(), Expr::Unroll { .. }) {
                warnings.push(LirWarning {
                    expr_id: Some(*id),
                    message: format!(
                        "match scrutinee should be unroll, got {:?}",
                        expr_kind(scrutinee)
                    ),
                });
            }
            validate_expr(warnings, scrutinee);
            for arm in arms {
                validate_match_arm(warnings, arm);
            }
        }

        // Recurse into sub-expressions
        Expr::Ctor { args, .. } => {
            for arg in args {
                validate_expr(warnings, arg);
            }
        }
        Expr::Thunk { expr: inner, .. }
        | Expr::Produce { expr: inner, .. }
        | Expr::Force { expr: inner, .. }
        | Expr::Unroll { expr: inner, .. }
        | Expr::Ann { expr: inner, .. } => {
            validate_expr(warnings, inner);
        }
        Expr::Lambda { body, .. } => {
            validate_expr(warnings, body);
        }
        Expr::Apply { callee, arg, .. } => {
            validate_expr(warnings, callee);
            validate_expr(warnings, arg);
        }
        Expr::Let { value, body, .. } => {
            validate_expr(warnings, value);
            validate_expr(warnings, body);
        }
        Expr::Handle { handler, body, .. } => {
            validate_expr(warnings, handler);
            validate_expr(warnings, body);
        }
        Expr::Member { object, .. } => {
            validate_expr(warnings, object);
        }
        Expr::Bundle { entries, .. } => {
            for entry in entries {
                validate_bundle_entry(warnings, entry);
            }
        }
        Expr::Ident { .. }
        | Expr::String { .. }
        | Expr::Number { .. }
        | Expr::Perform { .. }
        | Expr::Error { .. } => {}
    }
}

fn validate_match_arm(warnings: &mut Vec<LirWarning>, arm: &MatchArm) {
    validate_expr(warnings, &arm.body);
}

fn validate_bundle_entry(warnings: &mut Vec<LirWarning>, entry: &BundleEntry) {
    validate_expr(warnings, &entry.body);
}

fn expr_kind(expr: &Expr) -> &'static str {
    match expr {
        Expr::Ident { .. } => "Ident",
        Expr::String { .. } => "String",
        Expr::Number { .. } => "Number",
        Expr::Ctor { .. } => "Ctor",
        Expr::Thunk { .. } => "Thunk",
        Expr::Roll { .. } => "Roll",
        Expr::Bundle { .. } => "Bundle",
        Expr::Produce { .. } => "Produce",
        Expr::Force { .. } => "Force",
        Expr::Lambda { .. } => "Lambda",
        Expr::Apply { .. } => "Apply",
        Expr::Let { .. } => "Let",
        Expr::Match { .. } => "Match",
        Expr::Unroll { .. } => "Unroll",
        Expr::Perform { .. } => "Perform",
        Expr::Handle { .. } => "Handle",
        Expr::Member { .. } => "Member",
        Expr::Ann { .. } => "Ann",
        Expr::Error { .. } => "Error",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use lumo_span::Span;
    use lumo_types::{ContentHash, ExprId, Spanned, TypeExpr};

    fn dummy_span() -> Span {
        Span::new(0, 0)
    }

    fn spanned<T>(value: T) -> Spanned<T> {
        Spanned {
            value,
            span: dummy_span(),
        }
    }

    fn id(n: u32) -> ExprId {
        ExprId(n)
    }

    #[test]
    fn valid_roll_ctor() {
        let file = File {
            items: vec![Item::Fn(FnDecl {
                name: "f".into(),
                generics: vec![],
                params: vec![],
                return_type: None,
                cap: None,
                value: Expr::Roll {
                    id: id(0),
                    expr: Box::new(Expr::Ctor {
                        id: id(1),
                        name: "Bool.true".into(),
                        called: false,
                        args: vec![],
                    }),
                },
                inline: false,
                span: dummy_span(),
            })],
            content_hash: ContentHash(0),
            spans: vec![dummy_span(); 2],
        };
        let warnings = validate(&file);
        assert!(warnings.is_empty(), "unexpected: {warnings:?}");
    }

    #[test]
    fn invalid_roll_without_ctor() {
        let file = File {
            items: vec![Item::Fn(FnDecl {
                name: "f".into(),
                generics: vec![],
                params: vec![],
                return_type: None,
                cap: None,
                value: Expr::Roll {
                    id: id(0),
                    expr: Box::new(Expr::Ident {
                        id: id(1),
                        name: "x".into(),
                    }),
                },
                inline: false,
                span: dummy_span(),
            })],
            content_hash: ContentHash(0),
            spans: vec![dummy_span(); 2],
        };
        let warnings = validate(&file);
        assert!(
            warnings.iter().any(|w| w.message.contains("roll should wrap a ctor")),
            "expected roll warning: {warnings:?}"
        );
    }

    #[test]
    fn valid_match_unroll() {
        let file = File {
            items: vec![Item::Fn(FnDecl {
                name: "f".into(),
                generics: vec![],
                params: vec![],
                return_type: None,
                cap: None,
                value: Expr::Match {
                    id: id(0),
                    scrutinee: Box::new(Expr::Unroll {
                        id: id(1),
                        expr: Box::new(Expr::Ident {
                            id: id(2),
                            name: "b".into(),
                        }),
                    }),
                    arms: vec![],
                },
                inline: false,
                span: dummy_span(),
            })],
            content_hash: ContentHash(0),
            spans: vec![dummy_span(); 3],
        };
        let warnings = validate(&file);
        assert!(warnings.is_empty(), "unexpected: {warnings:?}");
    }

    #[test]
    fn invalid_match_without_unroll() {
        let file = File {
            items: vec![Item::Fn(FnDecl {
                name: "f".into(),
                generics: vec![],
                params: vec![],
                return_type: None,
                cap: None,
                value: Expr::Match {
                    id: id(0),
                    scrutinee: Box::new(Expr::Ident {
                        id: id(1),
                        name: "b".into(),
                    }),
                    arms: vec![],
                },
                inline: false,
                span: dummy_span(),
            })],
            content_hash: ContentHash(0),
            spans: vec![dummy_span(); 2],
        };
        let warnings = validate(&file);
        assert!(
            warnings.iter().any(|w| w.message.contains("match scrutinee should be unroll")),
            "expected match warning: {warnings:?}"
        );
    }

    #[test]
    fn fn_with_params_needs_lambda_spine() {
        let file = File {
            items: vec![Item::Fn(FnDecl {
                name: "f".into(),
                generics: vec![],
                params: vec![crate::Param {
                    name: "x".into(),
                    ty: spanned(TypeExpr::Named("Number".into())),
                    span: dummy_span(),
                }],
                return_type: None,
                cap: None,
                value: Expr::Thunk {
                    id: id(0),
                    expr: Box::new(Expr::Lambda {
                        id: id(1),
                        param: "x".into(),
                        body: Box::new(Expr::Produce {
                            id: id(2),
                            expr: Box::new(Expr::Ident {
                                id: id(3),
                                name: "x".into(),
                            }),
                        }),
                    }),
                },
                inline: false,
                span: dummy_span(),
            })],
            content_hash: ContentHash(0),
            spans: vec![dummy_span(); 4],
        };
        let warnings = validate(&file);
        assert!(warnings.is_empty(), "unexpected: {warnings:?}");
    }

    #[test]
    fn fn_spine_mismatch() {
        let file = File {
            items: vec![Item::Fn(FnDecl {
                name: "f".into(),
                generics: vec![],
                params: vec![
                    crate::Param {
                        name: "a".into(),
                        ty: spanned(TypeExpr::Named("Number".into())),
                        span: dummy_span(),
                    },
                    crate::Param {
                        name: "b".into(),
                        ty: spanned(TypeExpr::Named("Number".into())),
                        span: dummy_span(),
                    },
                ],
                return_type: None,
                cap: None,
                // Only 1 lambda for 2 params
                value: Expr::Thunk {
                    id: id(0),
                    expr: Box::new(Expr::Lambda {
                        id: id(1),
                        param: "a".into(),
                        body: Box::new(Expr::Produce {
                            id: id(2),
                            expr: Box::new(Expr::Ident {
                                id: id(3),
                                name: "a".into(),
                            }),
                        }),
                    }),
                },
                inline: false,
                span: dummy_span(),
            })],
            content_hash: ContentHash(0),
            spans: vec![dummy_span(); 4],
        };
        let warnings = validate(&file);
        assert!(
            warnings.iter().any(|w| w.message.contains("2 params but lambda spine has 1")),
            "expected spine mismatch: {warnings:?}"
        );
    }
}
