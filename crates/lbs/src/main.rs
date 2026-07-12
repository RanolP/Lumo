//! `lbs <check|build|run> [dir] [--target SPEC]` (D-53): assemble the
//! package at `dir` (default: cwd, walking up to the nearest
//! `lumo.toml`), typecheck, and for bin packages emit + run
//! `dist/{name}.js`.

use std::path::PathBuf;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first().map(String::as_str) else {
        eprintln!("usage: lbs <check|build|run> [dir] [--target SPEC]");
        exit(2);
    };
    let mut dir: Option<PathBuf> = None;
    let mut target_flag: Option<String> = None;
    let mut rest = args[1..].iter();
    while let Some(arg) = rest.next() {
        if arg == "--target" {
            target_flag = rest.next().cloned();
            if target_flag.is_none() {
                fail("--target needs a value");
            }
        } else if dir.is_none() {
            dir = Some(PathBuf::from(arg));
        } else {
            fail(&format!("unexpected argument: {arg}"));
        }
    }

    let start = dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|e| fail(&e.to_string())));
    let Some(root) = lbs::find_package_root(&start) else {
        fail(&format!("no lumo.toml found from {} upwards", start.display()));
    };
    let manifest = lbs::load_manifest(&root).unwrap_or_else(|e| fail(&e));
    let target =
        lbs::resolve_target(&manifest, target_flag.as_deref()).unwrap_or_else(|e| fail(&e));
    let unit = lbs::assemble(&root, &target).unwrap_or_else(|e| fail(&e));

    match cmd {
        "check" => {
            check(&unit);
            println!("check ok: {} ({} files, target {target})", unit.name, unit.parts.len());
        }
        "build" | "run" => {
            check(&unit);
            let out = build(&root, &unit);
            match out {
                Some(out) if cmd == "run" => {
                    let status = std::process::Command::new("node")
                        .arg(&out)
                        .status()
                        .unwrap_or_else(|e| fail(&format!("node: {e}")));
                    exit(status.code().unwrap_or(1));
                }
                Some(out) => println!("built {}", out.display()),
                None => println!("check ok: {} is a library, no artifact in this slice", unit.name),
            }
        }
        other => {
            eprintln!("unknown command: {other}");
            eprintln!("usage: lbs <check|build|run> [dir] [--target SPEC]");
            exit(2);
        }
    }
}

fn fail(msg: &str) -> ! {
    eprintln!("error: {msg}");
    exit(1);
}

/// Parse + infer the unit; exits with mapped diagnostics on failure.
fn check(unit: &lbs::Unit) {
    let parsed = lumo_syntax::lumo::parser::parse(&unit.text);
    if !parsed.errors.is_empty() {
        for e in &parsed.errors {
            let at = lbs::locate(unit, e.span.start as usize)
                .unwrap_or_else(|| format!("offset {}", e.span.start));
            eprintln!("parse error: {at}: {}", e.message);
        }
        exit(1);
    }
    let report = lumo_syntax::judge_driver::infer_report(&unit.text);
    if !report.errors.is_empty() {
        for e in &report.errors {
            eprintln!("error: {e}");
        }
        exit(1);
    }
    if report.output.starts_with("ERROR") {
        eprintln!("type error: {}", report.output);
        exit(1);
    }
}

/// Compile and emit `dist/{name}.js` for a bin package; `None` for libs.
fn build(root: &std::path::Path, unit: &lbs::Unit) -> Option<PathBuf> {
    if !unit.is_bin {
        return None;
    }
    let report = lumo_syntax::compile_driver::compile_report(&unit.text);
    if !report.errors.is_empty() {
        for e in &report.errors {
            eprintln!("error: {e}");
        }
        exit(1);
    }
    let mut js = String::new();
    for prelude in &unit.preludes {
        let src = std::fs::read_to_string(prelude)
            .unwrap_or_else(|e| fail(&format!("{}: {e}", prelude.display())));
        js.push_str(&src);
        js.push('\n');
    }
    js.push_str(&report.output);
    js.push_str("\nmain();\n");

    let dist = root.join("dist");
    std::fs::create_dir_all(&dist).unwrap_or_else(|e| fail(&format!("{}: {e}", dist.display())));
    let out = dist.join(format!("{}.js", unit.name));
    std::fs::write(&out, js).unwrap_or_else(|e| fail(&format!("{}: {e}", out.display())));
    Some(out)
}
