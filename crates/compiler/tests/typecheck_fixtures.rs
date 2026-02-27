use std::fs;
use std::path::Path;

use lumo_compiler::{
    lexer::lex,
    parser::parse,
    typecheck::{render_type, typecheck_and_bindings},
};

#[test]
fn typecheck_fixtures() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/typecheck.txt");
    let all = fs::read_to_string(&path).expect("fixtures/typecheck.txt");

    for (idx, raw_case) in split_cases(&all).into_iter().enumerate() {
        let case_name = format!("fixtures/typecheck.txt#{}", idx + 1);
        let (source, expected) = split_source_expected(&raw_case, &case_name);

        let lexed = lex(&source);
        let parsed = parse(&lexed.tokens, &lexed.errors);
        let (bindings, errors) = typecheck_and_bindings(&parsed.file);

        if expected.starts_with("ERROR:") {
            let expected_messages = expected
                .lines()
                .filter_map(|l| l.strip_prefix("ERROR:"))
                .map(str::trim)
                .collect::<Vec<_>>();
            let actual = errors
                .iter()
                .map(|e| e.message.as_str())
                .collect::<Vec<_>>();
            for msg in expected_messages {
                assert!(
                    actual.iter().any(|m| m.contains(msg)),
                    "missing expected error `{msg}` in {case_name}; actual={actual:?}"
                );
            }
            continue;
        }

        assert!(
            errors.is_empty(),
            "unexpected errors in {case_name}: {errors:?}"
        );
        let actual = bindings
            .iter()
            .map(|b| format!("{} : {}", b.name, render_type(&b.ty)))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(actual, expected, "binding mismatch in {case_name}");
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
