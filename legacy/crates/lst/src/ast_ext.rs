/// Manual extensions to the generated AST types.
///
/// These live in a separate file so they survive `gen_langue.sh` regeneration.
use crate::ast::{BindPattern, GenericParam};

impl<'a> GenericParam<'a> {
    /// True when the generic param is a capability-row binder (has a `cap` keyword child).
    pub fn is_cap_param(&self) -> bool {
        use crate::lossless::SyntaxElement;
        use crate::SyntaxKind;
        self.0.children.iter().any(|c| matches!(c, SyntaxElement::Token(t) if t.kind == SyntaxKind::CAP_KW))
    }
}

impl<'a> BindPattern<'a> {
    /// True when the pattern was written as `ident(...)` — a constructor without leading `.`.
    pub fn has_call_args(&self) -> bool {
        use crate::lossless::SyntaxElement;
        self.0.children.iter().any(|c| matches!(c, SyntaxElement::Token(t) if t.text == "("))
    }
}
