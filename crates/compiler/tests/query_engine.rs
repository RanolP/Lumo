use lumo_compiler::{
    backend::{self, CodegenTarget},
    lst::lossless::{node_text, SyntaxKind},
    query::QueryEngine,
    typecheck,
};

#[test]
fn parse_lower_diagnostics_are_callable() {
    let mut q = QueryEngine::new();
    q.set_file("main.lumo", "data X { .a } fn id() := produce a+");

    let parsed = q.parse("main.lumo").expect("parse result");
    assert_eq!(parsed.file.items.len(), 2);
    assert_eq!(parsed.lossless.root.kind, SyntaxKind::File);
    assert_eq!(
        node_text(&parsed.lossless.root),
        "data X { .a } fn id() := produce a+"
    );

    let lowered = q.lower("main.lumo").expect("lower result");
    assert_eq!(lowered.items.len(), 2);

    let diagnostics = q.diagnostics("main.lumo").expect("diagnostics result");
    assert!(!diagnostics.is_empty());
}

#[test]
fn cache_is_reused_when_source_is_unchanged() {
    let mut q = QueryEngine::new();
    q.set_file("main.lumo", "fn id() := produce a");

    let _ = q.parse("main.lumo");
    let _ = q.lower("main.lumo");
    let _ = q.diagnostics("main.lumo");
    let stats_after_first = q.stats();

    let _ = q.parse("main.lumo");
    let _ = q.lower("main.lumo");
    let _ = q.diagnostics("main.lumo");
    let stats_after_second = q.stats();

    assert_eq!(
        stats_after_first.parse_computes,
        stats_after_second.parse_computes
    );
    assert_eq!(
        stats_after_first.lower_computes,
        stats_after_second.lower_computes
    );
    assert_eq!(
        stats_after_first.diagnostics_computes,
        stats_after_second.diagnostics_computes
    );
}

#[test]
fn cache_is_invalidated_when_source_changes() {
    let mut q = QueryEngine::new();
    q.set_file("main.lumo", "fn id() := produce a");

    let _ = q.parse("main.lumo");
    let _ = q.lower("main.lumo");
    let _ = q.diagnostics("main.lumo");
    let stats_before = q.stats();

    q.set_file("main.lumo", "fn id() := produce b");
    let _ = q.parse("main.lumo");
    let _ = q.lower("main.lumo");
    let _ = q.diagnostics("main.lumo");
    let stats_after = q.stats();

    assert!(stats_after.parse_computes > stats_before.parse_computes);
    assert!(stats_after.lower_computes > stats_before.lower_computes);
    assert!(stats_after.diagnostics_computes > stats_before.diagnostics_computes);
}

#[test]
fn eof_diagnostics_use_eof_span_instead_of_zero_zero() {
    let mut q = QueryEngine::new();
    let src = "fn broken() := produce";
    q.set_file("main.lumo", src);

    let diagnostics = q.diagnostics("main.lumo").expect("diagnostics result");
    assert!(diagnostics.iter().any(|d| d
        .message
        .contains("expected payload expression after `produce`")));
    assert!(diagnostics
        .iter()
        .any(|d| d.message.contains("expected expression")));
    assert!(
        diagnostics
            .iter()
            .all(|d| d.start == src.len() && d.end == src.len()),
        "expected EOF diagnostics at byte {}, got {:?}",
        src.len(),
        diagnostics
    );
}

#[test]
fn extern_fn_declaration_without_body_is_not_parsed_as_fn_decl() {
    let mut q = QueryEngine::new();
    let src = "#[extern(name = \"string\")] extern type String;\nextern fn console_log(s: String): produce Unit;";
    q.set_file("main.lumo", src);

    let diagnostics = q.diagnostics("main.lumo").expect("diagnostics result");
    assert!(
        diagnostics.is_empty(),
        "expected no diagnostics, got {diagnostics:?}"
    );
}

// --- Multi-file module tests ---

#[test]
fn multi_file_shares_data_type() {
    let mut q = QueryEngine::new();
    q.set_file("types.lumo", "data Bool { .true, .false }");
    q.set_file(
        "fns.lumo",
        "fn not(x: Bool): produce Bool / {} := match x { .true => Bool.false, .false => Bool.true }",
    );

    let merged = q
        .lower_module(&["types.lumo", "fns.lumo"])
        .expect("merged module");
    let errors = typecheck::typecheck_file(&merged);
    assert!(
        errors.is_empty(),
        "cross-file data type should typecheck, got: {errors:?}"
    );
}

#[test]
fn multi_file_cross_fn_reference() {
    let mut q = QueryEngine::new();
    q.set_file(
        "a.lumo",
        "fn id(x: A): produce A / {} := produce x",
    );
    q.set_file(
        "b.lumo",
        "fn use_id(x: A): produce A / {} := id(x)",
    );

    let merged = q
        .lower_module(&["a.lumo", "b.lumo"])
        .expect("merged module");
    let errors = typecheck::typecheck_file(&merged);
    assert!(
        errors.is_empty(),
        "cross-file fn ref should typecheck, got: {errors:?}"
    );
}

#[test]
fn multi_file_backend_emits_all_items() {
    let mut q = QueryEngine::new();
    q.set_file("types.lumo", "data Bool { .true, .false }");
    q.set_file(
        "fns.lumo",
        "fn not(x: Bool): produce Bool / {} := match x { .true => Bool.false, .false => Bool.true }",
    );

    let merged = q
        .lower_module(&["types.lumo", "fns.lumo"])
        .expect("merged module");
    let output = backend::emit(&merged, CodegenTarget::TypeScript).expect("backend emit");
    assert!(output.contains("Bool"), "output should contain Bool type");
    assert!(output.contains("not"), "output should contain not function");
}
