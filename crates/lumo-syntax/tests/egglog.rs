//! M1 step 8 golden test (D-37): the compiled `between MIR` egglog
//! program matches the committed fixture. Bless by copying the
//! generated `mir::between::PROGRAM` into the fixture.

use std::path::Path;

#[test]
fn between_mir_egglog_golden() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/egglog/MIR.egg");
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    assert_eq!(
        lumo_syntax::mir::between::PROGRAM.trim_start_matches('\n'),
        expected,
        "compiled egglog program drifted from the golden fixture — \
         review and re-bless tests/fixtures/egglog/MIR.egg"
    );
}
