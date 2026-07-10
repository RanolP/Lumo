//! `langc check`: every static guarantee over the merged, DCE'd
//! definition. Collision errors are emitted earlier, during merge.

use std::collections::{BTreeMap, BTreeSet};

use crate::diag::Diagnostic;
use crate::project::first::{first_sets, shape_first, FirstSets};
use crate::project::model::{Definition, Language, RuleDef, START_RULE};
use crate::project::praat::{classify_row, RowKind, TailPart};
use crate::syntax::ast::{RuleBody, Shape, ShapeKind, StageKind, TokenPattern};

pub fn check_definition(def: &Definition) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    check_pipelines(def, &mut diags);
    for (lang_name, lang) in &def.languages {
        check_language(lang_name, lang, &mut diags);
    }
    diags.sort();
    diags
}

fn check_pipelines(def: &Definition, diags: &mut Vec<Diagnostic>) {
    if def.pipelines.is_empty() {
        // Diagnostic against the project as a whole; no single file owns it.
        diags.push(Diagnostic::warning(
            "<project>",
            langue_rt::Span::default(),
            "no manifest pipeline found — DCE has no roots (D-27)",
        ));
        return;
    }
    for pipeline in def.pipelines.values() {
        match pipeline.stages.first().map(|s| &s.kind) {
            Some(StageKind::Parse { .. }) => {}
            _ => diags.push(Diagnostic::error(
                &pipeline.origin.file,
                pipeline.origin.span,
                format!("pipeline `{}` must start with a `parse` stage (D-33)", pipeline.name),
            )),
        }
        for stage in &pipeline.stages {
            let langs: Vec<&String> = match &stage.kind {
                StageKind::Parse { lang } => vec![lang],
                StageKind::Elab { from, to } => vec![from, to],
                StageKind::Judgment { lang, .. } => vec![lang],
            };
            for lang in langs {
                if !def.languages.contains_key(lang) {
                    diags.push(Diagnostic::error(
                        &pipeline.origin.file,
                        stage.span,
                        format!("unknown language `{lang}` in pipeline `{}`", pipeline.name),
                    ));
                }
            }
            if let StageKind::Parse { lang } = &stage.kind {
                if let Some(l) = def.languages.get(lang) {
                    if !l.rules.contains_key(START_RULE) {
                        diags.push(Diagnostic::error(
                            &pipeline.origin.file,
                            stage.span,
                            format!(
                                "language `{lang}` is parsed but declares no `{START_RULE}` start rule"
                            ),
                        ));
                    }
                }
            }
        }
    }
}

fn check_language(lang_name: &str, lang: &Language, diags: &mut Vec<Diagnostic>) {
    check_tokens(lang_name, lang, diags);
    check_kind_names(lang_name, lang, diags);

    let sets = first_sets(lang);
    for rule in lang.rules.values() {
        match &rule.body {
            RuleBody::Plain(shape) => {
                let mut labels = BTreeMap::new();
                check_shape(lang_name, lang, rule, shape, &sets, &mut labels, diags);
            }
            RuleBody::Praat(_) => check_praat(lang_name, lang, rule, &sets, diags),
        }
    }

    for (rule_name, origin) in &lang.extern_recovers {
        if !lang.rules.contains_key(rule_name) {
            diags.push(Diagnostic::error(
                &origin.file,
                origin.span,
                format!("extern recover names unknown rule `{rule_name}` in language `{lang_name}`"),
            ));
        }
    }
}

