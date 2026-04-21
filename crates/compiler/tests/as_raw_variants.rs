//! Tests for the `#[as__raw(true|false)]` attribute on data variants.
//!
//! The attribute tells the TS/JS backend to emit the raw JS literal for
//! that variant instead of the default `{ [LUMO_TAG]: "name" }` tagged
//! object. Pattern matching uses `=== true` / `=== false` comparisons.
//!
//! Platform-specific overrides: a common `data Bool { .true, .false }`
//! decl in `src/` can coexist with a redefinition in `src#js/` that adds
//! `#[as__raw(...)]` to each variant. The merge-lumo-files text-concat in
//! `lbs` produces a single source with two `data Bool` decls; the HIR
//! pipeline keeps only the one carrying `as_raw`.

use lumo_compiler::{
    backend::{self, CodegenTarget},
    hir,
    lexer::lex,
    lir,
    parser::parse,
};

fn compile_js(src: &str) -> String {
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    let h = hir::lower(&parsed.file);
    let l = lir::lower(&h);
    backend::emit(&l, CodegenTarget::JavaScript).expect("js emit")
}

fn compile_ts(src: &str) -> String {
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    let h = hir::lower(&parsed.file);
    let l = lir::lower(&h);
    backend::emit(&l, CodegenTarget::TypeScript).expect("ts emit")
}

#[test]
fn as_raw_variants_emit_raw_literals_in_js() {
    // Simulates the concatenated output of `src/prelude.lumo` +
    // `src#js/prelude.lumo` produced by `lbs::merge_lumo_files`.
    let src = r#"
        data Bool { .true, .false }

        data Bool {
          #[as__raw(true)]
          .true,
          #[as__raw(false)]
          .false,
        }

        fn yes(): Bool { Bool.true }
        fn no(): Bool { Bool.false }
    "#;

    let js = compile_js(src);

    // Ctor uses: `Bool.true` should become the raw `true` literal, not
    // a bundle index like `Bool["true"]`.
    assert!(
        js.contains("return true"),
        "expected `return true` in generated JS, got:\n{js}"
    );
    assert!(
        js.contains("return false"),
        "expected `return false` in generated JS, got:\n{js}"
    );

    // The bundle itself should also carry raw values (so any remaining
    // `Bool["true"]` accesses still produce a raw literal).
    assert!(
        js.contains("\"true\": true") || js.contains("'true': true"),
        "expected bundle with raw `true`, got:\n{js}"
    );
    assert!(
        js.contains("\"false\": false") || js.contains("'false': false"),
        "expected bundle with raw `false`, got:\n{js}"
    );

    // No tagged-object emission for this type.
    assert!(
        !js.contains("LUMO_TAG]: 'true'") && !js.contains("LUMO_TAG]: \"true\""),
        "should not emit tagged Bool.true, got:\n{js}"
    );
}

#[test]
fn as_raw_variants_use_direct_eqeq_in_match() {
    let src = r#"
        data Bool {
          #[as__raw(true)]
          .true,
          #[as__raw(false)]
          .false,
        }

        extern type Number;

        fn to_num(b: Bool): Number {
          match b {
            .true => 1,
            .false => 0,
          }
        }
    "#;

    let js = compile_js(src);

    // Match on Bool: `b === true` is simplified to just `b` by bool optimization,
    // and `[LUMO_TAG]` tag comparison is never used for #[as__raw] variants.
    assert!(js.contains("if (b)"), "expected simplified bool condition, got:\n{js}");
    assert!(
        !js.contains("[LUMO_TAG] === \"true\""),
        "should not use tagged comparison, got:\n{js}"
    );
}

#[test]
fn as_raw_ts_type_uses_literal_types() {
    let src = r#"
        data Bool {
          #[as__raw(true)]
          .true,
          #[as__raw(false)]
          .false,
        }
    "#;

    let ts = compile_ts(src);

    // The TS union should be `true | false` rather than tagged objects.
    assert!(
        ts.contains("export type Bool = true | false"),
        "expected raw literal TS union, got:\n{ts}"
    );
}

#[test]
fn duplicate_data_without_as_raw_still_errors() {
    // When NEITHER decl carries `#[as__raw]`, preserve the existing
    // duplicate-type error (no silent shadowing).
    let src = r#"
        data Foo { .a, .b }
        data Foo { .a, .b }
    "#;
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    let h = hir::lower(&parsed.file);
    let errs = hir::check::check_file(&h);
    assert!(
        errs.iter().any(|e| format!("{e:?}").contains("duplicate type `Foo`")),
        "expected duplicate type error, got: {errs:?}"
    );
}
