mod manifest;
mod resolve;

use std::path::PathBuf;
use std::process;

use lumo_compiler::backend::{self, CodegenTarget};
use lumo_compiler::lir;
use lumo_compiler::query::QueryEngine;
use lumo_compiler::typecheck;

use manifest::EntryKind;

/// A build target. The `spec` is a dotted path like `"js"`, `"js.node"`, or `"js.web"`.
/// Directory resolution scans `src#{prefix}/` for each dotted prefix, so
/// `js.node` enables `src#js/` + `src#js.node/`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Target {
    backend: Backend,
    spec: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Backend {
    Js,
    Rust,
}

impl Target {
    /// All directory suffixes to scan, in merge order (base first, variant last).
    /// e.g. `"js.node"` → `["js", "js.node"]`, `"js"` → `["js"]`.
    fn suffixes(&self) -> Vec<String> {
        let parts: Vec<&str> = self.spec.split('.').collect();
        (1..=parts.len()).map(|n| parts[..n].join(".")).collect()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let subcommand = args.first().map(|s| s.as_str());
    match subcommand {
        Some("build") => cmd_build(&args[1..]),
        Some("check") => cmd_check(&args[1..]),
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

/// Returns the user's explicit `--target` flag, or None if unspecified.
fn parse_target_flag(args: &[String]) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--target" {
            if let Some(val) = args.get(i + 1) {
                return Some(val.clone());
            }
        }
        i += 1;
    }
    None
}

fn target_from_spec(raw: &str) -> Target {
    let normalized = match raw {
        "javascript" => "js",
        "rust" => "rs",
        other => other,
    };
    let base = normalized.split('.').next().unwrap_or("");
    let backend = match base {
        "js" => Backend::Js,
        "rs" => Backend::Rust,
        _ => {
            eprintln!("unknown target: {raw}");
            eprintln!("supported bases: js (js.node, js.web, ...), rs");
            process::exit(1);
        }
    };
    Target {
        backend,
        spec: normalized.to_owned(),
    }
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

fn compile(
    manifest: &manifest::Manifest,
    project_root: &std::path::Path,
    target: &Target,
) -> lir::File {
    let mut engine = QueryEngine::new();
    let mut sources: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    let suffixes = target.suffixes();

    // Load common .lumo files from src/
    let src_dir = project_root.join("src");
    collect_lumo_files(&src_dir, &mut sources);

    // Merge platform-specific .lumo files from src#{suffix}/ for each target prefix.
    // Earlier suffixes are base (e.g. "js"), later are variants (e.g. "js.node").
    for suffix in &suffixes {
        let platform_dir = project_root.join(format!("src#{suffix}"));
        merge_lumo_files(&platform_dir, &mut sources);
    }

    if sources.is_empty() {
        eprintln!("error: no .lumo files found in {}", src_dir.display());
        process::exit(1);
    }

    let mut file_names: Vec<String> = sources.keys().cloned().collect();
    file_names.sort();
    for name in &file_names {
        let source = sources.get(name).expect("present by construction");
        engine.set_file(name, source);
    }

    let file_refs: Vec<&str> = file_names.iter().map(|s| s.as_str()).collect();
    let mut resolver = resolve::make_resolver(manifest.deps.clone(), suffixes.clone());
    match engine.compile_with_deps(&file_refs, &mut resolver) {
        Some(lir) => lir,
        None => {
            eprintln!("error: compilation failed");
            process::exit(1);
        }
    }
}

/// Scan a directory for .lumo files and collect their sources.
fn collect_lumo_files(
    dir: &std::path::Path,
    sources: &mut std::collections::HashMap<String, String>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("lumo") {
            continue;
        }
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: cannot read {}: {e}", path.display());
                process::exit(1);
            }
        };
        let basename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let name = format!("src/{basename}");
        sources.insert(name, source);
    }
}

/// Scan a platform directory for .lumo files and merge them with common sources.
/// If a common file with the same basename exists, the platform source is appended.
/// If no common file exists, the platform source stands alone.
fn merge_lumo_files(
    dir: &std::path::Path,
    sources: &mut std::collections::HashMap<String, String>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("lumo") {
            continue;
        }
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: cannot read {}: {e}", path.display());
                process::exit(1);
            }
        };
        let basename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let name = format!("src/{basename}");
        sources
            .entry(name)
            .and_modify(|existing| {
                existing.push('\n');
                existing.push_str(&source);
            })
            .or_insert(source);
    }
}

fn typecheck_or_exit(lir: &lir::File) {
    let type_errors = typecheck::typecheck_file(lir);
    if !type_errors.is_empty() {
        for e in &type_errors {
            eprintln!("error in `{}`: {}", e.fn_name, e.message);
        }
        process::exit(1);
    }
}

fn cmd_build(args: &[String]) {
    let requested = parse_target_flag(args);

    let (project_root, manifest) = match find_manifest() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    let targets_to_build = resolve_build_targets(&manifest, requested.as_deref());
    for target in &targets_to_build {
        build_target(&project_root, &manifest, target);
    }
}

/// Pick which targets to build based on manifest config and optional CLI flag.
/// - `--target X` with a manifest `targets` list: X must be listed (error otherwise).
/// - `--target X` with no manifest `targets`: build X (legacy).
/// - No `--target`, manifest `targets = [...]`: build each listed target.
/// - No `--target`, no `targets`: build default `js`.
fn resolve_build_targets(manifest: &manifest::Manifest, requested: Option<&str>) -> Vec<Target> {
    match (requested, manifest.targets.is_empty()) {
        (Some(spec), true) => vec![target_from_spec(spec)],
        (Some(spec), false) => {
            if !manifest.targets.iter().any(|t| t == spec) {
                eprintln!(
                    "error: target `{spec}` is not in manifest `targets = {:?}`",
                    manifest.targets
                );
                process::exit(1);
            }
            vec![target_from_spec(spec)]
        }
        (None, true) => vec![target_from_spec("js")],
        (None, false) => manifest
            .targets
            .iter()
            .map(|s| target_from_spec(s))
            .collect(),
    }
}

fn build_target(
    project_root: &std::path::Path,
    manifest: &manifest::Manifest,
    target: &Target,
) {
    let lir = compile(manifest, project_root, target);
    // Debug: print LIR items with their span info
    if std::env::var("LBS_DEBUG_SPANS").is_ok() {
        for item in &lir.items {
            match item {
                lir::Item::Fn(f) => {
                    eprintln!("FN {} span={}..{}", f.name, f.span.start, f.span.end);
                }
                _ => {}
            }
        }
    }
    typecheck_or_exit(&lir);

    match target.backend {
        Backend::Js => build_js(&manifest, &lir),
        Backend::Rust => build_rust(&manifest, &lir),
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

fn cmd_check(args: &[String]) {
    let requested = parse_target_flag(args);
    let (project_root, manifest) = match find_manifest() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    // For check, just use the first resolved target.
    let target = resolve_build_targets(&manifest, requested.as_deref())
        .into_iter()
        .next()
        .expect("at least one target");

    let lir = compile(&manifest, &project_root, &target);

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
