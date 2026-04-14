use lumo_span::Span;

// ---------------------------------------------------------------------------
// Content hash (used for file-level caching in query engine)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash(pub u64);

// ---------------------------------------------------------------------------
// Expression ID (LIR only — indexes into File::spans side-table)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub u32);

// ---------------------------------------------------------------------------
// Spanned wrapper
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// TypeExpr — replaces all ty_repr: String
// ---------------------------------------------------------------------------

/// A source-level type expression. Parsed once during HIR lowering,
/// consumed by typechecker and backends without re-parsing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeExpr {
    /// Simple named type: `String`, `A`, `Unit`
    Named(String),
    /// Parameterized type: `List[A]`, `Option[Nat]`
    App { head: String, args: Vec<TypeExpr> },
    /// Computation type: `produce A` (HIR/LIR internal; not in surface syntax)
    Produce(Box<TypeExpr>),
    /// Thunked computation: `thunk T`
    Thunk(Box<TypeExpr>),
    /// Capability type: `Add for Number`, or bare `IO`
    Cap { name: String, for_type: Option<Box<TypeExpr>> },
}

impl TypeExpr {
    /// Parse from the text representation in `TypeSig.repr`.
    pub fn parse(repr: &str) -> Option<TypeExpr> {
        let text = repr.trim();
        if text.is_empty() {
            return None;
        }
        Some(parse_type_expr_full(text))
    }

    /// Canonical display form for error messages.
    pub fn display(&self) -> String {
        match self {
            TypeExpr::Named(n) => n.clone(),
            TypeExpr::App { head, args } => {
                let args_str = args
                    .iter()
                    .map(|a| a.display())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{head}[{args_str}]")
            }
            TypeExpr::Produce(inner) => format!("produce {}", inner.display()),
            TypeExpr::Thunk(inner) => format!("thunk {}", inner.display()),
            TypeExpr::Cap { name, for_type: Some(ty) } => format!("{name} for {}", ty.display()),
            TypeExpr::Cap { name, for_type: None } => name.clone(),
        }
    }

    /// The head name (for data type lookup).
    pub fn head_name(&self) -> &str {
        match self {
            TypeExpr::Named(n) => n,
            TypeExpr::App { head, .. } => head,
            TypeExpr::Produce(inner) | TypeExpr::Thunk(inner) => inner.head_name(),
            TypeExpr::Cap { name, .. } => name,
        }
    }

    /// Check if this type references a specific name anywhere.
    pub fn references_name(&self, target: &str) -> bool {
        match self {
            TypeExpr::Named(n) => n == target,
            TypeExpr::App { head, args } => {
                head == target || args.iter().any(|a| a.references_name(target))
            }
            TypeExpr::Produce(inner) | TypeExpr::Thunk(inner) => inner.references_name(target),
            TypeExpr::Cap { name, for_type } => {
                name == target || for_type.as_ref().map_or(false, |t| t.references_name(target))
            }
        }
    }

    // -----------------------------------------------------------------
    // Capability-specific helpers (for TypeExpr::Cap entries in CapRef)
    // -----------------------------------------------------------------

    /// For Cap: returns the capability name. For Named: returns the name.
    pub fn cap_name(&self) -> &str {
        match self {
            TypeExpr::Cap { name, .. } => name,
            TypeExpr::Named(name) => name,
            _ => "",
        }
    }

    /// For Cap: returns the for_type if present.
    pub fn cap_for_type(&self) -> Option<&TypeExpr> {
        match self {
            TypeExpr::Cap { for_type, .. } => for_type.as_deref(),
            _ => None,
        }
    }

    /// Mangled runtime parameter name: `__cap_Add_Number` or `__cap_IO`.
    pub fn cap_mangled_param(&self) -> String {
        match self {
            TypeExpr::Cap { name, for_type: Some(ty) } => format!("__cap_{}_{}", name, ty.display()),
            TypeExpr::Cap { name, for_type: None } => format!("__cap_{}", name),
            TypeExpr::Named(name) => format!("__cap_{}", name),
            _ => "__cap_unknown".to_string(),
        }
    }
}

