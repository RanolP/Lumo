//! Handwritten extern implementations for the generated elab modules
//! (D-01/D-38). The generated registry constructs these by convention:
//! `crate::elab_externs::{from}_to_{to}()`.
//!
//! Reference semantics: the legacy HIR→LIR lowering
//! (`legacy/crates/lir/src/lib.rs`) — curried thunk spines for fn decls,
//! force-then-apply calls, ctors wrapped in roll, match auto-unroll —
//! except capabilities, which pass lexically since D-51:
//! `Cap.op` → `sel <provider> . op` (handle binding, row param, or
//! default impl), never `perform`.

use std::collections::BTreeSet;

use langue_rt::{ElabCtx, Frag, PassPhase};

use crate::elab::lumo_to_mir::{elab_node, Externs, ToFrag};
use crate::lumo::ast::{self as l, AstNode as _};
use crate::lumo::lossless::SyntaxNode as LumoNode;
use crate::lumo::syntax_kind::SyntaxKind as LumoKind;
use crate::mir::ast::{self as m, AstNode as _};
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

#[derive(Default)]
pub struct LumoToMir {
    /// `(owner, variant)` pairs of every `data` declaration in scope.
    variants: BTreeSet<(String, String)>,
    /// Names of every `cap` declaration in scope.
    caps: BTreeSet<String>,
    /// Caps with a bare impl (`impl Cap { … }`) in scope (D-44).
    impl_caps: BTreeSet<String>,
    /// `(cap, target)` pairs of typeclass impls (`impl T: Cap`, D-48).
    typeclass_impls: BTreeSet<(String, String)>,
    /// `(target, method)` → return type-constructor head of inherent
    /// impl methods (`impl T { fn m(self, …) }`, D-49/D-50).
    inherent_methods: std::collections::BTreeMap<(String, String), Option<String>>,
    /// Variant tag → field type-constructor heads, `None` when the
    /// tag is ambiguous across data decls (D-50 match-binder typing).
    variant_fields: std::collections::BTreeMap<String, Option<Vec<Option<String>>>>,
    /// fn/extern-fn names → simple named return types (D-48 dispatch).
    fn_returns: std::collections::BTreeMap<String, String>,
    /// `(cap, op)` → simple named return types (D-48 dispatch).
    cap_op_returns: std::collections::BTreeMap<(String, String), String>,
    /// Annotated fn/lambda params in scope: name → simple type name.
    scope: Vec<(String, String)>,
    /// fn name → declared row caps, in source order (D-51 threading).
    fn_rows: std::collections::BTreeMap<String, Vec<String>>,
    /// Lexical capability providers, innermost last: cap name → the
    /// MIR value text (a var) that carries it (D-51).
    cap_env: Vec<(String, String)>,
    /// When elaborating a typeclass impl: `Self` in types reads as
    /// this target (D-48).
    self_target: Option<String>,
}

/// The syntactic shape of an `impl` head (D-44/D-48/D-50). Membership
/// checks (is the cap declared?) are the caller's.
pub enum ImplForm {
    /// `impl Cap { … }`
    Bare(String),
    /// `impl Target: Cap { … }` with a ground target
    Typeclass { cap: String, target: String },
    /// `impl[A, …] Target[A, …] { … }` — head args are exactly the
    /// binders (D-50)
    GenericInherent { target: String, binders: Vec<String> },
    /// named / assoc / generic typeclass / non-ground — later slices
    Other,
}

/// A type expression as a plain type name (`Number`, no args/assoc) —
/// the ground test of impl-head classification.
fn plain_type_name(ty: &l::TypeExpr<'_>) -> Option<String> {
    let l::TypeExpr::NamedTypeExpr(n) = ty else { return None };
    if n.generic_args().is_some() || n.assoc().is_some() {
        return None;
    }
    Some(n.name()?.text.clone())
}

/// The type-constructor head name (`List[Number]` → `List`) — the
/// currency of the D-48/D-50 syntactic dispatch table.
fn head_type_name(ty: &l::TypeExpr<'_>) -> Option<String> {
    let l::TypeExpr::NamedTypeExpr(n) = ty else { return None };
    if n.assoc().is_some() {
        return None;
    }
    Some(n.name()?.text.clone())
}

/// The named caps of a row annotation, in source order (`..`/`..c`
/// rests are vestigial under capability passing, D-51).
fn row_caps(annotation: Option<l::CapAnnotation<'_>>) -> Vec<String> {
    let mut out = Vec::new();
    let Some(set) = annotation.and_then(|a| a.cap()) else { return out };
    for entry in set.entries() {
        if let Some(l::CapEntryBody::CapSig(sig)) = entry.body() {
            if let Some(name) = sig.name() {
                out.push(name.text.clone());
            }
        }
    }
    out
}

/// An impl's methods whose first parameter is `self` — the ones that
/// participate in D-49 UFCS dispatch and instance-cap derivation.
pub fn self_methods<'a>(i: &l::ImplDecl<'a>) -> Vec<l::ImplMethod<'a>> {
    let mut out = Vec::new();
    for item in i.items() {
        let Some(l::ImplItemBody::ImplMethod(m)) = item.body() else { continue };
        let first = m.param_list().and_then(|l| l.params().next());
        if first.and_then(|p| p.name().map(|t| t.text == "self")).unwrap_or(false) {
            out.push(m);
        }
    }
    out
}

/// Classify an impl head.
pub fn impl_form(i: &l::ImplDecl<'_>) -> ImplForm {
    if i.assign().is_some() {
        return ImplForm::Other;
    }
    // Generic (D-50): `impl[A] Target[A] { … }` — inherent only, and
    // the head's args must be exactly the declared binders.
    if let Some(generics) = i.generic_params() {
        if i.cap().is_some() {
            return ImplForm::Other;
        }
        let mut binders = Vec::new();
        for p in generics.params() {
            if p.constraint().is_some() {
                return ImplForm::Other;
            }
            let Some(n) = p.name() else { return ImplForm::Other };
            binders.push(n.text.clone());
        }
        let Some(l::TypeExpr::NamedTypeExpr(head)) = i.head() else {
            return ImplForm::Other;
        };
        if head.assoc().is_some() {
            return ImplForm::Other;
        }
        let Some(name) = head.name() else { return ImplForm::Other };
        let mut args = Vec::new();
        if let Some(generic_args) = head.generic_args() {
            for arg in generic_args.args() {
                let Some(a) = plain_type_name(&arg) else { return ImplForm::Other };
                args.push(a);
            }
        }
        if args != binders || binders.is_empty() {
            return ImplForm::Other;
        }
        return ImplForm::GenericInherent { target: name.text.clone(), binders };
    }
    let Some(head) = i.head().as_ref().and_then(plain_type_name) else {
        return ImplForm::Other;
    };
    match i.cap() {
        None => ImplForm::Bare(head),
        Some(c) => match c.cap().as_ref().and_then(plain_type_name) {
            Some(cap) => ImplForm::Typeclass { cap, target: head },
            None => ImplForm::Other,
        },
    }
}

impl LumoToMir {
    /// A frag as a computation: values return (`ret v`, F-intro), and
    /// pending binders flush here (a computation is a binder site).
    fn to_comp(&mut self, frag: ToFrag) -> ToFrag {
        let mut frag = frag;
        if frag.kind.is_some_and(is_value) {
            frag = ToFrag {
                kind: Some(MirKind::RET_C),
                text: builder::ret_c(&frag.text),
                pending: frag.pending,
            };
        }
        self.flush(frag)
    }

    /// A frag as a value: computations bind to a fresh `__tN` (D-38).
    fn to_value(&mut self, ctx: &mut ElabCtx, frag: ToFrag) -> ToFrag {
        if frag.kind.is_some_and(is_comp) {
            let var = ctx.fresh();
            let mut out = Frag::node(MirKind::VAR_V, builder::var_v(&var));
            out.pending = frag.pending;
            out.pending.push((var, frag.text));
            return out;
        }
        frag
    }

