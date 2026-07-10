//! AST for the `.langue` format. One `File` per source file; the project
//! model (D-05) cats files into a single namespace afterwards.

use langue_rt::Span;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct File {
    pub items: Vec<Item>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Item {
    /// `token keyword.fn = 'fn'` / `trivia comment.line = /…/`
    Token(TokenDecl),
    /// `Name = <shape>` or `Name = praat { … }`
    Rule(RuleDecl),
    /// `extern recover Expr` — names a Rust-side recovery hook (D-01/D-02).
    ExternRecover(ExternRecover),
    /// `main = parse Lumo | elab Lumo to MIR | …` (manifest files only, D-27/D-33)
    Pipeline(Pipeline),
    /// `from Lumo to MIR { pattern ==> construction … }` (elab files, D-13/D-35)
    ElabBlock(ElabBlock),
    /// `between MIR { lhs === rhs … }` (elab files, D-14)
    BetweenBlock(BetweenBlock),
    /// `extern rule member_classify from Lumo to MIR` (D-01/D-38)
    ExternRule(ExternRule),
    /// `extern pass scc_fix` (D-01/D-38)
    ExternPass(ExternPass),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TokenDecl {
    /// Dotted names double as highlight scopes (D-09).
    pub name: String,
    pub name_span: Span,
    pub pattern: TokenPattern,
    pub pattern_span: Span,
    pub is_trivia: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TokenPattern {
    /// `'fn'` — literals beat regexes on equal-length matches (D-09).
    Literal(String),
    /// `/[0-9]+/` — raw pattern handed to regex-automata.
    Regex(String),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RuleDecl {
    pub name: String,
    pub name_span: Span,
    pub body: RuleBody,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RuleBody {
    Plain(Shape),
    Praat(Praat),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Shape {
    pub kind: ShapeKind,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ShapeKind {
    /// `A B C`
    Seq(Vec<Shape>),
    /// `A | B | C`
    Alt(Vec<Shape>),
    /// `A?`
    Opt(Box<Shape>),
    /// `A*`
    Rep(Box<Shape>),
    /// `name:A` — labels become accessors (D-03).
    Label { label: String, shape: Box<Shape> },
    /// `'fn'` — a literal token by its text.
    Lit(String),
    /// `ident` — a named (regex) token; lowercase or dotted first segment.
    TokenRef(String),
    /// `FnDecl` — another rule; uppercase first segment.
    NodeRef(String),
    /// `sep(Param, ',')`
    Sep { item: Box<Shape>, sep: String },
}

impl Shape {
    pub fn new(kind: ShapeKind, span: Span) -> Self {
        Shape { kind, span }
    }
}

/// Whether a bare name in shape position refers to a rule (uppercase first
/// letter) or a token (anything else, incl. dotted names).
pub fn name_is_node_ref(name: &str) -> bool {
    !name.contains('.') && name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

/// `praat { simple = … operators { … } }`
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Praat {
    /// Atom alternatives: `simple = Lit | Ident | ParenExpr`.
    pub simple: Vec<(String, Span)>,
    pub rows: Vec<OpRow>,
}

/// One `operators` row, e.g. `@89 '**' @90` or `'+' | '-' @100`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OpRow {
    pub elems: Vec<OpElem>,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum OpElem {
    /// `@N` — an expression operand at binding power N.
    Operand(u16),
    /// `'+' | '-'` — token alternatives at this position.
    Toks(Vec<String>),
    /// `CallArgs` — a rule parsed in place; only valid in a postfix tail
    /// (`@110 '(' CallArgs ')'` — the call-expr form).
    Node(String),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExternRecover {
    /// The rule whose recovery is implemented (or defaulted) in Rust.
    pub rule: String,
    pub span: Span,
}

/// One named pipeline from a manifest file (D-33):
/// `main = parse Lumo | elab Lumo to MIR | check_V LIR`
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Pipeline {
    pub name: String,
    pub name_span: Span,
    pub stages: Vec<Stage>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Stage {
    pub kind: StageKind,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum StageKind {
    /// `parse Lumo`
    Parse { lang: String },
    /// `elab Lumo to MIR`
    Elab { from: String, to: String },
    /// `check_V LIR` — any declared judgment applied to a language.
    Judgment { judgment: String, lang: String },
}

impl Pipeline {
    /// The DCE root (D-05): the language of the first `parse` stage.
    pub fn root_language(&self) -> Option<&str> {
        self.stages.iter().find_map(|s| match &s.kind {
            StageKind::Parse { lang } => Some(lang.as_str()),
            _ => None,
        })
    }
}

// === elab items (D-35) ===

/// `from Lumo to MIR { rules }` — merged across files by (from, to).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ElabBlock {
    pub from: String,
    pub to: String,
    /// Span of the `from A to B` header.
    pub span: Span,
    pub rules: Vec<ElabRule>,
}

/// `pattern ==> construction`
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ElabRule {
    pub pattern: Pat,
    pub construction: Con,
    pub span: Span,
}

/// `between MIR { relations }` — merged across files by language.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BetweenBlock {
    pub lang: String,
    /// Span of the `between L` header.
    pub span: Span,
    pub relations: Vec<Relation>,
}

/// `lhs === rhs`
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Relation {
    pub lhs: Pat,
    pub rhs: Con,
    pub span: Span,
}

/// `extern rule member_classify from Lumo to MIR`
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExternRule {
    pub name: String,
    pub from: String,
    pub to: String,
    pub span: Span,
}

/// `extern pass scc_fix`
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExternPass {
    pub name: String,
    pub span: Span,
}

/// One side of an elab rule: what the source tree must look like.
/// Fields are matched by syn label; omitted fields match anything (D-35).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Pat {
    /// `FnDecl { name: $n, … }` / `Lumo::FnDecl` (bare name = no field
    /// constraints).
    Node { lang: Option<String>, name: String, fields: Vec<(String, Pat)>, span: Span },
    /// `$x`
    Var { name: String, span: Span },
    /// `[$x*]` — captures a labeled sep/rep field as a list.
    ListVar { name: String, span: Span },
    /// `'literal'` — a token's text.
    Lit { text: String, span: Span },
}

/// The other side: how to build the target tree.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Con {
    Node { lang: Option<String>, name: String, fields: Vec<(String, Con)>, span: Span },
    /// `$x` — splice a binding as-is.
    Var { name: String, span: Span },
    /// `$x to MIR` — recursive elaboration (strict subtree, D-28).
    VarTo { name: String, lang: String, span: Span },
    /// `[$x* to MIR]` — elementwise recursive elaboration of a list capture.
    ListVarTo { name: String, lang: String, span: Span },
    /// `$e[$b := $a]` — built-in subst (D-24).
    Subst { target: String, var: String, replacement: String, span: Span },
    /// `'literal'`
    Lit { text: String, span: Span },
}

impl Pat {
    pub fn span(&self) -> Span {
        match self {
            Pat::Node { span, .. }
            | Pat::Var { span, .. }
            | Pat::ListVar { span, .. }
            | Pat::Lit { span, .. } => *span,
        }
    }
}

impl Con {
    pub fn span(&self) -> Span {
        match self {
            Con::Node { span, .. }
            | Con::Var { span, .. }
            | Con::VarTo { span, .. }
            | Con::ListVarTo { span, .. }
            | Con::Subst { span, .. }
            | Con::Lit { span, .. } => *span,
        }
    }
}
