//! Builder emitter: per-node render-to-text functions derived from the
//! target grammar shape (M1 step 4). Constructions render target text
//! and reparse it with the target's generated parser — correct by
//! construction. Praat row builders parenthesize operands exactly when
//! reparsing the bare concatenation would attach an operator inside the
//! operand (binding-power comparison; see `r_bound`/`b_bound`).

use crate::project::fields::{kind_set, struct_fields};
use crate::project::model::Language;
use crate::project::praat::{classify_row, RowKind, TailPart};
use crate::syntax::ast::{Praat, RuleBody, Shape, ShapeKind};

use super::naming::snake;
use super::Buf;

pub fn generate(lang: &Language) -> String {
    let mut buf = Buf::new();
    buf.line("#![allow(dead_code)]");
    buf.blank();
    buf.line("use super::syntax_kind::SyntaxKind;");
    buf.blank();
    buf.line("/// A praat operand: rendered text plus its root kind, when known.");
    buf.line("/// `None` renders defensively (always parenthesized).");
    buf.open("pub struct Operand<'a> {");
    buf.line("pub text: &'a str,");
    buf.line("pub kind: Option<SyntaxKind>,");
    buf.close("}");
    buf.blank();
    buf.open("fn push(out: &mut String, s: &str) {");
    buf.open("if s.is_empty() {");
    buf.line("return;");
    buf.close("}");
    buf.open("if !out.is_empty() {");
    buf.line("out.push(' ');");
    buf.close("}");
    buf.line("out.push_str(s);");
    buf.close("}");

    for (name, rule) in &lang.rules {
        match &rule.body {
            RuleBody::Plain(shape) => {
                if crate::codegen::parser::enum_arms(shape).is_some() {
                    continue; // transparent — nothing to build
                }
                buf.blank();
                match buildable_plan(shape) {
                    Ok(()) => emit_struct_builder(&mut buf, name, shape),
                    Err(why) => {
                        buf.line(&format!("// `{name}` is not buildable: {why}."));
                    }
                }
            }
            RuleBody::Praat(praat) => emit_praat_builders(&mut buf, lang, name, praat),
        }
    }
    buf.finish()
}

/// Rust keywords that can appear as field labels.
fn param_name(label: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "as", "box", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
        "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
        "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true", "type",
        "unsafe", "use", "where", "while",
    ];
    if KEYWORDS.contains(&label) {
        format!("{label}_")
    } else {
        label.to_owned()
    }
}

/// Why a shape cannot be rendered from field values alone.
fn buildable_plan(shape: &Shape) -> Result<(), String> {
    match &shape.kind {
        ShapeKind::Seq(parts) => parts.iter().try_for_each(buildable_plan),
        ShapeKind::Alt(_) => Err("alternatives cannot be chosen from field values".to_owned()),
        ShapeKind::Opt(inner) => buildable_plan(inner),
        ShapeKind::Rep(inner) | ShapeKind::Sep { item: inner, .. } => {
            // A repetition renders per item — it needs exactly one
            // labeled field inside (plus literals).
            match count_labels(inner) {
                1 => buildable_leaf_group(inner),
                n => Err(format!("repetition with {n} labeled fields")),
            }
        }
        ShapeKind::Label { shape: inner, .. } => match &inner.kind {
            ShapeKind::NodeRef(_) | ShapeKind::TokenRef(_) | ShapeKind::Lit(_) => Ok(()),
            ShapeKind::Opt(o) => buildable_plan(&Shape::new(
                ShapeKind::Label { label: String::new(), shape: o.clone() },
                inner.span,
            )),
            ShapeKind::Rep(r) | ShapeKind::Sep { item: r, .. } => match &r.kind {
                ShapeKind::NodeRef(_) | ShapeKind::TokenRef(_) => Ok(()),
                _ => Err("label over a composite repetition".to_owned()),
            },
            _ => Err("label over a composite shape".to_owned()),
        },
        ShapeKind::Lit(_) => Ok(()),
        ShapeKind::TokenRef(t) => Err(format!("unlabeled token `{t}`")),
        ShapeKind::NodeRef(n) => Err(format!("unlabeled node `{n}`")),
    }
}