    fn flush(&mut self, mut frag: ToFrag) -> ToFrag {
        if let Some(kind) = frag.kind {
            if !frag.pending.is_empty() && self.is_binder_site(kind) {
                let pending = std::mem::take(&mut frag.pending);
                let (text, new_kind) = self.wrap_pending(&pending, &frag.text, kind);
                frag.text = text;
                frag.kind = Some(new_kind);
            }
        }
        frag
    }

    /// Elaborate a Lumo fn body (`= expr` or a block) to a computation.
    fn elab_body(&mut self, ctx: &mut ElabCtx, body: l::FnBody<'_>) -> Option<ToFrag> {
        let frag = elab_node(ctx, self, body.syntax())?;
        Some(self.to_comp(frag))
    }

    /// The curried thunk spine (legacy `lower_fn_value`): fold params
    /// right-to-left into single-param lambdas, then thunk.
    fn curry(&mut self, params: &[String], body: ToFrag) -> ToFrag {
        let mut text = body.text;
        for param in params.iter().rev() {
            text = builder::lam_c(param, &text);
        }
        Frag::node(MirKind::THUNK_V, builder::thunk_v(&text))
    }

    /// Lumo TypeExpr → MIR TypeV text. `None` = not spellable in MIR
    /// (assoc types are deferred, D-39) — the caller drops the
    /// annotation and leaves the def to inference.
    fn type_v(&mut self, ty: &l::TypeExpr<'_>) -> Option<String> {
        match ty {
            l::TypeExpr::NamedTypeExpr(n) => {
                if n.assoc().is_some() {
                    return None;
                }
                let mut name = n.name()?.text.clone();
                // D-48: inside a typeclass impl, `Self` is the target.
                if name == "Self" {
                    name = self.self_target.clone()?;
                }
                let args = match n.generic_args() {
                    None => None,
                    Some(args) => Some(self.type_args(args)?),
                };
                Some(builder::named_type_v(&name, args.as_deref()))
            }
            // `thunk T` is U of T-as-computation.
            l::TypeExpr::ThunkTypeExpr(t) => {
                let inner = self.type_c(&t.inner()?)?;
                Some(builder::u_type_v(&inner))
            }
            // A fn type in value position is an implicit thunk.
            l::TypeExpr::FnTypeExpr(f) => {
                let inner = self.fn_type_c(f)?;
                Some(builder::u_type_v(&inner))
            }
        }
    }

    /// Lumo TypeExpr → MIR TypeC text: fn types curry, anything else
    /// is a returner `F(V)`.
    fn type_c(&mut self, ty: &l::TypeExpr<'_>) -> Option<String> {
        match ty {
            l::TypeExpr::FnTypeExpr(f) => self.fn_type_c(f),
            _ => Some(builder::f_type_c(&self.type_v(ty)?, None)),
        }
    }

    /// `(A, B) -> R / {row}` curries one param per arrow to mirror
    /// `curry()`; zero params yield the bare `F` (a nullary fn is just
    /// a thunk). The row sits on the innermost F (D-41). The return is
    /// always a value type — a fn-typed return is an implicit thunk
    /// (`F(U(…))`), matching the `ret`-wrapped term the elab emits.
    fn fn_type_c(&mut self, f: &l::FnTypeExpr<'_>) -> Option<String> {
        // FnTypeExpr's params and return share the TypeExpr kind
        // class, so the generated `params()` accessor yields the
        // return as its last element (M0 offset scheme limitation) —
        // split it off by position.
        let mut params: Vec<l::TypeExpr<'_>> = f.params().collect();
        let ret = params.pop()?;
        let row = match f.cap_annotation() {
            None => None,
            Some(a) => Some(self.cap_row(a)?),
        };
        let mut text = builder::f_type_c(&self.type_v(&ret)?, row.as_deref());
        for param in params.iter().rev() {
            text = builder::fn_type_c(&[&self.type_v(param)?], &text);
        }
        Some(text)
    }

    fn type_args(&mut self, args: l::GenericArgs<'_>) -> Option<String> {
        let mut texts: Vec<String> = Vec::new();
        for arg in args.args() {
            texts.push(self.type_v(&arg)?);
        }
        let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
        Some(builder::type_args(&refs))
    }

    /// CapAnnotation → CapRow: `/ { State[Number], ..c }` carries over
    /// entry by entry (D-41).
    fn cap_row(&mut self, annotation: l::CapAnnotation<'_>) -> Option<String> {
        let mut entries: Vec<String> = Vec::new();
        for entry in annotation.cap()?.entries() {
            let body = match entry.body()? {
                l::CapEntryBody::CapSig(sig) => {
                    let name = sig.name()?.text.clone();
                    let args = match sig.generic_args() {
                        None => None,
                        Some(args) => Some(self.type_args(args)?),
                    };
                    builder::cap_sig(&name, args.as_deref())
                }
                l::CapEntryBody::CapRest(rest) => {
                    builder::cap_rest(rest.name().map(|t| t.text.as_str()))
                }
            };
            entries.push(builder::cap_entry(&body));
        }
        let refs: Vec<&str> = entries.iter().map(String::as_str).collect();
        Some(builder::cap_row(&builder::cap_set(&refs)))
    }

    /// The def-level annotation for a fully annotated fn signature
    /// (M2 step 2): every param typed and a return type present —
    /// otherwise `None` and the def is left to inference. Bounded
    /// generics also bail (bounds are deferred, D-39). Under
    /// capability passing (D-51) the declared row becomes leading
    /// capability parameters and leaves the type.
    fn fn_signature_type(&mut self, f: &l::FnDecl<'_>) -> Option<String> {
        let ret = f.return_type()?;
        let mut param_types: Vec<String> = Vec::new();
        for param in f.param_list()?.params() {
            param_types.push(self.type_v(&param.ty()?)?);
        }
        let mut text = builder::f_type_c(&self.type_v(&ret)?, None);
        for ty in param_types.iter().rev() {
            text = builder::fn_type_c(&[ty], &text);
        }
        for cap in row_caps(f.cap_annotation()).iter().rev() {
            text = builder::fn_type_c(&[&builder::named_type_v(cap, None)], &text);
        }
        if let Some(generics) = f.generic_params() {
            let mut binders: Vec<String> = Vec::new();
            for param in generics.params() {
                if param.constraint().is_some() {
                    return None;
                }
                binders.push(param.name()?.text.clone());
            }
            if !binders.is_empty() {
                let refs: Vec<&str> = binders.iter().map(String::as_str).collect();
                text = builder::forall_type_c(&refs, &text);
            }
        }
        Some(builder::u_type_v(&text))
    }

    /// Legacy member classification: data ctor / cap op / value method.
    fn classify_member(
        &mut self,
        ctx: &mut ElabCtx,
        object: &LumoNode,
        member: &str,
        args: Option<Vec<ToFrag>>,
    ) -> Option<ToFrag> {
        // `Owner.member` with an ident owner: ctor or cap op.
        if let Some(ident) = l::IdentExpr::cast(object) {
            let owner = ident.name()?.text.clone();
            if self.variants.contains(&(owner.clone(), member.to_owned())) {
                // Data constructor: `.member(args)` wrapped in roll.
                let ctor = match &args {
                    None => builder::ctor_v(member, None),
                    Some(args) => {
                        let texts: Vec<&str> = args.iter().map(|f| f.text.as_str()).collect();
                        builder::ctor_v(member, Some(&builder::ctor_args(&texts)))
                    }
                };
                let mut frag = Frag::node(MirKind::ROLL_V, builder::roll_v(&ctor));
                for mut arg in args.into_iter().flatten() {
                    frag.absorb(&mut arg);
                }
                return Some(frag);
            }
            if self.caps.contains(&owner) {
                // D-51 capability passing: the innermost lexical
                // provider (handle binding, row param) wins, else the
                // default impl (D-44); no provider is a loud error.
                let Some(provider) = self.cap_provider(&owner) else {
                    ctx.error(format!(
                        "unhandled capability `{owner}` — declare it in the row, \
                         handle it, or provide an impl (D-51)"
                    ));
                    return None;
                };
                let sel = Frag::node(MirKind::SEL_C, builder::sel_c(&provider, member));
                return Some(self.apply(sel, args));
            }
        }
        // D-49: inherent method dispatch on the syntactic type —
        // `x.m(args)` becomes `sel __impl_T . m (x, args…)`.
        if let Some(t) = self.syn_type(object) {
            if self.inherent_methods.contains_key(&(t.clone(), member.to_owned())) {
                let obj = elab_node(ctx, self, object)?;
                let obj = self.to_value(ctx, obj);
                let mut full_args = vec![obj];
                full_args.extend(args.into_iter().flatten());
                let sel = Frag::node(
                    MirKind::SEL_C,
                    builder::sel_c(&format!("__impl_{t}"), member),
                );
                return Some(self.apply(sel, Some(full_args)));
            }
        }
        // Value method / plain member: `sel object . member`.
        let obj = elab_node(ctx, self, object)?;
        let obj = self.to_value(ctx, obj);
        let mut sel = Frag::node(MirKind::SEL_C, builder::sel_c(&obj.text, member));
        sel.pending = obj.pending;
        Some(self.apply(sel, args))
    }

