//! Per-node field tables shared by the AST-accessor emitter, the elab
//! checks, and the elab codegen. One source of truth for "which labeled
//! fields does node `N` have, what do they point at, and at which
//! same-class occurrence offset" — the M0 accessor scheme.

use std::collections::BTreeSet;

use crate::syntax::ast::{Praat, RuleBody, Shape, ShapeKind};

use super::model::Language;
use super::praat::{classify_row, RowKind, TailPart};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FieldTarget {
    /// A named token, by its declared name (`ident`, `lit.number`).
    Token(String),
    /// A literal token, by its text (`'extern'`).
    LitToken(String),
    /// A rule — concrete kinds are `kind_set(lang, rule)`.
    Node(String),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Field {
    pub label: String,
    pub target: FieldTarget,
    /// Under a `*` / `sep(…)` — the accessor is an iterator.
    pub many: bool,
    /// Not under `?` / `*` / `sep(…)` — a parse without it is an error.
    pub required: bool,
    /// Same-class occurrence offset (`value:Expr … body:Expr` → 0, 1).
    pub skip: usize,
}

impl Field {
    /// The occurrence-counting key: two fields share offsets iff they
    /// share this (rule name for nodes, SyntaxKind name for tokens).
    pub fn class_key(&self, lang: &Language) -> String {
        match &self.target {
            FieldTarget::Node(rule) => format!("n:{rule}"),
            FieldTarget::Token(token) => {
                format!("t:{}", crate::codegen::naming::kind_name(token))
            }
            FieldTarget::LitToken(text) => match lang.literal_token(text) {
                Some(t) => format!("t:{}", crate::codegen::naming::kind_name(&t.name)),
                None => format!("t:'{text}'"),
            },
        }
    }
}

/// Labeled fields of a plain struct rule shape, with occurrence offsets.
pub fn struct_fields(lang: &Language, shape: &Shape) -> Vec<Field> {
    let mut raw = Vec::new();
    walk(shape, false, true, &mut raw);
    assign_offsets(lang, raw)
}

fn assign_offsets(lang: &Language, raw: Vec<Field>) -> Vec<Field> {
    let mut out: Vec<Field> = Vec::new();
    for field in raw {
        let key = field.class_key(lang);
        let skip = out
            .iter()
            .filter(|prev| !prev.many && prev.class_key(lang) == key)
            .count();
        out.push(Field { skip, ..field });
    }
    out
}

fn walk(shape: &Shape, many: bool, required: bool, out: &mut Vec<Field>) {
    match &shape.kind {
        ShapeKind::Label { label, shape: inner } => {
            if let Some((target, inner_many)) = label_target(inner) {
                out.push(Field {
                    label: label.clone(),
                    target,
                    many: many || inner_many,
                    required: required && !inner_many && !is_optional(inner),
                    skip: 0,
                });
            }
        }
        ShapeKind::Seq(parts) => {
            for p in parts {
                walk(p, many, required, out);
            }
        }
        ShapeKind::Alt(parts) => {
            for p in parts {
                walk(p, many, false, out);
            }
        }
        ShapeKind::Opt(inner) => walk(inner, many, false, out),
        ShapeKind::Rep(inner) => walk(inner, true, false, out),
        ShapeKind::Sep { item, .. } => walk(item, true, false, out),
        _ => {}
    }
}

fn is_optional(shape: &Shape) -> bool {
    matches!(&shape.kind, ShapeKind::Opt(_))
}

/// What a label points at, unwrapping `?`/`*`/`sep(…)` around the atom.
fn label_target(shape: &Shape) -> Option<(FieldTarget, bool)> {
    match &shape.kind {
        ShapeKind::NodeRef(rule) => Some((FieldTarget::Node(rule.clone()), false)),
        ShapeKind::TokenRef(token) => Some((FieldTarget::Token(token.clone()), false)),
        ShapeKind::Lit(text) => Some((FieldTarget::LitToken(text.clone()), false)),
        ShapeKind::Opt(inner) => label_target(inner),
        ShapeKind::Rep(inner) | ShapeKind::Sep { item: inner, .. } => {
            label_target(inner).map(|(t, _)| (t, true))
        }
        // A label over a composite shape has no obvious accessor; skip.
        _ => None,
    }
}

/// The concrete SyntaxKind-bearing node names a rule can produce:
/// itself for a struct rule, the arm union for a transparent enum rule,
/// atoms ∪ synthesized row kinds for a praat rule.
pub fn kind_set(lang: &Language, rule_name: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut work = vec![rule_name.to_owned()];
    let mut seen = BTreeSet::new();
    while let Some(name) = work.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let Some(rule) = lang.rules.get(&name) else { continue };
        match &rule.body {
            RuleBody::Plain(shape) => match crate::codegen::parser::enum_arms(shape) {
                Some(arms) => work.extend(arms),
                None => {
                    out.insert(name);
                }
            },
            RuleBody::Praat(praat) => {
                for (atom, _) in &praat.simple {
                    work.push(atom.clone());
                }
                out.extend(crate::codegen::syntax_kind::praat_kinds(&name, praat));
            }
        }
    }
    out
}