/// Tokens, rules, and synthesized praat row kinds share one generated
/// `SyntaxKind` enum — their uppercased names must be distinct (`ident`
/// vs `Ident` both map to `IDENT`).
fn check_kind_names(lang_name: &str, lang: &Language, diags: &mut Vec<Diagnostic>) {
    use crate::codegen::naming::kind_name;
    use crate::codegen::syntax_kind::praat_kinds;

    let mut seen: BTreeMap<String, (String, crate::project::model::Origin)> = BTreeMap::new();
    let mut visit =
        |kind: String, what: String, origin: &crate::project::model::Origin, diags: &mut Vec<Diagnostic>| {
            if let Some((prev_what, prev)) = seen.insert(kind.clone(), (what.clone(), origin.clone())) {
                diags.push(Diagnostic::error(
                    &origin.file,
                    origin.span,
                    format!(
                        "{what} and {prev_what} (at {}:{}) both generate SyntaxKind `{kind}` in language `{lang_name}`",
                        prev.file, prev.span
                    ),
                ));
            }
        };
    for token in lang.tokens.values() {
        visit(kind_name(&token.name), format!("token `{}`", token.name), &token.origin, diags);
    }
    for rule in lang.rules.values() {
        visit(kind_name(&rule.name), format!("rule `{}`", rule.name), &rule.origin, diags);
        if let RuleBody::Praat(praat) = &rule.body {
            for row_kind in praat_kinds(&rule.name, praat) {
                visit(
                    kind_name(&row_kind),
                    format!("praat rows of `{}`", rule.name),
                    &rule.origin,
                    diags,
                );
            }
        }
    }
}

fn check_tokens(lang_name: &str, lang: &Language, diags: &mut Vec<Diagnostic>) {
    // The generated lexer builds ONE dense DFA over the whole escaped
    // pattern table — validate exactly that construction here, not just
    // each pattern in isolation.
    let order = crate::codegen::token_order(lang);
    if !order.is_empty() {
        let patterns: Vec<String> = order
            .iter()
            .map(|t| match &t.pattern {
                TokenPattern::Literal(text) => langue_rt::regex_escape(text),
                TokenPattern::Regex(pattern) => pattern.clone(),
            })
            .collect();
        let refs: Vec<&str> = patterns.iter().map(String::as_str).collect();
        if let Err(e) = langue_rt::LexDfa::try_build(&refs) {
            diags.push(Diagnostic::error(
                &order[0].origin.file,
                langue_rt::Span::default(),
                format!("cannot build the lexer DFA for language `{lang_name}`: {e}"),
            ));
        }
    }
    // Two tokens with the same literal pattern make `'x'` in grammar (and
    // the lexer tie-break) ambiguous.
    let mut by_literal: BTreeMap<&str, &str> = BTreeMap::new();
    for token in lang.tokens.values() {
        match &token.pattern {
            TokenPattern::Literal(text) => {
                if let Some(first) = by_literal.insert(text, &token.name) {
                    diags.push(Diagnostic::error(
                        &token.origin.file,
                        token.origin.span,
                        format!(
                            "tokens `{first}` and `{}` in language `{lang_name}` both declare the literal `'{text}'`",
                            token.name
                        ),
                    ));
                }
            }
            TokenPattern::Regex(pattern) => {
                // Validate with the same backend the generated lexer uses
                // (dense DFA — rejects look-around etc., D-09).
                if let Err(e) = regex_automata::dfa::dense::DFA::new(pattern) {
                    diags.push(Diagnostic::error(
                        &token.origin.file,
                        token.origin.span,
                        format!("invalid regex for token `{}`: {e}", token.name),
                    ));
                }
            }
        }
    }
}

