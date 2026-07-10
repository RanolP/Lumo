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
