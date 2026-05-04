use std::fs;
use std::path::Path;

use lumo_compiler::{
    hir,
    lir,
    lst::lossless,
    query::QueryEngine,
};

#[test]
fn parser_fixtures_pipeline_consistency() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/syntax");
    let mut dir_entries: Vec<_> = fs::read_dir(&dir)
        .expect("tests/fixtures/syntax/ directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "txt"))
        .collect();
    dir_entries.sort_by_key(|e| e.file_name());

    assert!(
        !dir_entries.is_empty(),
        "no fixture files found in tests/fixtures/syntax/"
    );

    let mut total_cases = 0;

    for dir_entry in dir_entries {
        let path = dir_entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let all = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {file_name}: {e}"));
        let cases = split_cases(&all);

    for (index, raw_case) in cases.iter().enumerate() {
        let case_name = format!("syntax/{file_name}#{}", index + 1);
        let source = extract_source(raw_case, &case_name);

        let lossless_parsed = lossless::parse(&source);
        let hir_from_lossless = hir::lower_lossless(&lossless_parsed);
        let lir_from_lossless = lir::lower(&hir_from_lossless);

        let mut query = QueryEngine::new();
        let virtual_path = format!("fixture-{file_name}-{index}.lumo");
        query.set_file(virtual_path.clone(), source.clone());
        let q_parsed = query.parse(&virtual_path).expect("query parse result");
        let q_lowered_hir = query.lower_hir(&virtual_path).expect("query hir result");
        let q_lowered = query.lower(&virtual_path).expect("query lir result");

        assert_eq!(
            hir_from_lossless, q_parsed.file,
            "query parse mismatch on fixture {}",
            case_name
        );
        assert_eq!(
            hir_from_lossless, q_lowered_hir,
            "query HIR lower mismatch on fixture {}",
            case_name
        );
        assert_eq!(
            lir_from_lossless, q_lowered,
            "query lower mismatch on fixture {}",
            case_name
        );

        total_cases += 1;
    }
    }

    assert!(
        total_cases >= 12,
        "expected at least 12 syntax fixtures, got {}",
        total_cases
    );
}

fn split_cases(text: &str) -> Vec<String> {
    text.replace("\r\n", "\n")
        .split("\n==========\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn extract_source(case: &str, case_name: &str) -> String {
    let source = case
        .split_once("---")
        .unwrap_or_else(|| panic!("{case_name} missing --- separator"))
        .0;
    source.trim().to_owned()
}