fn buildable_leaf_group(shape: &Shape) -> Result<(), String> {
    match &shape.kind {
        ShapeKind::Seq(parts) => parts.iter().try_for_each(buildable_leaf_group),
        ShapeKind::Lit(_) => Ok(()),
        ShapeKind::Label { shape: inner, .. } => match &inner.kind {
            ShapeKind::NodeRef(_) | ShapeKind::TokenRef(_) => Ok(()),
            _ => Err("label over a composite shape in a repetition".to_owned()),
        },
        _ => Err("unsupported shape in a repetition".to_owned()),
    }
}

fn count_labels(shape: &Shape) -> usize {
    match &shape.kind {
        ShapeKind::Label { .. } => 1,
        ShapeKind::Seq(parts) | ShapeKind::Alt(parts) => parts.iter().map(count_labels).sum(),
        ShapeKind::Opt(inner) | ShapeKind::Rep(inner) => count_labels(inner),
        ShapeKind::Sep { item, .. } => count_labels(item),
        _ => 0,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParamKind {
    Required,
    Optional,
    Many,
}

/// Parameters in shape-walk order, mirroring `fields::struct_fields`.
fn params_of(shape: &Shape) -> Vec<(String, ParamKind)> {
    fn walk(shape: &Shape, many: bool, optional: bool, out: &mut Vec<(String, ParamKind)>) {
        match &shape.kind {
            ShapeKind::Label { label, shape: inner } => {
                let (inner_many, inner_opt) = wrap_of(inner);
                let kind = if many || inner_many {
                    ParamKind::Many
                } else if optional || inner_opt {
                    ParamKind::Optional
                } else {
                    ParamKind::Required
                };
                out.push((label.clone(), kind));
            }
            ShapeKind::Seq(parts) => {
                for p in parts {
                    walk(p, many, optional, out);
                }
            }
            ShapeKind::Alt(parts) => {
                for p in parts {
                    walk(p, many, true, out);
                }
            }
            ShapeKind::Opt(inner) => walk(inner, many, true, out),
            ShapeKind::Rep(inner) => walk(inner, true, optional, out),
            ShapeKind::Sep { item, .. } => walk(item, true, optional, out),
            _ => {}
        }
    }
    fn wrap_of(shape: &Shape) -> (bool, bool) {
        match &shape.kind {
            ShapeKind::Opt(inner) => (wrap_of(inner).0, true),
            ShapeKind::Rep(_) | ShapeKind::Sep { .. } => (true, false),
            _ => (false, false),
        }
    }
    let mut out = Vec::new();
    walk(shape, false, false, &mut out);
    out
}

fn emit_struct_builder(buf: &mut Buf, name: &str, shape: &Shape) {
    let params = params_of(shape);
    let sig: Vec<String> = params
        .iter()
        .map(|(label, kind)| {
            let p = param_name(label);
            match kind {
                ParamKind::Required => format!("{p}: &str"),
                ParamKind::Optional => format!("{p}: Option<&str>"),
                ParamKind::Many => format!("{p}: &[&str]"),
            }
        })
        .collect();
    buf.line(&format!("/// Render a `{name}` node as reparseable text."));
    buf.open(&format!("pub fn {}({}) -> String {{", snake(name), sig.join(", ")));
    buf.line("let mut out = String::new();");
    emit_render(buf, shape, &params);
    buf.line("out");
    buf.close("}");
}

fn emit_render(buf: &mut Buf, shape: &Shape, params: &[(String, ParamKind)]) {
    match &shape.kind {
        ShapeKind::Seq(parts) => {
            for p in parts {
                emit_render(buf, p, params);
            }
        }
        ShapeKind::Lit(text) => {
            buf.line(&format!("push(&mut out, {text:?});"));
        }
        ShapeKind::Label { label, shape: inner } => {
            let p = param_name(label);
            let kind = params
                .iter()
                .find(|(l, _)| l == label)
                .map(|(_, k)| *k)
                .expect("label collected as param");
            match kind {
                ParamKind::Required => buf.line(&format!("push(&mut out, {p});")),
                ParamKind::Optional => {
                    buf.open(&format!("if let Some(v) = {p} {{"));
                    buf.line("push(&mut out, v);");
                    buf.close("}");
                }
                ParamKind::Many => {
                    // Bare labeled repetition: space-joined items. The
                    // sep/rep-with-literals cases are handled below.
                    match find_sep(inner) {
                        Some(sep) => emit_sep_loop(buf, &p, sep),
                        None => {
                            buf.open(&format!("for item in {p} {{"));
                            buf.line("push(&mut out, item);");
                            buf.close("}");
                        }
                    }
                }
            }
        }
        ShapeKind::Opt(inner) => {
            let labels: Vec<(String, ParamKind)> = params_of(inner)
                .iter()
                .map(|(l, _)| {
                    let k = params.iter().find(|(pl, _)| pl == l).map(|(_, k)| *k).unwrap();
                    (l.clone(), k)
                })
                .collect();
            if labels.is_empty() {
                return; // optional literals render minimally: omitted
            }
            let cond: Vec<String> = labels
                .iter()
                .map(|(l, k)| {
                    let p = param_name(l);
                    match k {
                        ParamKind::Optional => format!("{p}.is_some()"),
                        ParamKind::Many => format!("!{p}.is_empty()"),
                        ParamKind::Required => "true".to_owned(),
                    }
                })
                .collect();
            buf.open(&format!("if {} {{", cond.join(" && ")));
            emit_render(buf, inner, params);
            buf.close("}");
        }
        ShapeKind::Rep(inner) | ShapeKind::Sep { item: inner, .. } => {
            let sep = match &shape.kind {
                ShapeKind::Sep { sep, .. } => Some(sep.as_str()),
                _ => None,
            };
            // Exactly one labeled field inside (checked by buildable_plan).
            let label = params_of(inner)
                .first()
                .map(|(l, _)| l.clone())
                .expect("checked: one label per repetition");
            let p = param_name(&label);
            emit_group_loop(buf, inner, &p, sep);
        }
        ShapeKind::TokenRef(_) | ShapeKind::NodeRef(_) | ShapeKind::Alt(_) => {
            unreachable!("checked by buildable_plan")
        }
    }
}

/// The separator of a (possibly `?`-wrapped) `sep(…)` shape.
fn find_sep(shape: &Shape) -> Option<&str> {
    match &shape.kind {
        ShapeKind::Sep { sep, .. } => Some(sep),
        ShapeKind::Opt(inner) => find_sep(inner),
        _ => None,
    }
}

fn emit_sep_loop(buf: &mut Buf, param: &str, sep: &str) {
    buf.open(&format!("for (i, item) in {param}.iter().enumerate() {{"));
    buf.open("if i > 0 {");
    buf.line(&format!("push(&mut out, {sep:?});"));
    buf.close("}");
    buf.line("push(&mut out, item);");
    buf.close("}");
}

/// A repetition whose body mixes literals with one labeled field:
/// render the literals around each item.
fn emit_group_loop(buf: &mut Buf, inner: &Shape, param: &str, sep: Option<&str>) {
    buf.open(&format!("for (i, item) in {param}.iter().enumerate() {{"));
    if let Some(sep) = sep {
        buf.open("if i > 0 {");
        buf.line(&format!("push(&mut out, {sep:?});"));
        buf.close("}");
    } else {
        buf.line("let _ = i;");
    }
    fn emit_item(buf: &mut Buf, shape: &Shape) {
        match &shape.kind {
            ShapeKind::Seq(parts) => {
                for p in parts {
                    emit_item(buf, p);
                }
            }
            ShapeKind::Lit(text) => buf.line(&format!("push(&mut out, {text:?});")),
            ShapeKind::Label { .. } => buf.line("push(&mut out, item);"),
            _ => unreachable!("checked by buildable_leaf_group"),
        }
    }
    emit_item(buf, inner);
    buf.close("}");
}

// === praat rows ===

struct RowInfo {
    kind: RowKind,
    /// Binding strength tested against the trailing openness of a left
    /// operand: `rbp` for infix/mixfix, `lbp` for postfix.
    bind: u16,
}

fn emit_praat_builders(buf: &mut Buf, lang: &Language, name: &str, praat: &Praat) {
    let rows: Vec<RowInfo> = praat
        .rows
        .iter()
        .filter_map(|r| classify_row(r).ok())
        .map(|kind| {
            let bind = match &kind {
                RowKind::Prefix { rbp, .. } => *rbp,
                RowKind::Infix { rbp, .. } => *rbp,
                RowKind::Postfix { lbp, .. } => *lbp,
                RowKind::Mixfix { rbp, .. } => *rbp,
            };
            RowInfo { kind, bind }
        })
        .collect();
    let sn = snake(name);
    let kinds = kind_set(lang, name);

    // Trailing openness per operand kind: the min_bp of the rightmost
    // expression position inside it. An operator binds *inside* a left
    // operand iff its bind strength exceeds this.
    buf.blank();
    buf.line(&format!("/// Trailing openness of a `{name}` operand (see module docs)."));
    buf.open(&format!("fn {sn}_r_bound(kind: Option<SyntaxKind>) -> u16 {{"));
    buf.line("let Some(kind) = kind else { return 0 };");
    buf.open("match kind {");
    let mut open_atoms: Vec<String> = Vec::new();
    for kind in &kinds {
        if let Some(rule) = lang.rules.get(kind) {
            if let RuleBody::Plain(shape) = &rule.body {
                if trailing_open(lang, shape, &kinds) {
                    open_atoms.push(super::naming::kind_name(kind));
                }
            }
        }
    }
    if !open_atoms.is_empty() {
        let pats: Vec<String> =
            open_atoms.iter().map(|k| format!("SyntaxKind::{k}")).collect();
        buf.line(&format!("{} => 0,", pats.join(" | ")));
    }
    let placement_min = |placement: &str| -> Option<u16> {
        rows.iter()
            .filter_map(|r| match (&r.kind, placement) {
                (RowKind::Prefix { rbp, .. }, "Prefix") => Some(*rbp),
                (RowKind::Infix { lbp, .. }, "Infix") => Some(*lbp),
                (RowKind::Mixfix { lbp, .. }, "Mixfix") => Some(*lbp),
                _ => None,
            })
            .min()
    };
    for placement in ["Prefix", "Infix", "Mixfix"] {
        if let Some(min) = placement_min(placement) {
            buf.line(&format!(
                "SyntaxKind::{} => {min},",
                super::naming::kind_name(&format!("{name}{placement}"))
            ));
        }
    }
    buf.line("_ => u16::MAX,");
    buf.close("}");
    buf.close("}");

    // Bind strength per operand kind, for right/inner positions parsed
    // at a known min_bp: the operand's own top row must still bind there.
    buf.blank();
    buf.open(&format!("fn {sn}_b_bound(kind: Option<SyntaxKind>) -> u16 {{"));
    buf.line("let Some(kind) = kind else { return 0 };");
    buf.open("match kind {");
    let bind_min = |pred: &dyn Fn(&RowKind) -> bool| -> Option<u16> {
        rows.iter().filter(|r| pred(&r.kind)).map(|r| r.bind).min()
    };
    for (placement, pred) in [
        ("Infix", (&|k: &RowKind| matches!(k, RowKind::Infix { .. })) as &dyn Fn(&RowKind) -> bool),
        ("Postfix", &|k: &RowKind| matches!(k, RowKind::Postfix { .. })),
        ("Mixfix", &|k: &RowKind| matches!(k, RowKind::Mixfix { .. })),
    ] {
        if let Some(min) = bind_min(pred) {
            buf.line(&format!(
                "SyntaxKind::{} => {min},",
                super::naming::kind_name(&format!("{name}{placement}"))
            ));
        }
    }
    buf.line("_ => u16::MAX,");
    buf.close("}");
    buf.close("}");

    // The paren atom, for wrapping unsafe operands.
    let paren = find_paren_atom(lang, praat, &kinds);
    buf.blank();
    buf.open(&format!("fn {sn}_paren(text: &str) -> String {{"));
    match &paren {
        Some((atom, params)) => {
            let args: Vec<String> = params
                .iter()
                .map(|(is_target, kind)| {
                    if *is_target {
                        "text".to_owned()
                    } else {
                        match kind {
                            ParamKind::Optional => "None".to_owned(),
                            ParamKind::Many => "&[]".to_owned(),
                            ParamKind::Required => unreachable!("paren atom has one required field"),
                        }
                    }
                })
                .collect();
            buf.line(&format!("{}({})", snake(atom), args.join(", ")));
        }
        None => {
            buf.line(&format!(
                "panic!(\"language has no paren atom for `{name}` — cannot wrap {{text}}\")"
            ));
        }
    }
    buf.close("}");

    buf.blank();
    buf.line("/// Left operand of an operator row: parenthesize iff the row would");
    buf.line("/// bind inside it.");
    buf.open(&format!("fn {sn}_left(op: Operand, bind: u16) -> String {{"));
    buf.open(&format!("if bind > {sn}_r_bound(op.kind) {{"));
    buf.line(&format!("{sn}_paren(op.text)"));
    buf.else_open("} else {");
    buf.line("op.text.to_owned()");
    buf.close("}");
    buf.close("}");
    buf.blank();
    buf.line("/// Operand parsed at a known min_bp (prefix/infix right side).");
    buf.open(&format!("fn {sn}_at(op: Operand, min_bp: u16) -> String {{"));
    buf.open(&format!("if {sn}_b_bound(op.kind) <= min_bp {{"));
    buf.line(&format!("{sn}_paren(op.text)"));
    buf.else_open("} else {");
    buf.line("op.text.to_owned()");
    buf.close("}");
    buf.close("}");

    // Row builders, numbered within their placement when ambiguous.
    let count = |placement: &str| {
        rows.iter()
            .filter(|r| match (&r.kind, placement) {
                (RowKind::Prefix { .. }, "Prefix")
                | (RowKind::Infix { .. }, "Infix")
                | (RowKind::Postfix { .. }, "Postfix")
                | (RowKind::Mixfix { .. }, "Mixfix") => true,
                _ => false,
            })
            .count()
    };
    let mut seen: std::collections::BTreeMap<&str, usize> = Default::default();
    for row in &rows {
        let placement = match &row.kind {
            RowKind::Prefix { .. } => "Prefix",
            RowKind::Infix { .. } => "Infix",
            RowKind::Postfix { .. } => "Postfix",
            RowKind::Mixfix { .. } => "Mixfix",
        };
        let i = *seen.entry(placement).and_modify(|i| *i += 1).or_insert(0);
        let fn_name = if count(placement) > 1 {
            format!("{sn}_{}_{i}", placement.to_ascii_lowercase())
        } else {
            format!("{sn}_{}", placement.to_ascii_lowercase())
        };
        buf.blank();
        match &row.kind {
            RowKind::Prefix { toks, rbp } => {
                buf.open(&format!("pub fn {fn_name}(op: &str, expr: Operand) -> String {{"));
                buf.line("let mut out = String::new();");
                buf.line(&format!("debug_assert!({:?}.contains(&op));", toks));
                buf.line("push(&mut out, op);");
                buf.line(&format!("push(&mut out, &{sn}_at(expr, {rbp}));"));
                buf.line("out");
                buf.close("}");
            }
            RowKind::Infix { lbp, toks, rbp } => {
                buf.open(&format!(
                    "pub fn {fn_name}(lhs: Operand, op: &str, rhs: Operand) -> String {{"
                ));
                buf.line("let mut out = String::new();");
                buf.line(&format!("debug_assert!({:?}.contains(&op));", toks));
                buf.line(&format!("push(&mut out, &{sn}_left(lhs, {rbp}));"));
                buf.line("push(&mut out, op);");
                buf.line(&format!("push(&mut out, &{sn}_at(rhs, {lbp}));"));
                buf.line("out");
                buf.close("}");
            }
            RowKind::Postfix { lbp, tail } => {
                let mut params: Vec<String> = vec!["expr: Operand".to_owned()];
                let mut payloads = Vec::new();
                let mut buildable = true;
                for part in tail {
                    match part {
                        TailPart::Toks(toks) if toks.len() > 1 => buildable = false,
                        TailPart::Toks(_) => {}
                        TailPart::Node(rule) => {
                            payloads.push(snake(rule));
                            params.push(format!("{}: &str", param_name(&snake(rule))));
                        }
                    }
                }
                if !buildable {
                    buf.line(&format!(
                        "// postfix row `{fn_name}` is not buildable: multi-token choice in tail."
                    ));
                    continue;
                }
                buf.open(&format!("pub fn {fn_name}({}) -> String {{", params.join(", ")));
                buf.line("let mut out = String::new();");
                buf.line(&format!("push(&mut out, &{sn}_left(expr, {lbp}));"));
                let mut payload_i = 0;
                for part in tail {
                    match part {
                        TailPart::Toks(toks) => {
                            buf.line(&format!("push(&mut out, {:?});", toks[0]));
                        }
                        TailPart::Node(_) => {
                            buf.line(&format!(
                                "push(&mut out, {});",
                                param_name(&payloads[payload_i])
                            ));
                            payload_i += 1;
                        }
                    }
                }
                buf.line("out");
                buf.close("}");
            }
            RowKind::Mixfix { .. } => {
                buf.line(&format!("// mixfix row `{fn_name}` builder deferred (unused in M1)."));
            }
        }
    }
}

/// Does this atom shape end in a position that keeps parsing this praat
/// rule (its last significant element resolves into the rule's kinds)?
fn trailing_open(lang: &Language, shape: &Shape, praat_kinds: &std::collections::BTreeSet<String>) -> bool {
    match &shape.kind {
        ShapeKind::Seq(parts) => {
            // Walk backwards; optional trailers extend the candidates.
            for part in parts.iter().rev() {
                if trailing_open(lang, part, praat_kinds) {
                    return true;
                }
                if !is_optional_shape(part) {
                    return false;
                }
            }
            false
        }
        ShapeKind::Label { shape: inner, .. } | ShapeKind::Opt(inner) => {
            trailing_open(lang, inner, praat_kinds)
        }
        ShapeKind::Rep(inner) | ShapeKind::Sep { item: inner, .. } => {
            trailing_open(lang, inner, praat_kinds)
        }
        ShapeKind::Alt(arms) => arms.iter().any(|a| trailing_open(lang, a, praat_kinds)),
        ShapeKind::NodeRef(rule) => {
            !kind_set(lang, rule).is_disjoint(praat_kinds)
        }
        _ => false,
    }
}

fn is_optional_shape(shape: &Shape) -> bool {
    match &shape.kind {
        ShapeKind::Opt(_) | ShapeKind::Rep(_) => true,
        ShapeKind::Label { shape: inner, .. } => is_optional_shape(inner),
        _ => false,
    }
}

/// An atom shaped `'(' x:Rule ')'` (extra optional fields allowed):
/// returns its name and parameter plan for wrapping.
fn find_paren_atom<'l>(
    lang: &'l Language,
    praat: &Praat,
    praat_kinds: &std::collections::BTreeSet<String>,
) -> Option<(String, Vec<(bool, ParamKind)>)> {
    for (atom, _) in &praat.simple {
        let Some(rule) = lang.rules.get(atom) else { continue };
        let RuleBody::Plain(shape) = &rule.body else { continue };
        let ShapeKind::Seq(parts) = &shape.kind else { continue };
        if !matches!(parts.first().map(|p| &p.kind), Some(ShapeKind::Lit(l)) if l == "(") {
            continue;
        }
        if !matches!(parts.last().map(|p| &p.kind), Some(ShapeKind::Lit(l)) if l == ")") {
            continue;
        }
        if buildable_plan(shape).is_err() {
            continue;
        }
        let fields = struct_fields(lang, shape);
        let required: Vec<_> = fields.iter().filter(|f| f.required && !f.many).collect();
        if required.len() != 1 {
            continue;
        }
        let target_ok = match &required[0].target {
            crate::project::fields::FieldTarget::Node(rule) => {
                !kind_set(lang, rule).is_disjoint(praat_kinds)
                    || kind_set(lang, rule).contains(atom)
            }
            _ => false,
        };
        if !target_ok {
            continue;
        }
        let target_label = required[0].label.clone();
        let params = params_of(shape)
            .into_iter()
            .map(|(label, kind)| (label == target_label, kind))
            .collect();
        return Some((atom.clone(), params));
    }
    None
}
