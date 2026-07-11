//! The `:optimize(MIR)` driver (M3 step 5, D-42): parse MIR, encode the
//! tree as an egglog expression, run the compiled `between MIR` program
//! through the langue-rt saturate/extract/reduce loop (host-side
//! `subst`), decode the winning term back to MIR text, and reparse for
//! the canonical output. Encode/decode failures and non-convergence
//! come back as an `ERROR: …` output line (D-26).
//!
//! Encoding rides the generated judgment term encoder
//! (`judgments::term_of`: node = `Struct(Name, fields)`, token =
//! `Atom(text)`, absent = `#none`, list = `#cons`/`#nil`) and maps it
//! onto the compiled program's conventions: standalone structs are
//! `Mk`-prefixed, lists are `(vec-of …)`/`(vec-empty)`, tokens are
//! strings. Optional fields follow D-42: absent list-carrying wrappers
//! (ctor args, case binders, type args, value args) encode as the
//! wrapper with an empty Vec; a bare `ParenV` encodes as its inner
//! value; any other absent optional (an `F` row, a bare `..`) is
//! unencodable.

use langue_rt::{EggTerm, Term};

use crate::mir::judgments;

/// Bounds for the D-42 loop: egglog iterations per saturation round,
/// and reduce/union rounds before giving up.
const RUN_ITERATIONS: usize = 10;
const MAX_ROUNDS: usize = 20;

pub fn optimize_report(source: &str) -> langue_rt::ElabReport {
    match run(source) {
        Ok(output) => langue_rt::ElabReport { output, errors: Vec::new() },
        Err(errors) => langue_rt::ElabReport { output: String::new(), errors },
    }
}

fn run(source: &str) -> Result<String, Vec<String>> {
    let mir = crate::mir::parser::parse(source);
    if !mir.errors.is_empty() {
        return Err(mir.errors.iter().map(|e| format!("parse: {}", e.message)).collect());
    }
    let root = match encode(&judgments::term_of(&mir.root)) {
        Ok(root) => root,
        Err(e) => return Ok(format!("ERROR: {e}")),
    };
    let best = match langue_rt::optimize_loop(
        crate::mir::between::PROGRAM,
        &root.to_sexpr(),
        RUN_ITERATIONS,
        MAX_ROUNDS,
        "subst",
        reduce_subst_call,
    ) {
        Ok(best) => best,
        Err(e) => return Ok(format!("ERROR: {e}")),
    };
    let text = match decode_file(&best) {
        Ok(text) => text,
        Err(e) => return Ok(format!("ERROR: {e}")),
    };
    let out = crate::mir::parser::parse(&text);
    if !out.errors.is_empty() {
        return Ok(format!(
            "ERROR: optimized output does not reparse as MIR: {} (in `{text}`)",
            out.errors[0].message
        ));
    }
    Ok(crate::mir::printer::canonical(&out.root))
}

// === Host-side subst reduction (D-24/D-42) ===

/// `(subst t b v)` → `t` with every `(VarV b)` replaced by `v`. Naive:
/// no shadow-stopping (the grammar has no binder markers, D-42). Nested
/// subst calls in the extracted arguments reduce first.
fn reduce_subst_call(call: &EggTerm) -> Result<EggTerm, String> {
    match call {
        EggTerm::App(head, args) if head == "subst" && args.len() == 3 => {
            let target = eliminate_substs(&args[0])?;
            let EggTerm::Str(binder) = &args[1] else {
                return Err(format!("subst binder is not a string: {:?}", args[1]));
            };
            let replacement = eliminate_substs(&args[2])?;
            Ok(replace_var(&target, binder, &replacement))
        }
        other => Err(format!("not a subst call: {other:?}")),
    }
}

fn eliminate_substs(term: &EggTerm) -> Result<EggTerm, String> {
    match term {
        EggTerm::Str(_) => Ok(term.clone()),
        EggTerm::App(head, _) if head == "subst" => reduce_subst_call(term),
        EggTerm::App(head, args) => {
            let args =
                args.iter().map(eliminate_substs).collect::<Result<Vec<_>, _>>()?;
            Ok(EggTerm::App(head.clone(), args))
        }
    }
}

fn replace_var(term: &EggTerm, binder: &str, replacement: &EggTerm) -> EggTerm {
    if let EggTerm::App(head, args) = term {
        if head == "VarV" && args == &[EggTerm::Str(binder.to_owned())] {
            return replacement.clone();
        }
        let args = args.iter().map(|a| replace_var(a, binder, replacement)).collect();
        return EggTerm::App(head.clone(), args);
    }
    term.clone()
}

// === Encoding: judgment term → egglog expression ===