/// Parse a type expression, handling the `thunk` keyword.
fn parse_type_expr_full(text: &str) -> TypeExpr {
    let text = text.trim();
    if let Some(rest) = text.strip_prefix("thunk") {
        let rest = rest.trim_start();
        if rest.is_empty() {
            return TypeExpr::Named("thunk".to_owned());
        }
        // Only treat as keyword if followed by whitespace (already stripped) or known prefix
        if rest.starts_with(|c: char| c.is_uppercase() || c.is_lowercase()) {
            return TypeExpr::Thunk(Box::new(parse_type_expr_full(rest)));
        }
        // fallthrough: treat "thunk" as a type name part
    }
    // No keywords — strip whitespace and parse as Named/App
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    parse_type_expr_compact(&compact)
}

fn parse_type_expr_compact(text: &str) -> TypeExpr {
    if let Some(bracket) = text.find('[') {
        let head = text[..bracket].to_owned();
        let inner = &text[bracket + 1..text.len().saturating_sub(1)];
        let args = split_type_args(inner)
            .into_iter()
            .map(|a| parse_type_expr_compact(&a))
            .collect();
        TypeExpr::App { head, args }
    } else {
        TypeExpr::Named(text.to_owned())
    }
}

fn split_type_args(text: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (i, ch) in text.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let s = text[start..i].trim();
                if !s.is_empty() {
                    args.push(s.to_owned());
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    let last = text[start..].trim();
    if !last.is_empty() {
        args.push(last.to_owned());
    }
    args
}

// ---------------------------------------------------------------------------
// CapRef — replaces cap_repr: Option<String>
// ---------------------------------------------------------------------------

/// A structured capability annotation.
/// Each entry is a `TypeExpr::Cap { name, for_type }`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CapRef {
    /// Pure function (no capability).
    Pure,
    /// Effectful function requiring one or more capabilities.
    /// Each element is a `TypeExpr::Cap`.
    Named(Vec<TypeExpr>),
}

impl CapRef {
    /// Parse from the repr string the parser produces.
    /// Supports: `"{ Add for Number, IO }"`, `"StrOps, NumOps"`, `"{ StrOps }"`.
    pub fn parse(repr: &str) -> CapRef {
        let s = repr.trim();
        let s = s.strip_prefix('{').unwrap_or(s);
        let s = s.strip_suffix('}').unwrap_or(s);
        let s = s.trim();
        if s.is_empty() {
            CapRef::Pure
        } else {
            let entries: Vec<TypeExpr> = s
                .split(',')
                .filter_map(|c| {
                    let c = c.trim();
                    if c.is_empty() {
                        return None;
                    }
                    if let Some((name, ty)) = c.split_once(" for ") {
                        Some(TypeExpr::Cap {
                            name: name.trim().to_owned(),
                            for_type: Some(Box::new(TypeExpr::parse(ty.trim())?)),
                        })
                    } else {
                        Some(TypeExpr::Cap {
                            name: c.to_owned(),
                            for_type: None,
                        })
                    }
                })
                .collect();
            if entries.is_empty() {
                CapRef::Pure
            } else {
                CapRef::Named(entries)
            }
        }
    }

    /// Returns the first capability name (for backward compatibility).
    pub fn name(&self) -> Option<&str> {
        match self {
            CapRef::Pure => None,
            CapRef::Named(entries) => entries.first().map(|e| e.cap_name()),
        }
    }

    /// Returns all capability names (without for_type info), deduplicated.
    pub fn cap_names(&self) -> Vec<&str> {
        match self {
            CapRef::Pure => vec![],
            CapRef::Named(entries) => {
                let mut names = Vec::new();
                for e in entries {
                    let name = e.cap_name();
                    if !names.contains(&name) {
                        names.push(name);
                    }
                }
                names
            }
        }
    }

