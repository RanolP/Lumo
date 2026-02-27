use std::collections::HashMap;

use crate::lexer::Span;
use crate::parser::{self, Expr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Named(String),
    Produce(Box<Type>),
    Thunk(Box<Type>),
    Fn {
        params: Vec<Type>,
        ret: Box<Type>,
        effect: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedBinding {
    pub name: String,
    pub ty: Type,
}

pub fn typecheck_file(file: &parser::File) -> Vec<TypeError> {
    typecheck_and_bindings(file).1
}

pub fn typecheck_and_bindings(file: &parser::File) -> (Vec<CheckedBinding>, Vec<TypeError>) {
    let mut tc = TypeChecker {
        errors: Vec::new(),
        bindings: Vec::new(),
        data_defs: HashMap::new(),
        variant_owner: HashMap::new(),
    };
    tc.check_file(file);
    (tc.bindings, tc.errors)
}

pub fn render_type(ty: &Type) -> String {
    match ty {
        Type::Named(n) => n.clone(),
        Type::Produce(inner) => format!("produce {}", render_type(inner)),
        Type::Thunk(inner) => format!("thunk {}", render_type(inner)),
        Type::Fn {
            params,
            ret,
            effect,
        } => {
            let ps = params
                .iter()
                .map(render_type)
                .collect::<Vec<_>>()
                .join(", ");
            match effect {
                Some(e) if !e.is_empty() => format!("({ps}) -> {} / {e}", render_type(ret)),
                _ => format!("({ps}) -> {}", render_type(ret)),
            }
        }
    }
}

struct TypeChecker {
    errors: Vec<TypeError>,
    bindings: Vec<CheckedBinding>,
    data_defs: HashMap<String, DataDef>,
    variant_owner: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DataDef {
    variants: HashMap<String, Vec<Type>>,
}

impl TypeChecker {
    fn check_file(&mut self, file: &parser::File) {
        for item in &file.items {
            if let parser::Item::Data(d) = item {
                for v in &d.variants {
                    self.variant_owner.insert(v.name.clone(), d.name.clone());
                }
                self.data_defs.insert(
                    d.name.clone(),
                    DataDef {
                        variants: d
                            .variants
                            .iter()
                            .map(|v| {
                                (
                                    v.name.clone(),
                                    v.payload.iter().map(|t| parse_type(&t.repr)).collect(),
                                )
                            })
                            .collect(),
                    },
                );
            }
        }

        for item in &file.items {
            if let parser::Item::Fn(f) = item {
                self.check_fn(f);
            }
        }
    }

    fn check_fn(&mut self, f: &parser::FnDecl) {
        let mut env = HashMap::new();
        let mut param_types = Vec::new();
        for p in &f.params {
            let ty = parse_type(&p.ty.repr);
            env.insert(p.name.clone(), ty.clone());
            param_types.push(ty);
        }

        let effect = f
            .effect
            .as_ref()
            .map(|e| normalize_effect_text(&normalize_type_text(&e.repr)));
        if let Some(ret) = &f.return_type {
            let expected = parse_type(&ret.repr);
            self.check_expr(&f.body, &expected, &env);
            self.bindings.push(CheckedBinding {
                name: f.name.clone(),
                ty: Type::Fn {
                    params: param_types,
                    ret: Box::new(expected),
                    effect,
                },
            });
        } else {
            if let Some(inferred) = self.synth_expr(&f.body, &env) {
                self.bindings.push(CheckedBinding {
                    name: f.name.clone(),
                    ty: Type::Fn {
                        params: param_types,
                        ret: Box::new(inferred),
                        effect,
                    },
                });
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr, expected: &Type, env: &HashMap<String, Type>) {
        match (expr, expected) {
            (Expr::Produce { expr, .. }, Type::Produce(inner)) => {
                self.check_expr(expr, inner, env);
            }
            (Expr::Thunk { expr, .. }, Type::Thunk(inner)) => {
                self.check_expr(expr, inner, env);
            }
            _ => {
                if let Some(actual) = self.synth_expr(expr, env) {
                    if &actual != expected {
                        self.errors.push(TypeError {
                            span: expr_span(expr),
                            message: format!(
                                "type mismatch: expected {}, got {}",
                                render_type(expected),
                                render_type(&actual)
                            ),
                        });
                    }
                }
            }
        }
    }

    fn synth_expr(&mut self, expr: &Expr, env: &HashMap<String, Type>) -> Option<Type> {
        match expr {
            Expr::Ident { name, span } => {
                if let Some(ty) = env.get(name) {
                    Some(ty.clone())
                } else {
                    self.errors.push(TypeError {
                        span: *span,
                        message: format!("unknown variable `{name}`"),
                    });
                    None
                }
            }
            Expr::Produce { expr, .. } => {
                let inner = self.synth_expr(expr, env)?;
                Some(Type::Produce(Box::new(inner)))
            }
            Expr::Thunk { expr, .. } => {
                let inner = self.synth_expr(expr, env)?;
                Some(Type::Thunk(Box::new(inner)))
            }
            Expr::Force { expr, .. } => {
                let inner = self.synth_expr(expr, env)?;
                if let Type::Thunk(thunked) = inner {
                    Some(*thunked)
                } else {
                    self.errors.push(TypeError {
                        span: expr_span(expr),
                        message: format!("cannot force non-thunk type {}", render_type(&inner)),
                    });
                    None
                }
            }
            Expr::LetIn {
                name, value, body, ..
            } => {
                let value_ty = self.synth_expr(value, env)?;
                let mut child = env.clone();
                child.insert(name.clone(), value_ty);
                self.synth_expr(body, &child)
            }
            Expr::Match {
                scrutinee,
                arms,
                span,
            } => {
                let scrutinee_ty = self.synth_expr(scrutinee, env);
                let scrutinee_data = scrutinee_ty
                    .as_ref()
                    .and_then(nominal_head_name)
                    .and_then(|name| self.data_defs.get(name).cloned());
                if let Some(ref data_def) = scrutinee_data {
                    self.check_match_exhaustive(arms, data_def, *span);
                }
                let mut body_ty = None;
                for arm in arms {
                    let mut parsed_pattern = parse_match_pattern(&arm.pattern);
                    if parsed_pattern.is_none() {
                        self.errors.push(TypeError {
                            span: arm.span,
                            message: invalid_pattern_message(&arm.pattern),
                        });
                    }
                    if let (Some(data_def), Some(Pattern::Bind(name))) =
                        (scrutinee_data.as_ref(), parsed_pattern.as_ref())
                    {
                        if data_def.variants.contains_key(name) {
                            self.errors.push(TypeError {
                                span: arm.span,
                                message: format!(
                                    "variant pattern `{name}` must be written `.{name}`"
                                ),
                            });
                            parsed_pattern = None;
                        }
                    }
                    let mut arm_env = env.clone();
                    if let (Some(ty), Some(pattern)) = (scrutinee_ty.as_ref(), parsed_pattern.as_ref())
                    {
                        self.bind_pattern(pattern, ty, &mut arm_env, arm.span);
                    } else if let Some(ty) = &scrutinee_ty {
                        for binding in parsed_pattern
                            .as_ref()
                            .map(pattern_bindings)
                            .unwrap_or_default()
                        {
                            arm_env.insert(binding, ty.clone());
                        }
                    }
                    let arm_ty = self.synth_expr(&arm.body, &arm_env)?;
                    if let Some(expected) = &body_ty {
                        if *expected != arm_ty {
                            self.errors.push(TypeError {
                                span: *span,
                                message: format!(
                                    "match arm type mismatch: expected {}, got {}",
                                    render_type(expected),
                                    render_type(&arm_ty)
                                ),
                            });
                            return None;
                        }
                    } else {
                        body_ty = Some(arm_ty);
                    }
                }
                body_ty
            }
            Expr::Apply {
                owner,
                member,
                args,
                span,
            } => {
                let Some(def) = self.data_defs.get(owner) else {
                    self.errors.push(TypeError {
                        span: *span,
                        message: format!("unknown data bundle `{owner}`"),
                    });
                    return None;
                };
                let Some(owner_by_variant) = self.variant_owner.get(member) else {
                    self.errors.push(TypeError {
                        span: *span,
                        message: format!("unknown constructor `{owner}.{member}`"),
                    });
                    return None;
                };
                if owner_by_variant != owner {
                    self.errors.push(TypeError {
                        span: *span,
                        message: format!("constructor `{owner}.{member}` does not belong to `{owner}`"),
                    });
                    return None;
                };
                let Some(payload_types) = def.variants.get(member).cloned() else {
                    self.errors.push(TypeError {
                        span: *span,
                        message: format!("unknown constructor `{owner}.{member}`"),
                    });
                    return None;
                };
                if payload_types.len() != args.len() {
                    self.errors.push(TypeError {
                        span: *span,
                        message: format!(
                            "constructor `{owner}.{member}` expects {} args, got {}",
                            payload_types.len(),
                            args.len()
                        ),
                    });
                    return None;
                }
                for (arg, expected) in args.iter().zip(payload_types.iter()) {
                    self.check_expr(arg, expected, env);
                }
                Some(Type::Named(owner.clone()))
            }
            Expr::Error { .. } => None,
        }
    }

    fn bind_pattern(
        &mut self,
        pattern: &Pattern,
        expected: &Type,
        env: &mut HashMap<String, Type>,
        span: Span,
    ) {
        match pattern {
            Pattern::Wildcard => {}
            Pattern::Bind(name) => {
                env.insert(name.clone(), expected.clone());
            }
            Pattern::Ctor { name, args } => {
                let Some(data_name) = nominal_head_name(expected) else {
                    self.errors.push(TypeError {
                        span,
                        message: format!(
                            "constructor pattern `{name}` used on non-data type {}",
                            render_type(expected)
                        ),
                    });
                    return;
                };
                let Some(data_def) = self.data_defs.get(data_name) else {
                    self.errors.push(TypeError {
                        span,
                        message: format!("unknown data type `{data_name}` in match scrutinee"),
                    });
                    return;
                };
                if let Some(payload_types) = data_def.variants.get(name) {
                    if payload_types.len() != args.len() {
                        self.errors.push(TypeError {
                            span,
                            message: format!(
                                "pattern `{name}` expects {} args, got {}",
                                payload_types.len(),
                                args.len()
                            ),
                        });
                        return;
                    }

                    let payload_types = payload_types.clone();
                    for (arg, payload_ty) in args.iter().zip(payload_types.iter()) {
                        self.bind_pattern(arg, payload_ty, env, span);
                    }
                } else {
                    self.errors.push(TypeError {
                        span,
                        message: format!("unknown variant `{name}` in match pattern"),
                    });
                }
            }
        }
    }

    fn check_match_exhaustive(
        &mut self,
        arms: &[parser::MatchArm],
        data_def: &DataDef,
        span: Span,
    ) {
        if arms.is_empty() {
            self.errors.push(TypeError {
                span,
                message: "non-exhaustive match: missing all variants".to_owned(),
            });
            return;
        }

        let mut covered = HashMap::new();
        let mut has_catchall = false;
        for arm in arms {
            if let Some(pattern) = parse_match_pattern(&arm.pattern) {
                match &pattern {
                    Pattern::Wildcard | Pattern::Bind(_) => {
                        if has_catchall {
                            self.errors.push(TypeError {
                                span: arm.span,
                                message: "unreachable match arm: prior catch-all pattern".to_owned(),
                            });
                        }
                        has_catchall = true;
                    }
                    Pattern::Ctor { name, .. } => {
                        if has_catchall {
                            self.errors.push(TypeError {
                                span: arm.span,
                                message: "unreachable match arm: prior catch-all pattern".to_owned(),
                            });
                            continue;
                        }
                        if covered.contains_key(name) {
                            self.errors.push(TypeError {
                                span: arm.span,
                                message: format!(
                                    "unreachable match arm: variant `{name}` already covered"
                                ),
                            });
                        }
                        covered.insert(name.clone(), true);
                    }
                }
            } else if has_catchall {
                self.errors.push(TypeError {
                    span: arm.span,
                    message: "unreachable match arm: prior catch-all pattern".to_owned(),
                });
            }
        }

        if has_catchall {
            return;
        }

        let mut missing = data_def
            .variants
            .keys()
            .filter(|name| !covered.contains_key(*name))
            .cloned()
            .collect::<Vec<_>>();
        missing.sort();
        if !missing.is_empty() {
            self.errors.push(TypeError {
                span,
                message: format!("non-exhaustive match: missing variants {}", missing.join(", ")),
            });
        }
    }
}

fn parse_type(repr: &str) -> Type {
    let text = repr.trim();
    if let Some(rest) = text.strip_prefix("produce") {
        return Type::Produce(Box::new(parse_type(rest)));
    }
    if let Some(rest) = text.strip_prefix("thunk") {
        return Type::Thunk(Box::new(parse_type(rest)));
    }
    Type::Named(normalize_type_text(text))
}

fn normalize_type_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_effect_text(text: &str) -> String {
    text.replace("{ }", "{}")
}

fn parse_match_pattern(pattern: &str) -> Option<Pattern> {
    let mut parser = PatternParser::new(pattern);
    let pat = parser.parse_pattern();
    if parser.failed || parser.peek().is_some() {
        return None;
    }
    Some(pat)
}

fn invalid_pattern_message(pattern: &str) -> String {
    let text = pattern.trim();
    if text.contains('(') && !text.starts_with('.') {
        "constructor pattern must start with `.`".to_owned()
    } else {
        "invalid match pattern".to_owned()
    }
}

fn pattern_bindings(pat: &Pattern) -> Vec<String> {
    let mut out = Vec::new();
    collect_pattern_bindings(pat, &mut out);
    out
}

fn collect_pattern_bindings(pat: &Pattern, out: &mut Vec<String>) {
    match pat {
        Pattern::Wildcard => {}
        Pattern::Bind(name) => out.push(name.clone()),
        Pattern::Ctor { args, .. } => {
            for arg in args {
                collect_pattern_bindings(arg, out);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Pattern {
    Wildcard,
    Bind(String),
    Ctor { name: String, args: Vec<Pattern> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PatternToken {
    Ident(String),
    Underscore,
    Dot,
    LParen,
    RParen,
    Comma,
}

struct PatternParser {
    tokens: Vec<PatternToken>,
    index: usize,
    failed: bool,
}

impl PatternParser {
    fn new(text: &str) -> Self {
        Self {
            tokens: lex_pattern(text),
            index: 0,
            failed: false,
        }
    }

    fn parse_pattern(&mut self) -> Pattern {
        let Some(token) = self.bump() else {
            self.failed = true;
            return Pattern::Wildcard;
        };
        match token {
            PatternToken::Underscore => Pattern::Wildcard,
            PatternToken::Dot => {
                let Some(PatternToken::Ident(name)) = self.bump() else {
                    self.failed = true;
                    return Pattern::Wildcard;
                };
                self.parse_constructor_pattern(name)
            }
            PatternToken::Ident(head) if head == "let" || head == "mut" => {
                let Some(PatternToken::Ident(name)) = self.bump() else {
                    self.failed = true;
                    return Pattern::Wildcard;
                };
                if is_binding_name(&name) {
                    Pattern::Bind(name)
                } else {
                    self.failed = true;
                    Pattern::Wildcard
                }
            }
            PatternToken::Ident(head) => {
                if is_binding_name(&head) {
                    Pattern::Bind(head)
                } else {
                    self.failed = true;
                    Pattern::Wildcard
                }
            }
            _ => {
                self.failed = true;
                Pattern::Wildcard
            }
        }
    }

    fn peek(&self) -> Option<&PatternToken> {
        self.tokens.get(self.index)
    }

    fn parse_constructor_pattern(&mut self, name: String) -> Pattern {
        if self.peek() == Some(&PatternToken::LParen) {
            self.bump(); // (
            let mut args = Vec::new();
            if self.peek() != Some(&PatternToken::RParen) {
                loop {
                    let arg = self.parse_pattern();
                    args.push(arg);
                    if self.peek() == Some(&PatternToken::Comma) {
                        self.bump();
                        continue;
                    }
                    break;
                }
            }
            if self.peek() == Some(&PatternToken::RParen) {
                self.bump();
                Pattern::Ctor { name, args }
            } else {
                self.failed = true;
                Pattern::Wildcard
            }
        } else {
            Pattern::Ctor {
                name,
                args: Vec::new(),
            }
        }
    }

    fn bump(&mut self) -> Option<PatternToken> {
        let out = self.tokens.get(self.index).cloned();
        if out.is_some() {
            self.index += 1;
        }
        out
    }
}

fn lex_pattern(text: &str) -> Vec<PatternToken> {
    let mut out = Vec::new();
    let mut i = 0;
    let bytes = text.as_bytes();
    while i < bytes.len() {
        let ch = text[i..].chars().next().unwrap_or('\0');
        if ch.is_whitespace() {
            i += ch.len_utf8();
            continue;
        }
        match ch {
            '_' => {
                out.push(PatternToken::Underscore);
                i += 1;
            }
            '.' => {
                out.push(PatternToken::Dot);
                i += 1;
            }
            '(' => {
                out.push(PatternToken::LParen);
                i += 1;
            }
            ')' => {
                out.push(PatternToken::RParen);
                i += 1;
            }
            ',' => {
                out.push(PatternToken::Comma);
                i += 1;
            }
            _ => {
                if ch == '_' || ch.is_alphabetic() {
                    let start = i;
                    i += ch.len_utf8();
                    while i < bytes.len() {
                        let c = text[i..].chars().next().unwrap_or('\0');
                        if c == '_' || c.is_alphanumeric() {
                            i += c.len_utf8();
                        } else {
                            break;
                        }
                    }
                    out.push(PatternToken::Ident(text[start..i].to_owned()));
                } else {
                    i += ch.len_utf8();
                }
            }
        }
    }
    out
}

fn is_binding_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_alphabetic()) && chars.all(|c| c == '_' || c.is_alphanumeric())
}

fn nominal_head_name(ty: &Type) -> Option<&str> {
    let Type::Named(n) = ty else {
        return None;
    };
    n.split_whitespace().next()
}

fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::Ident { span, .. } => *span,
        Expr::Produce { span, .. } => *span,
        Expr::Thunk { span, .. } => *span,
        Expr::Force { span, .. } => *span,
        Expr::LetIn { span, .. } => *span,
        Expr::Match { span, .. } => *span,
        Expr::Apply { span, .. } => *span,
        Expr::Error { span } => *span,
    }
}