    /// Apply a computation callee to value args (no force — the callee
    /// is already a computation), flushing binders at the application.
    fn apply(&mut self, callee: ToFrag, args: Option<Vec<ToFrag>>) -> ToFrag {
        let Some(args) = args else { return self.flush(callee) };
        let texts: Vec<&str> = args.iter().map(|f| f.text.as_str()).collect();
        let mut frag = Frag::node(
            MirKind::COMP_POSTFIX,
            builder::comp_postfix(
                builder::Operand { text: &callee.text, kind: callee.kind },
                &builder::value_args(&texts),
            ),
        );
        frag.pending = callee.pending;
        for mut arg in args {
            frag.absorb(&mut arg);
        }
        self.flush(frag)
    }

    /// D-44/D-48: an impl becomes an annotated bundle def — bare
    /// `impl Cap` as `def __impl_Cap = (bundle {…} : Cap)`, typeclass
    /// `impl T: Cap` as `def __impl_Cap_T = (bundle {…} : Cap_T)`
    /// against the seeded ground instance cap. Other forms error.
    fn impl_def(&mut self, ctx: &mut ElabCtx, i: l::ImplDecl<'_>) -> Option<String> {
        let (name, ty_name, self_target) = match impl_form(&i) {
            ImplForm::Bare(cap) if self.caps.contains(&cap) => {
                (format!("__impl_{cap}"), cap, None)
            }
            // D-49: inherent — checked against the driver-derived
            // `{T}_impl` instance cap.
            ImplForm::Bare(target) => {
                (format!("__impl_{target}"), format!("{target}_impl"), Some(target))
            }
            // D-50: generic inherent — binders are spelled as named
            // types in the seeded sig, nothing to substitute here.
            ImplForm::GenericInherent { target, .. } => {
                (format!("__impl_{target}"), format!("{target}_impl"), None)
            }
            ImplForm::Typeclass { cap, target } if self.caps.contains(&cap) => {
                (format!("__impl_{cap}_{target}"), format!("{cap}_{target}"), Some(target))
            }
            _ => {
                ctx.error(
                    "only ground impls are supported: bare cap (D-44), inherent (D-49), \
                     or typeclass `impl T: Cap` (D-48)"
                        .to_owned(),
                );
                return None;
            }
        };
        let saved = std::mem::replace(&mut self.self_target, self_target);
        let mut clauses: Vec<String> = Vec::new();
        for item in i.items() {
            match item.body()? {
                l::ImplItemBody::ImplMethod(m) => {
                    let frag = elab_node(ctx, self, m.syntax())?;
                    clauses.push(frag.text);
                }
                l::ImplItemBody::AssocTypeBinding(_) => {
                    self.self_target = saved;
                    ctx.error(
                        "assoc type bindings in impls are not supported (D-44)".to_owned(),
                    );
                    return None;
                }
            }
        }
        self.self_target = saved;
        let refs: Vec<&str> = clauses.iter().map(String::as_str).collect();
        let bundle = builder::bundle_v(&refs);
        let ty = builder::named_type_v(&ty_name, None);
        Some(builder::def(&name, &builder::paren_v(&bundle, Some(&ty))))
    }

    /// D-48 syntactic dispatch: the plain type name of an expression,
    /// when statically evident. `None` = unknown (the operator errors).
    fn syn_type(&self, node: &LumoNode) -> Option<String> {
        if l::NumberExpr::cast(node).is_some() {
            return Some("Number".to_owned());
        }
        if l::StringExpr::cast(node).is_some() {
            return Some("String".to_owned());
        }
        if let Some(i) = l::IdentExpr::cast(node) {
            let name = &i.name()?.text;
            return self.scope.iter().rev().find(|(n, _)| n == name).map(|(_, t)| t.clone());
        }
        if let Some(p) = l::ParenExpr::cast(node) {
            if let Some(ty) = p.ty() {
                return head_type_name(&ty);
            }
            return self.syn_type(p.inner()?.syntax());
        }
        if let Some(pf) = l::ExprPostfix::cast(node) {
            if let Some(m) = pf.member_name() {
                return self.member_syn_type(pf.expr()?.syntax(), &m.name()?.text);
            }
            if pf.call_args().is_some() {
                return match pf.expr()? {
                    l::Expr::Postfix(inner) => {
                        let m = inner.member_name()?;
                        self.member_syn_type(inner.expr()?.syntax(), &m.name()?.text)
                    }
                    l::Expr::IdentExpr(f) => self.fn_returns.get(&f.name()?.text).cloned(),
                    _ => None,
                };
            }
            return None;
        }
        if let Some(ie) = l::ExprInfix::cast(node) {
            return match ie.op()?.text.as_str() {
                "+" | "-" | "*" | "/" | "%" | "**" => self
                    .syn_type(ie.lhs()?.syntax())
                    .or_else(|| ie.rhs().and_then(|r| self.syn_type(r.syntax()))),
                "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&" | "||" => Some("Bool".to_owned()),
                _ => None,
            };
        }
        if let Some(pe) = l::ExprPrefix::cast(node) {
            return match pe.op()?.text.as_str() {
                "-" => self.syn_type(pe.expr()?.syntax()),
                "!" => Some("Bool".to_owned()),
                _ => None,
            };
        }
        None
    }

    /// `owner.member` as a type: a ctor's owner, a cap op's return, or
    /// an inherent method's return on the owner's syntactic type.
    fn member_syn_type(&self, owner: &LumoNode, member: &str) -> Option<String> {
        if let Some(ident) = l::IdentExpr::cast(owner) {
            let owner_name = ident.name()?.text.clone();
            if self.variants.contains(&(owner_name.clone(), member.to_owned())) {
                return Some(owner_name);
            }
            if let Some(ret) = self.cap_op_returns.get(&(owner_name, member.to_owned())) {
                return Some(ret.clone());
            }
            // An ident owner can also be a plain value — fall through.
        }
        let t = self.syn_type(owner)?;
        self.inherent_methods.get(&(t, member.to_owned())).and_then(Clone::clone)
    }

    /// `sel __impl_{cap}_{ty} . {method} (operands…)` — the D-48
    /// instance-cap selection, dispatched on the first operand whose
    /// type is syntactically known.
    fn instance_op(
        &mut self,
        ctx: &mut ElabCtx,
        cap: &str,
        method: &str,
        op: &str,
        operands: &[&LumoNode],
    ) -> Option<ToFrag> {
        let Some(ty) = operands.iter().find_map(|n| self.syn_type(n)) else {
            ctx.error(format!(
                "cannot resolve `{op}`: no operand type is syntactically known — \
                 annotate an operand (D-48)"
            ));
            return None;
        };
        if !self.typeclass_impls.contains(&(cap.to_owned(), ty.clone())) {
            ctx.error(format!("no `impl {ty}: {cap}` in scope for `{op}` (D-48)"));
            return None;
        }
        let mut args = Vec::new();
        for n in operands {
            let frag = elab_node(ctx, self, n)?;
            args.push(self.to_value(ctx, frag));
        }
        let sel = Frag::node(
            MirKind::SEL_C,
            builder::sel_c(&format!("__impl_{cap}_{ty}"), method),
        );
        Some(self.apply(sel, Some(args)))
    }

