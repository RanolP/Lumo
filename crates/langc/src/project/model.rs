//! The immutable definition value — one per project revision (design §5.1).
//! Everything is BTree-ordered so codegen over it is byte-stable.

use std::collections::BTreeMap;

use langue_rt::Span;

use crate::syntax::ast::{BodyGoal, Con, Pat, RuleBody, Stage, TermExpr, TokenPattern};

/// Every generated parser starts at this rule; a language that appears in a
/// `parse` stage must declare it.
pub const START_RULE: &str = "File";

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Origin {
    pub file: String,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TokenDef {
    pub name: String,
    pub pattern: TokenPattern,
    pub is_trivia: bool,
    pub origin: Origin,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RuleDef {
    pub name: String,
    pub body: RuleBody,
    pub origin: Origin,
}

/// One declared language (D-03): the additive merge of every
/// `<Name>.*.syn.langue` file with the same first name segment.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Language {
    pub tokens: BTreeMap<String, TokenDef>,
    pub rules: BTreeMap<String, RuleDef>,
    /// Rules whose recovery hook is declared `extern recover` (D-01/D-02).
    pub extern_recovers: BTreeMap<String, Origin>,
}

impl Language {
    /// The token a grammar literal like `'fn'` refers to.
    pub fn literal_token(&self, text: &str) -> Option<&TokenDef> {
        self.tokens
            .values()
            .find(|t| matches!(&t.pattern, TokenPattern::Literal(l) if l == text))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PipelineDef {
    pub name: String,
    pub stages: Vec<Stage>,
    pub origin: Origin,
}

/// One elab rule with provenance (D-35).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ElabRuleDef {
    pub pattern: Pat,
    pub construction: Con,
    pub origin: Origin,
}

/// The merged `from A to B` definition: every same-pair block across all
/// files (D-05/D-13), plus the pair's extern rules (D-38).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ElabDef {
    pub rules: Vec<ElabRuleDef>,
    /// Declaration order preserved — extern rules dispatch first.
    pub extern_rules: Vec<(String, Origin)>,
}

/// One `lhs === rhs` relation with provenance (D-14).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RelationDef {
    pub lhs: Pat,
    pub rhs: Con,
    pub origin: Origin,
}

/// The merged `between L` group for one language (D-14).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct BetweenDef {
    pub relations: Vec<RelationDef>,
}

/// `context Γ = [Ident: TypeV]` (D-16).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ContextDef {
    pub name: String,
    pub key_sort: String,
    pub value_sort: String,
    pub origin: Origin,
}

/// One `head := body` rule with provenance (D-17).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct JudgmentRuleDef {
    pub params: Vec<TermExpr>,
    pub body: Vec<BodyGoal>,
    pub origin: Origin,
}

/// A judgment: its declaration plus every rule for it across all type
/// files (additive merge, D-05/D-17).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct JudgmentDef {
    /// `(sort names, origin)` — set by the declaration; `None` until
    /// (and unless) one is seen, which check reports.
    pub decl: Option<(Vec<String>, Origin)>,
    pub contexts: Vec<String>,
    pub rules: Vec<JudgmentRuleDef>,
}

impl JudgmentDef {
    /// The declared subject language (first declared sort).
    pub fn subject_lang(&self) -> Option<&str> {
        self.decl.as_ref().and_then(|(params, _)| params.first()).map(String::as_str)
    }

    pub fn arity(&self) -> Option<usize> {
        self.decl.as_ref().map(|(params, _)| params.len())
    }
}

/// The whole merged project. Top-level names (languages, pipelines) share
/// one global namespace (design §1.2); rule/token names live under their
/// language and are qualified as `Lang::Rule` elsewhere.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Definition {
    pub languages: BTreeMap<String, Language>,
    pub pipelines: BTreeMap<String, PipelineDef>,
    /// `from A to B` blocks merged by (from, to) pair (D-05/D-13).
    pub elabs: BTreeMap<(String, String), ElabDef>,
    /// `between L` groups merged by language (D-14).
    pub betweens: BTreeMap<String, BetweenDef>,
    /// `extern pass` names — global, applied by the Rust registration
    /// (D-38); declaration order preserved.
    pub extern_passes: Vec<(String, Origin)>,
    /// `context` declarations — one global namespace (D-16).
    pub contexts: BTreeMap<String, ContextDef>,
    /// Judgments by name — declarations + rules merged (D-17).
    pub judgments: BTreeMap<String, JudgmentDef>,
}
