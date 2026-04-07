mod manifest;
mod resolve;

use std::path::PathBuf;
use std::process;

use lumo_compiler::backend::{self, CodegenTarget};
use lumo_compiler::lir;
use lumo_compiler::query::QueryEngine;
use lumo_compiler::typecheck;

use manifest::EntryKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    Js,
    Rust,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let subcommand = args.first().map(|s| s.as_str());
    match subcommand {
        Some("build") => cmd_build(&args[1..]),
        Some("check") => cmd_check(),
        Some(other) => {
            eprintln!("unknown command: {other}");
            eprintln!("usage: lbs <build|check> [--target js|rust]");
            process::exit(1);
        }
        None => {
            eprintln!("usage: lbs <build|check> [--target js|rust]");
            process::exit(1);
        }
    }
}

fn parse_target(args: &[String]) -> Target {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--target" {
            if let Some(val) = args.get(i + 1) {
                return match val.as_str() {
                    "js" | "javascript" => Target::Js,
                    "rust" | "rs" => Target::Rust,
                    other => {
                        eprintln!("unknown target: {other}");
                        eprintln!("supported: js, rust");
                        process::exit(1);
                    }
                };
            }
        }
        i += 1;
    }
    Target::Js
}

fn find_manifest() -> Result<(PathBuf, manifest::Manifest), String> {
    let mut dir = std::env::current_dir().map_err(|e| format!("cannot get cwd: {e}"))?;

    loop {
        let candidate = dir.join("lumo.toml");
        if candidate.exists() {
            let content =
                std::fs::read_to_string(&candidate).map_err(|e| format!("read error: {e}"))?;
            let m = manifest::parse(&content, &dir)?;
            return Ok((dir, m));
        }
        if !dir.pop() {
            return Err("lumo.toml not found (searched from cwd upwards)".into());
        }
    }
}

fn entry_path(entry: &EntryKind) -> &std::path::Path {
    match entry {
        EntryKind::Bin(p) | EntryKind::Lib(p) => p,
    }
}

fn compile(manifest: &manifest::Manifest, project_root: &std::path::Path) -> lir::File {
    let mut engine = QueryEngine::new();
    let mut file_names = Vec::new();

    // Load all .lumo files in src/ — they form a single compilation unit.
    let src_dir = project_root.join("src");
    if let Ok(entries) = std::fs::read_dir(&src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("lumo") {
                let source = match std::fs::read_to_string(&path) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("error: cannot read {}: {e}", path.display());
                        process::exit(1);
                    }
                };
                let name = path
                    .strip_prefix(project_root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                engine.set_file(&name, &source);
                file_names.push(name);
            }
        }
    }

    if file_names.is_empty() {
        eprintln!("error: no .lumo files found in {}", src_dir.display());
        process::exit(1);
    }

    let file_refs: Vec<&str> = file_names.iter().map(|s| s.as_str()).collect();
    let mut resolver = resolve::make_resolver(manifest.deps.clone());
    match engine.compile_with_deps(&file_refs, &mut resolver) {
        Some(lir) => lir,
        None => {
            eprintln!("error: compilation failed");
            process::exit(1);
        }
    }
}

fn typecheck_or_exit(lir: &lir::File) {
    let type_errors = typecheck::typecheck_file(lir);
    if !type_errors.is_empty() {
        for e in &type_errors {
            eprintln!("error: {}", e.message);
        }
        process::exit(1);
    }
}

fn cmd_build(args: &[String]) {
    let target = parse_target(args);

    let (project_root, manifest) = match find_manifest() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    let lir = compile(&manifest, &project_root);
    typecheck_or_exit(&lir);

    match target {
        Target::Js => build_js(&manifest, &lir),
        Target::Rust => build_rust(&manifest, &lir),
    }
}

fn build_js(manifest: &manifest::Manifest, lir: &lir::File) {
    let js = match backend::emit(lir, CodegenTarget::JavaScript) {
        Ok(js) => js,
        Err(e) => {
            eprintln!("error: codegen failed: {e:?}");
            process::exit(1);
        }
    };

    if let Err(e) = std::fs::create_dir_all(&manifest.out_dir) {
        eprintln!(
            "error: cannot create output dir {}: {e}",
            manifest.out_dir.display()
        );
        process::exit(1);
    }

    let out_file = manifest.out_dir.join(format!("{}.js", manifest.name));
    // For bin entries, auto-invoke main()
    let js = if matches!(manifest.entry, EntryKind::Bin(_)) {
        format!("{js}\nmain();\n")
    } else {
        js
    };
    if let Err(e) = std::fs::write(&out_file, &js) {
        eprintln!("error: cannot write {}: {e}", out_file.display());
        process::exit(1);
    }

    eprintln!("built {}", out_file.display());
}

fn build_rust(manifest: &manifest::Manifest, lir: &lir::File) {
    let rs_code = match backend::emit(lir, CodegenTarget::Rust) {
        Ok(rs) => rs,
        Err(e) => {
            eprintln!("error: codegen failed: {e:?}");
            process::exit(1);
        }
    };

    // Create Cargo project structure
    let src_dir = manifest.out_dir.join("src");
    if let Err(e) = std::fs::create_dir_all(&src_dir) {
        eprintln!(
            "error: cannot create output dir {}: {e}",
            src_dir.display()
        );
        process::exit(1);
    }

    // Write Cargo.toml (with [workspace] to prevent parent workspace detection)
    let cargo_toml = format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
        manifest.name
    );
    let cargo_toml_path = manifest.out_dir.join("Cargo.toml");
    if let Err(e) = std::fs::write(&cargo_toml_path, &cargo_toml) {
        eprintln!("error: cannot write {}: {e}", cargo_toml_path.display());
        process::exit(1);
    }

    // Write main.rs or lib.rs based on entry kind
    let rs_file = match &manifest.entry {
        EntryKind::Bin(_) => src_dir.join("main.rs"),
        EntryKind::Lib(_) => src_dir.join("lib.rs"),
    };
    if let Err(e) = std::fs::write(&rs_file, &rs_code) {
        eprintln!("error: cannot write {}: {e}", rs_file.display());
        process::exit(1);
    }

    eprintln!("built {}", manifest.out_dir.display());
}

fn cmd_check() {
    let (project_root, manifest) = match find_manifest() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    let lir = compile(&manifest, &project_root);

    let type_errors = typecheck::typecheck_file(&lir);
    if type_errors.is_empty() {
        eprintln!("no errors");
    } else {
        for e in &type_errors {
            eprintln!("error: {}", e.message);
        }
        process::exit(1);
    }
}
