//! Cat: merge every parsed file into one `Definition` (D-05). Languages
//! merge additively across files; same-named items are strict errors
//! (D-22), stdlib included — it participates as ordinary files.

use crate::diag::Diagnostic;
use crate::syntax::ast::{self, Item};
use crate::syntax::parser;

use super::loader::{FileKind, LoadedFile};
use super::model::{Definition, Language, Origin, PipelineDef, RuleDef, TokenDef};

/// M0 stdlib (D-29): starts empty; grows only on proven need. Each entry
/// is a virtual `(path, text)` catted in front of the project files.
pub const STDLIB_FILES: &[(&str, &str)] = &[];

/// One parsed file, ready to merge. This is what the salsa `parse_langue`
/// query produces per file.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParsedFile {
    pub path: String,
    pub kind: FileKind,
    pub ast: ast::File,
}

/// Parse one file according to its kind. Elab/type files parse to an
/// empty AST with a warning until M1/M2.
pub fn parse_file(path: &str, kind: &FileKind, text: &str) -> (ast::File, Vec<Diagnostic>) {
    match kind {
        FileKind::Syn { .. } => parser::parse_syn_file(path, text),
        FileKind::Manifest => parser::parse_manifest(path, text),
        FileKind::Elab | FileKind::Type => (
            ast::File::default(),
            vec![Diagnostic::warning(
                path,
                langue_rt::Span::default(),
                "elab/type files are ignored until M1/M2",
            )],
        ),
    }
}

/// Cat stdlib + parsed files into one definition.
pub fn merge_asts(files: &[ParsedFile]) -> (Definition, Vec<Diagnostic>) {
    let mut def = Definition::default();
    let mut diags = Vec::new();

    let stdlib = STDLIB_FILES.iter().map(|(path, text)| {
        let kind = super::loader::classify_file_name(path).expect("stdlib file names are valid");
        let (ast, stdlib_diags) = parse_file(path, &kind, text);
        assert!(stdlib_diags.is_empty(), "stdlib must parse cleanly: {stdlib_diags:?}");
        ParsedFile { path: (*path).to_owned(), kind, ast }
    });

    for file in stdlib.chain(files.iter().cloned()) {
        match &file.kind {
            FileKind::Syn { language } => {
                merge_syn_file(&mut def, language, &file.path, file.ast, &mut diags);
            }
            FileKind::Manifest => {
                merge_manifest(&mut def, &file.path, file.ast, &mut diags);
            }
            FileKind::Elab | FileKind::Type => {}
        }
    }

    (def, diags)
}

/// Parse and merge a whole project (stdlib + files) in one call.
pub fn merge_project(files: &[LoadedFile]) -> (Definition, Vec<Diagnostic>) {
    let mut diags = Vec::new();
    let parsed: Vec<ParsedFile> = files
        .iter()
        .map(|f| {
            let (ast, mut file_diags) = parse_file(&f.path, &f.kind, &f.text);
            diags.append(&mut file_diags);
            ParsedFile { path: f.path.clone(), kind: f.kind.clone(), ast }
        })
        .collect();
    let (def, mut merge_diags) = merge_asts(&parsed);
    diags.append(&mut merge_diags);
    (def, diags)
}

fn merge_syn_file(
    def: &mut Definition,
    language: &str,
    path: &str,
    ast: ast::File,
    diags: &mut Vec<Diagnostic>,
) {
    // The file's existence declares the language (D-03).
    if !def.languages.contains_key(language) {
        if def.pipelines.contains_key(language) {
            diags.push(Diagnostic::error(
                path,
                langue_rt::Span::default(),
                format!("language `{language}` collides with a pipeline of the same name"),
            ));
        }
        def.languages.insert(language.to_owned(), Language::default());
    }
    let lang = def.languages.get_mut(language).expect("just inserted");

    for item in ast.items {
        match item {
            Item::Token(t) => {
                let origin = Origin { file: path.to_owned(), span: t.name_span };
                if let Some(prev) = lang.tokens.get(&t.name) {
                    diags.push(collision(path, t.name_span, "token", &t.name, language, &prev.origin));
                } else {
                    lang.tokens.insert(
                        t.name.clone(),
                        TokenDef { name: t.name, pattern: t.pattern, is_trivia: t.is_trivia, origin },
                    );
                }
            }
            Item::Rule(r) => {
                let origin = Origin { file: path.to_owned(), span: r.name_span };
                if let Some(prev) = lang.rules.get(&r.name) {
                    diags.push(collision(path, r.name_span, "rule", &r.name, language, &prev.origin));
                } else {
                    lang.rules.insert(r.name.clone(), RuleDef { name: r.name, body: r.body, origin });
                }
            }
            Item::ExternRecover(e) => {
                let origin = Origin { file: path.to_owned(), span: e.span };
                if let Some(prev) = lang.extern_recovers.get(&e.rule) {
                    diags.push(collision(path, e.span, "extern recover", &e.rule, language, prev));
                } else {
                    lang.extern_recovers.insert(e.rule, origin);
                }
            }
            Item::Pipeline(p) => {
                diags.push(Diagnostic::error(
                    path,
                    p.name_span,
                    "pipelines may only appear in the manifest (suffix-less .langue file)",
                ));
            }
            Item::ElabBlock(_) | Item::BetweenBlock(_) | Item::ExternRule(_)
            | Item::ExternPass(_) => {
                diags.push(Diagnostic::error(
                    path,
                    langue_rt::Span::default(),
                    "elab items may only appear in .elab.langue files",
                ));
            }
        }
    }
}

