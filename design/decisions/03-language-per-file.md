# Every stage is a language declared by its file name

`MIR.syn.langue` existing puts `MIR` into the global namespace,
referenceable as a language. No marker keywords, no surface/internal
distinction, arbitrary chain length — Lumo, MIR, LIR, JS are all equal
individual languages. A language split across files uses the first name
segment (`Lumo.expr.syn.langue`). Per language, parser and pretty-printer
are derived; every stage round-trips as text; emitting JavaScript is
pretty-printing a JS tree.