/// Standalone structs collide with their own datatype name and get the
/// `Mk` prefix (mirrors `codegen/between.rs::ctor_name`).
const MK_STRUCTS: &[&str] = &[
    "BundleClause",
    "CapEntry",
    "CapRow",
    "CapSet",
    "CaseArm",
    "CaseBinders",
    "CtorArgs",
    "Def",
    "File",
    "TypeArgs",
    "ValueArgs",
];

fn ctor_name(node: &str) -> String {
    if MK_STRUCTS.contains(&node) {
        format!("Mk{node}")
    } else {
        node.to_owned()
    }
}

fn is_none(term: &Term) -> bool {
    matches!(term, Term::Atom(a) if a == "#none")
}

/// What an absent optional field encodes as (D-42): list-carrying
/// wrappers become the wrapper around an empty Vec (semantically
/// identical); everything else is unencodable.
fn absent_default(node: &str, index: usize) -> Result<EggTerm, String> {
    let empty = |wrapper: &str| {
        Ok(EggTerm::app(format!("Mk{wrapper}"), vec![EggTerm::app("vec-empty", vec![])]))
    };
    match (node, index) {
        ("CtorV", 1) => empty("CtorArgs"),
        ("CaseArm", 1) => empty("CaseBinders"),
        ("NamedTypeV", 1) | ("CapSig", 1) => empty("TypeArgs"),
        ("CompPostfix", 1) => empty("ValueArgs"),
        _ => Err(format!(
            "unencodable: `{node}` is missing a field the egglog grammar requires (D-42)"
        )),
    }
}

fn encode(term: &Term) -> Result<EggTerm, String> {
    match term {
        Term::Atom(a) if a == "#nil" => Ok(EggTerm::app("vec-empty", vec![])),
        Term::Atom(a) if a == "#none" => {
            Err("unencodable: absent field outside any node (D-42)".to_owned())
        }
        Term::Atom(a) => Ok(EggTerm::str(a.clone())),
        Term::Struct(f, _) if f == "#cons" => {
            let mut items = Vec::new();
            let mut cur = term;
            while let Term::Struct(f, args) = cur {
                if f != "#cons" || args.len() != 2 {
                    break;
                }
                items.push(encode(&args[0])?);
                cur = &args[1];
            }
            Ok(EggTerm::app("vec-of", items))
        }
        Term::Struct(f, args) => {
            // A bare paren is syntax — encode the inner value directly.
            if f == "ParenV" && args.len() == 2 && is_none(&args[1]) {
                return encode(&args[0]);
            }
            let mut out = Vec::new();
            for (i, arg) in args.iter().enumerate() {
                if is_none(arg) {
                    out.push(absent_default(f, i)?);
                } else {
                    out.push(encode(arg)?);
                }
            }
            Ok(EggTerm::App(ctor_name(f), out))
        }
        Term::Var(_) | Term::Set { .. } => {
            Err(format!("unencodable term shape: {term:?}"))
        }
    }
}

// === Decoding: extracted term → MIR surface text ===

fn args_of<'t>(term: &'t EggTerm, head: &str, n: usize) -> Result<&'t [EggTerm], String> {
    match term {
        EggTerm::App(h, args) if h == head && args.len() == n => Ok(args),
        other => Err(format!("expected ({head} ×{n}), got {other:?}")),
    }
}

fn text_of(term: &EggTerm) -> Result<&str, String> {
    match term {
        EggTerm::Str(s) => Ok(s),
        other => Err(format!("expected a token, got {other:?}")),
    }
}

/// Items of a `(vec-of …)` / `(vec-empty)` container.
fn vec_items(term: &EggTerm) -> Result<&[EggTerm], String> {
    match term {
        EggTerm::App(h, args) if h == "vec-of" => Ok(args),
        EggTerm::App(h, args) if h == "vec-empty" && args.is_empty() => Ok(&[]),
        other => Err(format!("expected a Vec container, got {other:?}")),
    }
}

fn decode_file(term: &EggTerm) -> Result<String, String> {
    let [defs] = args_of(term, "MkFile", 1)? else { unreachable!() };
    let mut lines = Vec::new();
    for def in vec_items(defs)? {
        let [name, value] = args_of(def, "MkDef", 2)? else { unreachable!() };
        lines.push(format!("def {} = {}", text_of(name)?, decode_v(value)?));
    }
    Ok(lines.join("\n"))
}

fn decode_list(term: &EggTerm, decode: fn(&EggTerm) -> Result<String, String>) -> Result<Vec<String>, String> {
    vec_items(term)?.iter().map(decode).collect()
}

