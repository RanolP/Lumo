# Manifest pipe syntax

Dictated (2026-07-11). The manifest binds a named entry point to a pipe of
stages:

```
main = parse Lumo | elab Lumo to MIR | elab MIR to LIR | check_V LIR | ...
```

Grammar:

```
Manifest = name '=' Stage ('|' Stage)*
Stage    = 'parse' Lang
         | 'elab' Lang 'to' Lang
         | judgment Lang          // e.g. check_V LIR — any judgment name
```

- `parse <Lang>` is the first stage; its language is the DCE root (D-05).
- `elab <A> to <B>` selects the merged `from A to B` block (D-13).
- `<judgment> <Lang>` runs a declared judgment over the language (D-17).
- Multiple pipelines may be named; `main` is the entry point.

Until elaboration exists (M1), only the first stage's language is
consumed (as the DCE root); later stages are parsed and validated for
shape only.
