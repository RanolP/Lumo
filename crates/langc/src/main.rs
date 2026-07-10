use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

use langc::db::{self, Project, SourceFile};
use langc::diag::Severity;
use langc::project::loader;

const USAGE: &str = "\
langc — Langue 2 definition compiler

USAGE:
    langc check <project-dir>
    langc gen <project-dir> -o <output-dir> [--check]
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") => match args.get(1) {
            Some(dir) => run_check(Path::new(dir)),
            None => usage(),
        },
        Some("gen") => {
            let dir = args.get(1);
            let out = args
                .iter()
                .position(|a| a == "-o")
                .and_then(|i| args.get(i + 1));
            let check_only = args.iter().any(|a| a == "--check");
            match (dir, out) {
                (Some(dir), Some(out)) => run_gen(Path::new(dir), Path::new(out), check_only),
                _ => usage(),
            }
        }
        _ => usage(),
    }
}

fn run_gen(root: &Path, out_dir: &Path, check_only: bool) -> ExitCode {
    let (salsa_db, project, texts) = match load_project(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };

    // Never generate from a broken definition.
    let diags = db::check_definition(&salsa_db, project);
    let mut errors = 0;
    for d in &diags {
        if d.severity == Severity::Error {
            errors += 1;
            let empty = String::new();
            eprintln!("{}", d.render(texts.get(&d.file).unwrap_or(&empty)));
        }
    }
    if errors > 0 {
        eprintln!("langc: {errors} error(s) — nothing generated");
        return ExitCode::FAILURE;
    }

    let files = db::generated_files(&salsa_db, project);
    let mut stale = 0;
    for (rel, content) in &files {
        let path = out_dir.join(rel);
        if check_only {
            let on_disk = std::fs::read_to_string(&path).unwrap_or_default();
            if &on_disk != content {
                eprintln!("langc gen --check: {} is out of date", path.display());
                stale += 1;
            }
        } else {
            if let Some(parent) = path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("langc: cannot create {}: {e}", parent.display());
                    return ExitCode::FAILURE;
                }
            }
            if let Err(e) = std::fs::write(&path, content) {
                eprintln!("langc: cannot write {}: {e}", path.display());
                return ExitCode::FAILURE;
            }
        }
    }
    if check_only {
        if stale > 0 {
            eprintln!("langc gen --check: {stale} file(s) out of date — rerun `langc gen`");
            return ExitCode::FAILURE;
        }
        println!("langc gen --check: {} file(s) up to date", files.len());
    } else {
        println!("langc gen: wrote {} file(s) under {}", files.len(), out_dir.display());
    }
    ExitCode::SUCCESS
}

fn usage() -> ExitCode {
    eprint!("{USAGE}");
    ExitCode::FAILURE
}

type LoadedProject = (salsa::DatabaseImpl, Project, BTreeMap<String, String>);

fn load_project(root: &Path) -> Result<LoadedProject, ExitCode> {
    let files = match loader::scan_project(root) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("langc: cannot read {}: {e}", root.display());
            return Err(ExitCode::FAILURE);
        }
    };
    if files.is_empty() {
        eprintln!("langc: no .langue files under {}", root.display());
        return Err(ExitCode::FAILURE);
    }

    let texts: BTreeMap<String, String> =
        files.iter().map(|f| (f.path.clone(), f.text.clone())).collect();

    let salsa_db = salsa::DatabaseImpl::default();
    let inputs: Vec<SourceFile> = files
        .into_iter()
        .map(|f| SourceFile::new(&salsa_db, f.path, f.kind, f.text))
        .collect();
    let project = Project::new(&salsa_db, inputs);
    Ok((salsa_db, project, texts))
}

fn run_check(root: &Path) -> ExitCode {
    let (salsa_db, project, texts) = match load_project(root) {
        Ok(loaded) => loaded,
        Err(code) => return code,
    };

    let diags = db::check_definition(&salsa_db, project);
    let mut errors = 0;
    for d in &diags {
        if d.severity == Severity::Error {
            errors += 1;
        }
        let empty = String::new();
        let text = texts.get(&d.file).unwrap_or(&empty);
        eprintln!("{}", d.render(text));
    }

    if errors > 0 {
        eprintln!("langc: {errors} error(s)");
        ExitCode::FAILURE
    } else {
        println!("langc: check clean ({} warning(s))", diags.len());
        ExitCode::SUCCESS
    }
}
