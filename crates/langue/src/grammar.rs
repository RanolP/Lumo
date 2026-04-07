/// Grammar IR — the in-memory representation of a `.langue` file.

#[derive(Debug, Clone)]
pub struct Grammar {
    /// Token names declared with `@token Ident StringLit ...`
    pub token_defs: Vec<String>,
    /// Node rules: `Name = ...`
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub body: RuleBody,
}

#[derive(Debug, Clone)]
pub enum RuleBody {
    /// `A B C` — a sequence of elements (generates a struct)
    Sequence(Vec<Element>),
    /// `| A | B | C` — alternatives (generates an enum)
    Alternatives(Vec<Alternative>),
}

#[derive(Debug, Clone)]
pub struct Alternative {
    /// The name of the referenced rule (e.g. "DataDecl" in `| DataDecl`)
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum Element {
    /// A token reference: `'data'`, `Ident`, `'('`
    Token(TokenRef),
    /// A node reference: `GenericParams`, `Variant`
    Node(NodeRef),
    /// A labeled child: `name:Ident`, `variants:Variant`
    Labeled(String, Box<Element>),
    /// Optional: `Element?`
    Optional(Box<Element>),
    /// Repeated: `Element*`
    Repeated(Box<Element>),
    /// Grouping: `(A B C)`
    Group(Vec<Element>),
}

#[derive(Debug, Clone)]
pub enum TokenRef {
    /// A keyword literal: `'data'`, `'fn'`, `'let'`
    Keyword(String),
    /// A punctuation/symbol literal: `'('`, `':='`, `'=>'`
    Symbol(String),
    /// A named token from `@token`: `Ident`, `StringLit`
    Named(String),
}

#[derive(Debug, Clone)]
pub struct NodeRef {
    pub name: String,
}

impl Grammar {
    /// Check whether a name was declared as a `@token`.
    pub fn is_token(&self, name: &str) -> bool {
        self.token_defs.iter().any(|t| t == name)
    }
}

impl TokenRef {
    /// Classify a quoted literal as keyword or symbol.
    /// Keywords must contain at least one alphabetic character.
    pub fn from_literal(text: &str) -> Self {
        if text.chars().any(|c| c.is_ascii_alphabetic())
            && text.chars().all(|c| c.is_ascii_alphabetic() || c == '_')
        {
            TokenRef::Keyword(text.to_owned())
        } else {
            TokenRef::Symbol(text.to_owned())
        }
    }
}