/// Where a concrete node name came from.
pub enum NodeInfo<'l> {
    /// A plain struct rule — fields via [`struct_fields`].
    Struct(&'l Shape),
    /// A synthesized praat row node (`ExprPostfix`): the praat rule it
    /// belongs to plus its placement rows.
    PraatRow { rule: String, praat: &'l Praat, placement: &'static str },
}

/// Look up a concrete node name: a struct rule or a `RulePlacement`
/// praat row kind.
pub fn node_info<'l>(lang: &'l Language, node: &str) -> Option<NodeInfo<'l>> {
    if let Some(rule) = lang.rules.get(node) {
        return match &rule.body {
            RuleBody::Plain(shape) if crate::codegen::parser::enum_arms(shape).is_none() => {
                Some(NodeInfo::Struct(shape))
            }
            _ => None, // transparent enum / praat rule — not a concrete node
        };
    }
    for placement in ["Prefix", "Infix", "Postfix", "Mixfix"] {
        let Some(rule_name) = node.strip_suffix(placement) else { continue };
        if let Some(rule) = lang.rules.get(rule_name) {
            if let RuleBody::Praat(praat) = &rule.body {
                if crate::codegen::syntax_kind::praat_kinds(rule_name, praat)
                    .contains(&node.to_owned())
                {
                    return Some(NodeInfo::PraatRow {
                        rule: rule_name.to_owned(),
                        praat,
                        placement,
                    });
                }
            }
        }
    }
    None
}

