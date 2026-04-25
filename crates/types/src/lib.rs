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
    /// Capability type: `Add[Number]`, or bare `IO`
    Cap { name: String, type_args: Vec<TypeExpr> },
    /// Function type in value position: `fn(T, U): R` or `fn(T): R / { IO }`
    Fn { params: Vec<TypeExpr>, ret: Box<TypeExpr>, cap: Vec<CapEntry> },
    /// Iso-recursive type binder: `mu X. T`
    Mu { var: String, body: Box<TypeExpr> },
    /// Bound type variable inside a `mu` binder
    Var(String),
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
            TypeExpr::Cap { name, type_args } if !type_args.is_empty() => {
                let args_str = type_args.iter().map(|a| a.display()).collect::<Vec<_>>().join(", ");
                format!("{name}[{args_str}]")
            }
            TypeExpr::Cap { name, .. } => name.clone(),
            TypeExpr::Fn { params, ret, cap } => {
                let ps = params.iter().map(|p| p.display()).collect::<Vec<_>>().join(", ");
                let cap_str = if cap.is_empty() {
                    String::new()
                } else {
                    format!(" / {{{}}}", cap.iter().map(|e| e.display()).collect::<Vec<_>>().join(", "))
                };
                format!("fn({ps}): {}{cap_str}", ret.display())
            }
            TypeExpr::Mu { var, body } => format!("mu {var}. {}", body.display()),
            TypeExpr::Var(v) => v.clone(),
        }
    }

    /// The head name (for data type lookup).
    pub fn head_name(&self) -> &str {
        match self {
            TypeExpr::Named(n) | TypeExpr::Var(n) => n,
            TypeExpr::App { head, .. } => head,
            TypeExpr::Produce(inner) | TypeExpr::Thunk(inner) => inner.head_name(),
            TypeExpr::Cap { name, .. } => name,
            TypeExpr::Fn { ret, .. } => ret.head_name(),
            TypeExpr::Mu { body, .. } => body.head_name(),
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
            TypeExpr::Cap { name, type_args } => {
                name == target || type_args.iter().any(|a| a.references_name(target))
            }
            TypeExpr::Fn { params, ret, .. } => {
                params.iter().any(|p| p.references_name(target)) || ret.references_name(target)
            }
            TypeExpr::Mu { var, body } => var == target || body.references_name(target),
            TypeExpr::Var(v) => v == target,
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

    /// For Cap: returns the first type arg (the for_type) if present.
    pub fn cap_for_type(&self) -> Option<&TypeExpr> {
        match self {
            TypeExpr::Cap { type_args, .. } => type_args.first(),
            _ => None,
        }
    }

    /// For Cap: returns all type args.
    pub fn cap_type_args(&self) -> &[TypeExpr] {
        match self {
            TypeExpr::Cap { type_args, .. } => type_args,
            _ => &[],
        }
    }

    /// Runtime parameter name: `__cap_Add_Number` or `__cap_IO`.
    pub fn cap_mangled_param(&self) -> String {
        match self {
            TypeExpr::Cap { name, type_args } if !type_args.is_empty() => {
                let args = type_args.iter().map(|a| a.display()).collect::<Vec<_>>().join("_");
                format!("__cap_{name}_{args}")
            }
            TypeExpr::Cap { name, .. } => format!("__cap_{name}"),
            TypeExpr::Named(name) => format!("__cap_{name}"),
            _ => "__cap_unknown".to_string(),
        }
    }
}

/// Parse a type expression, handling the `thunk` and `fn` keywords.
fn parse_type_expr_full(text: &str) -> TypeExpr {
    let text = text.trim();
    // Handle both `fn(` and `fn (` (spaced form from LST token join)
    let fn_rest = text.strip_prefix("fn(")
        .or_else(|| text.strip_prefix("fn").and_then(|r| r.trim_start().strip_prefix('(')));
    if let Some(rest) = fn_rest {
        // fn(T, U): R / { cap } — function type in value position
        if let Some(paren_end) = find_close_paren(rest) {
            let params_str = &rest[..paren_end];
            let after_paren = rest[paren_end + 1..].trim_start();
            let after_paren = after_paren
                .strip_prefix("->").map(str::trim_start)
                .or_else(|| after_paren.strip_prefix(':').map(str::trim_start))
                .unwrap_or(after_paren);
            // Parse cap annotation if present
            let (ret_str, cap) = if let Some(slash_pos) = after_paren.find('/') {
                let ret_part = after_paren[..slash_pos].trim();
                let cap_part = after_paren[slash_pos + 1..].trim();
                (ret_part, parse_cap_ref(cap_part))
            } else {
                (after_paren.trim(), vec![])
            };
            let params: Vec<TypeExpr> = if params_str.trim().is_empty() {
                vec![]
            } else {
                split_type_args(params_str)
                    .into_iter()
                    .map(|p| parse_type_expr_full(&p))
                    .collect()
            };
            let ret = parse_type_expr_full(ret_str);
            return TypeExpr::Fn { params, ret: Box::new(ret), cap };
        }
    }
    if let Some(rest) = text.strip_prefix("mu ") {
        // mu X. T — iso-recursive type binder
        if let Some(dot_pos) = rest.find('.') {
            let var = rest[..dot_pos].trim().to_owned();
            let body_str = rest[dot_pos + 1..].trim();
            return TypeExpr::Mu { var, body: Box::new(parse_type_expr_full(body_str)) };
        }
    }
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

/// Find the position of the closing `)` matching the first `(`.
/// The text starts AFTER the opening `(`.
fn find_close_paren(text: &str) -> Option<usize> {
    let mut depth = 1usize;
    for (i, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
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
// CapEntry + CapRef — structured capability annotations
// ---------------------------------------------------------------------------

/// One entry in a capability annotation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CapEntry {
    /// Concrete cap: `IO`, `Add[Number]`
    Cap(TypeExpr),
    /// Named row variable spread: `..c`
    Spread(String),
    /// Anonymous inference marker: `..`
    Infer,
}

impl CapEntry {
    pub fn cap_mangled_param(&self) -> Option<String> {
        match self {
            CapEntry::Cap(ty) => Some(ty.cap_mangled_param()),
            CapEntry::Spread(_) | CapEntry::Infer => None,
        }
    }

    pub fn display(&self) -> String {
        match self {
            CapEntry::Cap(ty) => ty.display(),
            CapEntry::Spread(v) => format!("..{v}"),
            CapEntry::Infer => "..".to_owned(),
        }
    }
}

/// Cap annotation: empty vec = pure `/ {}`, non-empty = annotated.
/// `None` (via `Option<CapRef>` at use sites) means fully unannotated (inferred).
pub type CapRef = Vec<CapEntry>;

/// Parse a cap repr string into a `CapRef` (`Vec<CapEntry>`).
/// Handles: `{}`, `{ IO }`, `{ IO, Print }`, `{ .. }`, `{ .., IO }`,
///          `{ ..c }`, `{ IO, ..c }`.
pub fn parse_cap_ref(repr: &str) -> CapRef {
    let s = repr.trim();
    let s = s.strip_prefix('{').unwrap_or(s);
    let s = s.strip_suffix('}').unwrap_or(s);
    let s = s.trim();
    if s.is_empty() {
        return vec![];
    }
    let mut entries = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if part == ".." {
            entries.push(CapEntry::Infer);
        } else if let Some(var) = part.strip_prefix("..") {
            entries.push(CapEntry::Spread(var.trim().to_owned()));
        } else if let Some((name, ty)) = part.split_once(" for ") {
            if let Some(parsed) = TypeExpr::parse(ty.trim()) {
                entries.push(CapEntry::Cap(TypeExpr::Cap {
                    name: name.trim().to_owned(),
                    type_args: vec![parsed],
                }));
            }
        } else {
            entries.push(CapEntry::Cap(TypeExpr::Cap {
                name: part.to_owned(),
                type_args: vec![],
            }));
        }
    }
    entries
}

pub fn cap_ref_mangled_params(cap: &[CapEntry]) -> Vec<String> {
    cap.iter().filter_map(|e| e.cap_mangled_param()).collect()
}

pub fn cap_ref_is_effectful(cap: &[CapEntry]) -> bool {
    cap.iter().any(|e| matches!(e, CapEntry::Cap(_) | CapEntry::Spread(_)))
}

pub fn cap_ref_is_open(cap: &[CapEntry]) -> bool {
    cap.iter().any(|e| matches!(e, CapEntry::Infer | CapEntry::Spread(_)))
}

pub fn cap_ref_concrete_entries(cap: &[CapEntry]) -> Vec<&TypeExpr> {
    cap.iter()
        .filter_map(|e| if let CapEntry::Cap(ty) = e { Some(ty) } else { None })
        .collect()
}

pub fn cap_ref_display(cap: &[CapEntry]) -> String {
    if cap.is_empty() {
        return "{}".to_owned();
    }
    format!(
        "{{{}}}",
        cap.iter().map(|e| e.display()).collect::<Vec<_>>().join(", ")
    )
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

    fn bare_cap(name: &str) -> CapEntry {
        CapEntry::Cap(TypeExpr::Cap { name: name.to_owned(), type_args: vec![] })
    }

    fn typed_cap(name: &str, ty: &str) -> CapEntry {
        CapEntry::Cap(TypeExpr::Cap {
            name: name.to_owned(),
            type_args: vec![TypeExpr::Named(ty.to_owned())],
        })
    }

    #[test]
    fn cap_ref_parse_bare() {
        assert_eq!(parse_cap_ref("E"), vec![bare_cap("E")]);
        assert_eq!(parse_cap_ref("{ E }"), vec![bare_cap("E")]);
        assert_eq!(parse_cap_ref("{}"), vec![] as Vec<CapEntry>);
        assert_eq!(parse_cap_ref("{ }"), vec![] as Vec<CapEntry>);
        assert_eq!(parse_cap_ref("IO"), vec![bare_cap("IO")]);
        assert_eq!(
            parse_cap_ref("StrOps, NumOps"),
            vec![bare_cap("StrOps"), bare_cap("NumOps")]
        );
        assert_eq!(
            parse_cap_ref("{ A, B, C }"),
            vec![bare_cap("A"), bare_cap("B"), bare_cap("C")]
        );
    }

    #[test]
    fn cap_ref_parse_typed() {
        assert_eq!(
            parse_cap_ref("{ Add for Number }"),
            vec![typed_cap("Add", "Number")]
        );
        assert_eq!(
            parse_cap_ref("{ Add for String, IO }"),
            vec![typed_cap("Add", "String"), bare_cap("IO")]
        );
        assert_eq!(
            parse_cap_ref("{ Add for Number, Add for String, IO }"),
            vec![
                typed_cap("Add", "Number"),
                typed_cap("Add", "String"),
                bare_cap("IO"),
            ]
        );
    }

    #[test]
    fn cap_ref_parse_infer() {
        assert_eq!(parse_cap_ref("{ .. }"), vec![CapEntry::Infer]);
        assert_eq!(parse_cap_ref("{ .., IO }"), vec![CapEntry::Infer, bare_cap("IO")]);
    }

    #[test]
    fn cap_ref_parse_spread() {
        assert_eq!(parse_cap_ref("{ ..c }"), vec![CapEntry::Spread("c".into())]);
        assert_eq!(
            parse_cap_ref("{ IO, ..c }"),
            vec![bare_cap("IO"), CapEntry::Spread("c".into())]
        );
    }

    #[test]
    fn cap_entry_mangled_param() {
        assert_eq!(bare_cap("IO").cap_mangled_param(), Some("__cap_IO".into()));
        assert_eq!(typed_cap("Add", "Number").cap_mangled_param(), Some("__cap_Add_Number".into()));
        assert_eq!(CapEntry::Infer.cap_mangled_param(), None);
        assert_eq!(CapEntry::Spread("c".into()).cap_mangled_param(), None);
    }

    #[test]
    fn cap_entry_display() {
        assert_eq!(bare_cap("IO").display(), "IO");
        assert_eq!(typed_cap("Add", "Number").display(), "Add[Number]");
        assert_eq!(CapEntry::Infer.display(), "..");
        assert_eq!(CapEntry::Spread("c".into()).display(), "..c");
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
