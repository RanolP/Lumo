//! Between-group emitter (M1 step 8, D-19): each `between L` group
//! compiles to an egglog program — the language's node grammar as
//! mutually recursive datatypes (per-constructor `:cost 1`, D-31) plus
//! one `(rewrite lhs rhs)` per relation. `subst` (the D-24 built-in
//! tactic) is a high-cost constructor reduced host-side by the D-42
//! saturate/extract/reduce loop.

use std::collections::{BTreeMap, BTreeSet};

use crate::project::fields::{kind_set, node_fields, FieldTarget};
use crate::project::model::{BetweenDef, Language};
use crate::syntax::ast::{Con, Pat, RuleBody};

use super::Buf;

pub fn generate(lang_name: &str, lang: &Language, between: &BetweenDef) -> String {
    let mut buf = Buf::new();
    buf.blank();
    buf.line(&format!(
        "/// Egglog program for `between {lang_name}` (compiled here, D-19;"
    ));
    buf.line("/// executed by the D-42 saturate/extract/reduce loop).");
    buf.line("pub static PROGRAM: &str = r#\"");
    let mut program = String::new();
    emit_program(&mut program, lang_name, lang, between);
    buf.line(&program);
    buf.line("\"#;");
    buf.finish()
}

/// Datatype name of the values a grammar ref `R` produces: sort rules
/// (enums / praat) are their own datatype; concrete structs too.
fn sort_of(rule: &str) -> String {
    rule.to_owned()
}

