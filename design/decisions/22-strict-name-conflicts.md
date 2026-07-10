# Name conflicts are strict errors — stdlib included

Two same-named items in the global namespace are an error, strictly —
both in-project and between a project and the stdlib. There is no
shadowing rule: the stdlib participates in the cat like any other file,
and colliding with it is the same error as colliding with a project file.
The only designed exception remains additive merging (same judgment, same
language, same from/to elab block across files).
