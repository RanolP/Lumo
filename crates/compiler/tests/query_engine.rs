use lumo_compiler::{
    lst::lossless::{node_text, SyntaxKind},
    query::QueryEngine,
};

#[test]
fn parse_lower_diagnostics_are_callable() {
    let mut q = QueryEngine::new();
    q.set_file("main.lumo", "data X { A } fn id := produce a+");

    let parsed = q.parse("main.lumo").expect("parse result");
    assert_eq!(parsed.file.items.len(), 2);
    assert_eq!(parsed.lossless.root.kind, SyntaxKind::File);
    assert_eq!(
        node_text(&parsed.lossless.root),
        "data X { A } fn id := produce a+"
    );

    let lowered = q.lower("main.lumo").expect("lower result");
    assert_eq!(lowered.items.len(), 2);

    let diagnostics = q.diagnostics("main.lumo").expect("diagnostics result");
    assert!(!diagnostics.is_empty());
}

#[test]
fn cache_is_reused_when_source_is_unchanged() {
    let mut q = QueryEngine::new();
    q.set_file("main.lumo", "fn id := produce a");

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
    q.set_file("main.lumo", "fn id := produce a");

    let _ = q.parse("main.lumo");
    let _ = q.lower("main.lumo");
    let _ = q.diagnostics("main.lumo");
    let stats_before = q.stats();

    q.set_file("main.lumo", "fn id := produce b");
    let _ = q.parse("main.lumo");
    let _ = q.lower("main.lumo");
    let _ = q.diagnostics("main.lumo");
    let stats_after = q.stats();

    assert!(stats_after.parse_computes > stats_before.parse_computes);
    assert!(stats_after.lower_computes > stats_before.lower_computes);
    assert!(stats_after.diagnostics_computes > stats_before.diagnostics_computes);
}
