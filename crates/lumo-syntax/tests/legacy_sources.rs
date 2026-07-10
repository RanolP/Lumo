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

/// The type/ and lto/ fixture *expectations* wait for the M2/M3
/// engines, but their case sources are Lumo programs — they must parse
/// today. Cases are `==========`-separated `input --- expected` blocks.
#[test]
fn legacy_fixture_sources_parse() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../legacy/crates/compiler/tests/fixtures");
    let mut files = Vec::new();
    for dir in ["syntax", "type", "lto"] {
        let mut group = Vec::new();
        collect_ext(&root.join(dir), "txt", &mut group);
        files.extend(group);
    }
    files.sort();
    assert!(!files.is_empty(), "no legacy fixture files under {}", root.display());

    let mut failures = Vec::new();
    let mut cases = 0;
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        let rel = file.strip_prefix(&root).unwrap_or(file).display().to_string();
        for (i, case) in text.split("\n==========\n").enumerate() {
            // A case's source ends at `---` (expected) or `===EXPECT===`
            // (lto golden section), whichever comes first.
            let source = case
                .split("\n---\n")
                .next()
                .unwrap_or(case)
                .split("\n===EXPECT===")
                .next()
                .unwrap_or(case)
                .trim();
            if source.is_empty() {
                continue;
            }
            cases += 1;
            let out = parse(source);
            if out.errors.is_empty() {
                continue;
            }
            // Some type-fixture cases lead with a prose title line.
            if let Some((_title, rest)) = source.split_once('\n') {
                if parse(rest.trim()).errors.is_empty() {
                    continue;
                }
            }
            let first = &out.errors[0];
            failures.push(format!(
                "{rel} case {}: {} error(s); first at {}: {}",
                i + 1,
                out.errors.len(),
                first.span,
                first.message
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {cases} legacy fixture sources failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn collect_ext(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_ext(&path, ext, out);
        } else if path.extension().is_some_and(|e| e == ext) {
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
