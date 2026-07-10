//! Salsa query layer (D-06). Kept deliberately thin — inputs and tracked
//! functions only delegate to the pure `project`/`check` modules, so the
//! query runtime stays swappable if salsa's model ever fights the
//! relational engine.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::check;
use crate::diag::Diagnostic;
use crate::project::dce::dce;
use crate::project::loader::FileKind;
use crate::project::merge::{self, ParsedFile};
use crate::project::model::Definition;
use crate::syntax::ast;

/// Executions of `parse_langue` — instrumentation for the incremental
/// smoke test; salsa memoization keeps this from growing on cache hits.
pub static PARSE_EXECUTIONS: AtomicUsize = AtomicUsize::new(0);

#[salsa::input]
pub struct SourceFile {
    pub path: String,
    pub kind: FileKind,
    pub text: String,
}

#[salsa::input]
pub struct Project {
    pub files: Vec<SourceFile>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Parsed {
    pub ast: ast::File,
    pub diags: Vec<Diagnostic>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Merged {
    pub definition: Definition,
    pub diags: Vec<Diagnostic>,
}

#[salsa::tracked]
pub fn parse_langue(db: &dyn salsa::Database, file: SourceFile) -> Parsed {
    PARSE_EXECUTIONS.fetch_add(1, Ordering::Relaxed);
    let (ast, diags) = merge::parse_file(&file.path(db), &file.kind(db), &file.text(db));
    Parsed { ast, diags }
}

/// Merged + DCE'd definition (the loaded-definition value of design §5.1).
#[salsa::tracked]
pub fn merged_definition(db: &dyn salsa::Database, project: Project) -> Merged {
    let mut diags = Vec::new();
    let parsed: Vec<ParsedFile> = project
        .files(db)
        .into_iter()
        .map(|f| {
            let p = parse_langue(db, f);
            diags.extend(p.diags.iter().cloned());
            ParsedFile { path: f.path(db), kind: f.kind(db), ast: p.ast.clone() }
        })
        .collect();
    let (def, mut merge_diags) = merge::merge_asts(&parsed);
    diags.append(&mut merge_diags);
    Merged { definition: dce(&def), diags }
}

/// Generated Rust sources as `(relative path, content)` pairs, in
/// deterministic order (D-21).
#[salsa::tracked]
pub fn generated_files(db: &dyn salsa::Database, project: Project) -> Vec<(String, String)> {
    crate::codegen::generate(&merged_definition(db, project).definition)
}

/// All diagnostics for the project: parse + merge + check.
#[salsa::tracked]
pub fn check_definition(db: &dyn salsa::Database, project: Project) -> Vec<Diagnostic> {
    let merged = merged_definition(db, project);
    let mut diags = merged.diags.clone();
    diags.extend(check::check_definition(&merged.definition));
    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use salsa::Setter as _;

    fn file(db: &dyn salsa::Database, path: &str, kind: FileKind, text: &str) -> SourceFile {
        SourceFile::new(db, path.to_owned(), kind, text.to_owned())
    }

    #[test]
    fn incremental_smoke() {
        let mut db = salsa::DatabaseImpl::default();
        let tokens = file(
            &db,
            "Lumo.tokens.syn.langue",
            FileKind::Syn { language: "Lumo".into() },
            "token kw.fn = 'fn'\ntoken ident = /[a-z]+/",
        );
        let grammar = file(
            &db,
            "Lumo.expr.syn.langue",
            FileKind::Syn { language: "Lumo".into() },
            "File = 'fn' name:ident",
        );
        let manifest = file(&db, "lumo.langue", FileKind::Manifest, "main = parse Lumo");
        let project = Project::new(&db, vec![tokens, grammar, manifest]);

        let diags = check_definition(&db, project);
        assert!(diags.is_empty(), "{diags:?}");
        let after_first = PARSE_EXECUTIONS.load(Ordering::Relaxed);

        // Re-query without changes: fully cached, nothing re-parses.
        let _ = check_definition(&db, project);
        assert_eq!(PARSE_EXECUTIONS.load(Ordering::Relaxed), after_first);

        // Break one file: only that file re-parses, and check goes red.
        grammar
            .set_text(&mut db)
            .to("File = 'fn' name:missing".to_owned());
        let diags = check_definition(&db, project);
        assert!(
            diags.iter().any(|d| d.message.contains("unknown token `missing`")),
            "{diags:?}"
        );
        assert_eq!(PARSE_EXECUTIONS.load(Ordering::Relaxed), after_first + 1);
    }
}
