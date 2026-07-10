# Tokens are named literals/regexes only

A token is a name bound to a string literal or a regex — nothing else:

```
token keyword.fn = 'fn'            // 'fn' displays as keyword.fn
token ident      = /[a-zA-Z_][a-zA-Z0-9_]*/
trivia comment.line = /\/\/[^\n]*/
```

The name is the token's display identity (debug dumps, syntax kinds,
diagnostics). No `keywords()` block or other special forms. Longest match
wins; on ties a literal beats a regex. Dotted names double as highlight
scopes. In grammar rules, literal tokens are written as their literal
(`'fn'`) and regex tokens by name (`name:ident`).