    /// Returns all cap type entries.
    pub fn entries(&self) -> &[TypeExpr] {
        match self {
            CapRef::Pure => &[],
            CapRef::Named(entries) => entries,
        }
    }

    pub fn is_effectful(&self) -> bool {
        matches!(self, CapRef::Named(_))
    }
}

// ---------------------------------------------------------------------------
// Pattern — consolidates 3 duplicate parsers
// ---------------------------------------------------------------------------

/// A structured match pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    /// `_` — matches anything, binds nothing
    Wildcard,
    /// `x` — matches anything, binds to name
    Bind(String),
    /// `.variant` or `.variant(p1, p2)` — matches a data constructor
    Ctor { name: String, args: Vec<Pattern> },
}

impl Pattern {
    /// Parse from the string representation in MatchArm.
    pub fn parse(text: &str) -> Option<Pattern> {
        let mut parser = PatternParser::new(text);
        let pat = parser.parse_pattern()?;
        if parser.has_remaining() {
            None
        } else {
            Some(pat)
        }
    }

    /// Collect all variable bindings introduced by this pattern.
    pub fn bindings(&self) -> Vec<String> {
        let mut out = Vec::new();
        self.collect_bindings(&mut out);
        out
    }

    fn collect_bindings(&self, out: &mut Vec<String>) {
        match self {
            Pattern::Wildcard => {}
            Pattern::Bind(name) => out.push(name.clone()),
            Pattern::Ctor { args, .. } => {
                for arg in args {
                    arg.collect_bindings(out);
                }
            }
        }
    }

