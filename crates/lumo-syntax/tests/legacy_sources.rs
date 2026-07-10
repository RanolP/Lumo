//! Migration gate: every real legacy Lumo source file must parse cleanly
//! with the generated parser, print losslessly, and survive the
//! canonical-print round-trip.

use std::path::{Path, PathBuf};

use lumo_syntax::lumo::parser::parse;
use lumo_syntax::lumo::printer::{canonical, sexpr};

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "lumo") {
            out.push(path);
        }
    }
}

#[test]
fn legacy_sources_parse_and_round_trip() {
    // `legacy/apps/lumoc/main.lumo` is a type-theory sketch (∑/μ/∀) the
    // legacy compiler never parsed either — the gate is the code that
    // actually compiled: the packages.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../legacy/packages");
    let mut files = Vec::new();
    collect(&root, &mut files);
    files.sort();
    assert!(!files.is_empty(), "no legacy .lumo files under {}", root.display());

    let mut failures = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        let rel = file.strip_prefix(&root).unwrap_or(file).display();
        let out = parse(&text);
        if !out.errors.is_empty() {
            let first = &out.errors[0];
            failures.push(format!(
                "{rel}: {} error(s); first at {}: {}",
                out.errors.len(),
                first.span,
                first.message
            ));
            continue;
        }
        if out.root.text() != text {
            failures.push(format!("{rel}: lossless print differs from source"));
            continue;
        }
        let reparse = parse(&canonical(&out.root));
        if sexpr(&reparse.root) != sexpr(&out.root) {
            failures.push(format!("{rel}: canonical round-trip changed the tree"));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} legacy sources failed:\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );
}