    /// `case unroll v { .true => then .false => else }` — the D-47/48
    /// Bool tag protocol.
    fn bool_case(&mut self, value: ToFrag, then_text: &str, else_text: &str) -> ToFrag {
        let arms = [
            builder::case_arm("true", None, then_text),
            builder::case_arm("false", None, else_text),
        ];
        let refs: Vec<&str> = arms.iter().map(String::as_str).collect();
        let mut frag = Frag::node(
            MirKind::CASE_C,
            builder::case_c(&builder::unroll_v(&value.text), &refs),
        );
        frag.pending = value.pending;
        self.flush(frag)
    }

    /// The innermost lexical capability provider (D-51): a handle
    /// binding or row param, else the default impl (D-44).
    fn cap_provider(&self, cap: &str) -> Option<String> {
        self.cap_env
            .iter()
            .rev()
            .find(|(c, _)| c == cap)
            .map(|(_, v)| v.clone())
            .or_else(|| self.impl_caps.contains(cap).then(|| format!("__impl_{cap}")))
    }

    /// The defs produced by one `use` decl (D-30): each imported name
    /// becomes `def name = thunk { let m = force require("module") in
    /// sel m . name }`.
    fn use_defs(&mut self, u: l::UseDecl<'_>) -> Option<Vec<String>> {
        let path = u.path()?;
        let mut segments = vec![path.head()?.text.clone()];
        let mut names: Vec<String> = Vec::new();
        let mut rest = path.rest();
        while let Some(r) = rest {
            match r.item()? {
                l::UsePathItem::UsePathBranch(b) => {
                    segments.push(b.next()?.text.clone());
                    rest = b.cont();
                }
                l::UsePathItem::UseTree(tree) => {
                    for item in tree.names() {
                        names.push(item.name()?.text.clone());
                    }
                    rest = None;
                }
            }
        }
        if names.is_empty() {
            // `use foo.bar;` — the last segment is the imported name.
            if segments.len() < 2 {
                return None;
            }
            names.push(segments.pop().expect("checked: at least two segments"));
        }
        let module = segments.join(".");
        let module_str = format!("{module:?}");
        let mut defs = Vec::new();
        for name in names {
            let require = builder::comp_postfix(
                builder::Operand {
                    text: &builder::force_c(&builder::var_v("require")),
                    kind: Some(MirKind::FORCE_C),
                },
                &builder::value_args(&[&builder::str_v(&module_str)]),
            );
            let body = builder::let_c("m", &require, &builder::sel_c("m", &name));
            defs.push(builder::def(&name, &builder::thunk_v(&body)));
        }
        Some(defs)
    }
}

impl Externs for LumoToMir {
    fn init(&mut self, root: &LumoNode) {
        let Some(file) = l::File::cast(root) else { return };
        for item in file.items() {
            match item.body() {
                Some(l::ItemBody::DataDecl(d)) => {
                    let Some(owner) = d.name() else { continue };
                    for variant in d.variants() {
                        let Some(v) = variant.name() else { continue };
                        self.variants.insert((owner.text.clone(), v.text.clone()));
                        // Tag → field heads for match-binder typing
                        // (D-50); ambiguous tags poison to None.
                        let heads: Vec<Option<String>> = variant
                            .fields()
                            .map(|fs| fs.fields().map(|ty| head_type_name(&ty)).collect())
                            .unwrap_or_default();
                        self.variant_fields
                            .entry(v.text.clone())
                            .and_modify(|e| *e = None)
                            .or_insert(Some(heads));
                    }
                }
                Some(l::ItemBody::CapDecl(c)) => {
                    let Some(name) = c.name() else { continue };
                    self.caps.insert(name.text.clone());
                    for item in c.items() {
                        let l::CapItem::OperationDecl(op) = item else { continue };
                        let (Some(op_name), Some(ret)) = (op.name(), op.return_type()) else {
                            continue;
                        };
                        if let Some(ret) = head_type_name(&ret) {
                            self.cap_op_returns
                                .insert((name.text.clone(), op_name.text.clone()), ret);
                        }
                    }
                }
                Some(l::ItemBody::FnDecl(f)) => {
                    let Some(name) = f.name() else { continue };
                    if let Some(ret) = f.return_type().as_ref().and_then(head_type_name) {
                        self.fn_returns.insert(name.text.clone(), ret);
                    }
                    let caps = row_caps(f.cap_annotation());
                    if !caps.is_empty() {
                        self.fn_rows.insert(name.text.clone(), caps);
                    }
                }
                Some(l::ItemBody::ExternDecl(e)) => {
                    let mut tails: Vec<l::ExternFnTail<'_>> = Vec::new();
                    match e.rest() {
                        Some(l::ExternRest::ExternFnTail(f)) => tails.push(f),
                        Some(l::ExternRest::ExternBlockTail(b)) => {
                            for item in b.items() {
                                if let Some(l::ExternBlockItemBody::ExternFnTail(f)) = item.body()
                                {
                                    tails.push(f);
                                }
                            }
                        }
                        _ => {}
                    }
                    for f in tails {
                        let (Some(name), Some(ret)) = (f.name(), f.return_type()) else {
                            continue;
                        };
                        if let Some(ret) = head_type_name(&ret) {
                            self.fn_returns.insert(name.text.clone(), ret);
                        }
                    }
                }
                _ => {}
            }
        }
        // Second pass: impls (D-44/D-48) — an `impl Console` textually
        // before `cap Console` must still register.
        for item in file.items() {
            if let Some(l::ItemBody::ImplDecl(i)) = item.body() {
                match impl_form(&i) {
                    ImplForm::Bare(cap) if self.caps.contains(&cap) => {
                        self.impl_caps.insert(cap);
                    }
                    // D-49/D-50: a bare (or generic) head that is not
                    // a cap is an inherent impl on that type.
                    ImplForm::Bare(target) | ImplForm::GenericInherent { target, .. } => {
                        for m in self_methods(&i) {
                            let Some(name) = m.name() else { continue };
                            let ret = m.return_type().as_ref().and_then(head_type_name);
                            self.inherent_methods
                                .insert((target.clone(), name.text.clone()), ret);
                        }
                    }
                    ImplForm::Typeclass { cap, target } if self.caps.contains(&cap) => {
                        self.typeclass_impls.insert((cap, target));
                    }
                    _ => {}
                }
            }
        }
    }

