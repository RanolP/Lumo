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
            eprintln!("langc gen: not implemented yet");
            ExitCode::FAILURE
        }
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprint!("{USAGE}");
    ExitCode::FAILURE
}

fn run_check(root: &Path) -> ExitCode {
    let files = match loader::scan_project(root) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("langc: cannot read {}: {e}", root.display());
            return ExitCode::FAILURE;
        }
    };
    if files.is_empty() {
        eprintln!("langc: no .langue files under {}", root.display());
        return ExitCode::FAILURE;
    }

    let texts: BTreeMap<String, String> =
        files.iter().map(|f| (f.path.clone(), f.text.clone())).collect();

    let salsa_db = salsa::DatabaseImpl::default();
    let inputs: Vec<SourceFile> = files
        .into_iter()
        .map(|f| SourceFile::new(&salsa_db, f.path, f.kind, f.text))
        .collect();
    let project = Project::new(&salsa_db, inputs);

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
