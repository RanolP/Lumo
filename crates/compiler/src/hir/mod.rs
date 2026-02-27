use crate::{
    lexer::LosslessTokenKind,
    lst::{
        self,
        lossless::{self, SyntaxElement, SyntaxKind, SyntaxNode},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub items: Vec<Item>,
    pub content_hash: ContentHash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Data(DataDecl),
    Fn(FnDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDecl {
    pub id: ContentHash,
    pub structural_hash: ContentHash,
    pub variants: Vec<VariantDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantDecl {
    pub id: ContentHash,
    pub structural_hash: ContentHash,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
    pub id: ContentHash,
    pub structural_hash: ContentHash,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Ident {
        id: ContentHash,
        structural_hash: ContentHash,
        name: String,
    },
    Produce {
        id: ContentHash,
        structural_hash: ContentHash,
        expr: Box<Expr>,
    },
    LetIn {
        id: ContentHash,
        structural_hash: ContentHash,
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    Error {
        id: ContentHash,
        structural_hash: ContentHash,
    },
}

pub fn lower_lossless(parsed: &lossless::ParseOutput) -> File {
    let mut items = Vec::new();

    for child in &parsed.root.children {
        let SyntaxElement::Node(node) = child else {
            continue;
        };

        match node.kind {
            SyntaxKind::DataDecl => items.push(Item::Data(lower_data_lossless(node))),
            SyntaxKind::FnDecl => items.push(Item::Fn(lower_fn_lossless(node))),
            _ => {}
        }
    }

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

pub fn lower(file: &lst::File) -> File {
    let items = file
        .items
        .iter()
        .map(|item| match item {
            lst::Item::Data(data) => Item::Data(lower_data(data)),
            lst::Item::Fn(func) => Item::Fn(lower_fn(func)),
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

fn lower_data(data: &lst::DataDecl) -> DataDecl {
    let variants = data.variants.iter().map(lower_variant).collect::<Vec<_>>();

    let mut hasher = Hasher::new();
    hasher.write_tag("data");
    for variant in &variants {
        hasher.write_u64(variant.id.0);
    }
    let hash = ContentHash(hasher.finish());

    DataDecl {
        id: hash,
        structural_hash: hash,
        variants,
    }
}

fn lower_variant(variant: &lst::VariantDecl) -> VariantDecl {
    let mut hasher = Hasher::new();
    hasher.write_tag("variant");
    hasher.write_str(&variant.name);
    let hash = ContentHash(hasher.finish());

    VariantDecl {
        id: hash,
        structural_hash: hash,
        name: variant.name.clone(),
    }
}

fn lower_fn(func: &lst::FnDecl) -> FnDecl {
    let body = lower_expr(&func.body);

    let mut hasher = Hasher::new();
    hasher.write_tag("fn");
    hasher.write_u64(expr_id(&body).0);
    let hash = ContentHash(hasher.finish());

    FnDecl {
        id: hash,
        structural_hash: hash,
        body,
    }
}

fn lower_expr(expr: &lst::Expr) -> Expr {
    match expr {
        lst::Expr::Ident { name, .. } => {
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
        lst::Expr::Produce { expr, .. } => {
            let expr = Box::new(lower_expr(expr));
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
        lst::Expr::LetIn {
            name, value, body, ..
        } => {
            let value = Box::new(lower_expr(value));
            let body = Box::new(lower_expr(body));
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
        lst::Expr::Error { .. } => {
            let mut hasher = Hasher::new();
            hasher.write_tag("error");
            let hash = ContentHash(hasher.finish());
            Expr::Error {
                id: hash,
                structural_hash: hash,
            }
        }
    }
}

fn lower_data_lossless(node: &SyntaxNode) -> DataDecl {
    let mut variants = Vec::new();
    let mut in_body = false;
    let mut prev_sig = "";

    for token in flatten_tokens(node) {
        match &token.kind {
            LosslessTokenKind::Symbol(_) if token.text == "{" => {
                in_body = true;
                prev_sig = "{";
            }
            LosslessTokenKind::Symbol(_) if token.text == "}" => {
                in_body = false;
            }
            LosslessTokenKind::Whitespace | LosslessTokenKind::Newline => {}
            LosslessTokenKind::Ident if in_body && (prev_sig == "{" || prev_sig == ",") => {
                let mut hasher = Hasher::new();
                hasher.write_tag("variant");
                hasher.write_str(&token.text);
                let hash = ContentHash(hasher.finish());
                variants.push(VariantDecl {
                    id: hash,
                    structural_hash: hash,
                    name: token.text.clone(),
                });
                prev_sig = "ident";
            }
            _ => {
                prev_sig = token.text.as_str();
            }
        }
    }

    let mut hasher = Hasher::new();
    hasher.write_tag("data");
    for variant in &variants {
        hasher.write_u64(variant.id.0);
    }
    let hash = ContentHash(hasher.finish());

    DataDecl {
        id: hash,
        structural_hash: hash,
        variants,
    }
}

fn lower_fn_lossless(node: &SyntaxNode) -> FnDecl {
    let body_node = node.children.iter().find_map(|child| match child {
        SyntaxElement::Node(n) => Some(n.as_ref()),
        SyntaxElement::Token(_) => None,
    });

    let body = body_node
        .map(lower_expr_lossless)
        .unwrap_or_else(error_expr);

    let mut hasher = Hasher::new();
    hasher.write_tag("fn");
    hasher.write_u64(expr_id(&body).0);
    let hash = ContentHash(hasher.finish());

    FnDecl {
        id: hash,
        structural_hash: hash,
        body,
    }
}

fn lower_expr_lossless(node: &SyntaxNode) -> Expr {
    match node.kind {
        SyntaxKind::IdentExpr => {
            let name = flatten_tokens(node)
                .find(|t| matches!(t.kind, LosslessTokenKind::Ident))
                .map(|t| t.text.clone())
                .unwrap_or_else(|| "<missing>".to_owned());
            let mut hasher = Hasher::new();
            hasher.write_tag("ident");
            hasher.write_str(&name);
            let hash = ContentHash(hasher.finish());
            Expr::Ident {
                id: hash,
                structural_hash: hash,
                name,
            }
        }
        SyntaxKind::ProduceExpr => {
            let payload = node.children.iter().find_map(|child| match child {
                SyntaxElement::Node(n) => Some(lower_expr_lossless(n)),
                SyntaxElement::Token(_) => None,
            });
            let expr = Box::new(payload.unwrap_or_else(error_expr));
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
        SyntaxKind::LetExpr => {
            let name = find_let_name(node).unwrap_or_else(|| "<missing>".to_owned());
            let mut expr_nodes = node.children.iter().filter_map(|c| match c {
                SyntaxElement::Node(n) => Some(n.as_ref()),
                SyntaxElement::Token(_) => None,
            });
            let value = Box::new(
                expr_nodes
                    .next()
                    .map(lower_expr_lossless)
                    .unwrap_or_else(error_expr),
            );
            let body = Box::new(
                expr_nodes
                    .next()
                    .map(lower_expr_lossless)
                    .unwrap_or_else(error_expr),
            );

            let mut hasher = Hasher::new();
            hasher.write_tag("let-in");
            hasher.write_str(&name);
            hasher.write_u64(expr_id(&value).0);
            hasher.write_u64(expr_id(&body).0);
            let hash = ContentHash(hasher.finish());

            Expr::LetIn {
                id: hash,
                structural_hash: hash,
                name,
                value,
                body,
            }
        }
        _ => error_expr(),
    }
}

fn find_let_name(node: &SyntaxNode) -> Option<String> {
    let mut seen_let = false;
    for token in flatten_tokens(node) {
        match &token.kind {
            LosslessTokenKind::Keyword(_) if token.text == "let" => seen_let = true,
            LosslessTokenKind::Ident if seen_let => return Some(token.text.clone()),
            _ => {}
        }
    }
    None
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

fn flatten_tokens<'a>(
    node: &'a SyntaxNode,
) -> Box<dyn Iterator<Item = &'a crate::lexer::LosslessToken> + 'a> {
    let mut out = Vec::new();
    collect_tokens(node, &mut out);
    Box::new(out.into_iter())
}

fn collect_tokens<'a>(node: &'a SyntaxNode, out: &mut Vec<&'a crate::lexer::LosslessToken>) {
    for child in &node.children {
        match child {
            SyntaxElement::Node(n) => collect_tokens(n, out),
            SyntaxElement::Token(t) => out.push(t),
        }
    }
}

fn item_id(item: &Item) -> ContentHash {
    match item {
        Item::Data(d) => d.id,
        Item::Fn(f) => f.id,
    }
}

fn expr_id(expr: &Expr) -> ContentHash {
    match expr {
        Expr::Ident { id, .. } => *id,
        Expr::Produce { id, .. } => *id,
        Expr::LetIn { id, .. } => *id,
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