    /// D-38 sort coercion between the two MIR sorts:
    /// - computation where a value is expected → bind it to a fresh
    ///   `__tN` (the binder is pending until the nearest computation);
    /// - value where a computation is expected → `ret v` (F-intro).
    fn coerce(&mut self, ctx: &mut ElabCtx, expected: &'static str, frag: &ToFrag) -> Option<ToFrag> {
        let kind = frag.kind?;
        if matches!(expected, "Value" | "ValueArgs") && is_comp(kind) {
            let var = ctx.fresh();
            let mut out = Frag::node(MirKind::VAR_V, builder::var_v(&var));
            out.pending.push((var, frag.text.clone()));
            return Some(out);
        }
        if expected == "Comp" && is_value(kind) {
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

    /// File assembly: type-level items produce no defs in M1
    /// (data/cap/extern); fn, use, and impl items become defs.
    fn rule_module(&mut self, ctx: &mut ElabCtx, node: &FromNodeAlias) -> Option<ToFrag> {
        let file = l::File::cast(node)?;
        let mut defs: Vec<String> = Vec::new();
        for item in file.items() {
            match item.body()? {
                l::ItemBody::DataDecl(_)
                | l::ItemBody::CapDecl(_)
                | l::ItemBody::ExternDecl(_) => {}
                l::ItemBody::ImplDecl(i) => {
                    defs.push(self.impl_def(ctx, i)?);
                }
                l::ItemBody::FnDecl(f) => {
                    let frag = elab_node(ctx, self, f.syntax())?;
                    defs.push(frag.text);
                }
                l::ItemBody::UseDecl(u) => {
                    defs.extend(self.use_defs(u)?);
                }
            }
        }
        let texts: Vec<&str> = defs.iter().map(String::as_str).collect();
        Some(Frag::node(MirKind::FILE, builder::file(&texts)))
    }

    /// Curried thunk spines for `fn` decls and lambdas (legacy
    /// `lower_fn_value` / `Expr::Lambda`).
    fn rule_fn_curry(&mut self, ctx: &mut ElabCtx, node: &FromNodeAlias) -> Option<ToFrag> {
        match node.kind {
            LumoKind::FN_DECL => {
                let f = l::FnDecl::cast(node)?;
                let name = f.name()?.text.clone();
                // D-51: the declared row becomes leading capability
                // params, lexically visible to the body.
                let caps = row_caps(f.cap_annotation());
                let mut params: Vec<String> =
                    caps.iter().map(|c| format!("__cap_{c}")).collect();
                params.extend(
                    f.param_list()?.params().filter_map(|p| p.name().map(|t| t.text.clone())),
                );
                // Annotated params join the D-48 syntactic scope.
                let mark = self.scope.len();
                for p in f.param_list()?.params() {
                    if let (Some(n), Some(ty)) = (p.name(), p.ty()) {
                        if let Some(t) = head_type_name(&ty) {
                            self.scope.push((n.text.clone(), t));
                        }
                    }
                }
                let env_mark = self.cap_env.len();
                for c in &caps {
                    self.cap_env.push((c.clone(), format!("__cap_{c}")));
                }
                let body = self.elab_body(ctx, f.body()?);
                self.cap_env.truncate(env_mark);
                self.scope.truncate(mark);
                let body = body?;
                let value = self.curry(&params, body);
                // M2 step 2: a fully annotated signature survives as a
                // def-level `(thunk {…} : U(…))` annotation.
                let text = match self.fn_signature_type(&f) {
                    Some(ty) => builder::paren_v(&value.text, Some(&ty)),
                    None => value.text,
                };
                Some(Frag::node(MirKind::DEF, builder::def(&name, &text)))
            }
            LumoKind::LAMBDA_EXPR => {
                let f = l::LambdaExpr::cast(node)?;
                let params: Vec<String> = f
                    .param_list()?
                    .params()
                    .filter_map(|p| p.name().map(|t| t.text.clone()))
                    .collect();
                let mark = self.scope.len();
                for p in f.param_list()?.params() {
                    if let (Some(n), Some(ty)) = (p.name(), p.ty()) {
                        if let Some(t) = head_type_name(&ty) {
                            self.scope.push((n.text.clone(), t));
                        }
                    }
                }
                let body = self.elab_body(ctx, f.body()?);
                self.scope.truncate(mark);
                Some(self.curry(&params, body?))
            }
            _ => None,
        }
    }

    /// `(e : T)` keeps its annotation as a ParenV when the type
    /// translates (D-17's judgments consume it); plain parens and
    /// untranslatable types stay transparent. A computation inner
    /// coerces through a fresh binder first (D-38).
    fn rule_paren_annot(&mut self, ctx: &mut ElabCtx, node: &FromNodeAlias) -> Option<ToFrag> {
        let p = l::ParenExpr::cast(node)?;
        let inner = elab_node(ctx, self, p.inner()?.syntax())?;
        let Some(ty) = p.ty() else { return Some(inner) };
        let Some(tv) = self.type_v(&ty) else { return Some(inner) };
        let value = self.to_value(ctx, inner);
        let mut frag =
            Frag::node(MirKind::PAREN_V, builder::paren_v(&value.text, Some(&tv)));
        frag.pending = value.pending;
        Some(frag)
    }

    /// Blocks fold right-to-left: `let x = e;` becomes `let x = c in …`,
    /// a non-final expression statement sequences as `let _ = c in …`,
    /// and the final expression is the block's computation.
    fn rule_block(&mut self, ctx: &mut ElabCtx, node: &FromNodeAlias) -> Option<ToFrag> {
        let block = l::BlockExpr::cast(node)?;
        let stmts: Vec<l::BlockStmt<'_>> = block.stmts().collect();
        let mut comp: Option<ToFrag> = None;
        for stmt in stmts.iter().rev() {
            match stmt {
                l::BlockStmt::ExprStmt(e) => {
                    let frag = elab_node(ctx, self, e.expr()?.syntax())?;
                    let frag = self.to_comp(frag);
                    comp = Some(match comp {
                        None => frag,
                        Some(rest) => Frag::node(
                            MirKind::LET_C,
                            builder::let_c("_", &frag.text, &rest.text),
                        ),
                    });
                }
                l::BlockStmt::LetStmt(s) => {
                    let Some(rest) = comp else {
                        ctx.error("a block must end with an expression".to_owned());
                        return None;
                    };
                    let value = elab_node(ctx, self, s.value()?.syntax())?;
                    let value = self.to_comp(value);
                    let name = s.name()?.text.clone();
                    comp = Some(Frag::node(
                        MirKind::LET_C,
                        builder::let_c(&name, &value.text, &rest.text),
                    ));
                }
            }
        }
        match comp {
            Some(c) => Some(c),
            None => {
                ctx.error("empty block has no computation".to_owned());
                None
            }
        }
    }

    /// Case arms: tag from the variant pattern, binder names from its
    /// field patterns (optional — no absent-field pattern form, D-35).
    fn rule_match_arm(&mut self, ctx: &mut ElabCtx, node: &FromNodeAlias) -> Option<ToFrag> {
        let arm = l::MatchArm::cast(node)?;
        let (tag, fields) = match arm.pattern()? {
            l::Pattern::VariantPattern(v) => (v.name()?.text.clone(), v.fields()),
            l::Pattern::IdentPattern(v) => (v.name()?.text.clone(), v.fields()),
            _ => {
                ctx.error("match arm patterns must name a variant in M1".to_owned());
                return None;
            }
        };
        let mut names: Vec<String> = Vec::new();
        let binders = match fields {
            None => None,
            Some(fields) => {
                for pattern in fields.fields() {
                    let name = match pattern {
                        l::Pattern::BindPattern(b) => b.name()?.text.clone(),
                        l::Pattern::IdentPattern(i) => i.name()?.text.clone(),
                        l::Pattern::WildcardPattern(_) => "_".to_owned(),
                        l::Pattern::VariantPattern(_) => {
                            ctx.error("nested variant patterns are not supported in M1".to_owned());
                            return None;
                        }
                    };
                    names.push(name);
                }
                let refs: Vec<&str> = names.iter().map(String::as_str).collect();
                Some(builder::case_binders(&refs))
            }
        };
        // D-50: binders join the syntactic scope with the variant
        // decl's field heads (unambiguous tags only).
        let mark = self.scope.len();
        if let Some(Some(heads)) = self.variant_fields.get(&tag).cloned() {
            for (name, head) in names.iter().zip(heads) {
                if let (false, Some(head)) = (name == "_", head) {
                    self.scope.push((name.clone(), head));
                }
            }
        }
        let body = elab_node(ctx, self, arm.body()?.syntax());
        self.scope.truncate(mark);
        let body = self.to_comp(body?);
        Some(Frag::node(
            MirKind::CASE_ARM,
            builder::case_arm(&tag, binders.as_deref(), &body.text),
        ))
    }

    /// Member classification (legacy `Expr::Member`/`Expr::Call`): data
    /// ctor → `roll .tag(args)`; cap op → `sel <provider> . op` (D-51);
    /// otherwise a value method (`sel object . member`). Claims member
    /// postfixes and calls whose callee is a member postfix.
    fn rule_member_classify(&mut self, ctx: &mut ElabCtx, node: &FromNodeAlias) -> Option<ToFrag> {
        let pf = l::ExprPostfix::cast(node)?;
        if let Some(m) = pf.member_name() {
            let member = m.name()?.text.clone();
            let object = pf.expr()?;
            return self.classify_member(ctx, object.syntax(), &member, None);
        }
        let call_args = pf.call_args()?;
        let l::Expr::Postfix(inner) = pf.expr()? else { return None };
        let m = inner.member_name()?;
        let member = m.name()?.text.clone();
        let object = inner.expr()?;
        let mut args: Vec<ToFrag> = Vec::new();
        for arg in call_args.args() {
            let frag = elab_node(ctx, self, arg.syntax())?;
            let frag = self.to_value(ctx, frag);
            args.push(frag);
        }
        self.classify_member(ctx, object.syntax(), &member, Some(args))
    }

    /// D-47: `if c { a } else { b }` is the Bool match the programmer
    /// could write — `case unroll c { .true => a .false => b }`. The
    /// tags resolve against whatever `data Bool` is in scope. Else-less
    /// ifs error (no Unit value yet).
    fn rule_if_else(&mut self, ctx: &mut ElabCtx, node: &FromNodeAlias) -> Option<ToFrag> {
        let ie = l::IfElseExpr::cast(node)?;
        // The generated `else_clause()` accessor also matches the
        // then-block (M0 offset scheme limitation, cf. `fn_type_c`) —
        // the real else clause is the second ElseClause-shaped child.
        let Some(else_clause) = node.child_nodes().filter_map(l::ElseClause::cast).nth(1)
        else {
            ctx.error(
                "`if` without `else` is not supported — MIR has no unit value (D-47)".to_owned(),
            );
            return None;
        };
        let cond = elab_node(ctx, self, ie.condition()?.syntax())?;
        let cond = self.to_value(ctx, cond);
        let then = elab_node(ctx, self, ie.then_body()?.syntax())?;
        let then = self.to_comp(then);
        let els = elab_node(ctx, self, else_clause.syntax())?;
        let els = self.to_comp(els);
        let arms = [
            builder::case_arm("true", None, &then.text),
            builder::case_arm("false", None, &els.text),
        ];
        let refs: Vec<&str> = arms.iter().map(String::as_str).collect();
        let mut frag = Frag::node(
            MirKind::CASE_C,
            builder::case_c(&builder::unroll_v(&cond.text), &refs),
        );
        frag.pending = cond.pending;
        Some(self.flush(frag))
    }

    /// D-48: operators desugar to instance-cap selections (arith,
    /// `==`) or Bool/Ordering cases (`!=`, comparisons, `&&`, `||`,
    /// `!`). `**` has no legacy cap and errors.
    fn rule_operators(&mut self, ctx: &mut ElabCtx, node: &FromNodeAlias) -> Option<ToFrag> {
        let ret_true = builder::ret_c(&builder::roll_v(&builder::ctor_v("true", None)));
        let ret_false = builder::ret_c(&builder::roll_v(&builder::ctor_v("false", None)));
        if let Some(pe) = l::ExprPrefix::cast(node) {
            let op = pe.op()?.text.clone();
            let expr = pe.expr()?;
            return match op.as_str() {
                "-" => self.instance_op(ctx, "Neg", "neg", &op, &[expr.syntax()]),
                "!" => {
                    let value = elab_node(ctx, self, expr.syntax())?;
                    let value = self.to_value(ctx, value);
                    Some(self.bool_case(value, &ret_false, &ret_true))
                }
                _ => {
                    ctx.error(format!("unsupported prefix operator `{op}` (D-48)"));
                    None
                }
            };
        }
        let ie = l::ExprInfix::cast(node)?;
        let op = ie.op()?.text.clone();
        let (lhs, rhs) = (ie.lhs()?, ie.rhs()?);
        let pair = [lhs.syntax(), rhs.syntax()];
        let arith = |o: &str| match o {
            "+" => Some(("Add", "add")),
            "-" => Some(("Sub", "sub")),
            "*" => Some(("Mul", "mul")),
            "/" => Some(("Div", "div")),
            "%" => Some(("Mod", "mod_")),
            "==" => Some(("PartialEq", "eq")),
            _ => None,
        };
        if let Some((cap, method)) = arith(&op) {
            return self.instance_op(ctx, cap, method, &op, &pair);
        }
        match op.as_str() {
            "!=" => {
                let eq = self.instance_op(ctx, "PartialEq", "eq", &op, &pair)?;
                let value = self.to_value(ctx, eq);
                Some(self.bool_case(value, &ret_false, &ret_true))
            }
            "<" | "<=" | ">" | ">=" => {
                let cmp = self.instance_op(ctx, "PartialOrd", "cmp", &op, &pair)?;
                let value = self.to_value(ctx, cmp);
                let truth = |yes: bool| if yes { &ret_true } else { &ret_false };
                let (lt, eq, gt) = match op.as_str() {
                    "<" => (true, false, false),
                    "<=" => (true, true, false),
                    ">" => (false, false, true),
                    _ => (false, true, true),
                };
                let arms = [
                    builder::case_arm("less", None, truth(lt)),
                    builder::case_arm("equal", None, truth(eq)),
                    builder::case_arm("greater", None, truth(gt)),
                ];
                let refs: Vec<&str> = arms.iter().map(String::as_str).collect();
                let mut frag = Frag::node(
                    MirKind::CASE_C,
                    builder::case_c(&builder::unroll_v(&value.text), &refs),
                );
                frag.pending = value.pending;
                Some(self.flush(frag))
            }
            // Lazy: the rhs stays inside its arm.
            "&&" | "||" => {
                let left = elab_node(ctx, self, lhs.syntax())?;
                let left = self.to_value(ctx, left);
                let right = elab_node(ctx, self, rhs.syntax())?;
                let right = self.to_comp(right);
                Some(if op == "&&" {
                    self.bool_case(left, &right.text, &ret_false)
                } else {
                    self.bool_case(left, &ret_true, &right.text)
                })
            }
            "**" => {
                ctx.error("`**` has no operator cap (D-48)".to_owned());
                None
            }
            _ => {
                ctx.error(format!("unsupported operator `{op}` (D-48)"));
                None
            }
        }
    }

    /// D-51: `handle E with h in body` is a lexical capability
    /// binding — `let __tN = ret (h : E) in body` with `E ↦ __tN`
    /// while the body elaborates. The annotation routes the judge's
    /// bundle-vs-cap check; nothing dynamic remains.
    fn rule_handle_bind(&mut self, ctx: &mut ElabCtx, node: &FromNodeAlias) -> Option<ToFrag> {
        let h = l::HandleExpr::cast(node)?;
        let cap = h.cap_name()?.text.clone();
        if !self.caps.contains(&cap) {
            ctx.error(format!("`handle {cap}`: unknown cap (D-51)"));
            return None;
        }
        let handler = elab_node(ctx, self, h.handler()?.syntax())?;
        let handler = self.to_value(ctx, handler);
        let annotated =
            builder::paren_v(&handler.text, Some(&builder::named_type_v(&cap, None)));
        let var = ctx.fresh();
        let mut pending = handler.pending;
        pending.push((var.clone(), builder::ret_c(&annotated)));
        self.cap_env.push((cap, var));
        let body = elab_node(ctx, self, h.body()?.syntax());
        self.cap_env.pop();
        let mut body = self.to_comp(body?);
        pending.append(&mut body.pending);
        body.pending = pending;
        Some(self.flush(body))
    }

    /// D-51: a call to a named fn with a declared row threads the
    /// lexical capability values as leading args.
    fn rule_call_caps(&mut self, ctx: &mut ElabCtx, node: &FromNodeAlias) -> Option<ToFrag> {
        let pf = l::ExprPostfix::cast(node)?;
        let call_args = pf.call_args()?;
        let l::Expr::IdentExpr(id) = pf.expr()? else { return None };
        let fname = id.name()?.text.clone();
        let caps = self.fn_rows.get(&fname)?.clone();
        let mut args: Vec<ToFrag> = Vec::new();
        for cap in &caps {
            let Some(provider) = self.cap_provider(cap) else {
                ctx.error(format!(
                    "unhandled capability `{cap}` in call to `{fname}` — declare it \
                     in the row, handle it, or provide an impl (D-51)"
                ));
                return None;
            };
            args.push(Frag::node(MirKind::VAR_V, builder::var_v(&provider)));
        }
        for arg in call_args.args() {
            let frag = elab_node(ctx, self, arg.syntax())?;
            args.push(self.to_value(ctx, frag));
        }
        let callee = Frag::node(
            MirKind::FORCE_C,
            builder::force_c(&builder::var_v(&fname)),
        );
        Some(self.apply(callee, Some(args)))
    }

    /// `use` decls become require-derived defs (D-30).
    fn rule_use_decl(&mut self, ctx: &mut ElabCtx, node: &FromNodeAlias) -> Option<ToFrag> {
        let u = l::UseDecl::cast(node)?;
        let defs = self.use_defs(u)?;
        let _ = ctx;
        Some(Frag::node(MirKind::FILE, defs.join(" ")))
    }

    /// D-12: a def whose value mentions its own name becomes
    /// `thunk { fix name => body }`. Self-recursion only in M1 —
    /// mutually recursive groups are left untouched (M2's typechecker
    /// will reject them).
    fn pass_scc_fix(&mut self, phase: PassPhase, text: &str) -> Option<String> {
        if phase != PassPhase::PostTarget {
            return None;
        }
        let parsed = crate::mir::parser::parse(text);
        if !parsed.errors.is_empty() {
            return None;
        }
        let file = m::File::cast(&parsed.root)?;
        let mut defs: Vec<String> = Vec::new();
        let mut changed = false;
        for def in file.defs() {
            let (Some(name), Some(value)) = (def.name(), def.value()) else {
                defs.push(crate::mir::printer::canonical(def.syntax()));
                continue;
            };
            let self_recursive = mentions_var(value.syntax(), &name.text);
            match (&value, self_recursive) {
                (m::Value::ThunkV(thunk), true) => {
                    let body = crate::mir::printer::canonical(thunk.body()?.syntax());
                    let fixed = builder::thunk_v(&builder::fix_c(&name.text, &body));
                    defs.push(builder::def(&name.text, &fixed));
                    changed = true;
                }
                // An annotated def keeps its annotation around the fix.
                (m::Value::ParenV(paren), true) => {
                    let (Some(m::Value::ThunkV(thunk)), Some(ty)) =
                        (paren.inner(), paren.ty())
                    else {
                        defs.push(crate::mir::printer::canonical(def.syntax()));
                        continue;
                    };
                    let body = crate::mir::printer::canonical(thunk.body()?.syntax());
                    let fixed = builder::thunk_v(&builder::fix_c(&name.text, &body));
                    let ty = crate::mir::printer::canonical(ty.syntax());
                    defs.push(builder::def(
                        &name.text,
                        &builder::paren_v(&fixed, Some(&ty)),
                    ));
                    changed = true;
                }
                _ => defs.push(crate::mir::printer::canonical(def.syntax())),
            }
        }
        if !changed {
            return None;
        }
        let refs: Vec<&str> = defs.iter().map(String::as_str).collect();
        Some(builder::file(&refs))
    }

    /// D-30: require-derived defs are hoisted before the others (uses
    /// are ordered first in the tree).
    fn pass_use_require(&mut self, phase: PassPhase, text: &str) -> Option<String> {
        if phase != PassPhase::PostTarget {
            return None;
        }
        let parsed = crate::mir::parser::parse(text);
        if !parsed.errors.is_empty() {
            return None;
        }
        let file = m::File::cast(&parsed.root)?;
        let mut requires: Vec<String> = Vec::new();
        let mut others: Vec<String> = Vec::new();
        for def in file.defs() {
            let uses_require =
                def.value().map(|v| mentions_var(v.syntax(), "require")).unwrap_or(false);
            let text = crate::mir::printer::canonical(def.syntax());
            if uses_require {
                requires.push(text);
            } else {
                others.push(text);
            }
        }
        if requires.is_empty() {
            return None;
        }
        let ordered: Vec<&str> =
            requires.iter().chain(others.iter()).map(String::as_str).collect();
        Some(builder::file(&ordered))
    }
}

/// Does any `VarV` in this subtree reference `name`? Ctor tags, sel
/// fields, and binder positions are not references. (M1 approximation:
/// shadowing binders are not tracked.)
fn mentions_var(node: &crate::mir::lossless::SyntaxNode, name: &str) -> bool {
    if node.kind == MirKind::VAR_V {
        if node.child_tokens().any(|t| t.kind == MirKind::IDENT && t.text == name) {
            return true;
        }
    }
    node.child_nodes().any(|n| mentions_var(n, name))
}

type FromNodeAlias = LumoNode;

pub fn lumo_to_mir() -> Box<dyn Externs> {
    Box::<LumoToMir>::default()
}

// === Type-translation helpers for the judge driver (M2 step 7) ===

/// A Lumo TypeExpr as MIR TypeV text; `None` = not spellable (D-39).
pub fn type_v_text(ty: &l::TypeExpr<'_>) -> Option<String> {
    LumoToMir::default().type_v(ty)
}

/// Like [`type_v_text`], with `Self` reading as `target` (D-48 —
/// instance-cap seeding).
pub fn type_v_text_self(ty: &l::TypeExpr<'_>, target: &str) -> Option<String> {
    let mut m = LumoToMir::default();
    m.self_target = Some(target.to_owned());
    m.type_v(ty)
}

/// A Lumo CapAnnotation as MIR CapRow text.
pub fn cap_row_text(annotation: l::CapAnnotation<'_>) -> Option<String> {
    LumoToMir::default().cap_row(annotation)
}

/// A signature (typed params, return, optional row) as a curried MIR
/// comp-type text — the shape shared by fn defs and cap operations.
pub fn signature_type_c_text(
    param_types: &[String],
    ret: &str,
    row: Option<&str>,
) -> String {
    let mut text = builder::f_type_c(ret, row);
    for ty in param_types.iter().rev() {
        text = builder::fn_type_c(&[ty], &text);
    }
    text
}

// === MIR → JS (M4, D-43) ===

use crate::elab::mir_to_js as js_elab;
use crate::js::builder as jsb;
use crate::js::syntax_kind::SyntaxKind as JsKind;
use crate::mir::lossless::SyntaxNode as MirNode;

type JsFrag = js_elab::ToFrag;

pub fn mir_to_js() -> Box<dyn js_elab::Externs> {
    Box::new(MirToJs)
}

pub struct MirToJs;

/// Wrap in the n-ary paren atom. The generated builder cannot
/// auto-parenthesize (no single-required-field paren atom in the JS
/// grammar, D-43), so risky operands are pre-wrapped here.
fn js_paren(text: &str) -> JsFrag {
    Frag::node(JsKind::PAREN_EXPR, jsb::paren_expr(&[text]))
}

fn js_operand(frag: &JsFrag) -> jsb::Operand<'_> {
    jsb::Operand { text: &frag.text, kind: frag.kind }
}

/// Callee-safe: anything with a weak right edge (arrows) gets parens
/// so a following postfix row binds to the whole expression.
fn js_tight(frag: JsFrag) -> JsFrag {
    match frag.kind {
        Some(JsKind::EXPR_INFIX) => js_paren(&frag.text),
        _ => frag,
    }
}

impl MirToJs {
    fn comp(&mut self, ctx: &mut ElabCtx, c: &m::Comp<'_>) -> Option<JsFrag> {
        js_elab::elab_node(ctx, self, c.syntax())
    }

