//! The `:infer(Lumo)` driver (M2 step 7): the full pipeline — parse
//! Lumo, elab to MIR, seed the judgment contexts from the *Lumo* tree
//! (MIR has no type-level declarations), then judge each MIR def in
//! order. Output is `name : Type` lines printed in the MIR type
//! sub-language, or `ERROR: …` when a def fails to type (D-26).
//!
//! Seeding follows the shape contracts in `lumo/MIR.type.langue`:
//! - `data` decls fill Δ: tag → `Variant(owner, binders, params)`;
//! - `cap` decls fill Σ: `Op(c, f)` → the op's comp type (ops with
//!   untyped parameters are left out — using them bails), `Ops(c)` →
//!   the op-name set;
//! - `extern fn` decls enter Γ (missing return types read as `Unit`).
//!
//! Each successfully judged def joins Γ for the defs after it —
//! definition order is binding order (mutual recursion is out of
//! scope until D-12's group encoding lands).

use langue_rt::{app, atom, set, Contexts, ElabReport, Term};

use crate::elab_externs;
use crate::lumo::ast::{self as l, AstNode as _};
use crate::mir::ast::{self as m, AstNode as _};
use crate::mir::judgments;

pub fn infer_report(source: &str) -> ElabReport {
    match run(source) {
        Ok(output) => ElabReport { output, errors: Vec::new() },
        Err(errors) => ElabReport { output: String::new(), errors },
    }
}

fn run(source: &str) -> Result<String, Vec<String>> {
    let lumo = crate::lumo::parser::parse(source);
    if !lumo.errors.is_empty() {
        return Err(lumo.errors.iter().map(|e| format!("parse: {}", e.message)).collect());
    }
    let mut externs = elab_externs::lumo_to_mir();
    let elab = crate::elab::lumo_to_mir::elab(source, externs.as_mut());
    if !elab.errors.is_empty() {
        return Err(elab.errors.iter().map(|e| format!("elab: {e}")).collect());
    }
    let mir = crate::mir::parser::parse(&elab.output);
    if !mir.errors.is_empty() {
        return Err(mir.errors.iter().map(|e| format!("elab output: {}", e.message)).collect());
    }

    let mut ctxs = Contexts::new();
    let mut lines = Vec::new();
    seed(&lumo.root, &mut ctxs, &mut lines)?;

    let file = m::File::cast(&mir.root).ok_or_else(|| vec!["no MIR file".to_owned()])?;
    for def in file.defs() {
        let (Some(name), Some(value)) = (def.name(), def.value()) else { continue };
        match judgments::solve("infer_V", value.syntax(), ctxs.clone()) {
            Ok(derivation) => {
                let ty = derivation.args[1].clone();
                lines.push(format!("{} : {}", name.text, print_v(&ty)));
                ctxs.entry("Γ".to_owned())
                    .or_default()
                    .push((atom(name.text.clone()), ty));
            }
            Err(bail) => return Ok(format!("ERROR: {}", bail.message)),
        }
    }
    Ok(lines.join("\n"))
}

// === Seeding from the Lumo tree ===