fn check_shape(
    lang_name: &str,
    lang: &Language,
    rule: &RuleDef,
    shape: &Shape,
    sets: &FirstSets,
    labels: &mut BTreeMap<String, ()>,
    diags: &mut Vec<Diagnostic>,
) {
    match &shape.kind {
        ShapeKind::Seq(parts) => {
            for p in parts {
                check_shape(lang_name, lang, rule, p, sets, labels, diags);
            }
        }
        ShapeKind::Alt(arms) => {
            // LL(1): arms must be first-token disjoint.
            let mut seen: BTreeMap<String, usize> = BTreeMap::new();
            for (i, arm) in arms.iter().enumerate() {
                for token in shape_first(lang, sets, arm).tokens {
                    if let Some(prev) = seen.insert(token.clone(), i) {
                        diags.push(Diagnostic::error(
                            &rule.origin.file,
                            arm.span,
                            format!(
                                "in rule `{}`: alternative arms {} and {} can both start with token `{token}` — the generated parser is LL(1)",
                                rule.name, prev + 1, i + 1
                            ),
                        ));
                    }
                }
                check_shape(lang_name, lang, rule, arm, sets, labels, diags);
            }
        }
        ShapeKind::Opt(inner) => {
            check_shape(lang_name, lang, rule, inner, sets, labels, diags);
        }
        ShapeKind::Rep(inner) => {
            // A repetition of a possibly-empty shape would loop forever.
            if shape_first(lang, sets, inner).nullable {
                diags.push(Diagnostic::error(
                    &rule.origin.file,
                    shape.span,
                    format!("repetition of a possibly-empty shape in rule `{}`", rule.name),
                ));
            }
            check_shape(lang_name, lang, rule, inner, sets, labels, diags);
        }
        ShapeKind::Label { label, shape: inner } => {
            if labels.insert(label.clone(), ()).is_some() {
                diags.push(Diagnostic::error(
                    &rule.origin.file,
                    shape.span,
                    format!("duplicate label `{label}` in rule `{}`", rule.name),
                ));
            }
            check_shape(lang_name, lang, rule, inner, sets, labels, diags);
        }
        ShapeKind::Lit(text) => check_literal(lang_name, lang, rule, shape.span, text, diags),
        ShapeKind::TokenRef(name) => match lang.tokens.get(name) {
            None => diags.push(Diagnostic::error(
                &rule.origin.file,
                shape.span,
                format!("unknown token `{name}` in rule `{}` of language `{lang_name}`", rule.name),
            )),
            Some(t) if t.is_trivia => diags.push(Diagnostic::error(
                &rule.origin.file,
                shape.span,
                format!("trivia token `{name}` cannot appear in grammar (rule `{}`)", rule.name),
            )),
            Some(_) => {}
        },
        ShapeKind::NodeRef(name) => {
            if !lang.rules.contains_key(name) {
                diags.push(Diagnostic::error(
                    &rule.origin.file,
                    shape.span,
                    format!("unknown rule `{name}` in rule `{}` of language `{lang_name}`", rule.name),
                ));
            }
        }
        ShapeKind::Sep { item, sep } => {
            check_literal(lang_name, lang, rule, shape.span, sep, diags);
            if shape_first(lang, sets, item).nullable {
                diags.push(Diagnostic::error(
                    &rule.origin.file,
                    shape.span,
                    format!("sep() over a possibly-empty shape in rule `{}`", rule.name),
                ));
            }
            check_shape(lang_name, lang, rule, item, sets, labels, diags);
        }
    }
}

fn check_literal(
    lang_name: &str,
    lang: &Language,
    rule: &RuleDef,
    span: langue_rt::Span,
    text: &str,
    diags: &mut Vec<Diagnostic>,
) {
    if lang.literal_token(text).is_none() {
        diags.push(Diagnostic::error(
            &rule.origin.file,
            span,
            format!(
                "no token in language `{lang_name}` declares the literal `'{text}'` (used in rule `{}`)",
                rule.name
            ),
        ));
    }
}

