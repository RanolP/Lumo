use derive_more::Debug;

use crate::hir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub items: Vec<Item>,
    #[debug(skip)]
    pub content_hash: ContentHash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    ExternType(ExternTypeDecl),
    ExternFn(ExternFnDecl),
    Data(DataDecl),
    Fn(FnDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternTypeDecl {
    #[debug(skip)]
    pub id: ContentHash,
    #[debug(skip)]
    pub structural_hash: ContentHash,
    pub name: String,
    pub extern_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternFnDecl {
    #[debug(skip)]
    pub id: ContentHash,
    #[debug(skip)]
    pub structural_hash: ContentHash,
    pub name: String,
    pub extern_name: Option<String>,
    pub params: Vec<ParamDecl>,
    pub return_type_repr: Option<String>,
    pub effect_repr: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDecl {
    pub name: String,
    pub generics: Vec<String>,
    #[debug(skip)]
    pub id: ContentHash,
    #[debug(skip)]
    pub structural_hash: ContentHash,
    pub variants: Vec<VariantDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantDecl {
    #[debug(skip)]
    pub id: ContentHash,
    #[debug(skip)]
    pub structural_hash: ContentHash,
    pub name: String,
    pub payload_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
    pub name: String,
    pub generics: Vec<String>,
    #[debug(skip)]
    pub id: ContentHash,
    #[debug(skip)]
    pub structural_hash: ContentHash,
    pub params: Vec<ParamDecl>,
    pub return_type_repr: Option<String>,
    pub effect_repr: Option<String>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamDecl {
    pub name: String,
    pub ty_repr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Ident {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        name: String,
    },
    String {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        value: String,
    },
    Produce {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        expr: Box<Expr>,
    },
    Thunk {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        expr: Box<Expr>,
    },
    Force {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        expr: Box<Expr>,
    },
    Unroll {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        expr: Box<Expr>,
    },
    LetIn {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    Match {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    Ctor {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        name: String,
        args: Vec<Expr>,
    },
    Roll {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
        expr: Box<Expr>,
    },
    Error {
        #[debug(skip)]
        id: ContentHash,
        #[debug(skip)]
        structural_hash: ContentHash,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: String,
    pub body: Expr,
}

pub fn lower(file: &hir::File) -> File {
    let variants = collect_variants(file);
    let items = file
        .items
        .iter()
        .map(|item| match item {
            hir::Item::ExternType(ext) => Item::ExternType(lower_extern_type(ext)),
            hir::Item::ExternFn(ext) => Item::ExternFn(lower_extern_fn(ext)),
            hir::Item::Data(data) => Item::Data(lower_data(data)),
            hir::Item::Fn(func) => Item::Fn(lower_fn(func, &variants)),
        })
        .collect::<Vec<_>>();

    let mut hasher = Hasher::new();
    hasher.write_tag("file");
    for item in &items {
        hasher.write_u64(item_id(item).0);
    }

    File {
        items,
        content_hash: ContentHash(hasher.finish()),
    }
}

fn lower_extern_type(ext: &hir::ExternTypeDecl) -> ExternTypeDecl {
    ExternTypeDecl {
        id: ContentHash(ext.id.0),
        structural_hash: ContentHash(ext.structural_hash.0),
        name: ext.name.clone(),
        extern_name: ext.extern_name.clone(),
    }
}

fn lower_extern_fn(ext: &hir::ExternFnDecl) -> ExternFnDecl {
    ExternFnDecl {
        id: ContentHash(ext.id.0),
        structural_hash: ContentHash(ext.structural_hash.0),
        name: ext.name.clone(),
        extern_name: ext.extern_name.clone(),
        params: ext
            .params
            .iter()
            .map(|param| ParamDecl {
                name: param.name.clone(),
                ty_repr: param.ty_repr.clone(),
            })
            .collect(),
        return_type_repr: ext.return_type_repr.clone(),
        effect_repr: ext.effect_repr.clone(),
    }
}

fn lower_data(data: &hir::DataDecl) -> DataDecl {
    DataDecl {
        name: data.name.clone(),
        generics: data.generics.clone(),
        id: ContentHash(data.id.0),
        structural_hash: ContentHash(data.structural_hash.0),
        variants: data
            .variants
            .iter()
            .map(|variant| VariantDecl {
                id: ContentHash(variant.id.0),
                structural_hash: ContentHash(variant.structural_hash.0),
                name: variant.name.clone(),
                payload_types: variant.payload_types.clone(),
            })
            .collect(),
    }
}

fn lower_fn(func: &hir::FnDecl, variants: &[(String, String)]) -> FnDecl {
    let body = lower_expr(&func.body, variants);
    let params = func
        .params
        .iter()
        .map(|param| ParamDecl {
            name: param.name.clone(),
            ty_repr: param.ty_repr.clone(),
        })
        .collect::<Vec<_>>();
    let mut hasher = Hasher::new();
    hasher.write_tag("fn");
    hasher.write_u64(expr_id(&body).0);
    let hash = ContentHash(hasher.finish());

    FnDecl {
        name: func.name.clone(),
        generics: func.generics.clone(),
        id: hash,
        structural_hash: hash,
        params,
        return_type_repr: func.return_type_repr.clone(),
        effect_repr: func.effect_repr.clone(),
        body,
    }
}

fn lower_expr(expr: &hir::Expr, variants: &[(String, String)]) -> Expr {
    match expr {
        hir::Expr::Ident { name, .. } => {
            let mut hasher = Hasher::new();
            hasher.write_tag("ident");
            hasher.write_str(name);
            let hash = ContentHash(hasher.finish());
            Expr::Ident {
                id: hash,
                structural_hash: hash,
                name: name.clone(),
            }
        }
        hir::Expr::String { value, .. } => {
            let mut hasher = Hasher::new();
            hasher.write_tag("string");
            hasher.write_str(value);
            let hash = ContentHash(hasher.finish());
            Expr::String {
                id: hash,
                structural_hash: hash,
                value: value.clone(),
            }
        }
        hir::Expr::Produce { expr, .. } => {
            let expr = Box::new(lower_expr(expr, variants));
            let mut hasher = Hasher::new();
            hasher.write_tag("produce");
            hasher.write_u64(expr_id(&expr).0);
            let hash = ContentHash(hasher.finish());
            Expr::Produce {
                id: hash,
                structural_hash: hash,
                expr,
            }
        }
        hir::Expr::Thunk { expr, .. } => {
            let expr = Box::new(lower_expr(expr, variants));
            let mut hasher = Hasher::new();
            hasher.write_tag("thunk");
            hasher.write_u64(expr_id(&expr).0);
            let hash = ContentHash(hasher.finish());
            Expr::Thunk {
                id: hash,
                structural_hash: hash,
                expr,
            }
        }
        hir::Expr::Force { expr, .. } => {
            let expr = Box::new(lower_expr(expr, variants));
            let mut hasher = Hasher::new();
            hasher.write_tag("force");
            hasher.write_u64(expr_id(&expr).0);
            let hash = ContentHash(hasher.finish());
            Expr::Force {
                id: hash,
                structural_hash: hash,
                expr,
            }
        }
        hir::Expr::LetIn {
            name, value, body, ..
        } => {
            let value = Box::new(lower_expr(value, variants));
            let body = Box::new(lower_expr(body, variants));
            let mut hasher = Hasher::new();
            hasher.write_tag("let-in");
            hasher.write_str(name);
            hasher.write_u64(expr_id(&value).0);
            hasher.write_u64(expr_id(&body).0);
            let hash = ContentHash(hasher.finish());
            Expr::LetIn {
                id: hash,
                structural_hash: hash,
                name: name.clone(),
                value,
                body,
            }
        }
        hir::Expr::Match {
            scrutinee, arms, ..
        } => {
            let lowered_scrutinee = lower_expr(scrutinee, variants);
            let scrutinee = Box::new(with_unroll(lowered_scrutinee));
            let arms = arms
                .iter()
                .map(|arm| MatchArm {
                    pattern: arm.pattern.clone(),
                    body: lower_expr(&arm.body, variants),
                })
                .collect::<Vec<_>>();
            let mut hasher = Hasher::new();
            hasher.write_tag("match");
            hasher.write_u64(expr_id(&scrutinee).0);
            for arm in &arms {
                hasher.write_str(&arm.pattern);
                hasher.write_u64(expr_id(&arm.body).0);
            }
            let hash = ContentHash(hasher.finish());
            Expr::Match {
                id: hash,
                structural_hash: hash,
                scrutinee,
                arms,
            }
        }
        hir::Expr::Member { object, member, .. } => {
            let hir::Expr::Ident { name: owner, .. } = object.as_ref() else {
                return error_expr();
            };
            let ctor = ctor_expr(&format!("{owner}.{member}"), Vec::new());
            if variants.iter().any(|(o, v)| o == owner && v == member) {
                with_roll(ctor)
            } else {
                ctor
            }
        }
        hir::Expr::Call { callee, args, .. } => {
            let args = args
                .iter()
                .map(|arg| lower_expr(arg, variants))
                .collect::<Vec<_>>();
            if let hir::Expr::Member { object, member, .. } = callee.as_ref() {
                if let hir::Expr::Ident { name: owner, .. } = object.as_ref() {
                    let ctor = ctor_expr(&format!("{owner}.{member}"), args);
                    if variants.iter().any(|(o, v)| o == owner && v == member) {
                        return with_roll(ctor);
                    }
                    return ctor;
                }
            }
            if let hir::Expr::Ident { name, .. } = callee.as_ref() {
                return ctor_expr(name, args);
            }
            error_expr()
        }
        hir::Expr::Error { .. } => error_expr(),
    }
}

fn with_unroll(expr: Expr) -> Expr {
    let expr = Box::new(expr);
    let mut hasher = Hasher::new();
    hasher.write_tag("unroll");
    hasher.write_u64(expr_id(&expr).0);
    let hash = ContentHash(hasher.finish());
    Expr::Unroll {
        id: hash,
        structural_hash: hash,
        expr,
    }
}

fn with_roll(expr: Expr) -> Expr {
    let expr = Box::new(expr);
    let mut hasher = Hasher::new();
    hasher.write_tag("roll");
    hasher.write_u64(expr_id(&expr).0);
    let hash = ContentHash(hasher.finish());
    Expr::Roll {
        id: hash,
        structural_hash: hash,
        expr,
    }
}

fn ctor_expr(name: &str, args: Vec<Expr>) -> Expr {
    let mut hasher = Hasher::new();
    hasher.write_tag("ctor");
    hasher.write_str(name);
    for arg in &args {
        hasher.write_u64(expr_id(arg).0);
    }
    let hash = ContentHash(hasher.finish());
    Expr::Ctor {
        id: hash,
        structural_hash: hash,
        name: name.to_owned(),
        args,
    }
}

fn error_expr() -> Expr {
    let mut hasher = Hasher::new();
    hasher.write_tag("error");
    let hash = ContentHash(hasher.finish());
    Expr::Error {
        id: hash,
        structural_hash: hash,
    }
}

fn collect_variants(file: &hir::File) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for item in &file.items {
        if let hir::Item::Data(d) = item {
            for v in &d.variants {
                out.push((d.name.clone(), v.name.clone()));
            }
        }
    }
    out
}

fn item_id(item: &Item) -> ContentHash {
    match item {
        Item::ExternType(ext) => ext.id,
        Item::ExternFn(ext) => ext.id,
        Item::Data(d) => d.id,
        Item::Fn(f) => f.id,
    }
}

fn expr_id(expr: &Expr) -> ContentHash {
    match expr {
        Expr::Ident { id, .. } => *id,
        Expr::String { id, .. } => *id,
        Expr::Produce { id, .. } => *id,
        Expr::Thunk { id, .. } => *id,
        Expr::Force { id, .. } => *id,
        Expr::Unroll { id, .. } => *id,
        Expr::LetIn { id, .. } => *id,
        Expr::Match { id, .. } => *id,
        Expr::Ctor { id, .. } => *id,
        Expr::Roll { id, .. } => *id,
        Expr::Error { id, .. } => *id,
    }
}

struct Hasher {
    state: u64,
}

impl Hasher {
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
