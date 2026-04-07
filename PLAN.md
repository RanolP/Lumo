1. 언어는 Rust로 한다
2. 그 어떤 라이브러리도 사용하지 않는다 (부트스트래핑에 용이)
3. Query-based Compiler를 짜서 compiler-language server 간 코드를 공유한다
4. Lossless Syntax Tree를 사용한다
5. Tree Sitter 논문에서 error-recovering parser 접근을 취한다
6. IR 전략
   6.1. IR는 우선 분리한다 (LST→HIR→LIR)
   6.2. 추적성은 노드 Unique ID로 보장한다 (ID=정체성)
   6.3. 성능/증분 처리를 위해 Structural Hash를 병행한다 (Hash=동등성/캐시 키)

## Done

- M1 Lexer: 키워드/식별자/기호 토큰화 및 span, 골든 테스트
- M2 Parser: data, fn, let-in, produce, match, thunk/force, error recovery
- M3 IR: LST→HIR→LIR lowering, Unique ID, Structural Hash
- M4 Query 엔진: parse/lower/diagnostics query, 캐시 재사용
- M5 LSP 하이라이팅: semantic tokens (full + delta)
- Pratt operator parsing (binary/unary, precedence, associativity)
- Algebraic capabilities: `cap`, `perform`, `handle`, bundles, multi-shot resume
- Data types: generic ADTs, dot-variant syntax, recursive types
- Pattern matching: nested destructuring, variant tags, exhaustiveness checking, wildcards, unreachable detection
- Operator desugaring to capability method calls
- `use` declarations (parsing only)
- Extern types/functions with `#[extern(...)]` attributes
- Typechecker: CBPV, bidirectional, capability checking
- TS/JS/d.ts backend with deep CPS for capabilities
- effect → cap rename
- Build system (`lbs`): `lumo.toml` manifest, filesystem module resolver, `lbs build`/`lbs check`, `--target` flag
- Rust backend: direct type mapping (String→String, Number→f64, ADTs→enums), pattern matching, extern fn mapping, Cargo project generation
- Rust backend: recursive ADTs with Box<T>, generic bounds (Clone+Debug)
- Stdlib: Bool, List[A] (recursive ADT), string ops, number ops, file I/O, process args
- `impl` blocks: inherent impls (vtable/const object), capability impls (unnamed/named), generic impls, `self` sugar, TS + Rust backend emission

- Langue phase 1: `crates/langue` — `.langue` grammar DSL parser + codegen (SyntaxKind enum + typed AST accessors), `lumo.langue` grammar (51 rules)

## To Do

- Langue phase 2: integrate generated SyntaxKind + AST accessors into compiler, migrate lossless parser
- Langue phase 3: self-hosting — rewrite langue in Lumo using generated infrastructure
- Pattern matching extensions: guards, literal patterns, or-patterns, as-patterns
- Generic type bound enforcement (syntax parses, checking is missing)
- LSP: diagnostics publishing, hover, go-to-definition, completions
- Source maps for generated JS
- Optimization passes (dead code elimination, inlining)
- Additional backends (Python, WASM)
