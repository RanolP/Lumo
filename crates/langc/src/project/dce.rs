//! DCE (D-05): unused items are eliminated when the definition is loaded.
//! Roots are the manifest pipelines — every language a stage mentions is
//! live; within a live language, rules reachable from `File` and the
//! tokens they use (plus all trivia) are live.

use std::collections::BTreeSet;

use crate::syntax::ast::{OpElem, RuleBody, Shape, ShapeKind, StageKind, TokenPattern};

use super::model::{Definition, Language, START_RULE};

/// Returns the pruned definition. With no pipelines there are no roots;
/// everything is kept so partial projects stay checkable (check reports
/// the missing manifest separately).
pub fn dce(def: &Definition) -> Definition {
    if def.pipelines.is_empty() {
        return def.clone();
    }

    let mut live_langs: BTreeSet<&str> = BTreeSet::new();
    for pipeline in def.pipelines.values() {
        for stage in &pipeline.stages {
            match &stage.kind {
                StageKind::Parse { lang } => {
                    live_langs.insert(lang);
                }
                StageKind::Elab { from, to } => {
                    live_langs.insert(from);
                    live_langs.insert(to);
                }
                StageKind::Judgment { lang, .. } => {
                    live_langs.insert(lang);
                }
            }
        }
    }

    let mut out = Definition { languages: Default::default(), pipelines: def.pipelines.clone() };
    for (name, lang) in &def.languages {
        if live_langs.contains(name.as_str()) {
            out.languages.insert(name.clone(), prune_language(lang));
        }
    }
    out
}

fn prune_language(lang: &Language) -> Language {
    // Without a start rule reachability has no root; keep everything and
    // let check report the missing `File`.
    if !lang.rules.contains_key(START_RULE) {
        return lang.clone();
    }

    let mut live_rules: BTreeSet<String> = BTreeSet::new();
    let mut live_tokens: BTreeSet<String> = BTreeSet::new();
    let mut work = vec![START_RULE.to_owned()];

    while let Some(rule_name) = work.pop() {
        if !live_rules.insert(rule_name.clone()) {
            continue;
        }
        let Some(rule) = lang.rules.get(&rule_name) else { continue };
        match &rule.body {
            RuleBody::Plain(shape) => {
                mark_shape(lang, shape, &mut live_tokens, &mut work);
            }
            RuleBody::Praat(praat) => {
                for (atom, _) in &praat.simple {
                    work.push(atom.clone());
                }
                for row in &praat.rows {
                    for elem in &row.elems {
                        match elem {
                            OpElem::Toks(toks) => {
                                for t in toks {
                                    mark_literal(lang, t, &mut live_tokens);
                                }
                            }
                            OpElem::Node(name) => work.push(name.clone()),
                            OpElem::Operand(_) => {}
                        }
                    }
                }
            }
        }
    }

    let mut out = Language::default();
    for (name, token) in &lang.tokens {
        // Trivia is attachable everywhere, so it is always live.
        if token.is_trivia || live_tokens.contains(name) {
            out.tokens.insert(name.clone(), token.clone());
        }
    }
    for (name, rule) in &lang.rules {
        if live_rules.contains(name) {
            out.rules.insert(name.clone(), rule.clone());
        }
    }
    for (name, origin) in &lang.extern_recovers {
        if live_rules.contains(name) {
            out.extern_recovers.insert(name.clone(), origin.clone());
        }
    }
    out
}

fn mark_shape(
    lang: &Language,
    shape: &Shape,
    live_tokens: &mut BTreeSet<String>,
    work: &mut Vec<String>,
) {
    match &shape.kind {
        ShapeKind::Seq(parts) | ShapeKind::Alt(parts) => {
            for p in parts {
                mark_shape(lang, p, live_tokens, work);
            }
        }
        ShapeKind::Opt(inner) | ShapeKind::Rep(inner) => {
            mark_shape(lang, inner, live_tokens, work);
        }
        ShapeKind::Label { shape, .. } => mark_shape(lang, shape, live_tokens, work),
        ShapeKind::Lit(text) => mark_literal(lang, text, live_tokens),
        ShapeKind::TokenRef(name) => {
            live_tokens.insert(name.clone());
        }
        ShapeKind::NodeRef(name) => work.push(name.clone()),
        ShapeKind::Sep { item, sep } => {
            mark_shape(lang, item, live_tokens, work);
            mark_literal(lang, sep, live_tokens);
        }
    }
}

