use lumo_lir as lir;
use lumo_span::Span;
use lumo_types::{CapRef, ContentHash, ExprId, Spanned, TypeExpr};
pub use lumo_hir::GenericParam;

// Re-export all lir types that do NOT contain Expr.
pub use lir::{
    AsRawValue, BundleEntry, CapDecl, DataDecl, ExternFnDecl, ExternTypeDecl,
    MatchArm, OperationDecl, Param, UseDecl, VariantDecl,
};

// ---------------------------------------------------------------------------
// Memory-aware expression type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// A pure sub-tree from the functional LIR — no RC operations inside.
    Pure(lir::Expr),

    /// Increment the refcount before this use (all non-consuming uses of a
    /// binding that is referenced N ≥ 2 times).
    Dup { id: ExprId, expr: Box<Expr> },

    /// Release `name` before evaluating `body` (binding used 0 times).
    Drop { id: ExprId, name: String, body: Box<Expr> },

    /// FBIP branch: unique ownership → `unique_branch`, shared → `shared_branch`.
    /// Not inserted by the elaboration pass; reserved for a future FBIP pass.
    IsUnique {
        id: ExprId,
        expr: Box<Expr>,
        unique_branch: Box<Expr>,
        shared_branch: Box<Expr>,
    },
}

// ---------------------------------------------------------------------------
// Types re-declared to swap in lir_memaware::Expr for lir::Expr
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub cap: Option<CapRef>,
    pub value: Expr,
    pub inline: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplMethodDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Spanned<TypeExpr>>,
    pub value: Expr,
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
pub enum Item {
    ExternType(lir::ExternTypeDecl),
    ExternFn(lir::ExternFnDecl),
    Data(lir::DataDecl),
    Cap(lir::CapDecl),
    Fn(FnDecl),
    Use(lir::UseDecl),
    Impl(ImplDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub items: Vec<Item>,
    pub content_hash: ContentHash,
    pub spans: Vec<Span>,
}
