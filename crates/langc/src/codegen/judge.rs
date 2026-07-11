//! Judgment codegen (M2 step 4, D-17/D-21): the merged `.type.langue`
//! judgments of one subject language compile to a `judgments.rs` module
//! holding a `langue_rt::judge` rule table plus the tree→term encoder.
//!
//! Term encoding (shared contract between `rules()` and `term_of`):
//! - a concrete node is `Struct(NodeName, fields)` with fields in
//!   `node_fields` order, praat `<op>` fields skipped;
//! - a token field is `Atom(text)`;
//! - a missing optional field is `Atom("#none")`; a list field is
//!   `Struct("#list", items)`.
//!
//! Rule heads are patterns — omitted fields become fresh variables
//! (D-35 omitted-is-wildcard); body terms are constructions — omitted
//! fields are `#none`/empty lists. A call used as an expression is
//! padded with fresh variables up to its declared arity and evaluates
//! to its last argument (D-17: `$x = (check_V $e <- $t)` runs the call
//! and unifies `$x` with `$t`).

use std::collections::BTreeMap;

use crate::project::fields::{kind_set, node_fields, Field, FieldTarget};
use crate::project::model::{Definition, JudgmentDef, Language};
use crate::syntax::ast::{BodyGoal, CallGoal, TermExpr};

use super::naming::kind_name;
use super::Buf;

/// The judgments whose declared subject is `lang_name`.
pub fn judgments_of<'d>(
    def: &'d Definition,
    lang_name: &str,
) -> BTreeMap<&'d str, &'d JudgmentDef> {
    def.judgments
        .iter()
        .filter(|(_, j)| j.subject_lang() == Some(lang_name))
        .map(|(n, j)| (n.as_str(), j))
        .collect()
}

pub fn generate(def: &Definition, lang_name: &str, lang: &Language) -> String {
    let judgments = judgments_of(def, lang_name);
    let g = Gen { def, lang };
    let mut buf = Buf::new();
    buf.line("#![allow(dead_code, clippy::all)]");
    buf.blank();
    buf.line("use langue_rt::{app, atom, var, Bail, Contexts, Derivation, Engine, Goal, Rule, Term};");
    buf.blank();
    buf.line("use super::lossless::SyntaxNode;");
    buf.line("use super::syntax_kind::SyntaxKind;");

    // Declared arities.
    buf.blank();
    buf.open("pub fn arity(judgment: &str) -> Option<usize> {");
    buf.open("match judgment {");
    for (name, judgment) in &judgments {
        if let Some(arity) = judgment.arity() {
            buf.line(&format!("{name:?} => Some({arity}),"));
        }
    }
    buf.line("_ => None,");
    buf.close("}");
    buf.close("}");

    // The rule table.
    buf.blank();
    buf.line(&format!("/// Every `{lang_name}` judgment rule, in definition order (D-17)."));
    buf.open("pub fn rules() -> Vec<Rule> {");
    buf.open("vec![");
    for (name, judgment) in &judgments {
        for rule in &judgment.rules {
            g.emit_rule(&mut buf, name, rule);
        }
    }
    buf.close("]");
    buf.close("}");

    g.emit_term_of(&mut buf);

    // The entry point: subject first, fresh variables for the
    // remaining (inout) parameters.
    buf.blank();
    buf.open("pub fn solve(judgment: &str, subject: &SyntaxNode, ctxs: Contexts) -> Result<Derivation, Bail> {");
    buf.open("let Some(arity) = arity(judgment) else {");
    buf.line("return Err(Bail { message: format!(\"unknown judgment `{judgment}` (D-17)\"), hard: true });");
    buf.close("};");
    buf.line("let mut args = vec![term_of(subject)];");
    buf.open("for i in 1..arity {");
    buf.line("args.push(var((i - 1) as u32));");
    buf.close("}");
    buf.line("Engine::new(rules()).solve(judgment, args, ctxs)");
    buf.close("}");

    buf.finish()
}

struct Gen<'d> {
    def: &'d Definition,
    lang: &'d Language,
}

/// Per-rule metavariable numbering: named metavars first-come, fresh
/// anonymous variables after.
#[derive(Default)]
struct VarAlloc {
    map: BTreeMap<String, u32>,
    next: u32,
}

impl VarAlloc {
    fn named(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.map.get(name) {
            return id;
        }
        let id = self.next;
        self.next += 1;
        self.map.insert(name.to_owned(), id);
        id
    }

