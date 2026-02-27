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
    Thunk {
        id: ContentHash,
        structural_hash: ContentHash,
        expr: Box<Expr>,
    },
    Force {
        id: ContentHash,
        structural_hash: ContentHash,
        expr: Box<Expr>,
    },
    Unroll {
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
    Match {
        id: ContentHash,
        structural_hash: ContentHash,
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    Ctor {
        id: ContentHash,
        structural_hash: ContentHash,
        name: String,
        args: Vec<Expr>,
    },
    Roll {
        id: ContentHash,
        structural_hash: ContentHash,
        expr: Box<Expr>,
    },
    Error {
        id: ContentHash,
        structural_hash: ContentHash,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: String,
    pub body: Expr,
}

pub fn lower_lossless(parsed: &lossless::ParseOutput) -> File {
    let variants = collect_variants_lossless(parsed);
    let mut items = Vec::new();

    for child in &parsed.root.children {
        let SyntaxElement::Node(node) = child else {
            continue;
        };

        match node.kind {
            SyntaxKind::DataDecl => items.push(Item::Data(lower_data_lossless(node))),
            SyntaxKind::FnDecl => items.push(Item::Fn(lower_fn_lossless(node, &variants))),
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
    let variants = collect_variants(file);
    let items = file
        .items
        .iter()
        .map(|item| match item {
            lst::Item::Data(data) => Item::Data(lower_data(data)),
            lst::Item::Fn(func) => Item::Fn(lower_fn(func, &variants)),
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

fn lower_fn(func: &lst::FnDecl, variants: &[(String, String)]) -> FnDecl {
    let body = lower_expr(&func.body, variants);

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

fn lower_expr(expr: &lst::Expr, variants: &[(String, String)]) -> Expr {
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
        lst::Expr::Thunk { expr, .. } => {
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
        lst::Expr::Force { expr, .. } => {
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
        lst::Expr::Match { scrutinee, arms, .. } => {
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
        lst::Expr::LetIn {
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
        lst::Expr::Apply {
            owner,
            member,
            args,
            ..
        } => {
            let args = args
                .iter()
                .map(|arg| lower_expr(arg, variants))
                .collect::<Vec<_>>();
            let ctor = ctor_expr(&format!("{owner}.{member}"), args);
            if variants.iter().any(|(o, v)| o == owner && v == member) {
                with_roll(ctor)
            } else {
                ctor
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
            LosslessTokenKind::Ident if in_body && prev_sig == "." => {
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

fn lower_fn_lossless(node: &SyntaxNode, variants: &[(String, String)]) -> FnDecl {
    let body_node = node.children.iter().find_map(|child| match child {
        SyntaxElement::Node(n) => Some(n.as_ref()),
        SyntaxElement::Token(_) => None,
    });

    let body = body_node
        .map(|n| lower_expr_lossless(n, variants))
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

fn lower_expr_lossless(node: &SyntaxNode, variants: &[(String, String)]) -> Expr {
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
                SyntaxElement::Node(n) => Some(lower_expr_lossless(n, variants)),
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
        SyntaxKind::ThunkExpr => {
            let payload = node.children.iter().find_map(|child| match child {
                SyntaxElement::Node(n) => Some(lower_expr_lossless(n, variants)),
                SyntaxElement::Token(_) => None,
            });
            let expr = Box::new(payload.unwrap_or_else(error_expr));
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
        SyntaxKind::ForceExpr => {
            let payload = node.children.iter().find_map(|child| match child {
                SyntaxElement::Node(n) => Some(lower_expr_lossless(n, variants)),
                SyntaxElement::Token(_) => None,
            });
            let expr = Box::new(payload.unwrap_or_else(error_expr));
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
        SyntaxKind::MatchExpr => {
            let patterns = find_match_patterns(node);
            let mut expr_nodes = node.children.iter().filter_map(|c| match c {
                SyntaxElement::Node(n) => Some(n.as_ref()),
                SyntaxElement::Token(_) => None,
            });
            let lowered_scrutinee = expr_nodes
                .next()
                .map(|n| lower_expr_lossless(n, variants))
                .unwrap_or_else(error_expr);
            let scrutinee = Box::new(with_unroll(lowered_scrutinee));
            let mut arms = Vec::new();
            for (index, body) in expr_nodes.enumerate() {
                arms.push(MatchArm {
                    pattern: patterns
                        .get(index)
                        .cloned()
                        .unwrap_or_else(|| "<arm>".to_owned()),
                    body: lower_expr_lossless(body, variants),
                });
            }

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
        SyntaxKind::LetExpr => {
            let name = find_let_name(node).unwrap_or_else(|| "<missing>".to_owned());
            let mut expr_nodes = node.children.iter().filter_map(|c| match c {
                SyntaxElement::Node(n) => Some(n.as_ref()),
                SyntaxElement::Token(_) => None,
            });
            let value = Box::new(
                expr_nodes
                    .next()
                    .map(|n| lower_expr_lossless(n, variants))
                    .unwrap_or_else(error_expr),
            );
            let body = Box::new(
                expr_nodes
                    .next()
                    .map(|n| lower_expr_lossless(n, variants))
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
        SyntaxKind::CallExpr => {
            let mut idents = flatten_tokens(node)
                .filter(|t| matches!(t.kind, LosslessTokenKind::Ident))
                .map(|t| t.text.clone());
            let owner = idents.next().unwrap_or_else(|| "<missing>".to_owned());
            let member = idents.next().unwrap_or_else(|| "<missing>".to_owned());
            let args = node
                .children
                .iter()
                .filter_map(|c| match c {
                    SyntaxElement::Node(n) => Some(lower_expr_lossless(n, variants)),
                    SyntaxElement::Token(_) => None,
                })
                .collect::<Vec<_>>();
            let ctor = ctor_expr(&format!("{owner}.{member}"), args);
            if variants.iter().any(|(o, v)| o == &owner && v == &member) {
                with_roll(ctor)
            } else {
                ctor
            }
        }
        _ => error_expr(),
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

fn find_match_patterns(node: &SyntaxNode) -> Vec<String> {
    let mut patterns = Vec::new();
    let mut in_arms = false;
    let mut collecting = false;
    let mut current = Vec::new();

    for child in &node.children {
        match child {
            SyntaxElement::Token(t) => {
                if t.text == "{" {
                    in_arms = true;
                    collecting = true;
                    current.clear();
                    continue;
                }
                if !in_arms {
                    continue;
                }
                if t.text == "=>" {
                    patterns.push(current.join(" "));
                    collecting = false;
                    current.clear();
                    continue;
                }
                if t.text == "," {
                    collecting = true;
                    current.clear();
                    continue;
                }
                if t.text == "}" {
                    break;
                }
                if collecting
                    && !matches!(
                        t.kind,
                        LosslessTokenKind::Whitespace | LosslessTokenKind::Newline
                    )
                {
                    current.push(t.text.clone());
                }
            }
            SyntaxElement::Node(_) => {}
        }
    }

    patterns
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

fn collect_variants(file: &lst::File) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for item in &file.items {
        if let lst::Item::Data(d) = item {
            for v in &d.variants {
                out.push((d.name.clone(), v.name.clone()));
            }
        }
    }
    out
}

fn collect_variants_lossless(parsed: &lossless::ParseOutput) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for child in &parsed.root.children {
        let SyntaxElement::Node(node) = child else {
            continue;
        };
        if node.kind != SyntaxKind::DataDecl {
            continue;
        }
        let data_name = flatten_tokens(node)
            .skip_while(|t| t.text != "data")
            .skip(1)
            .find(|t| matches!(t.kind, LosslessTokenKind::Ident))
            .map(|t| t.text.clone())
            .unwrap_or_else(|| "<missing>".to_owned());
        let mut in_body = false;
        let mut prev = "";
        for token in flatten_tokens(node) {
            match &token.kind {
                LosslessTokenKind::Symbol(_) if token.text == "{" => {
                    in_body = true;
                    prev = "{";
                }
                LosslessTokenKind::Symbol(_) if token.text == "}" => in_body = false,
                LosslessTokenKind::Whitespace | LosslessTokenKind::Newline => {}
                LosslessTokenKind::Ident if in_body && prev == "." => {
                    out.push((data_name.clone(), token.text.clone()));
                    prev = "ident";
                }
                _ => prev = token.text.as_str(),
            }
        }
    }
    out
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