fn check_praat(
    lang_name: &str,
    lang: &Language,
    rule: &RuleDef,
    sets: &FirstSets,
    diags: &mut Vec<Diagnostic>,
) {
    let RuleBody::Praat(praat) = &rule.body else { unreachable!() };

    // Atom alternatives must exist and be LL(1)-disjoint.
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (i, (atom, span)) in praat.simple.iter().enumerate() {
        match lang.rules.get(atom) {
            None => {
                diags.push(Diagnostic::error(
                    &rule.origin.file,
                    *span,
                    format!("unknown rule `{atom}` in `simple` of praat rule `{}`", rule.name),
                ));
                continue;
            }
            Some(atom_rule) => {
                let f = match &atom_rule.body {
                    RuleBody::Plain(shape) => shape_first(lang, sets, shape),
                    RuleBody::Praat(_) => sets.get(atom).cloned().unwrap_or_default(),
                };
                for token in f.tokens {
                    if let Some(prev) = seen.insert(token.clone(), i) {
                        diags.push(Diagnostic::error(
                            &rule.origin.file,
                            *span,
                            format!(
                                "in praat rule `{}`: simple atoms {} and {} can both start with token `{token}`",
                                rule.name, prev + 1, i + 1
                            ),
                        ));
                    }
                }
            }
        }
    }

    let mut lead_tokens_by_position: BTreeMap<(bool, String), usize> = BTreeMap::new();
    for (i, row) in praat.rows.iter().enumerate() {
        let kind = match classify_row(row) {
            Ok(kind) => kind,
            Err(msg) => {
                diags.push(Diagnostic::error(&rule.origin.file, row.span, msg));
                continue;
            }
        };
        if let RowKind::Infix { lbp, rbp, .. } | RowKind::Mixfix { lbp, rbp, .. } = &kind {
            if lbp == rbp {
                diags.push(Diagnostic::error(
                    &rule.origin.file,
                    row.span,
                    format!(
                        "row {} of praat rule `{}` has equal binding powers @{lbp} — associativity is ambiguous",
                        i + 1,
                        rule.name
                    ),
                ));
            }
        }
        // A token may lead at most one prefix row (atom position) and one
        // non-prefix row (loop position).
        let is_prefix = matches!(kind, RowKind::Prefix { .. });
        let mut check_tok = |tok: &String| {
            if let Some(prev) = lead_tokens_by_position.insert((is_prefix, tok.clone()), i) {
                diags.push(Diagnostic::error(
                    &rule.origin.file,
                    row.span,
                    format!(
                        "rows {} and {} of praat rule `{}` both use `'{tok}'` in {} position",
                        prev + 1,
                        i + 1,
                        rule.name,
                        if is_prefix { "prefix" } else { "operator" }
                    ),
                ));
            }
        };
        for tok in kind.lead_toks() {
            check_tok(tok);
        }
        // Every row token must be a declared literal; node payloads in a
        // postfix tail must be rules.
        let all_toks: Vec<&String> = match &kind {
            RowKind::Prefix { toks, .. } | RowKind::Infix { toks, .. } => toks.iter().collect(),
            RowKind::Postfix { tail, .. } => tail
                .iter()
                .flat_map(|p| match p {
                    TailPart::Toks(toks) => toks.iter(),
                    TailPart::Node(_) => [].iter(),
                })
                .collect(),
            RowKind::Mixfix { head, inner, .. } => {
                head.iter().chain(inner.iter().flat_map(|(_, t)| t.iter())).collect()
            }
        };
        for tok in all_toks {
            check_literal(lang_name, lang, rule, row.span, tok, diags);
        }
        if let RowKind::Postfix { tail, .. } = &kind {
            for part in tail {
                if let TailPart::Node(name) = part {
                    if !lang.rules.contains_key(name) {
                        diags.push(Diagnostic::error(
                            &rule.origin.file,
                            row.span,
                            format!(
                                "unknown rule `{name}` in postfix row of praat rule `{}`",
                                rule.name
                            ),
                        ));
                    }
                }
            }
        }
    }

    // Prefix row tokens must not collide with the atoms' first tokens
    // (atom dispatch is LL(1) too).
    let mut atom_firsts = BTreeSet::new();
    for (atom, _) in &praat.simple {
        if let Some(f) = sets.get(atom) {
            atom_firsts.extend(f.tokens.iter().cloned());
        }
    }
    for row in &praat.rows {
        if let Ok(RowKind::Prefix { toks, .. }) = classify_row(row) {
            for tok in &toks {
                if let Some(t) = lang.literal_token(tok) {
                    if atom_firsts.contains(&t.name) {
                        diags.push(Diagnostic::error(
                            &rule.origin.file,
                            row.span,
                            format!(
                                "prefix operator `'{tok}'` of praat rule `{}` collides with a simple atom's first token",
                                rule.name
                            ),
                        ));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::dce::dce;
    use crate::project::loader::{FileKind, LoadedFile};
    use crate::project::merge::merge_project;

    fn check_syn(text: &str) -> Vec<String> {
        let (def, merge_diags) = merge_project(&[LoadedFile {
            path: "L.syn.langue".into(),
            kind: FileKind::Syn { language: "L".into() },
            text: text.into(),
        }]);
        assert!(merge_diags.is_empty(), "{merge_diags:?}");
        check_definition(&dce(&def))
            .into_iter()
            .filter(|d| d.severity == crate::diag::Severity::Error)
            .map(|d| d.message)
            .collect()
    }

    #[test]
    fn clean_grammar_passes() {
        let msgs = check_syn("\
token kw.fn = 'fn'
token ident = /[a-z]+/
File = FnDecl*
FnDecl = 'fn' name:ident
");
        assert!(msgs.is_empty(), "{msgs:?}");
    }

    #[test]
    fn unknown_refs_and_undeclared_literal() {
        let msgs = check_syn("token ident = /[a-z]+/\nFile = 'fn' Missing other");
        assert!(msgs.iter().any(|m| m.contains("literal `'fn'`")), "{msgs:?}");
        assert!(msgs.iter().any(|m| m.contains("unknown rule `Missing`")));
        assert!(msgs.iter().any(|m| m.contains("unknown token `other`")));
    }

    #[test]
    fn duplicate_label_and_trivia_in_grammar() {
        let msgs = check_syn("\
token ident = /[a-z]+/
trivia ws = / +/
File = a:ident a:ident ws
");
        assert!(msgs.iter().any(|m| m.contains("duplicate label `a`")), "{msgs:?}");
        assert!(msgs.iter().any(|m| m.contains("trivia token `ws`")));
    }

    #[test]
    fn invalid_regex_reported() {
        let msgs = check_syn("token bad = /[unclosed/\nFile = bad");
        assert!(msgs.iter().any(|m| m.contains("invalid regex")), "{msgs:?}");
    }

    #[test]
    fn ll1_overlap_reported() {
        let msgs = check_syn("\
token kw.fn = 'fn'
token ident = /[a-z]+/
File = A | B
A = 'fn' name:ident
B = 'fn'
");
        assert!(
            msgs.iter().any(|m| m.contains("can both start with token `kw.fn`")),
            "{msgs:?}"
        );
    }

    #[test]
    fn praat_equal_bp_is_error() {
        let msgs = check_syn("\
token ident = /[a-z]+/
token op.plus = '+'
File = Expr
Ident = name:ident
Expr = praat {
  simple = Ident
  operators {
    @70 '+' @70,
  }
}
");
        assert!(msgs.iter().any(|m| m.contains("equal binding powers")), "{msgs:?}");
    }

    #[test]
    fn manifest_checks() {
        let files = [
            LoadedFile {
                path: "L.syn.langue".into(),
                kind: FileKind::Syn { language: "L".into() },
                text: "token x = 'x'\nStart = 'x'".into(),
            },
            LoadedFile {
                path: "p.langue".into(),
                kind: FileKind::Manifest,
                text: "main = parse L | elab L to Ghost".into(),
            },
        ];
        let (def, merge_diags) = merge_project(&files);
        assert!(merge_diags.is_empty());
        let msgs: Vec<String> =
            check_definition(&dce(&def)).into_iter().map(|d| d.message).collect();
        assert!(msgs.iter().any(|m| m.contains("unknown language `Ghost`")), "{msgs:?}");
        assert!(msgs.iter().any(|m| m.contains("no `File` start rule")), "{msgs:?}");
    }
}
