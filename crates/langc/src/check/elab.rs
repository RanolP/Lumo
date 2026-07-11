//! Static checks over the merged elab definitions (D-13/D-14/D-28/D-35):
//! languages and nodes exist, fields are real syn labels, metavariables
//! bind before use, `to` recursion stays inside the block's pair, and
//! same-root rules must be literal/ctor-disjoint (conflict detection).

use std::collections::BTreeMap;

use crate::diag::Diagnostic;
use crate::project::fields::{kind_set, node_fields, node_info, Field, FieldTarget};
use crate::project::model::{Definition, ElabDef, Language, Origin};
use crate::syntax::ast::{Con, Pat, StageKind};

pub fn check_elabs(def: &Definition, diags: &mut Vec<Diagnostic>) {
    for ((from, to), elab) in &def.elabs {
        check_pair(def, from, to, elab, diags);
    }
    for (lang_name, between) in &def.betweens {
        let Some(lang) = def.languages.get(lang_name) else {
            if let Some(rel) = between.relations.first() {
                diags.push(Diagnostic::error(
                    &rel.origin.file,
                    rel.origin.span,
                    format!("unknown language `{lang_name}` in `between` block"),
                ));
            }
            continue;
        };
        for rel in &between.relations {
            let mut bindings = BTreeMap::new();
            // Nonlinear patterns are fine here: egglog reads a repeated
            // metavariable as an equality constraint.
            check_pattern_root(lang_name, lang, &rel.lhs, &rel.origin, true, &mut bindings, diags);
            check_con(
                &ConCtx { lang_name, lang, rec_target: None, bindings: &bindings },
                &rel.rhs,
                None,
                &rel.origin,
                diags,
            );
        }
    }
    // An `elab A to B` stage without rules elaborates nothing — warn.
    for pipeline in def.pipelines.values() {
        for stage in &pipeline.stages {
            if let StageKind::Elab { from, to } = &stage.kind {
                if !def.elabs.contains_key(&(from.clone(), to.clone())) {
                    diags.push(Diagnostic::warning(
                        &pipeline.origin.file,
                        stage.span,
                        format!("no `from {from} to {to}` rules are defined"),
                    ));
                }
            }
        }
    }
}