/// Fields of a concrete node — struct labels, or the synthesized praat
/// row accessors (`op`/`expr`/`lhs`/`rhs` + snake-named payloads).
pub fn node_fields(lang: &Language, node: &str) -> Option<Vec<Field>> {
    match node_info(lang, node)? {
        NodeInfo::Struct(shape) => Some(struct_fields(lang, shape)),
        NodeInfo::PraatRow { rule, praat, placement } => {
            let mut fields = Vec::new();
            let op = |fields: &mut Vec<Field>| {
                fields.push(Field {
                    label: "op".to_owned(),
                    target: FieldTarget::Token("<op>".to_owned()),
                    many: false,
                    required: true,
                    skip: 0,
                });
            };
            match placement {
                "Prefix" | "Postfix" => {
                    op(&mut fields);
                    fields.push(Field {
                        label: "expr".to_owned(),
                        target: FieldTarget::Node(rule.clone()),
                        many: false,
                        required: true,
                        skip: 0,
                    });
                }
                "Infix" => {
                    op(&mut fields);
                    for (i, label) in ["lhs", "rhs"].iter().enumerate() {
                        fields.push(Field {
                            label: (*label).to_owned(),
                            target: FieldTarget::Node(rule.clone()),
                            many: false,
                            required: true,
                            skip: i,
                        });
                    }
                }
                _ => op(&mut fields), // Mixfix: op only (operands unnamed)
            }
            if placement == "Postfix" {
                let mut payloads: Vec<String> = Vec::new();
                for row in &praat.rows {
                    if let Ok(RowKind::Postfix { tail, .. }) = classify_row(row) {
                        for part in tail {
                            if let TailPart::Node(payload) = part {
                                if !payloads.contains(&payload) {
                                    payloads.push(payload);
                                }
                            }
                        }
                    }
                }
                for payload in payloads {
                    fields.push(Field {
                        label: crate::codegen::naming::snake(&payload),
                        target: FieldTarget::Node(payload),
                        many: false,
                        required: false,
                        skip: 0,
                    });
                }
            }
            Some(fields)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::loader::{FileKind, LoadedFile};
    use crate::project::merge::merge_project;

    fn lang(text: &str) -> Language {
        let (def, diags) = merge_project(&[LoadedFile {
            path: "L.syn.langue".into(),
            kind: FileKind::Syn { language: "L".into() },
            text: text.into(),
        }]);
        assert!(diags.is_empty(), "{diags:?}");
        def.languages["L"].clone()
    }

    const SYN: &str = "\
token kw.fn = 'fn'
token ident = /[a-z]+/
token num = /[0-9]+/
token comma = ','
token plus = '+'
token paren.open = '('
token paren.close = ')'
File = decls:Decl*
Decl = 'fn' name:ident '(' params:sep(Param, ',')? ')' (':' ret:ident)? value:Expr other:Expr
token colon = ':'
Param = name:ident
Lit = value:num
Body =
  | Decl
  | Lit
Expr = praat {
  simple = Lit
  operators {
    @70 '+' @69,
    @110 '(' Args ')',
  }
}
Args = args:sep(Expr, ',')?
";

    #[test]
    fn struct_fields_with_offsets_and_optionality() {
        let l = lang(SYN);
        let fields = node_fields(&l, "Decl").unwrap();
        let by_label: std::collections::BTreeMap<&str, &Field> =
            fields.iter().map(|f| (f.label.as_str(), f)).collect();
        assert_eq!(by_label["name"].target, FieldTarget::Token("ident".into()));
        assert!(by_label["name"].required);
        assert!(by_label["params"].many);
        assert!(!by_label["params"].required);
        assert!(!by_label["ret"].required);
        assert_eq!(by_label["value"].skip, 0);
        assert_eq!(by_label["other"].skip, 1);
        assert!(matches!(&by_label["value"].target, FieldTarget::Node(n) if n == "Expr"));
    }

    #[test]
    fn kind_sets_unfold_transparent_rules() {
        let l = lang(SYN);
        let body = kind_set(&l, "Body");
        assert!(body.contains("Decl") && body.contains("Lit"));
        let expr = kind_set(&l, "Expr");
        assert!(expr.contains("Lit"));
        assert!(expr.contains("ExprInfix"));
        assert!(expr.contains("ExprPostfix"));
        assert!(!expr.contains("Expr"));
    }

    #[test]
    fn praat_row_fields() {
        let l = lang(SYN);
        let infix = node_fields(&l, "ExprInfix").unwrap();
        assert!(infix.iter().any(|f| f.label == "lhs"));
        assert!(infix.iter().any(|f| f.label == "rhs" && f.skip == 1));
        let postfix = node_fields(&l, "ExprPostfix").unwrap();
        assert!(postfix
            .iter()
            .any(|f| f.label == "args" && matches!(&f.target, FieldTarget::Node(n) if n == "Args")));
        assert!(node_fields(&l, "Expr").is_none(), "praat rule itself is transparent");
        assert!(node_fields(&l, "Body").is_none(), "enum rule is transparent");
    }
}