fn seed(
    root: &crate::lumo::lossless::SyntaxNode,
    ctxs: &mut Contexts,
    lines: &mut Vec<String>,
) -> Result<(), Vec<String>> {
    let Some(file) = l::File::cast(root) else { return Ok(()) };
    let mut cap_decls: Vec<l::CapDecl<'_>> = Vec::new();
    for item in file.items() {
        match item.body() {
            Some(l::ItemBody::DataDecl(d)) => seed_data(&d, ctxs)?,
            Some(l::ItemBody::CapDecl(c)) => {
                seed_cap(&c, ctxs, None, None)?;
                cap_decls.push(c);
            }
            Some(l::ItemBody::ExternDecl(e)) => seed_extern(&e, ctxs, lines)?,
            _ => {}
        }
    }
    // D-48: typeclass impls seed ground instance caps — `impl Number:
    // Add` seeds `Add_Number` with every op type's `Self := Number`.
    // D-49/D-50: inherent impls (ground or generic) seed `{T}_impl`
    // derived from their own method signatures. Every impl def also
    // pre-seeds Γ — a method body may reference its own def (D-50).
    for item in file.items() {
        let Some(l::ItemBody::ImplDecl(i)) = item.body() else { continue };
        let is_cap = |name: &str| cap_decls.iter().any(|c| c.name().is_some_and(|n| n.text == name));
        let (def_name, cap_name) = match elab_externs::impl_form(&i) {
            elab_externs::ImplForm::Typeclass { cap, target } if is_cap(&cap) => {
                let decl = cap_decls
                    .iter()
                    .find(|c| c.name().is_some_and(|n| n.text == cap))
                    .expect("checked: cap is declared");
                let instance = format!("{cap}_{target}");
                seed_cap(decl, ctxs, Some(&instance), Some(&target))?;
                (format!("__impl_{cap}_{target}"), instance)
            }
            elab_externs::ImplForm::Bare(name) if is_cap(&name) => {
                (format!("__impl_{name}"), name)
            }
            elab_externs::ImplForm::Bare(target) => {
                seed_inherent(&i, &target, &[], ctxs)?;
                (format!("__impl_{target}"), format!("{target}_impl"))
            }
            elab_externs::ImplForm::GenericInherent { target, binders } => {
                seed_inherent(&i, &target, &binders, ctxs)?;
                (format!("__impl_{target}"), format!("{target}_impl"))
            }
            _ => continue,
        };
        let sem = sem_v(&crate::mir::builder::named_type_v(&cap_name, None))?;
        ctxs.entry("Γ".to_owned()).or_default().push((atom(def_name), sem));
    }
    // D-52: mutual groups seed their module cap from the members'
    // declared signatures (same derivation as the elab's projections).
    let mut fn_decls: Vec<l::FnDecl<'_>> = Vec::new();
    for item in file.items() {
        if let Some(l::ItemBody::FnDecl(f)) = item.body() {
            fn_decls.push(f);
        }
    }
    for members in elab_externs::mutual_groups(root) {
        let (_, cap_name) = elab_externs::group_names(&members);
        let mut op_names = Vec::new();
        for m in &members {
            op_names.push(atom(m.clone()));
            let Some(f) =
                fn_decls.iter().find(|f| f.name().is_some_and(|n| &n.text == m))
            else {
                continue;
            };
            // Members without full ground sigs stay out of Σ — the
            // elab errors on them first.
            let Some(comp) = elab_externs::fn_signature_comp_text(f) else { continue };
            let sem = sem_c(&comp)?;
            ctxs.entry("Σ".to_owned()).or_default().push((
                app("Op", vec![atom(cap_name.clone()), atom(m.clone())]),
                app("Sig", vec![cons(Vec::new()), sem]),
            ));
        }
        ctxs.entry("Σ".to_owned())
            .or_default()
            .push((app("Ops", vec![atom(cap_name)]), set(op_names, None)));
    }
    Ok(())
}

