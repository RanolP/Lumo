//! Compile the ported stdlib unit (D-45) to JS on stdout — the
//! `scripts/stdlib_smoke.sh` half that produces the program text.

use std::path::Path;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let manifest = std::fs::read_to_string(root.join("stdlib.manifest")).unwrap();
    let mut parts = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        parts.push(
            std::fs::read_to_string(root.join(line)).unwrap_or_else(|e| panic!("{line}: {e}")),
        );
    }
    let report = lumo_syntax::compile_driver::compile_report(&parts.join("\n"));
    if !report.errors.is_empty() {
        eprintln!("{}", report.errors.join("\n"));
        std::process::exit(1);
    }
    println!("{}", report.output);
}
