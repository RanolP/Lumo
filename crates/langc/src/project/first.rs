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

/// FOLLOW sets (token names that may come right after each rule) — the
/// sync targets for default `extern recover` hooks (D-02). Praat rules
/// are approximated: every row token may follow an operand or atom.
pub fn follow_sets(lang: &Language, firsts: &FirstSets) -> BTreeMap<String, BTreeSet<String>> {
    let mut follow: BTreeMap<String, BTreeSet<String>> =
        lang.rules.keys().map(|n| (n.clone(), BTreeSet::new())).collect();
    loop {
        let mut changed = false;
        for (name, rule) in &lang.rules {
            let cur = follow.get(name).cloned().unwrap_or_default();
            match &rule.body {
                RuleBody::Plain(shape) => {
                    visit_follow(lang, firsts, shape, &cur, &mut follow, &mut changed);
                }
                RuleBody::Praat(praat) => {
                    let mut after = cur.clone();
                    for row in &praat.rows {
                        for (i, elem) in row.elems.iter().enumerate() {
                            match elem {
                                OpElem::Toks(toks) => {
                                    for t in toks {
                                        add_literal(lang, t, &mut after);
                                    }
                                }
                                // A node payload is followed by the next
                                // token group in its row (`CallArgs` by `)`).
                                OpElem::Node(payload) => {
                                    let mut payload_after = BTreeSet::new();
                                    if let Some(OpElem::Toks(next)) = row.elems.get(i + 1) {
                                        for t in next {
                                            add_literal(lang, t, &mut payload_after);
                                        }
                                    } else {
                                        payload_after.extend(cur.iter().cloned());
                                    }
                                    extend(&mut follow, payload, &payload_after, &mut changed);
                                }
                                OpElem::Operand(_) => {}
                            }
                        }
                    }
                    // Operands are this rule again; atoms inherit too.
                    extend(&mut follow, name, &after, &mut changed);
                    for (atom, _) in &praat.simple {
                        extend(&mut follow, atom, &after, &mut changed);
                    }
                }
            }
        }
        if !changed {
            return follow;
        }
    }
}

fn extend(
    follow: &mut BTreeMap<String, BTreeSet<String>>,
    rule: &str,
    tokens: &BTreeSet<String>,
    changed: &mut bool,
) {
    if let Some(set) = follow.get_mut(rule) {
        for t in tokens {
            *changed |= set.insert(t.clone());
        }
    }
}

/// Walk `shape` given the tokens that may follow the whole shape.
fn visit_follow(
    lang: &Language,
    firsts: &FirstSets,
    shape: &Shape,
    after: &BTreeSet<String>,
    follow: &mut BTreeMap<String, BTreeSet<String>>,
    changed: &mut bool,
) {
    match &shape.kind {
        ShapeKind::Seq(parts) => {
            for (i, part) in parts.iter().enumerate() {
                let mut part_after = BTreeSet::new();
                let mut nullable_rest = true;
                for rest in &parts[i + 1..] {
                    let f = shape_first(lang, firsts, rest);
                    part_after.extend(f.tokens);
                    if !f.nullable {
                        nullable_rest = false;
                        break;
                    }
                }
                if nullable_rest {
                    part_after.extend(after.iter().cloned());
                }
                visit_follow(lang, firsts, part, &part_after, follow, changed);
            }
        }
        ShapeKind::Alt(arms) => {
            for arm in arms {
                visit_follow(lang, firsts, arm, after, follow, changed);
            }
        }
        ShapeKind::Opt(inner) => visit_follow(lang, firsts, inner, after, follow, changed),
        ShapeKind::Rep(inner) => {
            let mut rep_after = shape_first(lang, firsts, inner).tokens;
            rep_after.extend(after.iter().cloned());
            visit_follow(lang, firsts, inner, &rep_after, follow, changed);
        }
        ShapeKind::Label { shape: inner, .. } => {
            visit_follow(lang, firsts, inner, after, follow, changed);
        }
        ShapeKind::NodeRef(name) => extend(follow, name, after, changed),
        ShapeKind::Sep { item, sep } => {
            let mut item_after = after.clone();
            if let Some(t) = lang.literal_token(sep) {
                item_after.insert(t.name.clone());
            }
            visit_follow(lang, firsts, item, &item_after, follow, changed);
        }
        ShapeKind::Lit(_) | ShapeKind::TokenRef(_) => {}
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
