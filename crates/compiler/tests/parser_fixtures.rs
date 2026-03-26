use std::fs;
use std::path::Path;

use lumo_compiler::{
    hir,
    lexer::lex,
    lir,
    lst::lossless,
    parser::{parse, parse_lossless, Expr, Item},
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
        let hir_from_lossless_typed = hir::lower(&typed_from_lossless.file);
        assert_eq!(
            hir_typed, hir_from_lossless_typed,
            "HIR mismatch on fixture {}",
            case_name
        );
        let lir_typed = lir::lower(&hir_typed);

        let mut query = QueryEngine::new();
        let virtual_path = format!("fixture-{file_name}-{index}.lumo");
        query.set_file(virtual_path.clone(), source.clone());
        let q_parsed = query.parse(&virtual_path).expect("query parse result");
        let q_lowered_hir = query.lower_hir(&virtual_path).expect("query hir result");
        let q_lowered = query.lower(&virtual_path).expect("query lir result");
        let q_diags = query
            .diagnostics(&virtual_path)
            .expect("query diagnostics result");

        assert_eq!(
            typed_from_lossless.file, q_parsed.file,
            "query parse mismatch on fixture {}",
            case_name
        );
        assert_eq!(
            hir_typed, q_lowered_hir,
            "query HIR lower mismatch on fixture {}",
            case_name
        );
        assert_eq!(
            lir_typed, q_lowered,
            "query lower mismatch on fixture {}",
            case_name
        );

        assert_expected(expected, &q_parsed.file.items, &q_diags, &case_name);
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
        Item::ExternType(ext) => format!("ExternType(name=\"{}\")", ext.name),
        Item::ExternFn(ext) => format!("ExternFn(name=\"{}\")", ext.name),
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
        Item::Effect(e) => {
            let ops = e
                .operations
                .iter()
                .map(|op| format!("\"{}\"", op.name))
                .collect::<Vec<_>>()
                .join(", ");
            format!("Effect(name=\"{}\", ops=[{}])", e.name, ops)
        }
    }
}

fn render_expr(expr: &Expr) -> String {
    match expr {
        Expr::Ident { name, .. } => format!("Variable(\"{}\")", name),
        Expr::String { value, .. } => format!("String(\"{}\")", value),
        Expr::Member { object, member, .. } => {
            format!(
                "Member(object={}, member=\"{}\")",
                render_expr(object),
                member
            )
        }
        Expr::Call { callee, args, .. } => format!(
            "Call(callee={}, args=[{}])",
            render_expr(callee),
            args.iter().map(render_expr).collect::<Vec<_>>().join(", ")
        ),
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
        Expr::Match {
            scrutinee, arms, ..
        } => format!(
            "Match(scrutinee={}, arms=[{}])",
            render_expr(scrutinee),
            arms.iter()
                .map(|arm| format!("{} => {}", arm.pattern, render_expr(&arm.body)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Perform { effect, .. } => {
            format!("Perform(effect=\"{effect}\")")
        }
        Expr::Handle {
            effect, handler, body, ..
        } => format!(
            "Handle(effect=\"{effect}\", handler={}, body={})",
            render_expr(handler),
            render_expr(body)
        ),
        Expr::Ann { expr, ty, .. } => format!("Ann({}, \"{}\")", render_expr(expr), ty.repr),
        Expr::Bundle { entries, .. } => {
            let es = entries
                .iter()
                .map(|e| {
                    let params = e
                        .params
                        .iter()
                        .map(|p| p.name.clone())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}({}) := {}", e.name, params, render_expr(&e.body))
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("Bundle([{}])", es)
        }
        Expr::Error { .. } => "Error".to_owned(),
    }
}
