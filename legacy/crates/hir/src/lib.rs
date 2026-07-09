pub mod check;
pub mod from_cst;
pub mod from_hir_cst;
pub mod lossless;
pub mod syntax_kind;
pub mod ast;
pub mod print;

pub use syntax_kind::SyntaxKind;
pub use lossless::{LosslessToken, SyntaxElement, SyntaxNode};

use lumo_span::Span;
use lumo_lst as lst;
use lumo_types::{CapRef, ContentHash, Pattern, Spanned, TypeExpr};

/// A generic parameter in a function declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenericParam {
    /// Type variable with optional bounds: `A`, `A: Add`, `A: Add + Eq`
    Type(String, Vec<String>),
    /// Capability row variable: `cap c`
    CapRow(String),
}

impl GenericParam {
    pub fn name(&self) -> &str {
        match self {
            GenericParam::Type(n, _) | GenericParam::CapRow(n) => n,
        }
    }
    pub fn bounds(&self) -> &[String] {
        match self {
            GenericParam::Type(_, b) => b,
            GenericParam::CapRow(_) => &[],
        }
    }
    pub fn is_cap_row(&self) -> bool {
        matches!(self, GenericParam::CapRow(_))
    }
}

// ---------------------------------------------------------------------------
// HIR types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub items: Vec<Item>,
    pub content_hash: ContentHash,
    pub errors: Vec<HirError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirError {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    ExternType(ExternTypeDecl),
    ExternFn(ExternFnDecl),
    Data(DataDecl),
    Cap(CapDecl),
    Fn(FnDecl),
    Use(UseDecl),
    Impl(ImplDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternTypeDecl {
    pub name: String,
    pub extern_name: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternFnDecl {
    pub name: String,
    pub extern_name: Option<String>,
    /// Module import via `#[link(module = "...")]` — (module, js_name).
    pub link_module: Option<(String, String)>,
    pub inline: bool,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub cap: Option<CapRef>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: Vec<VariantDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsRawValue {
    True,
    False,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantDecl {
    pub name: String,
    pub payload: Vec<Spanned<TypeExpr>>,
    pub as_raw: Option<AsRawValue>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapDecl {
    pub name: String,
    pub assoc_types: Vec<String>,
    pub operations: Vec<OperationDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub cap: Option<CapRef>,
    pub body: Expr,
    pub inline: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseDecl {
    pub path: Vec<String>,
    pub names: Option<Vec<String>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplDecl {
    pub name: Option<String>,
    pub generics: Vec<GenericParam>,
    pub target_type: Spanned<TypeExpr>,
    pub capability: Option<Spanned<TypeExpr>>,
    pub assoc_types: Vec<(String, Spanned<TypeExpr>)>,
    pub methods: Vec<ImplMethodDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplMethodDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Spanned<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Ident { name: String, span: Span },
    String { value: String, span: Span },
    Number { value: String, span: Span },
    Call { callee: Box<Expr>, args: Vec<Expr>, span: Span },
    Member { object: Box<Expr>, member: String, span: Span },
    Produce { expr: Box<Expr>, span: Span },
    Thunk { expr: Box<Expr>, span: Span },
    /// Anonymous function: `fn(x, y) { body }`
    Lambda { params: Vec<(String, Option<Spanned<TypeExpr>>)>, body: Box<Expr>, span: Span },
    Force { expr: Box<Expr>, span: Span },
    Let { name: String, value: Box<Expr>, body: Box<Expr>, span: Span },
    Match { scrutinee: Box<Expr>, arms: Vec<MatchArm>, span: Span },
    Perform { cap: String, span: Span },
    Handle { cap: String, type_args: Vec<String>, handler: Box<Expr>, body: Box<Expr>, span: Span },
    Bundle { entries: Vec<BundleEntry>, span: Span },
    Ann { expr: Box<Expr>, ty: Spanned<TypeExpr>, span: Span },
    Error { span: Span },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleEntry {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Expr,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Span accessor
// ---------------------------------------------------------------------------

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Ident { span, .. }
            | Expr::String { span, .. }
            | Expr::Number { span, .. }
            | Expr::Call { span, .. }
            | Expr::Member { span, .. }
            | Expr::Produce { span, .. }
            | Expr::Thunk { span, .. }
            | Expr::Lambda { span, .. }
            | Expr::Force { span, .. }
            | Expr::Let { span, .. }
            | Expr::Match { span, .. }
            | Expr::Perform { span, .. }
            | Expr::Handle { span, .. }
            | Expr::Bundle { span, .. }
            | Expr::Ann { span, .. }
            | Expr::Error { span } => *span,
        }
    }
}

// ---------------------------------------------------------------------------
// Lowering: LST → HIR
// ---------------------------------------------------------------------------

pub fn lower_lossless(parsed: &crate::lst::lossless::ParseOutput) -> File {
    from_cst::lower_from_cst(parsed)
}


pub fn merge_files(files: &[File]) -> File {
    let mut items = Vec::new();
    let mut errors = Vec::new();
    for file in files {
        items.extend(file.items.iter().cloned());
        errors.extend(file.errors.iter().cloned());
    }
    items = dedupe_data_with_as_raw(items);
    let content_hash = hash_file(&items);
    File {
        items,
        content_hash,
        errors,
    }
}

pub fn dedupe_data_with_as_raw(items: Vec<Item>) -> Vec<Item> {
    // For each data name, decide whether any of its decls carries `as_raw`.
    use std::collections::HashMap;
    let mut any_as_raw: HashMap<String, bool> = HashMap::new();
    for item in &items {
        if let Item::Data(d) = item {
            let has = d.variants.iter().any(|v| v.as_raw.is_some());
            let entry = any_as_raw.entry(d.name.clone()).or_insert(false);
            if has {
                *entry = true;
            }
        }
    }

    // Walk items in order. For data decls whose name has an as_raw variant
    // somewhere, keep only the decl that carries as_raw (drop the others).
    let mut seen_kept: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        if let Item::Data(ref d) = item {
            if let Some(true) = any_as_raw.get(&d.name).copied() {
                let this_has = d.variants.iter().any(|v| v.as_raw.is_some());
                if !this_has {
                    continue;
                }
                if !seen_kept.insert(d.name.clone()) {
                    continue;
                }
            }
        }
        out.push(item);
    }
    out
}

pub fn hash_file_pub(items: &[Item]) -> ContentHash {
    hash_file(items)
}

fn hash_file(items: &[Item]) -> ContentHash {
    let mut h = FnvHasher::new();
    h.write_tag("file");
    for item in items {
        hash_item(&mut h, item);
    }
    ContentHash(h.finish())
}

fn hash_item(h: &mut FnvHasher, item: &Item) {
    match item {
        Item::ExternType(ext) => {
            h.write_tag("extern-type");
            h.write_str(&ext.name);
            if let Some(name) = &ext.extern_name {
                h.write_str(name);
            }
        }
        Item::ExternFn(ext) => {
            h.write_tag("extern-fn");
            h.write_str(&ext.name);
            if let Some(name) = &ext.extern_name {
                h.write_str(name);
            }
            for param in &ext.params {
                h.write_str(&param.name);
                h.write_str(&param.ty.value.display());
            }
            if let Some(ret) = &ext.return_type {
                h.write_str(&ret.value.display());
            }
        }
        Item::Data(d) => {
            h.write_tag("data");
            h.write_str(&d.name);
            for v in &d.variants {
                h.write_str(&v.name);
                for ty in &v.payload {
                    h.write_str(&ty.value.display());
                }
            }
        }
        Item::Cap(c) => {
            h.write_tag("cap");
            h.write_str(&c.name);
            for op in &c.operations {
                h.write_str(&op.name);
            }
        }
        Item::Fn(f) => {
            h.write_tag("fn");
            h.write_str(&f.name);
            hash_expr(h, &f.body);
        }
        Item::Use(u) => {
            h.write_tag("use");
            for seg in &u.path {
                h.write_str(seg);
            }
            if let Some(names) = &u.names {
                for name in names {
                    h.write_str(name);
                }
            }
        }
        Item::Impl(i) => {
            h.write_tag("impl");
            h.write_str(&i.target_type.value.display());
            if let Some(cap) = &i.capability {
                h.write_str(&cap.value.display());
            }
            for m in &i.methods {
                h.write_str(&m.name);
                hash_expr(h, &m.body);
            }
        }
    }
}

fn hash_expr(h: &mut FnvHasher, expr: &Expr) {
    match expr {
        Expr::Ident { name, .. } => {
            h.write_tag("ident");
            h.write_str(name);
        }
        Expr::String { value, .. } => {
            h.write_tag("string");
            h.write_str(value);
        }
        Expr::Number { value, .. } => {
            h.write_tag("number");
            h.write_str(value);
        }
        Expr::Call { callee, args, .. } => {
            h.write_tag("call");
            hash_expr(h, callee);
            for arg in args {
                hash_expr(h, arg);
            }
        }
        Expr::Member { object, member, .. } => {
            h.write_tag("member");
            hash_expr(h, object);
            h.write_str(member);
        }
        Expr::Produce { expr, .. } => {
            h.write_tag("produce");
            hash_expr(h, expr);
        }
        Expr::Thunk { expr, .. } => {
            h.write_tag("thunk");
            hash_expr(h, expr);
        }
        Expr::Lambda { params, body, .. } => {
            h.write_tag("lambda");
            for (name, _) in params {
                h.write_str(name);
            }
            hash_expr(h, body);
        }
        Expr::Force { expr, .. } => {
            h.write_tag("force");
            hash_expr(h, expr);
        }
        Expr::Let { name, value, body, .. } => {
            h.write_tag("let");
            h.write_str(name);
            hash_expr(h, value);
            hash_expr(h, body);
        }
        Expr::Match { scrutinee, arms, .. } => {
            h.write_tag("match");
            hash_expr(h, scrutinee);
            for arm in arms {
                h.write_str(&arm.pattern.display());
                hash_expr(h, &arm.body);
            }
        }
        Expr::Perform { cap, .. } => {
            h.write_tag("perform");
            h.write_str(cap);
        }
        Expr::Handle { cap, handler, body, .. } => {
            h.write_tag("handle");
            h.write_str(cap);
            hash_expr(h, handler);
            hash_expr(h, body);
        }
        Expr::Bundle { entries, .. } => {
            h.write_tag("bundle");
            for e in entries {
                h.write_str(&e.name);
                hash_expr(h, &e.body);
            }
        }
        Expr::Ann { expr, ty, .. } => {
            h.write_tag("ann");
            h.write_str(&ty.value.display());
            hash_expr(h, expr);
        }
        Expr::Error { .. } => {
            h.write_tag("error");
        }
    }
}

struct FnvHasher {
    state: u64,
}

impl FnvHasher {
    fn new() -> Self {
        Self {
            state: 0xcbf29ce484222325,
        }
    }

    fn write_tag(&mut self, tag: &str) {
        self.write_str(tag);
        self.write_byte(0xff);
    }

    fn write_str(&mut self, value: &str) {
        self.write_u64(value.len() as u64);
        for b in value.as_bytes() {
            self.write_byte(*b);
        }
    }

    fn write_u64(&mut self, value: u64) {
        for b in value.to_le_bytes() {
            self.write_byte(b);
        }
    }

    fn write_byte(&mut self, value: u8) {
        self.state ^= value as u64;
        self.state = self.state.wrapping_mul(0x100000001b3);
    }

    fn finish(&self) -> u64 {
        self.state
    }
}
