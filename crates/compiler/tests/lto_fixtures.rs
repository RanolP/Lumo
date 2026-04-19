use std::fs;
use std::path::Path;

use lumo_compiler::backend::{self, CodegenTarget};
use lumo_compiler::query::QueryEngine;

fn read_fixture(path: &Path) -> (String, String) {
    let content = fs::read_to_string(path).unwrap();
    let mut parts = content.splitn(2, "===EXPECT===");
    let src = parts.next().unwrap().trim().to_owned();
    let expect = parts.next().unwrap_or("").trim().to_owned();
    (src, expect)
}

fn compile(src: &str) -> String {
    let mut q = QueryEngine::new();
    q.set_file("main.lumo", src.to_owned());
    let lir = q.lower_module(&["main.lumo"]).expect("lower_module failed");
    backend::emit(&lir, CodegenTarget::JavaScript).expect("js emit")
}

#[test]
fn lto_fixtures_pass() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lto");
    if !dir.exists() {
        return; // No fixtures yet; harness compiles but nothing to exercise.
    }
    let mut failures = Vec::new();
    for entry in fs::read_dir(&dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|e| e.to_str()) != Some("txt") {
            continue;
        }
        let (src, expect) = read_fixture(&entry.path());
        let js = compile(&src);
        // Each line of `expect`: substring assertion. Lines starting with `!`
        // are negative assertions (must NOT be present). Lines starting with
        // `#` are comments and ignored.
        for line in expect.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(neg) = line.strip_prefix('!') {
                if js.contains(neg.trim()) {
                    failures.push(format!(
                        "{}: unexpected substring `{}` in output:\n{}",
                        entry.path().display(),
                        neg.trim(),
                        js
                    ));
                }
            } else if !js.contains(line) {
                failures.push(format!(
                    "{}: missing substring `{}` in output:\n{}",
                    entry.path().display(),
                    line,
                    js
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "lto fixture failures:\n{}",
        failures.join("\n---\n")
    );
}
