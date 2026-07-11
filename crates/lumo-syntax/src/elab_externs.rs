//! Handwritten extern implementations for the generated elab modules
//! (D-01/D-38). The generated registry constructs these by convention:
//! `crate::elab_externs::{from}_to_{to}()`.
//!
//! Reference semantics: the legacy HIR→LIR lowering
//! (`legacy/crates/lir/src/lib.rs`) — curried thunk spines for fn decls,
//! force-then-apply calls, ctors wrapped in roll, match auto-unroll,
//! `Cap.op` → `sel (perform Cap) . op`.

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
                let name = n.name()?.text.clone();
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
    /// a thunk). The row sits on the innermost F (D-41).
    fn fn_type_c(&mut self, f: &l::FnTypeExpr<'_>) -> Option<String> {
        let params: Vec<l::TypeExpr<'_>> = f.params().collect();
        let row = match f.cap_annotation() {
            None => None,
            Some(a) => Some(self.cap_row(a)?),
        };
        let ret = f.return_type()?;
        let mut text = match (&ret, &row) {
            (l::TypeExpr::FnTypeExpr(_), None) => self.type_c(&ret)?,
            // A row on an arrow is the latent-effect case — not
            // spellable in MIR (D-41).
            (l::TypeExpr::FnTypeExpr(_), Some(_)) => return None,
            _ => builder::f_type_c(&self.type_v(&ret)?, row.as_deref()),
        };
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
    /// generics also bail (bounds are deferred, D-39).
    fn fn_signature_type(&mut self, f: &l::FnDecl<'_>) -> Option<String> {
        let ret = f.return_type()?;
        let mut param_types: Vec<String> = Vec::new();
        for param in f.param_list()?.params() {
            param_types.push(self.type_v(&param.ty()?)?);
        }
        let row = match f.cap_annotation() {
            None => None,
            Some(a) => Some(self.cap_row(a)?),
        };
        let mut text = match (&ret, &row) {
            (l::TypeExpr::FnTypeExpr(_), None) => self.type_c(&ret)?,
            (l::TypeExpr::FnTypeExpr(_), Some(_)) => return None,
            _ => builder::f_type_c(&self.type_v(&ret)?, row.as_deref()),
        };
        for ty in param_types.iter().rev() {
            text = builder::fn_type_c(&[ty], &text);
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
                // Cap operation: `sel (perform Cap) . member` — the
                // perform is a computation, so it binds to a fresh value.
                let var = ctx.fresh();
                let mut sel = Frag::node(MirKind::SEL_C, builder::sel_c(&var, member));
                sel.pending.push((var, builder::perform_c(&owner)));
                return Some(self.apply(sel, args));
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
                        if let Some(v) = variant.name() {
                            self.variants.insert((owner.text.clone(), v.text.clone()));
                        }
                    }
                }
                Some(l::ItemBody::CapDecl(c)) => {
                    if let Some(name) = c.name() {
                        self.caps.insert(name.text.clone());
                    }
                }
                _ => {}
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
    /// (data/cap/extern/impl); fn and use items become defs.
    fn rule_module(&mut self, ctx: &mut ElabCtx, node: &FromNodeAlias) -> Option<ToFrag> {
        let file = l::File::cast(node)?;
        let mut defs: Vec<String> = Vec::new();
        for item in file.items() {
            match item.body()? {
                l::ItemBody::DataDecl(_)
                | l::ItemBody::CapDecl(_)
                | l::ItemBody::ExternDecl(_)
                | l::ItemBody::ImplDecl(_) => {}
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
                let params: Vec<String> = f
                    .param_list()?
                    .params()
                    .filter_map(|p| p.name().map(|t| t.text.clone()))
                    .collect();
                let body = self.elab_body(ctx, f.body()?)?;
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
                let body = self.elab_body(ctx, f.body()?)?;
                Some(self.curry(&params, body))
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
        let binders = match fields {
            None => None,
            Some(fields) => {
                let mut names: Vec<String> = Vec::new();
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
        let body = elab_node(ctx, self, arm.body()?.syntax())?;
        let body = self.to_comp(body);
        Some(Frag::node(
            MirKind::CASE_ARM,
            builder::case_arm(&tag, binders.as_deref(), &body.text),
        ))
    }

    /// Member classification (legacy `Expr::Member`/`Expr::Call`): data
    /// ctor → `roll .tag(args)`; cap op → `sel (perform Cap) . op`;
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
