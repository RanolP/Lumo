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
    /// `context Γ = [Ident: TypeV]` (type files, D-16)
    ContextDecl(ContextDecl),
    /// `infer_C MIR -> TypeC with Γ` (type files, D-17)
    JudgmentDecl(JudgmentDecl),
    /// `infer_V Ident { name: $n } -> $t := $t = Γ.$n` (type files, D-17)
    JudgmentRule(JudgmentRule),
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

// === type items (D-16/D-17/D-23) ===

/// `context Γ = [Ident: TypeV]` — a named multimap (D-16).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ContextDecl {
    pub name: String,
    pub name_span: Span,
    pub key_sort: String,
    pub value_sort: String,
    pub span: Span,
}

/// `infer_C MIR -> TypeC with Γ` — arrows are separators, both sides
/// are parameters (D-17). The first parameter names the subject
/// language; the rest are its node sorts.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct JudgmentDecl {
    pub name: String,
    pub name_span: Span,
    pub params: Vec<(String, Span)>,
    pub contexts: Vec<(String, Span)>,
    pub span: Span,
}

/// `head := body` — head params are patterns (omitted fields are
/// wildcards, D-35); body terms are constructions (omitted optional
/// fields are absent).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct JudgmentRule {
    pub judgment: String,
    pub judgment_span: Span,
    pub params: Vec<TermExpr>,
    pub body: Vec<BodyGoal>,
    pub span: Span,
}

/// A term of the judgment language — used on both sides of `=`, so
/// unlike elab there is no pattern/construction split in the grammar.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TermExpr {
    /// `$x`
    Var { name: String, span: Span },
    /// `NamedTypeV { name: 'Number' }` / bare `NumV`
    Node { name: String, fields: Vec<(String, TermExpr)>, span: Span },
    /// `'Number'` — a token's text.
    Lit { text: String, span: Span },
    /// `Γ.$name` (D-16).
    CtxRead { ctx: String, key: Box<TermExpr>, span: Span },
    /// `(check_V $e <- $t)` — as an expression its value is the last
    /// argument.
    Call(CallGoal),
    /// `$e[$b := $a]` — the built-in subst tactic (D-24).
    Subst { target: String, var: String, replacement: String, span: Span },
    /// `[]` / `[$h | $t]` — cons-cell lists (`#cons`/`#nil` terms).
    List { head: Option<Box<(TermExpr, TermExpr)>>, span: Span },
    /// `{ a, b | rest }` — a hash-keyed set, optionally open (D-25).
    SetExt { entries: Vec<TermExpr>, rest: Option<Box<TermExpr>>, span: Span },
    /// `Variant($o, $ps)` — a raw functor term (seed-shape contracts
    /// between the driver and the rules; not a syntax node).
    Apply { name: String, args: Vec<TermExpr>, span: Span },
}

/// A judgment call: `(check_V $e <- $t with Γ+{a: b})` or the bare
/// goal form `check_C $a $b with Γ+{a: b}` (D-23).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CallGoal {
    pub judgment: String,
    pub judgment_span: Span,
    pub args: Vec<TermExpr>,
    pub extends: Vec<CtxExt>,
    pub span: Span,
}

/// `Γ+{a: b}` on a call (D-23).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CtxExt {
    pub ctx: String,
    pub ctx_span: Span,
    pub key: TermExpr,
    pub value: TermExpr,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BodyGoal {
    /// `a = b`
    Unify(TermExpr, TermExpr),
    /// A call for its success alone.
    Call(CallGoal),
}

impl TermExpr {
    pub fn span(&self) -> Span {
        match self {
            TermExpr::Var { span, .. }
            | TermExpr::Node { span, .. }
            | TermExpr::Lit { span, .. }
            | TermExpr::CtxRead { span, .. }
            | TermExpr::Subst { span, .. }
            | TermExpr::List { span, .. }
            | TermExpr::SetExt { span, .. }
            | TermExpr::Apply { span, .. } => *span,
            TermExpr::Call(c) => c.span,
        }
    }
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
