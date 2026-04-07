use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: langue <input.langue> [-o <output_dir>]");
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_dir = if args.len() >= 4 && args[2] == "-o" {
        PathBuf::from(&args[3])
    } else {
        // Default: same directory as input
        input_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    };

    let source = match std::fs::read_to_string(&input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {e}", input_path.display());
            std::process::exit(1);
        }
    };

    let grammar = match langue::parser::parse(&source) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Parse error in {}: {e}", input_path.display());
            std::process::exit(1);
        }
    };

    let syntax_kind = langue::codegen::generate_syntax_kind(&grammar);
    let ast = langue::codegen::generate_ast(&grammar);

    std::fs::create_dir_all(&output_dir).unwrap_or_else(|e| {
        eprintln!("Error creating {}: {e}", output_dir.display());
        std::process::exit(1);
    });

    let sk_path = output_dir.join("syntax_kind.rs");
    std::fs::write(&sk_path, &syntax_kind).unwrap_or_else(|e| {
        eprintln!("Error writing {}: {e}", sk_path.display());
        std::process::exit(1);
    });

    let ast_path = output_dir.join("ast.rs");
    std::fs::write(&ast_path, &ast).unwrap_or_else(|e| {
        eprintln!("Error writing {}: {e}", ast_path.display());
        std::process::exit(1);
    });

    eprintln!(
        "Generated {} rules → {} + {}",
        grammar.rules.len(),
        sk_path.display(),
        ast_path.display()
    );
}
