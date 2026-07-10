use std::process::ExitCode;

const USAGE: &str = "\
langc — Langue 2 definition compiler

USAGE:
    langc check <project-dir>
    langc gen <project-dir> -o <output-dir> [--check]
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") | Some("gen") => {
            eprintln!("langc: not implemented yet");
            ExitCode::FAILURE
        }
        _ => {
            eprint!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}