    /// Display form for error messages.
    pub fn display(&self) -> String {
        match self {
            Pattern::Wildcard => "_".to_owned(),
            Pattern::Bind(n) => n.clone(),
            Pattern::Ctor { name, args } if args.is_empty() => format!(".{name}"),
            Pattern::Ctor { name, args } => format!(
                ".{name}({})",
                args.iter()
                    .map(|p| p.display())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Pattern lexer + parser (consolidation of 3 identical implementations)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum PatternToken {
    Ident(String),
    Underscore,
    Dot,
    LParen,
    RParen,
    Comma,
}

struct PatternParser {
    tokens: Vec<PatternToken>,
    index: usize,
}

impl PatternParser {
    fn new(text: &str) -> Self {
        Self {
            tokens: lex_pattern(text),
            index: 0,
        }
    }

    fn has_remaining(&self) -> bool {
        self.index < self.tokens.len()
    }

    fn peek(&self) -> Option<&PatternToken> {
        self.tokens.get(self.index)
    }

    fn bump(&mut self) -> Option<PatternToken> {
        let out = self.tokens.get(self.index).cloned();
        if out.is_some() {
            self.index += 1;
        }
        out
    }

    fn parse_pattern(&mut self) -> Option<Pattern> {
        let token = self.bump()?;
        match token {
            PatternToken::Underscore => Some(Pattern::Wildcard),
            PatternToken::Dot => {
                let PatternToken::Ident(name) = self.bump()? else {
                    return None;
                };
                Some(self.parse_ctor(name))
            }
            PatternToken::Ident(head) => {
                if head == "let" || head == "mut" {
                    let PatternToken::Ident(name) = self.bump()? else {
                        return None;
                    };
                    if is_binding_name(&name) {
                        Some(Pattern::Bind(name))
                    } else {
                        None
                    }
                } else if self.peek() == Some(&PatternToken::Dot) {
                    // Type.variant(args) form
                    self.bump(); // consume dot
                    let PatternToken::Ident(variant) = self.bump()? else {
                        return None;
                    };
                    let full_name = format!("{head}.{variant}");
                    Some(self.parse_ctor(full_name))
                } else if is_binding_name(&head) {
                    Some(Pattern::Bind(head))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_ctor(&mut self, name: String) -> Pattern {
        if self.peek() == Some(&PatternToken::LParen) {
            self.bump(); // consume (
            let mut args = Vec::new();
            if self.peek() != Some(&PatternToken::RParen) {
                loop {
                    if let Some(arg) = self.parse_pattern() {
                        args.push(arg);
                    } else {
                        break;
                    }
                    if self.peek() == Some(&PatternToken::Comma) {
                        self.bump();
                        continue;
                    }
                    break;
                }
            }
            if self.peek() == Some(&PatternToken::RParen) {
                self.bump();
            }
            Pattern::Ctor { name, args }
        } else {
            Pattern::Ctor {
                name,
                args: Vec::new(),
            }
        }
    }
}

fn is_binding_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_alphabetic()) && chars.all(|c| c == '_' || c.is_alphanumeric())
}

fn lex_pattern(text: &str) -> Vec<PatternToken> {
    let mut out = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '_' => {
                chars.next();
                // Standalone wildcard or start of an identifier like `_foo`?
                if chars.peek().map_or(true, |c| !c.is_alphanumeric() && *c != '_') {
                    out.push(PatternToken::Underscore);
                } else {
                    let mut s = String::from('_');
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            s.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    out.push(PatternToken::Ident(s));
                }
            }
            '.' => {
                chars.next();
                out.push(PatternToken::Dot);
            }
            '(' => {
                chars.next();
                out.push(PatternToken::LParen);
            }
            ')' => {
                chars.next();
                out.push(PatternToken::RParen);
            }
            ',' => {
                chars.next();
                out.push(PatternToken::Comma);
            }
            _ if ch.is_alphabetic() => {
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        s.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                out.push(PatternToken::Ident(s));
            }
            _ => {
                chars.next();
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_expr_parse_named() {
        assert_eq!(TypeExpr::parse("String"), Some(TypeExpr::Named("String".into())));
        assert_eq!(TypeExpr::parse("A"), Some(TypeExpr::Named("A".into())));
        assert_eq!(TypeExpr::parse("  Unit  "), Some(TypeExpr::Named("Unit".into())));
        assert_eq!(TypeExpr::parse(""), None);
    }

    #[test]
    fn type_expr_parse_app() {
        assert_eq!(
            TypeExpr::parse("List[A]"),
            Some(TypeExpr::App {
                head: "List".into(),
                args: vec![TypeExpr::Named("A".into())]
            })
        );
        assert_eq!(
            TypeExpr::parse("Map[String, Number]"),
            Some(TypeExpr::App {
                head: "Map".into(),
                args: vec![
                    TypeExpr::Named("String".into()),
                    TypeExpr::Named("Number".into()),
                ]
            })
        );
    }

    #[test]
    fn type_expr_parse_nested() {
        assert_eq!(
            TypeExpr::parse("List[Option[A]]"),
            Some(TypeExpr::App {
                head: "List".into(),
                args: vec![TypeExpr::App {
                    head: "Option".into(),
                    args: vec![TypeExpr::Named("A".into())]
                }]
            })
        );
    }

    #[test]
    fn type_expr_display() {
        let ty = TypeExpr::App {
            head: "List".into(),
            args: vec![TypeExpr::Named("A".into())],
        };
        assert_eq!(ty.display(), "List[A]");
    }

    #[test]
    fn type_expr_references_name() {
        let ty = TypeExpr::App {
            head: "List".into(),
            args: vec![TypeExpr::Named("A".into())],
        };
        assert!(ty.references_name("List"));
        assert!(ty.references_name("A"));
        assert!(!ty.references_name("B"));
    }

    fn bare_cap(name: &str) -> TypeExpr {
        TypeExpr::Cap { name: name.to_owned(), for_type: None }
    }

    fn typed_cap(name: &str, ty: &str) -> TypeExpr {
        TypeExpr::Cap { name: name.to_owned(), for_type: Some(Box::new(TypeExpr::Named(ty.to_owned()))) }
    }

    #[test]
    fn cap_ref_parse_bare() {
        assert_eq!(CapRef::parse("E"), CapRef::Named(vec![bare_cap("E")]));
        assert_eq!(CapRef::parse("{ E }"), CapRef::Named(vec![bare_cap("E")]));
        assert_eq!(CapRef::parse("{}"), CapRef::Pure);
        assert_eq!(CapRef::parse("{ }"), CapRef::Pure);
        assert_eq!(CapRef::parse("IO"), CapRef::Named(vec![bare_cap("IO")]));
        assert_eq!(
            CapRef::parse("StrOps, NumOps"),
            CapRef::Named(vec![bare_cap("StrOps"), bare_cap("NumOps")])
        );
        assert_eq!(
            CapRef::parse("{ A, B, C }"),
            CapRef::Named(vec![bare_cap("A"), bare_cap("B"), bare_cap("C")])
        );
    }

    #[test]
    fn cap_ref_parse_typed() {
        assert_eq!(
            CapRef::parse("{ Add for Number }"),
            CapRef::Named(vec![typed_cap("Add", "Number")])
        );
        assert_eq!(
            CapRef::parse("{ Add for String, IO }"),
            CapRef::Named(vec![typed_cap("Add", "String"), bare_cap("IO")])
        );
        assert_eq!(
            CapRef::parse("{ Add for Number, Add for String, IO }"),
            CapRef::Named(vec![
                typed_cap("Add", "Number"),
                typed_cap("Add", "String"),
                bare_cap("IO"),
            ])
        );
    }

    #[test]
    fn cap_type_expr_mangled_param() {
        assert_eq!(bare_cap("IO").cap_mangled_param(), "__cap_IO");
        assert_eq!(typed_cap("Add", "Number").cap_mangled_param(), "__cap_Add_Number");
    }

    #[test]
    fn cap_type_expr_display() {
        assert_eq!(bare_cap("IO").display(), "IO");
        assert_eq!(typed_cap("Add", "Number").display(), "Add for Number");
    }

    #[test]
    fn pattern_parse_wildcard() {
        assert_eq!(Pattern::parse("_"), Some(Pattern::Wildcard));
    }

    #[test]
    fn pattern_parse_bind() {
        assert_eq!(Pattern::parse("x"), Some(Pattern::Bind("x".into())));
        assert_eq!(Pattern::parse("let x"), Some(Pattern::Bind("x".into())));
    }

    #[test]
    fn pattern_parse_ctor_no_args() {
        assert_eq!(
            Pattern::parse(".true"),
            Some(Pattern::Ctor {
                name: "true".into(),
                args: vec![],
            })
        );
        assert_eq!(
            Pattern::parse(".none"),
            Some(Pattern::Ctor {
                name: "none".into(),
                args: vec![],
            })
        );
    }

    #[test]
    fn pattern_parse_ctor_with_args() {
        assert_eq!(
            Pattern::parse(".cons(x, xs)"),
            Some(Pattern::Ctor {
                name: "cons".into(),
                args: vec![
                    Pattern::Bind("x".into()),
                    Pattern::Bind("xs".into()),
                ],
            })
        );
    }

    #[test]
    fn pattern_parse_nested() {
        assert_eq!(
            Pattern::parse(".some(.pair(x, _))"),
            Some(Pattern::Ctor {
                name: "some".into(),
                args: vec![Pattern::Ctor {
                    name: "pair".into(),
                    args: vec![
                        Pattern::Bind("x".into()),
                        Pattern::Wildcard,
                    ],
                }],
            })
        );
    }

    #[test]
    fn pattern_bindings() {
        let pat = Pattern::parse(".cons(x, .cons(y, _))").unwrap();
        assert_eq!(pat.bindings(), vec!["x", "y"]);
    }

    #[test]
    fn pattern_display() {
        let pat = Pattern::Ctor {
            name: "cons".into(),
            args: vec![
                Pattern::Bind("x".into()),
                Pattern::Wildcard,
            ],
        };
        assert_eq!(pat.display(), ".cons(x, _)");
    }
}
