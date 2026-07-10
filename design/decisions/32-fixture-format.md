# Golden fixture format: tree-sitter-corpus shape

Files at `tests/fixtures/{syn,elab,type}/**/*.test`, glob-discovered,
many cases per file. Each case: `=`-fenced title + `:`-attributes,
source, `---`, expected. The attribute names the check:

- `:parse(L)` — expected is the named-node S-expression; every parse
  fixture automatically also checks parse → pretty-print → re-parse tree
  equality.
- `:elab(A -> B)` — input is A-text, expected is B-text; comparison is
  canonicalize-then-compare (parse expected with B's parser,
  pretty-print both sides, compare strings) — whitespace-robust, and
  each expected block doubles as a B round-trip test.
- `:optimize(L)` — between saturation + extraction; input and expected
  both L-text.
- `:infer(L)` — expected is `name : Type` lines, types printed by the
  type sub-language's own printer.
- `:fails` — no expected section (matches deferred diagnostics; messages
  can be added under `---` later without a format change).

`--update` bless mode regenerates expected blocks. Fixture sources:
tree-sitter and the legacy project
(`legacy/crates/compiler/tests/fixtures/` — already `input / --- /
expected`, so migration is cheap).