/// Constructor name for a concrete node: the node name, `Mk`-prefixed
/// when it would collide with its own datatype (standalone structs).
fn ctor_name(node: &str, datatype: &str) -> String {
    if node == datatype {
        format!("Mk{node}")
    } else {
        node.to_owned()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum EggType {
    Str,
    Sort(String),
    Vec(Box<EggType>),
}

impl EggType {
    fn name(&self) -> String {
        match self {
            EggType::Str => "String".to_owned(),
            EggType::Sort(s) => s.clone(),
            EggType::Vec(inner) => format!("{}Vec", inner.name()),
        }
    }
}

fn field_type(field_target: &FieldTarget, many: bool) -> EggType {
    let base = match field_target {
        FieldTarget::Token(_) | FieldTarget::LitToken(_) => EggType::Str,
        FieldTarget::Node(rule) => EggType::Sort(sort_of(rule)),
    };
    if many {
        EggType::Vec(Box::new(base))
    } else {
        base
    }
}

fn emit_program(out: &mut String, lang_name: &str, lang: &Language, between: &BetweenDef) {
    out.push_str(&format!(
        "; between {lang_name} — compiled by langc (D-14/D-19). Costs default to 1\n\
         ; per constructor (D-31). Modeling notes (D-42): optional fields are\n\
         ; required — drivers encode absent lists as empty Vecs and bare\n\
         ; parens as their inner node; list fields are Vec sorts.\n"
    ));

    // Sort rules (enum/praat) own their member constructors; concrete
    // nodes not claimed by any sort become standalone datatypes.
    let mut sort_rules: Vec<String> = Vec::new();
    let mut claimed: BTreeMap<String, String> = BTreeMap::new(); // node → sort
    for (name, rule) in &lang.rules {
        let is_sort = match &rule.body {
            RuleBody::Praat(_) => true,
            RuleBody::Plain(shape) => crate::codegen::parser::enum_arms(shape).is_some(),
        };
        if is_sort {
            sort_rules.push(name.clone());
            for node in kind_set(lang, name) {
                claimed.entry(node).or_insert_with(|| name.clone());
            }
        }
    }
    let mut standalone: Vec<String> = Vec::new();
    for name in lang.rules.keys() {
        if node_fields(lang, name).is_some() && !claimed.contains_key(name) {
            standalone.push(name.clone());
        }
    }

    // Collect Vec sorts used by any constructor.
    let mut vec_sorts: BTreeSet<String> = BTreeSet::new();
    let ctor_line = |node: &str, datatype: &str, vec_sorts: &mut BTreeSet<String>| {
        let fields = node_fields(lang, node).expect("concrete node has fields");
        let mut args = Vec::new();
        for f in &fields {
            let ty = field_type(&f.target, f.many);
            if let EggType::Vec(_) = &ty {
                vec_sorts.insert(ty.name());
            }
            args.push(ty.name());
        }
        let args = if args.is_empty() { String::new() } else { format!(" {}", args.join(" ")) };
        format!("    ({}{args} :cost 1)", ctor_name(node, datatype))
    };

    let mut body = String::new();
    for sort in &sort_rules {
        body.push_str(&format!("  ({sort}\n"));
        for node in kind_set(lang, sort) {
            if claimed.get(&node).map(String::as_str) == Some(sort.as_str()) {
                body.push_str(&ctor_line(&node, sort, &mut vec_sorts));
                body.push('\n');
            }
        }
        body.push_str("  )\n");
    }
    for node in &standalone {
        body.push_str(&format!("  ({node}\n"));
        body.push_str(&ctor_line(node, node, &mut vec_sorts));
        body.push_str("\n  )\n");
    }

    out.push_str("(datatype*\n");
    out.push_str(&body);
    for vec_sort in &vec_sorts {
        let elem = vec_sort.strip_suffix("Vec").expect("vec sort naming");
        out.push_str(&format!("  (sort {vec_sort} (Vec {elem}))\n"));
    }
    out.push_str(")\n");

    // Rewrites. Bindings are typed by their LHS position so `subst` can
    // be declared with concrete sorts.
    let mut subst_sigs: BTreeSet<(String, String, String, String)> = BTreeSet::new();
    let mut rewrites = String::new();
    for rel in &between.relations {
        let mut bindings: BTreeMap<String, EggType> = BTreeMap::new();
        let mut wildcards = 0usize;
        let lhs = pat_sexpr(lang, &rel.lhs, None, &mut bindings, &mut wildcards);
        let rhs = con_sexpr(lang, &rel.rhs, &bindings, &mut subst_sigs);
        rewrites.push_str(&format!("(rewrite {lhs} {rhs})\n"));
    }
    for (target, var, replacement, result) in &subst_sigs {
        out.push_str(&format!(
            "; built-in tactic (D-24): substitution, reduced host-side (D-42).\n\
             ; High cost steers extraction to subst-free forms when they exist.\n\
             (constructor subst ({target} {var} {replacement}) {result} :cost 1000)\n"
        ));
    }
    out.push_str(&rewrites);
}

/// The datatype a node's constructed values belong to.
fn node_sort(lang: &Language, node: &str) -> String {
    for (name, rule) in &lang.rules {
        let is_sort = match &rule.body {
            RuleBody::Praat(_) => true,
            RuleBody::Plain(shape) => crate::codegen::parser::enum_arms(shape).is_some(),
        };
        if is_sort && kind_set(lang, name).contains(node) {
            return name.clone();
        }
    }
    node.to_owned()
}

fn pat_sexpr(
    lang: &Language,
    pat: &Pat,
    ty: Option<&EggType>,
    bindings: &mut BTreeMap<String, EggType>,
    wildcards: &mut usize,
) -> String {
    match pat {
        Pat::Var { name, .. } => {
            if let Some(ty) = ty {
                bindings.insert(name.clone(), ty.clone());
            }
            name.clone()
        }
        Pat::ListVar { name, .. } => {
            if let Some(ty) = ty {
                bindings.insert(name.clone(), ty.clone());
            }
            name.clone()
        }
        Pat::Lit { text, .. } => format!("{text:?}"),
        Pat::Node { name, fields: pat_fields, .. } => {
            let table = node_fields(lang, name).expect("checked: concrete node");
            let datatype = node_sort(lang, name);
            let mut args = Vec::new();
            for field in &table {
                match pat_fields.iter().find(|(l, _)| l == &field.label) {
                    Some((_, sub)) => {
                        let ty = field_type(&field.target, field.many);
                        args.push(pat_sexpr(lang, sub, Some(&ty), bindings, wildcards));
                    }
                    None => {
                        // Omitted fields match anything: a fresh variable.
                        let w = format!("w{wildcards}");
                        *wildcards += 1;
                        args.push(w);
                    }
                }
            }
            let args =
                if args.is_empty() { String::new() } else { format!(" {}", args.join(" ")) };
            format!("({}{args})", ctor_name(name, &datatype))
        }
    }
}

fn con_sexpr(
    lang: &Language,
    con: &Con,
    bindings: &BTreeMap<String, EggType>,
    subst_sigs: &mut BTreeSet<(String, String, String, String)>,
) -> String {
    match con {
        Con::Var { name, .. } => name.clone(),
        Con::Lit { text, .. } => format!("{text:?}"),
        Con::Subst { target, var, replacement, .. } => {
            let ty = |n: &String| {
                bindings.get(n).map(EggType::name).unwrap_or_else(|| "String".to_owned())
            };
            subst_sigs.insert((ty(target), ty(var), ty(replacement), ty(target)));
            format!("(subst {target} {var} {replacement})")
        }
        Con::Node { name, fields: con_fields, .. } => {
            let table = node_fields(lang, name).expect("checked: concrete node");
            let datatype = node_sort(lang, name);
            let mut args = Vec::new();
            for field in &table {
                match con_fields.iter().find(|(l, _)| l == &field.label) {
                    Some((_, sub)) => args.push(con_sexpr(lang, sub, bindings, subst_sigs)),
                    None => panic!(
                        "between rhs construction of `{name}` omits field `{}` — \
                         rhs must be total",
                        field.label
                    ),
                }
            }
            let args =
                if args.is_empty() { String::new() } else { format!(" {}", args.join(" ")) };
            format!("({}{args})", ctor_name(name, &datatype))
        }
        Con::VarTo { .. } | Con::ListVarTo { .. } => {
            // Check rejects `to` recursion in between relations.
            unreachable!("checked: no recursion in between relations")
        }
    }
}
