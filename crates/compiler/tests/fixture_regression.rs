use std::fs;
use std::path::Path;

use lumo_compiler::{
    hir,
    lexer::lex,
    lst::lossless,
    parser::{parse, parse_lossless},
    query::QueryEngine,
};

#[test]
fn fixture_regression_pipeline_consistency() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut entries = fs::read_dir(&fixture_dir)
        .expect("fixture dir")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.path());

    assert!(!entries.is_empty(), "no fixtures found");

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("lumo") {
            continue;
        }
        if path
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|name| name.starts_with("broken_"))
        {
            continue;
        }

        let src = fs::read_to_string(&path).expect("fixture source");

        let lexed = lex(&src);
        let typed_from_lex = parse(&lexed.tokens, &lexed.errors);

        let lossless_parsed = lossless::parse(&src);
        let typed_from_lossless = parse_lossless(&lossless_parsed);

        assert_eq!(
            typed_from_lex.file,
            typed_from_lossless.file,
            "typed AST mismatch on fixture {}",
            path.display()
        );

        let hir_typed = hir::lower(&typed_from_lex.file);
        let hir_lossless = hir::lower_lossless(&lossless_parsed);
        assert_eq!(
            hir_typed,
            hir_lossless,
            "HIR mismatch on fixture {}",
            path.display()
        );

        let mut query = QueryEngine::new();
        query.set_file(path.to_string_lossy().to_string(), src.clone());
        let q_parsed = query
            .parse(&path.to_string_lossy())
            .expect("query parse result");
        let q_lowered = query
            .lower(&path.to_string_lossy())
            .expect("query lower result");

        assert_eq!(
            typed_from_lossless.file,
            q_parsed.file,
            "query parse mismatch on fixture {}",
            path.display()
        );
        assert_eq!(
            hir_lossless,
            q_lowered,
            "query lower mismatch on fixture {}",
            path.display()
        );
    }
}

#[test]
fn fixture_recovery_cases_emit_errors_consistently() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut entries = fs::read_dir(&fixture_dir)
        .expect("fixture dir")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.path());

    let mut checked = 0usize;

    for entry in entries {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.starts_with("broken_") || path.extension().and_then(|x| x.to_str()) != Some("lumo")
        {
            continue;
        }

        let src = fs::read_to_string(&path).expect("fixture source");
        let mut query = QueryEngine::new();
        query.set_file(path.to_string_lossy().to_string(), src.clone());

        let parsed = query
            .parse(&path.to_string_lossy())
            .expect("query parse result");
        let diagnostics = query
            .diagnostics(&path.to_string_lossy())
            .expect("query diagnostics result");

        assert!(
            !parsed.errors.is_empty(),
            "expected parse errors for broken fixture {}",
            path.display()
        );
        assert!(
            !diagnostics.is_empty(),
            "expected diagnostics for broken fixture {}",
            path.display()
        );

        let lossless = lossless::parse(&src);
        let lowered = hir::lower_lossless(&lossless);
        assert!(
            !lowered.items.is_empty(),
            "recovery should still produce some items for fixture {}",
            path.display()
        );

        checked += 1;
    }

    assert!(checked >= 2, "expected at least 2 broken fixtures");
}
