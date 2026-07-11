//! Static checks over the merged judgment definitions (D-16/D-17/D-23):
//! judgments are declared before use, their subject language and node
//! sorts exist, rule heads match the declared arity, node patterns use
//! real fields, and every context named anywhere is declared.

use crate::diag::Diagnostic;
use crate::project::fields::node_fields;
use crate::project::model::{Definition, JudgmentDef, Language, Origin};
use crate::syntax::ast::{BodyGoal, CallGoal, TermExpr};

pub fn check_judgments(def: &Definition, diags: &mut Vec<Diagnostic>) {
    for (name, judgment) in &def.judgments {
        check_judgment(def, name, judgment, diags);
    }
}

fn check_judgment(
    def: &Definition,
    name: &str,
    judgment: &JudgmentDef,
    diags: &mut Vec<Diagnostic>,
) {
    let Some((params, decl_origin)) = &judgment.decl else {
        if let Some(rule) = judgment.rules.first() {
            diags.push(Diagnostic::error(
                &rule.origin.file,
                rule.origin.span,
                format!("judgment `{name}` has rules but no declaration (D-17)"),
            ));
        }
        return;
    };
    // The first declared sort is the subject language (D-17); the rest
    // are its node sorts (checked loosely — a sort may also name a
    // whole language).
    let Some(lang) = def.languages.get(&params[0]) else {
        diags.push(Diagnostic::error(
            &decl_origin.file,
            decl_origin.span,
            format!(
                "judgment `{name}`: unknown subject language `{}` (the first sort \
                 must name a language)",
                params[0]
            ),
        ));
        return;
    };
    for sort in &params[1..] {
        if !lang.rules.contains_key(sort) && !def.languages.contains_key(sort) {
            diags.push(Diagnostic::error(
                &decl_origin.file,
                decl_origin.span,
                format!("judgment `{name}`: unknown sort `{sort}`"),
            ));
        }
    }
    for ctx in &judgment.contexts {
        if !def.contexts.contains_key(ctx) {
            diags.push(Diagnostic::error(
                &decl_origin.file,
                decl_origin.span,
                format!("judgment `{name}`: undeclared context `{ctx}` (D-16)"),
            ));
        }
    }
    for rule in &judgment.rules {
        if rule.params.len() != params.len() {
            diags.push(Diagnostic::error(
                &rule.origin.file,
                rule.origin.span,
                format!(
                    "rule for `{name}` has {} parameters but the declaration has {}",
                    rule.params.len(),
                    params.len()
                ),
            ));
        }
        for param in &rule.params {
            check_term(def, lang, param, &rule.origin, diags);
        }
        for goal in &rule.body {
            match goal {
                BodyGoal::Unify(a, b) => {
                    check_term(def, lang, a, &rule.origin, diags);
                    check_term(def, lang, b, &rule.origin, diags);
                }
                BodyGoal::Call(call) => check_call(def, lang, call, &rule.origin, diags),
            }
        }
    }
}

fn check_call(
    def: &Definition,
    lang: &Language,
    call: &CallGoal,
    origin: &Origin,
    diags: &mut Vec<Diagnostic>,
) {
    // `hash` and `subset` are built-in row tactics (D-25).
    let arity = if call.judgment == "hash" || call.judgment == "subset" {
        Some(2)
    } else {
        def.judgments.get(&call.judgment).and_then(|j| j.arity())
    };
    match arity {
        None => diags.push(Diagnostic::error(
            &origin.file,
            call.judgment_span,
            format!("call to undeclared judgment `{}`", call.judgment),
        )),
        // Trailing arguments may be omitted — codegen pads them with
        // fresh variables and the call's value is the last one.
        Some(arity) if call.args.len() > arity => diags.push(Diagnostic::error(
            &origin.file,
            call.span,
            format!(
                "`{}` takes {} parameters but is called with {}",
                call.judgment,
                arity,
                call.args.len()
            ),
        )),
        Some(_) => {}
    }
    for arg in &call.args {
        check_term(def, lang, arg, origin, diags);
    }
    for ext in &call.extends {
        if !def.contexts.contains_key(&ext.ctx) {
            diags.push(Diagnostic::error(
                &origin.file,
                ext.ctx_span,
                format!("undeclared context `{}` (D-16)", ext.ctx),
            ));
        }
        check_term(def, lang, &ext.key, origin, diags);
        check_term(def, lang, &ext.value, origin, diags);
    }
}