/// D-49/D-50: derive and seed the `{T}_impl` instance cap from an
/// inherent impl's method signatures — self reads as the impl head,
/// remaining params and the return come from annotations (untyped
/// ones stay out of Σ, so using them bails at the bundle check).
/// `binders` are the impl-level generics; method-level generics join
/// them per sig.
fn seed_inherent(
    i: &l::ImplDecl<'_>,
    target: &str,
    binders: &[String],
    ctxs: &mut Contexts,
) -> Result<(), Vec<String>> {
    let cap_name = format!("{target}_impl");
    // Ground impls substitute `Self`; generic heads are spelled out.
    let self_target = binders.is_empty().then_some(target);
    let self_type = match i.head().as_ref() {
        Some(head) if !binders.is_empty() => match elab_externs::type_v_text(head) {
            Some(t) => t,
            None => return Ok(()),
        },
        _ => crate::mir::builder::named_type_v(target, None),
    };
    let type_text = |ty: &l::TypeExpr<'_>| match self_target {
        Some(t) => elab_externs::type_v_text_self(ty, t),
        None => elab_externs::type_v_text(ty),
    };
    let mut op_names = Vec::new();
    for m in elab_externs::self_methods(i) {
        let Some(op_name) = m.name() else { continue };
        op_names.push(atom(op_name.text.clone()));
        let mut sig_binders: Vec<Term> = binders.iter().map(|b| atom(b.clone())).collect();
        if let Some(generics) = m.generic_params() {
            let mut ok = true;
            for p in generics.params() {
                if p.constraint().is_some() {
                    ok = false;
                    break;
                }
                match p.name() {
                    Some(n) => sig_binders.push(atom(n.text.clone())),
                    None => {
                        ok = false;
                        break;
                    }
                }
            }
            if !ok {
                continue;
            }
        }
        let Some(params) = m.param_list() else { continue };
        let mut texts = vec![self_type.clone()];
        let mut typed = true;
        for param in params.params().skip(1) {
            match param.ty().as_ref().and_then(|ty| type_text(ty)) {
                Some(t) => texts.push(t),
                None => {
                    typed = false;
                    break;
                }
            }
        }
        if !typed {
            continue;
        }
        let ret = match m.return_type() {
            Some(ty) => match type_text(&ty) {
                Some(text) => text,
                None => continue,
            },
            None => crate::mir::builder::named_type_v("Unit", None),
        };
        let row = m.cap_annotation().and_then(elab_externs::cap_row_text);
        let comp = elab_externs::signature_type_c_text(&texts, &ret, row.as_deref());
        let sem = sem_c(&comp)?;
        ctxs.entry("Σ".to_owned()).or_default().push((
            app("Op", vec![atom(cap_name.clone()), atom(op_name.text.clone())]),
            app("Sig", vec![cons(sig_binders), sem]),
        ));
    }
    ctxs.entry("Σ".to_owned())
        .or_default()
        .push((app("Ops", vec![atom(cap_name)]), set(op_names, None)));
    Ok(())
}

fn seed_data(d: &l::DataDecl<'_>, ctxs: &mut Contexts) -> Result<(), Vec<String>> {
    let Some(name) = d.name() else { return Ok(()) };
    let mut binders: Vec<String> = Vec::new();
    if let Some(generics) = d.generic_params() {
        binders.extend(generics.params().filter_map(|p| p.name().map(|t| t.text.clone())));
    }
    let owner_text = if binders.is_empty() {
        crate::mir::builder::named_type_v(&name.text, None)
    } else {
        let args: Vec<String> =
            binders.iter().map(|b| crate::mir::builder::named_type_v(b, None)).collect();
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        crate::mir::builder::named_type_v(
            &name.text,
            Some(&crate::mir::builder::type_args(&refs)),
        )
    };
    let owner = sem_v(&owner_text)?;
    let binder_atoms = cons(binders.iter().map(|b| atom(b.clone())).collect());
    for variant in d.variants() {
        let Some(tag) = variant.name() else { continue };
        let mut params = Vec::new();
        if let Some(fields) = variant.fields() {
            for field in fields.fields() {
                let Some(text) = elab_externs::type_v_text(&field) else {
                    return Err(vec![format!(
                        "data `{}`: variant `{}` has an unspellable field type",
                        name.text, tag.text
                    )]);
                };
                params.push(sem_v(&text)?);
            }
        }
        ctxs.entry("Δ".to_owned()).or_default().push((
            atom(tag.text.clone()),
            app("Variant", vec![owner.clone(), binder_atoms.clone(), cons(params)]),
        ));
    }
    Ok(())
}

