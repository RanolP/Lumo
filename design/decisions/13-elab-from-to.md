# Elab rule form: `from A to B` blocks

```
from Lumo to MIR {
  FnDecl { ... } ==> Lambda { ..., body: Lumo::FnDecl { ... } to MIR }
  ...
}

// merged across multiple definitions, multiple files
from Lumo to MIR { ... }
```

A source pattern `==>` a target construction; `<subtree> to <Lang>` inside
a construction is recursive elaboration; node names qualify as
`Lumo::FnDecl`; same from/to blocks merge across files. Two checked
constraints: **only strictly decreasing allowed** (a recursive `to` call
takes a strictly smaller input, so elaboration terminates) and
**conflicting disallowed** (two rules that can fire on the same input are
an error; no rule ordering or priority).
