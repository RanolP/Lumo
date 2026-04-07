use std::fs;
use std::path::Path;

use lumo_compiler::{
    hir,
    lexer::lex,
    lir,
    parser::parse,
    typecheck::{render_type, typecheck_and_bindings},
};

#[test]
fn typecheck_fixtures() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/type");
    let mut dir_entries: Vec<_> = fs::read_dir(&dir)
        .expect("tests/fixtures/type/ directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "txt"))
        .collect();
    dir_entries.sort_by_key(|e| e.file_name());

    assert!(
        !dir_entries.is_empty(),
        "no fixture files found in tests/fixtures/type/"
    );

    for dir_entry in dir_entries {
        let path = dir_entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let all = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {file_name}: {e}"));

    for (idx, raw_case) in split_cases(&all).into_iter().enumerate() {
        let case_name = format!("type/{file_name}#{}", idx + 1);
        let (source, expected) = split_source_expected(&raw_case, &case_name);

        let lexed = lex(&source);
        let parsed = parse(&lexed.tokens, &lexed.errors);
        let hir = hir::lower(&parsed.file);
        let lir = lir::lower(&hir);
        let (bindings, errors) = typecheck_and_bindings(&lir);

        // Collect all errors: HIR lowering errors + typecheck errors
        let mut all_error_msgs: Vec<String> = hir
            .errors
            .iter()
            .map(|e| e.message.clone())
            .collect();
        all_error_msgs.extend(errors.iter().map(|e| e.message.clone()));

        if expected.starts_with("ERROR:") {
            let expected_messages = expected
                .lines()
                .filter_map(|l| l.strip_prefix("ERROR:"))
                .map(str::trim)
                .collect::<Vec<_>>();
            for msg in expected_messages {
                assert!(
                    all_error_msgs.iter().any(|m| m.contains(msg)),
                    "missing expected error `{msg}` in {case_name}; actual={all_error_msgs:?}"
                );
            }
            continue;
        }

        assert!(
            all_error_msgs.is_empty(),
            "unexpected errors in {case_name}: {all_error_msgs:?}"
        );
        let actual = bindings
            .iter()
            .map(|b| format!("{} : {}", b.name, render_type(&b.ty)))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(actual, expected, "binding mismatch in {case_name}");
    }
    }
}

fn split_cases(text: &str) -> Vec<String> {
    text.replace("\r\n", "\n")
        .split("\n==========\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn split_source_expected(case: &str, case_name: &str) -> (String, String) {
    let (source, expected) = case
        .split_once("---")
        .unwrap_or_else(|| panic!("{case_name} missing --- separator"));
    (source.trim().to_owned(), expected.trim().to_owned())
}