    fn value(&mut self, ctx: &mut ElabCtx, v: &m::Value<'_>) -> Option<JsFrag> {
        js_elab::elab_node(ctx, self, v.syntax())
    }

    /// `(p1, p2) => body`
    fn arrow(&self, params: &[&str], body: &JsFrag) -> JsFrag {
        let idents: Vec<String> = params.iter().map(|p| jsb::ident_expr(p)).collect();
        let refs: Vec<&str> = idents.iter().map(String::as_str).collect();
        let lhs = jsb::paren_expr(&refs);
        let text = jsb::expr_infix_1(
            jsb::Operand { text: &lhs, kind: Some(JsKind::PAREN_EXPR) },
            "=>",
            js_operand(body),
        );
        Frag::node(JsKind::EXPR_INFIX, text)
    }

    /// `callee(args…)`
    fn call(&self, callee: JsFrag, args: &[&str]) -> JsFrag {
        let callee = js_tight(callee);
        let text = jsb::expr_postfix_1(js_operand(&callee), &jsb::call_args(args));
        Frag::node(JsKind::EXPR_POSTFIX, text)
    }

    fn runtime_call(&self, name: &str, args: &[&str]) -> JsFrag {
        self.call(Frag::node(JsKind::IDENT_EXPR, jsb::ident_expr(name)), args)
    }
}

/// An ident token as a JS string literal (`Cons` → `"Cons"`).
fn js_quoted(name: &str) -> String {
    jsb::str_expr(&format!("{name:?}"))
}

impl js_elab::Externs for MirToJs {
    // The SCC/use passes are Lumo→MIR concerns (D-12/D-30); they are
    // declared project-wide, so this pair implements them as no-ops.
    fn pass_scc_fix(&mut self, _phase: PassPhase, _text: &str) -> Option<String> {
        None
    }

