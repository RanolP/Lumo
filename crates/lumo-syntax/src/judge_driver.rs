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
    for item in file.items() {
        match item.body() {
            Some(l::ItemBody::DataDecl(d)) => seed_data(&d, ctxs)?,
            Some(l::ItemBody::CapDecl(c)) => seed_cap(&c, ctxs)?,
            Some(l::ItemBody::ExternDecl(e)) => seed_extern(&e, ctxs, lines)?,
            _ => {}
        }
    }
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

fn seed_cap(c: &l::CapDecl<'_>, ctxs: &mut Contexts) -> Result<(), Vec<String>> {
    let Some(cap) = c.name() else { return Ok(()) };
    let mut op_names = Vec::new();
    for item in c.items() {
        let l::CapItem::OperationDecl(op) = item else { continue };
        let Some(op_name) = op.name() else { continue };
        op_names.push(atom(op_name.text.clone()));
        // Ops with untyped parameters stay out of Σ — using them bails.
        let Some(texts) = op_param_texts(op.param_list()) else { continue };
        let ret = match op.return_type() {
            Some(ty) => match elab_externs::type_v_text(&ty) {
                Some(text) => text,
                None => continue,
            },
            None => crate::mir::builder::named_type_v("Unit", None),
        };
        let refs: Vec<String> = texts;
        let comp = elab_externs::signature_type_c_text(&refs, &ret, None);
        let sem = sem_c(&comp)?;
        ctxs.entry("Σ".to_owned()).or_default().push((
            app("Op", vec![atom(cap.text.clone()), atom(op_name.text.clone())]),
            sem,
        ));
    }
    ctxs.entry("Σ".to_owned())
        .or_default()
        .push((app("Ops", vec![atom(cap.text.clone())]), set(op_names, None)));
    Ok(())
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
        let Some(texts) = op_param_texts(f.param_list()) else { continue };
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
/// is untyped or unspellable.
fn op_param_texts(params: Option<l::ParamList<'_>>) -> Option<Vec<String>> {
    let mut texts = Vec::new();
    if let Some(list) = params {
        for param in list.params() {
            texts.push(elab_externs::type_v_text(&param.ty()?)?);
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