    fn fresh(&mut self) -> u32 {
        let id = self.next;
        self.next += 1;
        id
    }
}

/// Head terms are patterns, body terms are constructions.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Head,
    Body,
}

impl Gen<'_> {
    fn emit_rule(&self, buf: &mut Buf, judgment: &str, rule: &crate::project::model::JudgmentRuleDef) {
        let mut alloc = VarAlloc::default();
        let mut goals: Vec<String> = Vec::new();
        // Lowering a head param may hoist goals (e.g. a call in head
        // position) — they run before the written body.
        let params: Vec<String> = rule
            .params
            .iter()
            .map(|p| self.lower_term(p, Mode::Head, &mut goals, &mut alloc))
            .collect();
        for goal in &rule.body {
            match goal {
                BodyGoal::Unify(a, b) => {
                    let a = self.lower_term(a, Mode::Body, &mut goals, &mut alloc);
                    let b = self.lower_term(b, Mode::Body, &mut goals, &mut alloc);
                    goals.push(format!("Goal::Unify({a}, {b})"));
                }
                BodyGoal::Call(call) => {
                    let code = self.lower_call(call, &mut goals, &mut alloc);
                    goals.push(code.goal);
                }
            }
        }
        buf.open("Rule {");
        buf.line(&format!("judgment: {judgment:?}.to_owned(),"));
        buf.line(&format!("params: vec![{}],", params.join(", ")));
        buf.line(&format!("var_count: {},", alloc.next));
        buf.open("body: vec![");
        for goal in &goals {
            buf.line(&format!("{goal},"));
        }
        buf.close("],");
        buf.close("},");
    }

    /// Lower a term expression to emitted `Term` code, hoisting calls
    /// and context reads into `goals`.
    fn lower_term(
        &self,
        term: &TermExpr,
        mode: Mode,
        goals: &mut Vec<String>,
        alloc: &mut VarAlloc,
    ) -> String {
        match term {
            TermExpr::Var { name, .. } => format!("var({})", alloc.named(name)),
            TermExpr::Lit { text, .. } => format!("atom({text:?})"),
            TermExpr::Subst { target, var, replacement, .. } => {
                let target = alloc.named(target);
                let needle = alloc.named(var);
                let replacement = alloc.named(replacement);
                let out = alloc.fresh();
                goals.push(format!(
                    "Goal::Subst {{ target: var({target}), needle: var({needle}), \
                     replacement: var({replacement}), out: var({out}) }}"
                ));
                format!("var({out})")
            }
            TermExpr::CtxRead { ctx, key, .. } => {
                let key = self.lower_term(key, mode, goals, alloc);
                let value = alloc.fresh();
                goals.push(format!(
                    "Goal::CtxRead {{ ctx: {ctx:?}.to_owned(), key: {key}, value: var({value}) }}"
                ));
                format!("var({value})")
            }
            TermExpr::Call(call) => {
                let code = self.lower_call(call, goals, alloc);
                goals.push(code.goal);
                code.value
            }
            TermExpr::Node { name, fields, .. } => {
                let table = node_fields(self.lang, name).expect("checked: concrete node");
                let mut args: Vec<String> = Vec::new();
                for field in &table {
                    if matches!(&field.target, FieldTarget::Token(t) if t == "<op>") {
                        continue;
                    }
                    let named = fields.iter().find(|(l, _)| l == &field.label);
                    let code = match (named, mode) {
                        (Some((_, sub)), _) => self.lower_term(sub, mode, goals, alloc),
                        (None, Mode::Head) => format!("var({})", alloc.fresh()),
                        (None, Mode::Body) if field.many => "app(\"#list\", vec![])".to_owned(),
                        (None, Mode::Body) => "atom(\"#none\")".to_owned(),
                    };
                    args.push(code);
                }
                format!("app({name:?}, vec![{}])", args.join(", "))
            }
        }
    }

    /// Lower a call: pad missing trailing arguments with fresh
    /// variables up to the declared arity; the call's value is its
    /// last argument.
    fn lower_call(
        &self,
        call: &CallGoal,
        goals: &mut Vec<String>,
        alloc: &mut VarAlloc,
    ) -> CallCode {
        // `(hash $list)` is the built-in row tactic (D-25), not a
        // judgment call.
        if call.judgment == "hash" {
            let mut args: Vec<String> = call
                .args
                .iter()
                .map(|a| self.lower_term(a, Mode::Body, goals, alloc))
                .collect();
            while args.len() < 2 {
                args.push(format!("var({})", alloc.fresh()));
            }
            let goal = format!("Goal::Hash {{ input: {}, out: {} }}", args[0], args[1]);
            return CallCode { goal, value: args[1].clone() };
        }
        let arity = self
            .def
            .judgments
            .get(&call.judgment)
            .and_then(|j| j.arity())
            .unwrap_or(call.args.len());
        let mut args: Vec<String> = call
            .args
            .iter()
            .map(|a| self.lower_term(a, Mode::Body, goals, alloc))
            .collect();
        while args.len() < arity {
            args.push(format!("var({})", alloc.fresh()));
        }
        let value = args.last().cloned().unwrap_or_else(|| format!("var({})", alloc.fresh()));
        let extends: Vec<String> = call
            .extends
            .iter()
            .map(|ext| {
                let key = self.lower_term(&ext.key, Mode::Body, goals, alloc);
                let val = self.lower_term(&ext.value, Mode::Body, goals, alloc);
                format!("({:?}.to_owned(), {key}, {val})", ext.ctx)
            })
            .collect();
        let goal = format!(
            "Goal::Call {{ judgment: {:?}.to_owned(), args: vec![{}], extends: vec![{}] }}",
            call.judgment,
            args.join(", "),
            extends.join(", ")
        );
        CallCode { goal, value }
    }

    /// `term_of`: the canonical tree→term encoding for every concrete
    /// node of the language.
    fn emit_term_of(&self, buf: &mut Buf) {
        let mut concrete = std::collections::BTreeSet::new();
        for rule_name in self.lang.rules.keys() {
            concrete.extend(kind_set(self.lang, rule_name));
        }
        buf.blank();
        buf.line("/// Encode a syntax node as a judgment term (see module docs).");
        buf.open("pub fn term_of(node: &SyntaxNode) -> Term {");
        buf.open("match node.kind {");
        for node in &concrete {
            let Some(fields) = node_fields(self.lang, node) else { continue };
            let mut args: Vec<String> = Vec::new();
            for field in &fields {
                if matches!(&field.target, FieldTarget::Token(t) if t == "<op>") {
                    continue;
                }
                args.push(self.field_term(field));
            }
            buf.line(&format!(
                "SyntaxKind::{} => app({node:?}, vec![{}]),",
                kind_name(node),
                args.join(", ")
            ));
        }
        buf.line("_ => atom(\"#error\"),");
        buf.close("}");
        buf.close("}");
    }

    fn field_term(&self, field: &Field) -> String {
        match &field.target {
            FieldTarget::Token(_) | FieldTarget::LitToken(_) => {
                let kind = match &field.target {
                    FieldTarget::Token(t) => kind_name(t),
                    FieldTarget::LitToken(text) => kind_name(
                        &self
                            .lang
                            .literal_token(text)
                            .expect("checked: literal token exists")
                            .name,
                    ),
                    FieldTarget::Node(_) => unreachable!(),
                };
                if field.many {
                    format!(
                        "app(\"#list\", langue_rt::tokens_of(node, SyntaxKind::{kind}, {})\
                         .into_iter().map(|t| atom(t.text.clone())).collect())",
                        field.skip
                    )
                } else {
                    format!(
                        "match langue_rt::nth_token_of(node, SyntaxKind::{kind}, {}) \
                         {{ Some(t) => atom(t.text.clone()), None => atom(\"#none\") }}",
                        field.skip
                    )
                }
            }
            FieldTarget::Node(rule) => {
                let kinds: Vec<String> = kind_set(self.lang, rule)
                    .iter()
                    .map(|k| format!("SyntaxKind::{}", kind_name(k)))
                    .collect();
                let kinds = format!("&[{}]", kinds.join(", "));
                if field.many {
                    format!(
                        "app(\"#list\", langue_rt::nodes_in(node, {kinds}, {})\
                         .into_iter().map(term_of).collect())",
                        field.skip
                    )
                } else {
                    format!(
                        "match langue_rt::nth_node_in(node, {kinds}, {}) \
                         {{ Some(n) => term_of(n), None => atom(\"#none\") }}",
                        field.skip
                    )
                }
            }
        }
    }
}

struct CallCode {
    goal: String,
    value: String,
}
