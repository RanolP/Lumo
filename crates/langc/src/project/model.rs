//! The immutable definition value — one per project revision (design §5.1).
//! Everything is BTree-ordered so codegen over it is byte-stable.

use std::collections::BTreeMap;

use langue_rt::Span;

use crate::syntax::ast::{RuleBody, Stage, TokenPattern};

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

/// The whole merged project. Top-level names (languages, pipelines) share
/// one global namespace (design §1.2); rule/token names live under their
/// language and are qualified as `Lang::Rule` elsewhere.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Definition {
    pub languages: BTreeMap<String, Language>,
    pub pipelines: BTreeMap<String, PipelineDef>,
}
