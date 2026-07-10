//! Deterministic name mappings from `.langue` names to generated Rust.

/// `keyword.fn` → `KEYWORD_FN`, `FnDecl` → `FN_DECL`, `lit.number` →
/// `LIT_NUMBER`. Dots and dashes become underscores; camel boundaries get
/// underscores; everything is uppercased.
pub fn kind_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    let chars: Vec<char> = name.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        match c {
            '.' | '-' => out.push('_'),
            c if c.is_ascii_uppercase() => {
                let prev_lower = i > 0 && chars[i - 1].is_ascii_lowercase();
                let upper_run_ends =
                    i > 0 && chars[i - 1].is_ascii_uppercase()
                        && chars.get(i + 1).is_some_and(|n| n.is_ascii_lowercase());
                if prev_lower || upper_run_ends {
                    out.push('_');
                }
                out.push(c);
            }
            c => out.push(c.to_ascii_uppercase()),
        }
    }
    out
}

/// `FnDecl` → `fn_decl` — for generated function names.
pub fn snake(name: &str) -> String {
    kind_name(name).to_ascii_lowercase()
}

/// `Lumo` → `lumo` — the generated module directory for a language.
pub fn module_name(lang: &str) -> String {
    lang.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_names() {
        assert_eq!(kind_name("keyword.fn"), "KEYWORD_FN");
        assert_eq!(kind_name("lit.number"), "LIT_NUMBER");
        assert_eq!(kind_name("ident"), "IDENT");
        assert_eq!(kind_name("FnDecl"), "FN_DECL");
        assert_eq!(kind_name("ParenExpr"), "PAREN_EXPR");
        assert_eq!(kind_name("MIRExpr"), "MIR_EXPR");
        assert_eq!(snake("FnDecl"), "fn_decl");
    }
}
