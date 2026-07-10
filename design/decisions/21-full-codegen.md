# All three kinds are code-generated

syn, elab, and type are all generated as Rust code by `langc` — there are
no interpreted rule tables. Per kind: syn → token DFA, syntax kinds, AST
accessors, parser, pretty-printer (per declared language); elab →
`from`/`to` rewrites and `between` relations, compiled to Rust running on
egglog; type → judgment rules and diagnostic templates compiled to Rust on
the relational engine. One workflow everywhere: edit, `langc gen`, commit.