/// Seed one cap's Σ entries. With `name`/`self_target` set, seeds a
/// D-48 ground instance cap (`Add_Number`) instead of the declared
/// name, substituting `Self` in every op type.
fn seed_cap(
    c: &l::CapDecl<'_>,
    ctxs: &mut Contexts,
    name: Option<&str>,
    self_target: Option<&str>,
) -> Result<(), Vec<String>> {
    let Some(cap) = c.name() else { return Ok(()) };
    let cap_name = name.unwrap_or(&cap.text).to_owned();
    let mut op_names = Vec::new();
    for item in c.items() {
        let l::CapItem::OperationDecl(op) = item else { continue };
        let Some(op_name) = op.name() else { continue };
        op_names.push(atom(op_name.text.clone()));
        // Ops with untyped parameters stay out of Σ — using them bails.
        let Some(texts) = op_param_texts(op.param_list(), self_target) else { continue };
        let ret = match op.return_type() {
            Some(ty) => match type_v_text_maybe_self(&ty, self_target) {
                Some(text) => text,
                None => continue,
            },
            None => crate::mir::builder::named_type_v("Unit", None),
        };
        let refs: Vec<String> = texts;
        let comp = elab_externs::signature_type_c_text(&refs, &ret, None);
        let sem = sem_c(&comp)?;
        // D-50: sigs uniformly carry a binder list — none here.
        ctxs.entry("Σ".to_owned()).or_default().push((
            app("Op", vec![atom(cap_name.clone()), atom(op_name.text.clone())]),
            app("Sig", vec![cons(Vec::new()), sem]),
        ));
    }
    ctxs.entry("Σ".to_owned())
        .or_default()
        .push((app("Ops", vec![atom(cap_name)]), set(op_names, None)));
    Ok(())
}

fn type_v_text_maybe_self(ty: &l::TypeExpr<'_>, self_target: Option<&str>) -> Option<String> {
    match self_target {
        Some(t) => elab_externs::type_v_text_self(ty, t),
        None => elab_externs::type_v_text(ty),
    }
}

fn seed_extern(
    e: &l::ExternDecl<'_>,
    ctxs: &mut Contexts,
    lines: &mut Vec<String>,
) -> Result<(), Vec<String>> {
    let mut tails: Vec<l::ExternFnTail<'_>> = Vec::new();
    match e.rest() {
        Some(l::ExternRest::ExternFnTail(f)) => tails.push(f),
        Some(l::ExternRest::ExternBlockTail(b)) => {
            for item in b.items() {
                if let Some(l::ExternBlockItemBody::ExternFnTail(f)) = item.body() {
                    tails.push(f);
                }
            }
        }
        _ => {}
    }
    for f in tails {
        let Some(name) = f.name() else { continue };
        let Some(texts) = op_param_texts(f.param_list(), None) else { continue };
        let ret = match f.return_type() {
            Some(ty) => match elab_externs::type_v_text(&ty) {
                Some(text) => text,
                None => continue,
            },
            None => crate::mir::builder::named_type_v("Unit", None),
        };
        let row = f.cap_annotation().and_then(elab_externs::cap_row_text);
        let comp = elab_externs::signature_type_c_text(&texts, &ret, row.as_deref());
        let sem = sem_v(&crate::mir::builder::u_type_v(&comp))?;
        lines.push(format!("{} : {}", name.text, print_v(&sem)));
        ctxs.entry("Γ".to_owned()).or_default().push((atom(name.text.clone()), sem));
    }
    Ok(())
}

/// Typed parameter list as MIR TypeV texts; `None` if any parameter
/// is untyped or unspellable. `self_target` substitutes `Self` (D-48).
fn op_param_texts(
    params: Option<l::ParamList<'_>>,
    self_target: Option<&str>,
) -> Option<Vec<String>> {
    let mut texts = Vec::new();
    if let Some(list) = params {
        for param in list.params() {
            texts.push(type_v_text_maybe_self(&param.ty()?, self_target)?);
        }
    }
    Some(texts)
}

// === Syntactic type text → semantic term (parse + norm) ===

fn sem_v(ty_text: &str) -> Result<Term, Vec<String>> {
    let wrapped = format!("def __ty = (x : {ty_text})");
    let parsed = crate::mir::parser::parse(&wrapped);
    if !parsed.errors.is_empty() {
        return Err(vec![format!("type `{ty_text}` does not parse as MIR")]);
    }
    let node = m::File::cast(&parsed.root)
        .and_then(|f| f.defs().next())
        .and_then(|d| d.value())
        .and_then(|v| match v {
            m::Value::ParenV(p) => p.ty(),
            _ => None,
        })
        .ok_or_else(|| vec![format!("type `{ty_text}`: no annotation node")])?;
    judgments::solve("norm_v", node.syntax(), Contexts::new())
        .map(|d| d.args[1].clone())
        .map_err(|b| vec![format!("norm of `{ty_text}` failed: {}", b.message)])
}

