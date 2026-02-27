use std::fs;
use std::path::Path;

use lumo_compiler::{
    hir,
    lexer::lex,
    lst::lossless,
    parser::{parse, parse_lossless, Expr, Item},
    query::QueryEngine,
};

#[test]
fn parser_fixtures_pipeline_consistency() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/happy.txt");
    let all = fs::read_to_string(&path).expect("fixtures/happy.txt");
    let cases = split_cases(&all);

    assert!(
        cases.len() >= 12,
        "expected at least 12 happy fixtures, got {}",
        cases.len()
    );

    for (index, raw_case) in cases.iter().enumerate() {
        let case_name = format!("fixtures/happy.txt#{}", index + 1);
        let (source, expected) = split_source_expected(raw_case, &case_name);

        let lexed = lex(&source);
        let typed_from_lex = parse(&lexed.tokens, &lexed.errors);

        let lossless_parsed = lossless::parse(&source);
        let typed_from_lossless = parse_lossless(&lossless_parsed);

        assert_eq!(
            typed_from_lex.file, typed_from_lossless.file,
            "typed AST mismatch on fixture {}",
            case_name
        );

        let hir_typed = hir::lower(&typed_from_lex.file);
        let hir_lossless = hir::lower_lossless(&lossless_parsed);
        assert_eq!(
            hir_typed, hir_lossless,
            "HIR mismatch on fixture {}",
            case_name
        );

        let mut query = QueryEngine::new();
        let virtual_path = format!("fixture-{index}.lumo");
        query.set_file(virtual_path.clone(), source.clone());
        let q_parsed = query.parse(&virtual_path).expect("query parse result");
        let q_lowered = query.lower(&virtual_path).expect("query lower result");
        let q_diags = query
            .diagnostics(&virtual_path)
            .expect("query diagnostics result");

        assert_eq!(
            typed_from_lossless.file, q_parsed.file,
            "query parse mismatch on fixture {}",
            case_name
        );
        assert_eq!(
            hir_lossless, q_lowered,
            "query lower mismatch on fixture {}",
            case_name
        );

        assert_expected(expected, &q_parsed.file.items, &q_diags, &case_name);
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

fn assert_expected(
    expected: String,
    items: &[Item],
    diags: &[lumo_compiler::diagnostics::Diagnostic],
    case_name: &str,
) {
    if expected.starts_with("ERROR:") {
        let expected_messages = expected
            .lines()
            .filter_map(|line| line.strip_prefix("ERROR:"))
            .map(str::trim)
            .collect::<Vec<_>>();
        let actual_messages = diags.iter().map(|d| d.message.as_str()).collect::<Vec<_>>();
        for msg in expected_messages {
            assert!(
                actual_messages.iter().any(|m| m.contains(msg)),
                "missing expected error '{msg}' in {case_name}. actual: {:?}",
                actual_messages
            );
        }
        return;
    }

    let actual = render_items(items);
    assert_eq!(
        actual, expected,
        "AST mismatch for {case_name}\nactual:\n{actual}\nexpected:\n{expected}"
    );
}

fn render_items(items: &[Item]) -> String {
    items.iter().map(render_item).collect::<Vec<_>>().join("\n")
}

fn render_item(item: &Item) -> String {
    match item {
        Item::Data(d) => {
            let variants = d
                .variants
                .iter()
                .map(|v| format!("\"{}\"", v.name))
                .collect::<Vec<_>>()
                .join(", ");
            format!("Data(name=\"{}\", variants=[{}])", d.name, variants)
        }
        Item::Fn(f) => format!("Fn(name=\"{}\", body={})", f.name, render_expr(&f.body)),
    }
}

fn render_expr(expr: &Expr) -> String {
    match expr {
        Expr::Ident { name, .. } => format!("Variable(\"{}\")", name),
        Expr::Produce { expr, .. } => format!("Produce({})", render_expr(expr)),
        Expr::Thunk { expr, .. } => format!("Thunk({})", render_expr(expr)),
        Expr::Force { expr, .. } => format!("Force({})", render_expr(expr)),
        Expr::LetIn {
            name, value, body, ..
        } => format!(
            "Let(name=\"{}\", value={}, body={})",
            name,
            render_expr(value),
            render_expr(body)
        ),
        Expr::Match { scrutinee, arms, .. } => format!(
            "Match(scrutinee={}, arms=[{}])",
            render_expr(scrutinee),
            arms.iter()
                .map(|arm| format!("{} => {}", arm.pattern, render_expr(&arm.body)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Apply {
            owner,
            member,
            args,
            ..
        } => format!(
            "Apply(owner=\"{}\", member=\"{}\", args=[{}])",
            owner,
            member,
            args.iter().map(render_expr).collect::<Vec<_>>().join(", ")
        ),
        Expr::Error { .. } => "Error".to_owned(),
    }
}
