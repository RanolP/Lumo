//! Handwritten extern implementations for the generated elab modules
//! (D-01/D-38). The generated registry constructs these by convention:
//! `crate::elab_externs::{from}_to_{to}()`.

use langue_rt::{ElabCtx, Frag};

use crate::elab::lumo_to_mir::{Externs, ToFrag};
use crate::mir::builder;
use crate::mir::syntax_kind::SyntaxKind as MirKind;

/// Is this MIR kind a computation (the `Comp` sort of D-36)?
fn is_comp(kind: MirKind) -> bool {
    matches!(
        kind,
        MirKind::RET_C
            | MirKind::LET_C
            | MirKind::LAM_C
            | MirKind::FORCE_C
            | MirKind::CASE_C
            | MirKind::FIX_C
            | MirKind::PERFORM_C
            | MirKind::HANDLE_C
            | MirKind::SEL_C
            | MirKind::PAREN_C
            | MirKind::COMP_POSTFIX
    )
}

/// Is this MIR kind a value (the `Value` sort of D-36)?
fn is_value(kind: MirKind) -> bool {
    matches!(
        kind,
        MirKind::VAR_V
            | MirKind::NUM_V
            | MirKind::STR_V
            | MirKind::THUNK_V
            | MirKind::CTOR_V
            | MirKind::ROLL_V
            | MirKind::UNROLL_V
            | MirKind::BUNDLE_V
            | MirKind::PAREN_V
    )
}

pub struct LumoToMir;

impl Externs for LumoToMir {
    /// D-38 sort coercion between the two MIR sorts:
    /// - computation where a value is expected → bind it to a fresh
    ///   `__tN` (the binder is pending until the nearest computation);
    /// - value where a computation is expected → `ret v` (F-intro).
    fn coerce(&mut self, ctx: &mut ElabCtx, expected: &'static str, frag: &ToFrag) -> Option<ToFrag> {
        let kind = frag.kind?;
        let expects_value = matches!(expected, "Value" | "ValueArgs");
        let expects_comp = expected == "Comp";
        if expects_value && is_comp(kind) {
            let var = ctx.fresh();
            let mut out = Frag::node(MirKind::VAR_V, builder::var_v(&var));
            out.pending.push((var, frag.text.clone()));
            return Some(out);
        }
        if expects_comp && is_value(kind) {
            return Some(Frag::node(MirKind::RET_C, builder::ret_c(&frag.text)));
        }
        None
    }

    fn is_binder_site(&mut self, kind: MirKind) -> bool {
        is_comp(kind)
    }

    fn wrap_pending(
        &mut self,
        pending: &[(String, String)],
        body: &str,
        _kind: MirKind,
    ) -> (String, MirKind) {
        let mut text = body.to_owned();
        for (var, comp) in pending.iter().rev() {
            text = builder::let_c(var, comp, &text);
        }
        (text, MirKind::LET_C)
    }
}

pub fn lumo_to_mir() -> Box<dyn Externs> {
    Box::new(LumoToMir)
}