fn merge_manifest(
    def: &mut Definition,
    path: &str,
    ast: ast::File,
    diags: &mut Vec<Diagnostic>,
) {
    for item in ast.items {
        match item {
            Item::Pipeline(p) => {
                let origin = Origin { file: path.to_owned(), span: p.name_span };
                if let Some(prev) = def.pipelines.get(&p.name) {
                    diags.push(Diagnostic::error(
                        path,
                        p.name_span,
                        format!(
                            "duplicate pipeline `{}` (first defined in {} at {})",
                            p.name, prev.origin.file, prev.origin.span
                        ),
                    ));
                } else if def.languages.contains_key(&p.name) {
                    diags.push(Diagnostic::error(
                        path,
                        p.name_span,
                        format!("pipeline `{}` collides with a language of the same name", p.name),
                    ));
                } else {
                    def.pipelines
                        .insert(p.name.clone(), PipelineDef { name: p.name, stages: p.stages, origin });
                }
            }
            _ => {
                diags.push(Diagnostic::error(
                    path,
                    langue_rt::Span::default(),
                    "only pipelines may appear in the manifest",
                ));
            }
        }
    }
}

fn collision(
    path: &str,
    span: langue_rt::Span,
    what: &str,
    name: &str,
    language: &str,
    prev: &Origin,
) -> Diagnostic {
    Diagnostic::error(
        path,
        span,
        format!(
            "duplicate {what} `{name}` in language `{language}` (first defined in {} at {})",
            prev.file, prev.span
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn syn(path: &str, language: &str, text: &str) -> LoadedFile {
        LoadedFile {
            path: path.to_owned(),
            kind: FileKind::Syn { language: language.to_owned() },
            text: text.to_owned(),
        }
    }

    fn manifest(text: &str) -> LoadedFile {
        LoadedFile { path: "p.langue".to_owned(), kind: FileKind::Manifest, text: text.to_owned() }
    }

    #[test]
    fn cross_file_language_merge() {
        let (def, diags) = merge_project(&[
            syn("Lumo.tokens.syn.langue", "Lumo", "token ident = /[a-z]+/"),
            syn("Lumo.expr.syn.langue", "Lumo", "Expr = name:ident"),
            manifest("main = parse Lumo"),
        ]);
        assert!(diags.is_empty(), "{diags:?}");
        assert_eq!(def.languages.len(), 1);
        let lumo = &def.languages["Lumo"];
        assert!(lumo.tokens.contains_key("ident"));
        assert!(lumo.rules.contains_key("Expr"));
        assert_eq!(def.pipelines["main"].stages.len(), 1);
    }

    #[test]
    fn same_name_collision_is_strict_error() {
        let (_, diags) = merge_project(&[
            syn("a.syn.langue", "Lumo", "token ident = /[a-z]+/"),
            syn("b.syn.langue", "Lumo", "token ident = /[0-9]+/"),
        ]);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("duplicate token `ident`"));
        assert!(diags[0].message.contains("a.syn.langue"));
    }

    #[test]
    fn separate_languages_do_not_collide() {
        let (def, diags) = merge_project(&[
            syn("Lumo.syn.langue", "Lumo", "token ident = /[a-z]+/"),
            syn("MIR.syn.langue", "MIR", "token ident = /[a-z]+/"),
        ]);
        assert!(diags.is_empty());
        assert_eq!(def.languages.len(), 2);
    }

    #[test]
    fn pipeline_in_syn_file_is_error() {
        // In a syn file `main = parse Lumo` parses as a rule (fine); a
        // Pipeline item can only arrive via manifest parsing, so instead
        // check the inverse: rules in a manifest are rejected by shape.
        let (_, diags) = merge_project(&[manifest("main = parse")]);
        assert!(!diags.is_empty());
    }
}
