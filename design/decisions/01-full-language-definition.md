# Full-language-definition DSL

One DSL — **Langue 2** — defines all of Lumo: tokens, grammar, every
pipeline language, elaboration, typing. Rust code is engines that execute
the definition plus explicitly-declared escape hatches, never the
definition itself. The name stays **"Langue" (v2)**; tooling lives in
**Rust** (`crates/langc`); langue 1 (grammar-only) is an ancestor subset.