    fn pass_use_require(&mut self, _phase: PassPhase, _text: &str) -> Option<String> {
        None
    }

    fn rule_thunk_lambda(&mut self, ctx: &mut ElabCtx, node: &MirNode) -> Option<JsFrag> {
        if let Some(t) = m::ThunkV::cast(node) {
            let body = self.comp(ctx, &t.body()?)?;
            return Some(self.arrow(&[], &body));
        }
        if let Some(l) = m::LamC::cast(node) {
            let body = self.comp(ctx, &l.body()?)?;
            return Some(self.arrow(&[&l.param()?.text], &body));
        }
        None
    }

    fn rule_force_apply(&mut self, ctx: &mut ElabCtx, node: &MirNode) -> Option<JsFrag> {
        if let Some(f) = m::ForceC::cast(node) {
            let v = self.value(ctx, &f.value()?)?;
            return Some(self.call(v, &[]));
        }
        if let Some(a) = m::CompPostfix::cast(node) {
            let inner = a.expr()?;
            let mut args = Vec::new();
            if let Some(va) = a.value_args() {
                for arg in va.args() {
                    args.push(self.value(ctx, &arg)?.text);
                }
            }
            // The judge reads an empty application as identity; over a
            // force, `f()` already is the run — a second call would
            // double-invoke. Over a sel, the call stays: a nullary
            // bundle clause is a 0-ary arrow that still needs invoking.
            if args.is_empty() {
                if let m::Comp::ForceC(_) = inner {
                    return self.comp(ctx, &inner);
                }
            }
            let callee = self.comp(ctx, &inner)?;
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            return Some(self.call(callee, &refs));
        }
        None
    }

