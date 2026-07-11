//! End-to-end Lumo → JS (M4 step 4, D-43): the manifest pipe's elab
//! stages chained — parse Lumo, elab to MIR, reparse, elab to JS.
//! Wired into the corpus as `:elab(Lumo -> JS)`; the expected block is
//! canonicalized by the JS parser like any other elab pair (D-32).

use langue_rt::ElabReport;

pub fn compile_report(source: &str) -> ElabReport {
    let mut mir_externs = crate::elab_externs::lumo_to_mir();
    let mir = crate::elab::lumo_to_mir::elab(source, mir_externs.as_mut());
    if !mir.errors.is_empty() {
        return ElabReport {
            output: String::new(),
            errors: mir.errors.iter().map(|e| format!("elab Lumo->MIR: {e}")).collect(),
        };
    }
    let mut js_externs = crate::elab_externs::mir_to_js();
    let js = crate::elab::mir_to_js::elab(&mir.output, js_externs.as_mut());
    ElabReport {
        output: js.output,
        errors: js.errors.iter().map(|e| format!("elab MIR->JS: {e}")).collect(),
    }
}
