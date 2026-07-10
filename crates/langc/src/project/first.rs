//! FIRST sets over a language's rules. Elements are token names (grammar
//! literals resolve to their declaring literal token). Used by the LL(1)
//! `|`-arm overlap check and by parser codegen (`can_parse_X`).

use std::collections::{BTreeMap, BTreeSet};

use crate::syntax::ast::{OpElem, RuleBody, Shape, ShapeKind};

use super::model::Language;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RuleFirst {
    /// Token names that can start this rule. Literals with no declaring
    /// token are skipped (check reports those separately).
    pub tokens: BTreeSet<String>,
    pub nullable: bool,
}

pub type FirstSets = BTreeMap<String, RuleFirst>;

/// Fixed-point FIRST computation. Terminates because sets only grow.
pub fn first_sets(lang: &Language) -> FirstSets {
    let mut sets: FirstSets =
        lang.rules.keys().map(|n| (n.clone(), RuleFirst::default())).collect();
    loop {
        let mut changed = false;
        for (name, rule) in &lang.rules {
            let computed = rule_first(lang, &sets, &rule.body);
            let entry = sets.get_mut(name).expect("pre-seeded");
            if *entry != computed {
                *entry = computed;
                changed = true;
            }
        }
        if !changed {
            return sets;
        }
    }
}

fn rule_first(lang: &Language, sets: &FirstSets, body: &RuleBody) -> RuleFirst {
    match body {
        RuleBody::Plain(shape) => shape_first(lang, sets, shape),
        RuleBody::Praat(praat) => {
            let mut out = RuleFirst::default();
            for (atom, _) in &praat.simple {
                if let Some(f) = sets.get(atom) {
                    out.tokens.extend(f.tokens.iter().cloned());
                    out.nullable |= f.nullable;
                }
            }
            // Prefix rows start an expression with their tokens.
            for row in &praat.rows {
                if let Some(OpElem::Toks(toks)) = row.elems.first() {
                    for t in toks {
                        add_literal(lang, t, &mut out.tokens);
                    }
                }
            }
            out
        }
    }
}

/// FIRST of one shape given the current per-rule approximations.
pub fn shape_first(lang: &Language, sets: &FirstSets, shape: &Shape) -> RuleFirst {
    match &shape.kind {
        ShapeKind::Seq(parts) => {
            let mut out = RuleFirst { tokens: BTreeSet::new(), nullable: true };
            for part in parts {
                let f = shape_first(lang, sets, part);
                out.tokens.extend(f.tokens);
                if !f.nullable {
                    out.nullable = false;
                    break;
                }
            }
            out
        }
        ShapeKind::Alt(arms) => {
            let mut out = RuleFirst::default();
            for arm in arms {
                let f = shape_first(lang, sets, arm);
                out.tokens.extend(f.tokens);
                out.nullable |= f.nullable;
            }
            out
        }
        ShapeKind::Opt(inner) | ShapeKind::Rep(inner) => {
            let mut out = shape_first(lang, sets, inner);
            out.nullable = true;
            out
        }
        ShapeKind::Label { shape, .. } => shape_first(lang, sets, shape),
        ShapeKind::Lit(text) => {
            let mut tokens = BTreeSet::new();
            add_literal(lang, text, &mut tokens);
            RuleFirst { tokens, nullable: false }
        }
        ShapeKind::TokenRef(name) => RuleFirst {
            tokens: BTreeSet::from([name.clone()]),
            nullable: false,
        },
        ShapeKind::NodeRef(name) => sets.get(name).cloned().unwrap_or_default(),
        // sep(X, s) is one-or-more; use `sep(...)?` for possibly-empty lists.
        ShapeKind::Sep { item, .. } => shape_first(lang, sets, item),
    }
}

fn add_literal(lang: &Language, text: &str, tokens: &mut BTreeSet<String>) {
    if let Some(t) = lang.literal_token(text) {
        tokens.insert(t.name.clone());
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

    #[test]
    fn first_through_refs_opts_and_praat() {
        let l = lang("\
token kw.fn = 'fn'
token kw.let = 'let'
token ident = /[a-z]+/
token op.minus = '-'
File = Item*
Item = FnDecl | LetDecl
FnDecl = 'fn' name:ident
LetDecl = 'let' name:ident '=' Expr
Ident = name:ident
Expr = praat {
  simple = Ident
  operators {
    '-' @100,
    @70 '-' @69,
  }
}
");
        let sets = first_sets(&l);
        let item = &sets["Item"];
        assert!(item.tokens.contains("kw.fn") && item.tokens.contains("kw.let"));
        assert!(!item.nullable);
        assert!(sets["File"].nullable);
        let expr = &sets["Expr"];
        assert!(expr.tokens.contains("ident"), "simple atoms");
        assert!(expr.tokens.contains("op.minus"), "prefix row token");
    }
}