fn mark_literal(lang: &Language, text: &str, live_tokens: &mut BTreeSet<String>) {
    if let Some(token) = lang
        .tokens
        .values()
        .find(|t| matches!(&t.pattern, TokenPattern::Literal(l) if l == text))
    {
        live_tokens.insert(token.name.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::loader::{FileKind, LoadedFile};
    use crate::project::merge::merge_project;

    fn project(syn_text: &str, manifest_text: &str) -> Definition {
        let files = [
            LoadedFile {
                path: "Lumo.syn.langue".into(),
                kind: FileKind::Syn { language: "Lumo".into() },
                text: syn_text.into(),
            },
            LoadedFile {
                path: "p.langue".into(),
                kind: FileKind::Manifest,
                text: manifest_text.into(),
            },
        ];
        let (def, diags) = merge_project(&files);
        assert!(diags.is_empty(), "{diags:?}");
        dce(&def)
    }

    const SYN: &str = "\
token kw.fn = 'fn'
token kw.junk = 'junk'
token ident = /[a-z]+/
token number = /[0-9]+/
trivia ws = /[ \\t]+/
File = 'fn' name:ident
Dead = value:number
";

    #[test]
    fn unreachable_rule_and_tokens_are_dropped() {
        let def = project(SYN, "main = parse Lumo");
        let lumo = &def.languages["Lumo"];
        assert!(lumo.rules.contains_key("File"));
        assert!(!lumo.rules.contains_key("Dead"));
        assert!(lumo.tokens.contains_key("kw.fn"));
        assert!(lumo.tokens.contains_key("ident"));
        assert!(!lumo.tokens.contains_key("kw.junk"));
        assert!(!lumo.tokens.contains_key("number"));
        // Trivia always survives.
        assert!(lumo.tokens.contains_key("ws"));
    }

    #[test]
    fn unreferenced_language_is_dropped() {
        let files = [
            LoadedFile {
                path: "Lumo.syn.langue".into(),
                kind: FileKind::Syn { language: "Lumo".into() },
                text: "File = 'fn'\ntoken kw.fn = 'fn'".into(),
            },
            LoadedFile {
                path: "Scratch.syn.langue".into(),
                kind: FileKind::Syn { language: "Scratch".into() },
                text: "File = 'x'\ntoken x = 'x'".into(),
            },
            LoadedFile {
                path: "p.langue".into(),
                kind: FileKind::Manifest,
                text: "main = parse Lumo".into(),
            },
        ];
        let (def, diags) = merge_project(&files);
        assert!(diags.is_empty(), "{diags:?}");
        let def = dce(&def);
        assert!(def.languages.contains_key("Lumo"));
        assert!(!def.languages.contains_key("Scratch"));
    }

    #[test]
    fn no_pipelines_keeps_everything() {
        let files = [LoadedFile {
            path: "Lumo.syn.langue".into(),
            kind: FileKind::Syn { language: "Lumo".into() },
            text: SYN.into(),
        }];
        let (def, diags) = merge_project(&files);
        assert!(diags.is_empty(), "{diags:?}");
        let def = dce(&def);
        assert!(def.languages["Lumo"].rules.contains_key("Dead"));
    }

    #[test]
    fn praat_refs_are_roots() {
        let syn = "\
token ident = /[a-z]+/
token op.plus = '+'
token op.unused = '^'
File = body:Expr
Ident = name:ident
Expr = praat {
  simple = Ident
  operators {
    @70 '+' @69,
  }
}
";
        let def = project(syn, "main = parse Lumo");
        let lumo = &def.languages["Lumo"];
        assert!(lumo.rules.contains_key("Ident"));
        assert!(lumo.tokens.contains_key("op.plus"));
        assert!(!lumo.tokens.contains_key("op.unused"));
    }
}