fn check_pair(
    def: &Definition,
    from: &str,
    to: &str,
    elab: &ElabDef,
    diags: &mut Vec<Diagnostic>,
) {
    let origin = elab
        .rules
        .first()
        .map(|r| &r.origin)
        .or_else(|| elab.extern_rules.first().map(|(_, o)| o));
    let Some(origin) = origin else { return };
    let mut missing = false;
    for lang in [from, to] {
        if !def.languages.contains_key(lang) {
            diags.push(Diagnostic::error(
                &origin.file,
                origin.span,
                format!("unknown language `{lang}` in `from {from} to {to}`"),
            ));
            missing = true;
        }
    }
    if missing {
        return;
    }
    let from_lang = &def.languages[from];
    let to_lang = &def.languages[to];

    for rule in &elab.rules {
        let mut bindings = BTreeMap::new();
        check_pattern_root(from, from_lang, &rule.pattern, &rule.origin, false, &mut bindings, diags);
        check_con(
            &ConCtx { lang_name: to, lang: to_lang, rec_target: Some(to), bindings: &bindings },
            &rule.construction,
            None,
            &rule.origin,
            diags,
        );
    }

    // Conflict detection (D-13): two rules on the same root kind must be
    // statically disjoint — no ordering, no priority.
    let mut by_root: BTreeMap<&str, Vec<(usize, &Origin)>> = BTreeMap::new();
    for (i, rule) in elab.rules.iter().enumerate() {
        if let Pat::Node { name, .. } = &rule.pattern {
            by_root.entry(name).or_default().push((i, &rule.origin));
        }
    }
    for (root, rules) in by_root {
        for (a, &(i, _)) in rules.iter().enumerate() {
            for &(j, origin_j) in &rules[a + 1..] {
                if !disjoint(&elab.rules[i].pattern, &elab.rules[j].pattern) {
                    diags.push(Diagnostic::error(
                        &origin_j.file,
                        origin_j.span,
                        format!(
                            "conflicting rules on `{root}`: this rule and the one at {}:{} \
                             can fire on the same input (rules must be literal/ctor-disjoint, D-13)",
                            elab.rules[i].origin.file, elab.rules[i].origin.span
                        ),
                    ));
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Binding {
    TokenScalar,
    NodeScalar,
    TokenList,
    NodeList,
}

impl Binding {
    fn is_list(self) -> bool {
        matches!(self, Binding::TokenList | Binding::NodeList)
    }
}

/// The root of a pattern must be a concrete node — dispatch is by root
/// kind, and it makes every capture a strict subtree (D-28).
/// `nonlinear_ok`: between relations may bind a metavariable twice at
/// the same shape (an egglog equality constraint); accessor-scheme
/// from-rules cannot.
fn check_pattern_root(
    lang_name: &str,
    lang: &Language,
    pat: &Pat,
    origin: &Origin,
    nonlinear_ok: bool,
    bindings: &mut BTreeMap<String, Binding>,
    diags: &mut Vec<Diagnostic>,
) {
    match pat {
        Pat::Node { .. } => {
            check_pat(lang_name, lang, pat, None, origin, nonlinear_ok, bindings, diags)
        }
        other => diags.push(Diagnostic::error(
            &origin.file,
            other.span(),
            "a rule pattern must start with a node (dispatch is by root kind)",
        )),
    }
}

/// `expected`: the field this pattern sits in (None at the root).
fn check_pat(
    lang_name: &str,
    lang: &Language,
    pat: &Pat,
    expected: Option<&Field>,
    origin: &Origin,
    nonlinear_ok: bool,
    bindings: &mut BTreeMap<String, Binding>,
    diags: &mut Vec<Diagnostic>,
) {
    let error = |diags: &mut Vec<Diagnostic>, span, msg: String| {
        diags.push(Diagnostic::error(&origin.file, span, msg));
    };
    if let Some(field) = expected {
        let ok = match pat {
            Pat::ListVar { .. } => field.many,
            _ => !field.many,
        };
        if !ok {
            let msg = if field.many {
                format!("field `{}` is a list — capture it with `[$x*]`", field.label)
            } else {
                format!("field `{}` is not a list — `[$x*]` cannot match it", field.label)
            };
            error(diags, pat.span(), msg);
            return;
        }
    }
    match pat {
        Pat::Var { name, span } | Pat::ListVar { name, span } => {
            let is_list = matches!(pat, Pat::ListVar { .. });
            let is_token = !matches!(
                expected,
                None | Some(Field { target: FieldTarget::Node(_), .. })
            );
            let kind = match (is_token, is_list) {
                (true, false) => Binding::TokenScalar,
                (true, true) => Binding::TokenList,
                (false, false) => Binding::NodeScalar,
                (false, true) => Binding::NodeList,
            };
            if let Some(prev) = bindings.insert(name.clone(), kind) {
                if !(nonlinear_ok && prev == kind) {
                    error(diags, *span, format!("metavariable `${name}` is bound twice"));
                }
            }
        }
        Pat::Lit { text, span } => {
            if let Some(Field { target: FieldTarget::Node(rule), .. }) = expected {
                error(
                    diags,
                    *span,
                    format!("literal `'{text}'` cannot match a `{rule}` node field"),
                );
            }
        }
        Pat::Node { lang: qual, name, fields, span } => {
            if let Some(qual) = qual {
                if qual != lang_name {
                    error(
                        diags,
                        *span,
                        format!("pattern node `{qual}::{name}` is not in the source language `{lang_name}`"),
                    );
                    return;
                }
            }
            if let Some(field) = expected {
                match &field.target {
                    FieldTarget::Node(rule) => {
                        if !kind_set(lang, rule).contains(name) {
                            error(
                                diags,
                                *span,
                                format!(
                                    "node `{name}` cannot appear in field `{}` (expects `{rule}`)",
                                    field.label
                                ),
                            );
                            return;
                        }
                    }
                    FieldTarget::Token(_) | FieldTarget::LitToken(_) => {
                        error(
                            diags,
                            *span,
                            format!("field `{}` holds a token, not a `{name}` node", field.label),
                        );
                        return;
                    }
                }
            }
            let Some(node_fields) = node_fields(lang, name) else {
                error(
                    diags,
                    *span,
                    format!(
                        "`{name}` is not a concrete node of language `{lang_name}` \
                         (unknown, or a transparent alternative rule)"
                    ),
                );
                return;
            };
            for (label, sub) in fields {
                let Some(field) = node_fields.iter().find(|f| &f.label == label) else {
                    error(
                        diags,
                        sub.span(),
                        format!("node `{name}` has no field `{label}`"),
                    );
                    continue;
                };
                check_pat(lang_name, lang, sub, Some(field), origin, nonlinear_ok, bindings, diags);
            }
        }
    }
}

struct ConCtx<'c> {
    lang_name: &'c str,
    lang: &'c Language,
    /// `Some(to)` in from-blocks — `$x to L` must target it. `None` in
    /// between relations — no recursion there.
    rec_target: Option<&'c str>,
    bindings: &'c BTreeMap<String, Binding>,
}

fn check_con(
    ctx: &ConCtx,
    con: &Con,
    expected: Option<&Field>,
    origin: &Origin,
    diags: &mut Vec<Diagnostic>,
) {
    let error = |diags: &mut Vec<Diagnostic>, span, msg: String| {
        diags.push(Diagnostic::error(&origin.file, span, msg));
    };
    if let Some(field) = expected {
        let ok = match con {
            Con::ListVarTo { .. } => field.many,
            Con::Var { name, .. } => {
                ctx.bindings.get(name).is_none_or(|b| b.is_list() == field.many)
            }
            _ => !field.many,
        };
        if !ok {
            let msg = if field.many {
                format!("field `{}` is a list — build it with `[$x* to L]`", field.label)
            } else {
                format!("field `{}` is not a list", field.label)
            };
            error(diags, con.span(), msg);
        }
    }
    let check_bound = |diags: &mut Vec<Diagnostic>, name: &str, span, want: Binding| {
        match ctx.bindings.get(name) {
            None => error(
                diags,
                span,
                format!("metavariable `${name}` is not bound by the pattern"),
            ),
            Some(kind) if *kind != want => {
                let msg = match want {
                    _ if kind.is_list() && !want.is_list() => format!(
                        "`${name}` is bound as a list — use `[${name}* to L]`"
                    ),
                    _ if !kind.is_list() && want.is_list() => {
                        format!("`${name}` is not bound as a list")
                    }
                    Binding::NodeScalar | Binding::NodeList => format!(
                        "`${name}` is bound to a token — only node captures can recurse"
                    ),
                    _ => format!("`${name}` has the wrong binding kind here"),
                };
                error(diags, span, msg);
            }
            Some(_) => {}
        }
    };
    match con {
        Con::Var { name, span } => {
            match ctx.bindings.get(name) {
                None => error(
                    diags,
                    *span,
                    format!("metavariable `${name}` is not bound by the pattern"),
                ),
                Some(b) if b.is_list() && expected.is_none() => error(
                    diags,
                    *span,
                    format!("`${name}` is a list — it cannot be the whole construction"),
                ),
                Some(_) => {}
            }
        }
        Con::VarTo { name, lang, span } => match ctx.rec_target {
            None => error(
                diags,
                *span,
                "`to` recursion is not available in `between` relations".to_owned(),
            ),
            Some(target) => {
                check_bound(diags, name, *span, Binding::NodeScalar);
                if lang != target {
                    error(
                        diags,
                        *span,
                        format!("recursion must target the block's target language `{target}`"),
                    );
                }
            }
        },
        Con::ListVarTo { name, lang, span } => match ctx.rec_target {
            None => error(
                diags,
                *span,
                "`to` recursion is not available in `between` relations".to_owned(),
            ),
            Some(target) => {
                check_bound(diags, name, *span, Binding::NodeList);
                if lang != target {
                    error(
                        diags,
                        *span,
                        format!("recursion must target the block's target language `{target}`"),
                    );
                }
            }
        },
        Con::Subst { target, var, replacement, span } => {
            check_bound(diags, target, *span, Binding::NodeScalar);
            for name in [var, replacement] {
                if !ctx.bindings.contains_key(name) {
                    error(
                        diags,
                        *span,
                        format!("metavariable `${name}` is not bound by the pattern"),
                    );
                }
            }
        }
        Con::Lit { text, span } => {
            if let Some(Field { target: FieldTarget::Node(rule), .. }) = expected {
                error(
                    diags,
                    *span,
                    format!("literal `'{text}'` cannot fill a `{rule}` node field"),
                );
            }
        }
        Con::Node { lang: qual, name, fields, span } => {
            if let Some(qual) = qual {
                if qual != ctx.lang_name {
                    error(
                        diags,
                        *span,
                        format!(
                            "construction node `{qual}::{name}` is not in the target language `{}`",
                            ctx.lang_name
                        ),
                    );
                    return;
                }
            }
            if let Some(Field { target: FieldTarget::Node(rule), label, .. }) = expected {
                if !kind_set(ctx.lang, rule).contains(name) {
                    error(
                        diags,
                        *span,
                        format!("node `{name}` cannot fill field `{label}` (expects `{rule}`)"),
                    );
                    return;
                }
            }
            let Some(node_fields) = node_fields(ctx.lang, name) else {
                error(
                    diags,
                    *span,
                    format!(
                        "`{name}` is not a concrete node of language `{}` \
                         (unknown, or a transparent alternative rule)",
                        ctx.lang_name
                    ),
                );
                return;
            };
            for (label, sub) in fields {
                let Some(field) = node_fields.iter().find(|f| &f.label == label) else {
                    error(diags, sub.span(), format!("node `{name}` has no field `{label}`"));
                    continue;
                };
                check_con(ctx, sub, Some(field), origin, diags);
            }
            // A struct construction must supply every required field —
            // an incomplete node would not reparse (praat rows are
            // handled by the builder's own row selection).
            if matches!(node_info(ctx.lang, name), Some(crate::project::fields::NodeInfo::Struct(_))) {
                for field in &node_fields {
                    if field.required && !fields.iter().any(|(l, _)| l == &field.label) {
                        error(
                            diags,
                            *span,
                            format!(
                                "construction of `{name}` is missing the required field `{}`",
                                field.label
                            ),
                        );
                    }
                }
            }
        }
    }
}

/// Can `a` and `b` never match the same input? Conservative: only
/// literal texts and constructor (node-kind) mismatches separate rules.
fn disjoint(a: &Pat, b: &Pat) -> bool {
    match (a, b) {
        (Pat::Lit { text: ta, .. }, Pat::Lit { text: tb, .. }) => ta != tb,
        (Pat::Lit { .. }, Pat::Node { .. }) | (Pat::Node { .. }, Pat::Lit { .. }) => true,
        (
            Pat::Node { name: na, fields: fa, .. },
            Pat::Node { name: nb, fields: fb, .. },
        ) => {
            if na != nb {
                return true;
            }
            fa.iter().any(|(label, pa)| {
                fb.iter().any(|(lb, pb)| label == lb && disjoint(pa, pb))
            })
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::dce::dce;
    use crate::project::loader::{FileKind, LoadedFile};
    use crate::project::merge::merge_project;

    const FROM_SYN: &str = "\
token kw.fn = 'fn'
token kw.extern = 'extern'
token ident = /[a-z]+/
token paren.open = '('
token paren.close = ')'
token comma = ','
File = decls:Decl*
Decl = 'fn' name:ident '(' params:sep(Param, ',')? ')' body:Ref
Param = name:ident ('(' ')')?
Ref = name:ident
";

    const TO_SYN: &str = "\
token kw.def = 'def'
token op.eq = '='
token ident = /[a-z]+/
File = defs:Def*
Def = 'def' name:ident '=' value:Var
Var = name:ident
";

    fn diags_for(elab_text: &str) -> Vec<String> {
        let files = [
            LoadedFile {
                path: "A.syn.langue".into(),
                kind: FileKind::Syn { language: "A".into() },
                text: FROM_SYN.into(),
            },
            LoadedFile {
                path: "B.syn.langue".into(),
                kind: FileKind::Syn { language: "B".into() },
                text: TO_SYN.into(),
            },
            LoadedFile {
                path: "a_to_b.elab.langue".into(),
                kind: FileKind::Elab,
                text: elab_text.into(),
            },
            LoadedFile {
                path: "p.langue".into(),
                kind: FileKind::Manifest,
                text: "main = parse A | elab A to B".into(),
            },
        ];
        let (def, merge_diags) = merge_project(&files);
        assert!(merge_diags.is_empty(), "{merge_diags:?}");
        crate::check::check_definition(&dce(&def))
            .into_iter()
            .filter(|d| d.severity == crate::diag::Severity::Error)
            .map(|d| d.message)
            .collect()
    }

    #[test]
    fn clean_rules_pass() {
        let msgs = diags_for(
            "from A to B {\n  Decl { name: $n, body: $b } ==> Def { name: $n, value: $b to B }\n  Ref { name: $x } ==> Var { name: $x }\n}",
        );
        assert!(msgs.is_empty(), "{msgs:?}");
    }

    #[test]
    fn unknown_node_and_field() {
        let msgs = diags_for(
            "from A to B {\n  Missing {} ==> Def { name: $n }\n  Decl { nope: $x } ==> Var { name: $x }\n}",
        );
        assert!(msgs.iter().any(|m| m.contains("`Missing` is not a concrete node")), "{msgs:?}");
        assert!(msgs.iter().any(|m| m.contains("no field `nope`")), "{msgs:?}");
    }

    #[test]
    fn unknown_elab_language_reachable_via_pipeline() {
        // A `from A to Ghost` block not referenced by any pipeline is
        // dead code and DCE'd (D-05); referenced, it must error.
        let files = [
            LoadedFile {
                path: "A.syn.langue".into(),
                kind: FileKind::Syn { language: "A".into() },
                text: FROM_SYN.into(),
            },
            LoadedFile {
                path: "a.elab.langue".into(),
                kind: FileKind::Elab,
                text: "from A to Ghost {\n  Param { name: $n } ==> Param { name: $n }\n}".into(),
            },
            LoadedFile {
                path: "p.langue".into(),
                kind: FileKind::Manifest,
                text: "main = parse A | elab A to Ghost".into(),
            },
        ];
        let (def, merge_diags) = merge_project(&files);
        assert!(merge_diags.is_empty(), "{merge_diags:?}");
        let msgs: Vec<String> = crate::check::check_definition(&dce(&def))
            .into_iter()
            .map(|d| d.message)
            .collect();
        assert!(
            msgs.iter().any(|m| m.contains("unknown language `Ghost` in `from A to Ghost`")),
            "{msgs:?}"
        );
    }

    #[test]
    fn unbound_and_double_bound_metavars() {
        let msgs = diags_for(
            "from A to B {\n  Decl { name: $n, body: $n } ==> Def { name: $n, value: $ghost to B }\n}",
        );
        assert!(msgs.iter().any(|m| m.contains("`$n` is bound twice")), "{msgs:?}");
        assert!(msgs.iter().any(|m| m.contains("`$ghost` is not bound")), "{msgs:?}");
    }

    #[test]
    fn recursion_must_target_block_target() {
        let msgs = diags_for(
            "from A to B {\n  Decl { name: $n, body: $b } ==> Def { name: $n, value: $b to A }\n}",
        );
        assert!(
            msgs.iter().any(|m| m.contains("must target the block's target language `B`")),
            "{msgs:?}"
        );
    }

    #[test]
    fn list_shape_mismatches() {
        let msgs = diags_for(
            "from A to B {\n  Decl { params: $p, name: [$n*] } ==> Def { name: $x }\n}",
        );
        assert!(
            msgs.iter().any(|m| m.contains("field `params` is a list")),
            "{msgs:?}"
        );
        assert!(
            msgs.iter().any(|m| m.contains("field `name` is not a list")),
            "{msgs:?}"
        );
    }

    #[test]
    fn conflicting_rules_rejected_disjoint_allowed() {
        // Same root, no discriminating field: conflict.
        let msgs = diags_for(
            "from A to B {\n  Ref { name: $x } ==> Var { name: $x }\n  Ref {} ==> Var { name: $x }\n}",
        );
        assert!(msgs.iter().any(|m| m.contains("conflicting rules on `Ref`")), "{msgs:?}");
        // Ctor-disjoint via nested node kinds under the same field is
        // exercised in the fields tests; literal disjointness:
        let msgs = diags_for(
            "from A to B {\n  Decl { name: 'fn' } ==> Def { name: $x }\n  Decl { name: 'extern' } ==> Def { name: $x }\n}",
        );
        // Unbound `$x` errors are present, but no conflict error.
        assert!(
            !msgs.iter().any(|m| m.contains("conflicting rules")),
            "{msgs:?}"
        );
    }

    #[test]
    fn required_construction_fields_enforced() {
        let msgs = diags_for(
            "from A to B {\n  Ref { name: $x } ==> Def { name: $x }\n}",
        );
        assert!(
            msgs.iter()
                .any(|m| m.contains("missing the required field `value`")),
            "{msgs:?}"
        );
    }

    #[test]
    fn between_checks() {
        let files = [
            LoadedFile {
                path: "B.syn.langue".into(),
                kind: FileKind::Syn { language: "B".into() },
                text: TO_SYN.into(),
            },
            LoadedFile {
                path: "b.elab.langue".into(),
                kind: FileKind::Elab,
                text: "between B {\n  Var { name: $x } === $y\n  Def { name: $n, value: $v } === $v to B\n}".into(),
            },
            LoadedFile {
                path: "p.langue".into(),
                kind: FileKind::Manifest,
                text: "main = parse B".into(),
            },
        ];
        let (def, merge_diags) = merge_project(&files);
        assert!(merge_diags.is_empty(), "{merge_diags:?}");
        let msgs: Vec<String> = crate::check::check_definition(&dce(&def))
            .into_iter()
            .map(|d| d.message)
            .collect();
        assert!(msgs.iter().any(|m| m.contains("`$y` is not bound")), "{msgs:?}");
        assert!(
            msgs.iter().any(|m| m.contains("not available in `between`")),
            "{msgs:?}"
        );
    }

    #[test]
    fn between_allows_nonlinear_patterns() {
        // A repeated metavariable is an egglog equality constraint —
        // legal in between relations, still an error in from-rules.
        let files = [
            LoadedFile {
                path: "B.syn.langue".into(),
                kind: FileKind::Syn { language: "B".into() },
                text: TO_SYN.into(),
            },
            LoadedFile {
                path: "b.elab.langue".into(),
                kind: FileKind::Elab,
                text: "between B {\n  Def { name: $x, value: Var { name: $x } } === Def { name: $x, value: Var { name: $x } }\n}".into(),
            },
            LoadedFile {
                path: "p.langue".into(),
                kind: FileKind::Manifest,
                text: "main = parse B".into(),
            },
        ];
        let (def, merge_diags) = merge_project(&files);
        assert!(merge_diags.is_empty(), "{merge_diags:?}");
        let msgs: Vec<String> = crate::check::check_definition(&dce(&def))
            .into_iter()
            .map(|d| d.message)
            .collect();
        assert!(!msgs.iter().any(|m| m.contains("bound twice")), "{msgs:?}");
    }
}