    fn rule_let_fix(&mut self, ctx: &mut ElabCtx, node: &MirNode) -> Option<JsFrag> {
        if let Some(l) = m::LetC::cast(node) {
            let value = self.comp(ctx, &l.value()?)?;
            let body = self.comp(ctx, &l.body()?)?;
            let arrow = self.arrow(&[&l.name()?.text], &body);
            return Some(self.call(arrow, &[&value.text]));
        }
        if let Some(f) = m::FixC::cast(node) {
            let body = self.comp(ctx, &f.body()?)?;
            let fn_expr =
                Frag::node(JsKind::FN_EXPR, jsb::fn_expr(&f.name()?.text, &body.text));
            return Some(self.call(fn_expr, &[]));
        }
        None
    }

    fn rule_case_arms(&mut self, ctx: &mut ElabCtx, node: &MirNode) -> Option<JsFrag> {
        let case = m::CaseC::cast(node)?;
        let scrut = self.value(ctx, &case.scrutinee()?)?;
        let s = ctx.fresh();
        let s_ident = jsb::ident_expr(&s);
        let s_operand = || jsb::Operand { text: &s_ident, kind: Some(JsKind::IDENT_EXPR) };
        // Innermost alternative: the non-exhaustive fallthrough.
        let mut chain = self
            .runtime_call("__lumo_match_error", &[&s_ident])
            .text;
        let arms: Vec<_> = case.arms().collect();
        for arm in arms.iter().rev() {
            let tag = arm.tag()?;
            let body = self.comp(ctx, &arm.body()?)?;
            // `_` binders are never referenced, but JS forbids
            // duplicate parameter names — uniquify each one.
            let binders: Vec<String> = arm
                .binders()
                .map(|b| {
                    b.names()
                        .map(|t| if t.text == "_" { ctx.fresh() } else { t.text.clone() })
                        .collect()
                })
                .unwrap_or_default();
            let then_text = if binders.is_empty() {
                body.text.clone()
            } else {
                // `((b1, b2) => body)(s.args[0], s.args[1])`
                let refs: Vec<&str> = binders.iter().map(String::as_str).collect();
                let arrow = self.arrow(&refs, &body);
                let args_member = jsb::expr_postfix_0(s_operand(), &jsb::member_name("args"));
                let payloads: Vec<String> = (0..binders.len())
                    .map(|i| {
                        jsb::expr_postfix_2(
                            jsb::Operand {
                                text: &args_member,
                                kind: Some(JsKind::EXPR_POSTFIX),
                            },
                            &jsb::index_arg(&jsb::num_expr(&i.to_string())),
                        )
                    })
                    .collect();
                let payload_refs: Vec<&str> = payloads.iter().map(String::as_str).collect();
                self.call(arrow, &payload_refs).text
            };
            // `(s.$ === "Tag") ? then : chain` — the test is wrapped
            // because the builder cannot auto-parenthesize (D-43).
            let s_tag = jsb::expr_postfix_0(s_operand(), &jsb::member_name("$"));
            let tag_str = js_quoted(&tag.text);
            let eq = jsb::expr_infix_0(
                jsb::Operand { text: &s_tag, kind: Some(JsKind::EXPR_POSTFIX) },
                "===",
                jsb::Operand { text: &tag_str, kind: Some(JsKind::STR_EXPR) },
            );
            let eq_paren = jsb::paren_expr(&[&eq]);
            chain = jsb::expr_postfix_3(
                jsb::Operand { text: &eq_paren, kind: Some(JsKind::PAREN_EXPR) },
                &jsb::ternary_tail(&then_text, &chain),
            );
        }
        let chain_frag = Frag::node(JsKind::EXPR_POSTFIX, chain);
        let dispatcher = self.arrow(&[&s], &chain_frag);
        Some(self.call(dispatcher, &[&scrut.text]))
    }

    fn rule_ctor_bundle(&mut self, ctx: &mut ElabCtx, node: &MirNode) -> Option<JsFrag> {
        if let Some(c) = m::CtorV::cast(node) {
            let tag = c.tag()?;
            let mut args = Vec::new();
            if let Some(ctor_args) = c.args() {
                for arg in ctor_args.args() {
                    args.push(self.value(ctx, &arg)?.text);
                }
            }
            let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
            let tag_prop = jsb::object_prop("$", &js_quoted(&tag.text));
            let args_prop = jsb::object_prop("args", &jsb::array_expr(&arg_refs));
            let obj = jsb::object_expr(&[&tag_prop, &args_prop]);
            // Parenthesized: `{…}` as an arrow body reads as a block
            // in real JS (D-43).
            return Some(js_paren(&obj));
        }
        if let Some(b) = m::BundleV::cast(node) {
            let mut props = Vec::new();
            for clause in b.clauses() {
                let name = clause.name()?;
                let params: Vec<String> = clause.params().map(|t| t.text.clone()).collect();
                let refs: Vec<&str> = params.iter().map(String::as_str).collect();
                let body = self.comp(ctx, &clause.body()?)?;
                let arrow = self.arrow(&refs, &body);
                props.push(jsb::object_prop(&name.text, &arrow.text));
            }
            let prop_refs: Vec<&str> = props.iter().map(String::as_str).collect();
            return Some(js_paren(&jsb::object_expr(&prop_refs)));
        }
        None
    }

    fn rule_caps(&mut self, ctx: &mut ElabCtx, node: &MirNode) -> Option<JsFrag> {
        if let Some(p) = m::PerformC::cast(node) {
            return Some(self.runtime_call("__lumo_perform", &[&js_quoted(&p.cap()?.text)]));
        }
        if let Some(h) = m::HandleC::cast(node) {
            let handler = self.value(ctx, &h.handler()?)?;
            let body = self.comp(ctx, &h.body()?)?;
            let thunk = self.arrow(&[], &body);
            return Some(self.runtime_call(
                "__lumo_handle",
                &[&js_quoted(&h.cap()?.text), &handler.text, &thunk.text],
            ));
        }
        None
    }
}