fn check_term(
    def: &Definition,
    lang: &Language,
    term: &TermExpr,
    origin: &Origin,
    diags: &mut Vec<Diagnostic>,
) {
    match term {
        TermExpr::Var { .. } | TermExpr::Lit { .. } | TermExpr::Subst { .. } => {}
        TermExpr::List { head, .. } => {
            if let Some(pair) = head {
                check_term(def, lang, &pair.0, origin, diags);
                check_term(def, lang, &pair.1, origin, diags);
            }
        }
        TermExpr::SetExt { entries, rest, .. } => {
            for entry in entries {
                check_term(def, lang, entry, origin, diags);
            }
            if let Some(rest) = rest {
                check_term(def, lang, rest, origin, diags);
            }
        }
        // Raw functor terms are seed-shape contracts — args only.
        TermExpr::Apply { args, .. } => {
            for arg in args {
                check_term(def, lang, arg, origin, diags);
            }
        }
        TermExpr::CtxRead { ctx, key, span } => {
            if !def.contexts.contains_key(ctx) {
                diags.push(Diagnostic::error(
                    &origin.file,
                    *span,
                    format!("undeclared context `{ctx}` (D-16)"),
                ));
            }
            check_term(def, lang, key, origin, diags);
        }
        TermExpr::Call(call) => check_call(def, lang, call, origin, diags),
        TermExpr::Node { name, fields, span } => {
            let Some(node_fields) = node_fields(lang, name) else {
                diags.push(Diagnostic::error(
                    &origin.file,
                    *span,
                    format!("`{name}` is not a concrete node of the subject language"),
                ));
                return;
            };
            for (label, value) in fields {
                if !node_fields.iter().any(|f| f.label == *label) {
                    diags.push(Diagnostic::error(
                        &origin.file,
                        *span,
                        format!("node `{name}` has no field `{label}`"),
                    ));
                }
                check_term(def, lang, value, origin, diags);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::loader::{FileKind, LoadedFile};
    use crate::project::merge::merge_project;

    const SYN: &str = "\
token ident = /[a-z]+/
File = defs:NumV*
NumV = value:ident
";

    fn check(type_text: &str) -> Vec<String> {
        let (def, diags) = merge_project(&[
            LoadedFile {
                path: "L.syn.langue".into(),
                kind: FileKind::Syn { language: "L".into() },
                text: SYN.into(),
            },
            LoadedFile {
                path: "L.type.langue".into(),
                kind: FileKind::Type,
                text: type_text.into(),
            },
        ]);
        assert!(diags.is_empty(), "{diags:?}");
        let mut diags = Vec::new();
        check_judgments(&def, &mut diags);
        diags.into_iter().map(|d| d.message).collect()
    }

    #[test]
    fn clean_judgments_pass() {
        let msgs = check(
            "context Γ = [Ident: NumV]\n\
             infer L -> NumV with Γ\n\
             infer NumV { value: $v } -> $t := $t = Γ.$v\n",
        );
        assert!(msgs.is_empty(), "{msgs:?}");
    }

    #[test]
    fn missing_decl_arity_field_and_context_are_reported() {
        let msgs = check(
            "context Γ = [Ident: NumV]\n\
             infer L -> NumV with Γ\n\
             infer NumV -> $t := $t = Δ.$t, (other $t)\n\
             sized L -> L -> L\n\
             sized NumV { nope: $x } $a $b $c := $a = $b\n\
             undecl NumV -> $t := $t = 'x'\n",
        );
        assert!(msgs.iter().any(|m| m.contains("`undecl` has rules but no declaration")), "{msgs:?}");
        assert!(msgs.iter().any(|m| m.contains("undeclared context `Δ`")), "{msgs:?}");
        assert!(msgs.iter().any(|m| m.contains("undeclared judgment `other`")), "{msgs:?}");
        assert!(msgs.iter().any(|m| m.contains("no field `nope`")), "{msgs:?}");
        assert!(msgs.iter().any(|m| m.contains("4 parameters but the declaration has 3")), "{msgs:?}");
    }
}