fn decode_v(term: &EggTerm) -> Result<String, String> {
    let EggTerm::App(head, args) = term else {
        return Err(format!("expected a Value node, got {term:?}"));
    };
    match (head.as_str(), args.as_slice()) {
        ("VarV", [n]) | ("NumV", [n]) | ("StrV", [n]) => Ok(text_of(n)?.to_owned()),
        ("ThunkV", [c]) => Ok(format!("thunk {{ {} }}", decode_c(c)?)),
        ("CtorV", [tag, ctor_args]) => {
            let [items] = args_of(ctor_args, "MkCtorArgs", 1)? else { unreachable!() };
            let items = decode_list(items, decode_v)?;
            if items.is_empty() {
                Ok(format!(".{}", text_of(tag)?))
            } else {
                Ok(format!(".{}({})", text_of(tag)?, items.join(", ")))
            }
        }
        ("RollV", [v]) => Ok(format!("roll {}", decode_v(v)?)),
        ("UnrollV", [v]) => Ok(format!("unroll {}", decode_v(v)?)),
        ("BundleV", [clauses]) => {
            let mut parts = Vec::new();
            for clause in vec_items(clauses)? {
                let [name, params, body] = args_of(clause, "MkBundleClause", 3)? else {
                    unreachable!()
                };
                let params: Vec<String> = vec_items(params)?
                    .iter()
                    .map(|p| text_of(p).map(str::to_owned))
                    .collect::<Result<_, _>>()?;
                parts.push(format!(
                    "fn {}({}) => {};",
                    text_of(name)?,
                    params.join(", "),
                    decode_c(body)?
                ));
            }
            Ok(format!("bundle {{ {} }}", parts.join(" ")))
        }
        ("ParenV", [v, ty]) => Ok(format!("({} : {})", decode_v(v)?, decode_tv(ty)?)),
        _ => Err(format!("cannot decode Value node {head:?}/{}", args.len())),
    }
}

fn decode_c(term: &EggTerm) -> Result<String, String> {
    let EggTerm::App(head, args) = term else {
        return Err(format!("expected a Comp node, got {term:?}"));
    };
    match (head.as_str(), args.as_slice()) {
        ("RetC", [v]) => Ok(format!("ret {}", decode_v(v)?)),
        ("LetC", [n, value, body]) => Ok(format!(
            "let {} = {} in {}",
            text_of(n)?,
            decode_c(value)?,
            decode_c(body)?
        )),
        ("LamC", [p, body]) => Ok(format!("fn ({}) => {}", text_of(p)?, decode_c(body)?)),
        ("ForceC", [v]) => Ok(format!("force {}", decode_v(v)?)),
        ("CaseC", [scrutinee, arms]) => {
            let mut parts = Vec::new();
            for arm in vec_items(arms)? {
                let [tag, binders, body] = args_of(arm, "MkCaseArm", 3)? else {
                    unreachable!()
                };
                let [names] = args_of(binders, "MkCaseBinders", 1)? else { unreachable!() };
                let names: Vec<String> = vec_items(names)?
                    .iter()
                    .map(|n| text_of(n).map(str::to_owned))
                    .collect::<Result<_, _>>()?;
                let binders = if names.is_empty() {
                    String::new()
                } else {
                    format!("({})", names.join(", "))
                };
                parts.push(format!(".{}{binders} => {},", text_of(tag)?, decode_c(body)?));
            }
            Ok(format!("case {} {{ {} }}", decode_v(scrutinee)?, parts.join(" ")))
        }
        ("FixC", [n, body]) => Ok(format!("fix {} => {}", text_of(n)?, decode_c(body)?)),
        ("PerformC", [cap]) => Ok(format!("perform {}", text_of(cap)?)),
        ("HandleC", [cap, handler, body]) => Ok(format!(
            "handle {} with {} in {}",
            text_of(cap)?,
            decode_v(handler)?,
            decode_c(body)?
        )),
        ("SelC", [v, field]) => Ok(format!("sel {}.{}", decode_v(v)?, text_of(field)?)),
        ("ParenC", [c]) => Ok(format!("({})", decode_c(c)?)),
        ("CompPostfix", [callee, value_args]) => {
            let [items] = args_of(value_args, "MkValueArgs", 1)? else { unreachable!() };
            let items = decode_list(items, decode_v)?;
            // Comps that end in a nested Comp would swallow the `(…)`
            // postfix into that inner position on reparse — force the
            // grouping.
            let callee_text = match callee {
                EggTerm::App(h, _) if matches!(h.as_str(), "LetC" | "LamC" | "FixC" | "HandleC") => {
                    format!("({})", decode_c(callee)?)
                }
                _ => decode_c(callee)?,
            };
            Ok(format!("{callee_text}({})", items.join(", ")))
        }
        _ => Err(format!("cannot decode Comp node {head:?}/{}", args.len())),
    }
}