fn sem_c(comp_text: &str) -> Result<Term, Vec<String>> {
    let sem = sem_v(&crate::mir::builder::u_type_v(comp_text))?;
    match sem {
        Term::Struct(f, mut args) if f == "UTypeV" && args.len() == 1 => Ok(args.remove(0)),
        other => Err(vec![format!("expected U(…), normalized to {other:?}")]),
    }
}

// === Printing: semantic terms in the MIR type sub-language ===

fn cons(items: Vec<Term>) -> Term {
    items.into_iter().rev().fold(atom("#nil"), |t, h| app("#cons", vec![h, t]))
}

fn cons_items(term: &Term) -> Vec<&Term> {
    let mut items = Vec::new();
    let mut cur = term;
    while let Term::Struct(f, args) = cur {
        if f != "#cons" || args.len() != 2 {
            break;
        }
        items.push(&args[0]);
        cur = &args[1];
    }
    items
}

fn print_v(term: &Term) -> String {
    match term {
        Term::Var(v) => format!("?{v}"),
        Term::Atom(a) => a.clone(),
        Term::Struct(f, args) => match (f.as_str(), args.as_slice()) {
            ("UTypeV", [inner]) => format!("U({})", print_c(inner)),
            ("NamedTypeV", [name, ty_args]) => {
                let name = print_v(name);
                match ty_args {
                    Term::Atom(a) if a == "#none" => name,
                    Term::Struct(f, list) if f == "TypeArgs" && list.len() == 1 => {
                        let items: Vec<String> =
                            cons_items(&list[0]).into_iter().map(print_v).collect();
                        format!("{name}[{}]", items.join(", "))
                    }
                    other => format!("{name}[{}]", print_v(other)),
                }
            }
            _ => print_c(term),
        },
        Term::Set { .. } => row_suffix(term),
    }
}

fn print_c(term: &Term) -> String {
    match term {
        Term::Struct(f, args) => match (f.as_str(), args.as_slice()) {
            ("FTypeC", [inner, row]) => format!("F({}){}", print_v(inner), row_suffix(row)),
            ("FnTypeC", [params, ret]) => {
                let items: Vec<String> = cons_items(params).into_iter().map(print_v).collect();
                format!("({}) -> {}", items.join(", "), print_c(ret))
            }
            ("ForallTypeC", [binders, body]) => {
                let items: Vec<String> = cons_items(binders).into_iter().map(print_v).collect();
                format!("forall {}. {}", items.join(", "), print_c(body))
            }
            _ => format!("{term:?}"),
        },
        Term::Var(v) => format!("?{v}"),
        _ => format!("{term:?}"),
    }
}

/// ` / {entries, ..rest}` — empty closed rows print as nothing.
fn row_suffix(row: &Term) -> String {
    let (entries, rest): (Vec<String>, Option<String>) = match row {
        Term::Set { entries, rest } => (
            entries.iter().map(print_v).collect(),
            rest.as_ref().map(|r| rest_text(r)),
        ),
        // An empty open row resolves to its bare tail.
        other => (Vec::new(), Some(rest_text(other))),
    };
    if entries.is_empty() && rest.is_none() {
        return String::new();
    }
    let mut parts = entries;
    parts.extend(rest);
    format!(" / {{{}}}", parts.join(", "))
}

fn rest_text(rest: &Term) -> String {
    match rest {
        Term::Struct(f, args) if f == "RowVar" && args.len() == 1 => match &args[0] {
            Term::Atom(a) if a == "#none" => "..".to_owned(),
            other => format!("..{}", print_v(other)),
        },
        Term::Var(v) => format!("..?{v}"),
        other => format!("..{}", print_v(other)),
    }
}
