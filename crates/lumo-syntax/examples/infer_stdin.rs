//! Dev helper: run the `:infer(Lumo)` driver over stdin, print the
//! report. Used by scripts to debug the stdlib gate (D-45).

use std::io::Read;

fn main() {
    let mut source = String::new();
    std::io::stdin().read_to_string(&mut source).unwrap();
    let report = lumo_syntax::judge_driver::infer_report(&source);
    if !report.errors.is_empty() {
        eprintln!("{}", report.errors.join("\n"));
        std::process::exit(1);
    }
    println!("{}", report.output);
}