fn decode_tv(term: &EggTerm) -> Result<String, String> {
    let EggTerm::App(head, args) = term else {
        return Err(format!("expected a TypeV node, got {term:?}"));
    };
    match (head.as_str(), args.as_slice()) {
        ("UTypeV", [tc]) => Ok(format!("U({})", decode_tc(tc)?)),
        ("NamedTypeV", [n, type_args]) => {
            Ok(format!("{}{}", text_of(n)?, decode_type_args(type_args)?))
        }
        _ => Err(format!("cannot decode TypeV node {head:?}/{}", args.len())),
    }
}

fn decode_type_args(term: &EggTerm) -> Result<String, String> {
    let [items] = args_of(term, "MkTypeArgs", 1)? else { unreachable!() };
    let items = decode_list(items, decode_tv)?;
    if items.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("[{}]", items.join(", ")))
    }
}

fn decode_tc(term: &EggTerm) -> Result<String, String> {
    let EggTerm::App(head, args) = term else {
        return Err(format!("expected a TypeC node, got {term:?}"));
    };
    match (head.as_str(), args.as_slice()) {
        ("FTypeC", [tv, row]) => Ok(format!("F({}){}", decode_tv(tv)?, decode_row(row)?)),
        ("FnTypeC", [params, ret]) => {
            let params = decode_list(params, decode_tv)?;
            Ok(format!("({}) -> {}", params.join(", "), decode_tc(ret)?))
        }
        ("ForallTypeC", [binders, body]) => {
            let binders: Vec<String> = vec_items(binders)?
                .iter()
                .map(|b| text_of(b).map(str::to_owned))
                .collect::<Result<_, _>>()?;
            Ok(format!("forall {}. {}", binders.join(", "), decode_tc(body)?))
        }
        _ => Err(format!("cannot decode TypeC node {head:?}/{}", args.len())),
    }
}

fn decode_row(term: &EggTerm) -> Result<String, String> {
    let [set] = args_of(term, "MkCapRow", 1)? else { unreachable!() };
    let [entries] = args_of(set, "MkCapSet", 1)? else { unreachable!() };
    let mut parts = Vec::new();
    for entry in vec_items(entries)? {
        let [body] = args_of(entry, "MkCapEntry", 1)? else { unreachable!() };
        let EggTerm::App(head, args) = body else {
            return Err(format!("expected a cap entry body, got {body:?}"));
        };
        match (head.as_str(), args.as_slice()) {
            ("CapSig", [n, type_args]) => {
                parts.push(format!("{}{}", text_of(n)?, decode_type_args(type_args)?));
            }
            ("CapRest", [n]) => parts.push(format!("..{}", text_of(n)?)),
            _ => return Err(format!("cannot decode cap entry {head:?}/{}", args.len())),
        }
    }
    Ok(format!(" / {{ {} }}", parts.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use langue_rt::{app, atom};

    #[test]
    fn encode_defaults_and_paren_transparency() {
        // `.Nil` (no args) → empty CtorArgs; bare parens vanish.
        let term = app(
            "ParenV",
            vec![app("CtorV", vec![atom("Nil"), atom("#none")]), atom("#none")],
        );
        assert_eq!(
            encode(&term).unwrap().to_sexpr(),
            r#"(CtorV "Nil" (MkCtorArgs (vec-empty)))"#
        );
        // An absent F row is unencodable.
        let term = app("FTypeC", vec![app("NamedTypeV", vec![atom("N"), atom("#none")]), atom("#none")]);
        assert!(encode(&term).unwrap_err().contains("unencodable"));
    }

    #[test]
    fn subst_reduction_is_naive_var_replacement() {
        // (subst (RetC (VarV x)) "x" (NumV 1)) → (RetC (NumV 1))
        let call = EggTerm::app(
            "subst",
            vec![
                EggTerm::app("RetC", vec![EggTerm::app("VarV", vec![EggTerm::str("x")])]),
                EggTerm::str("x"),
                EggTerm::app("NumV", vec![EggTerm::str("1")]),
            ],
        );
        assert_eq!(
            reduce_subst_call(&call).unwrap(),
            EggTerm::app("RetC", vec![EggTerm::app("NumV", vec![EggTerm::str("1")])])
        );
    }

    #[test]
    fn encode_decode_round_trip_via_pipeline() {
        // The full driver on an already-optimal input is the identity
        // (modulo canonical printing).
        let source = "def t = thunk { case unroll roll .Cons(1, x) { .Cons(h, t) => ret h, .Nil => ret 0, } }";
        let report = optimize_report(source);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        let canonical = crate::mir::printer::canonical(&crate::mir::parser::parse(source).root);
        assert_eq!(report.output, canonical);
    }
}
