# How elaboration simulates scope

- **Name resolution = Γ contexts** — the judgment-context machinery does
  name resolution; no separate resolver.
- **`use` statements** are ordered to be first in the tree and translate
  as `λrequire. let x = require('x') in ...` — the module becomes a
  function of `require`, each use a `let` binding.
- **There is no dynamic scope.** Capability handlers take the
  Effekt-like approach: lexically scoped, explicitly passed capabilities.
