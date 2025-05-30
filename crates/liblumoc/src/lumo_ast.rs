use std::collections::HashMap;

use serde::Deserialize;

use crate::{LumoProductType, lumo_type::LumoFnType};

pub trait WithId {
    fn id(&self) -> usize;
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct LumoProgram {
    id: usize,
    items: Vec<LumoItem>,
}

impl WithId for LumoProgram {
    fn id(&self) -> usize {
        self.id
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub enum LumoItem {
    ExternFn {
        id: usize,
        name: String,
        ty: LumoFnType,
    },
    DefineEnum {
        id: usize,
        name: String,
        variants: Vec<(String, LumoProductType)>,
    },
    Expr(LumoExpr),
}

impl WithId for LumoItem {
    fn id(&self) -> usize {
        match self {
            LumoItem::ExternFn { id, .. } | LumoItem::DefineEnum { id, .. } => *id,
            LumoItem::Expr(expr) => expr.id(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub enum LumoExpr {
    Name {
        id: usize,
        content: String,
    },
    Call {
        id: usize,
        function: Box<LumoExpr>,
        parameters: Vec<LumoExpr>,
    },
    LitInteger {
        id: usize,
        content: String,
    },
    Match(LumoMatchExpr),
}

impl WithId for LumoExpr {
    fn id(&self) -> usize {
        match self {
            LumoExpr::Name { id, .. }
            | LumoExpr::Call { id, .. }
            | LumoExpr::LitInteger { id, .. } => *id,
            LumoExpr::Match(match_expr) => match_expr.id(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct LumoMatchExpr {
    id: usize,
    target: Box<LumoExpr>,
    match_arms: Vec<LumoMatchArm>,
}

impl WithId for LumoMatchExpr {
    fn id(&self) -> usize {
        self.id
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub enum LumoMatchArm {
    Discard {
        id: usize,
    },
    LetName {
        id: usize,
        content: String,
    },
    Destructure {
        id: usize,
        map: HashMap<String, Box<LumoMatchArm>>,
    },
    Equals {
        id: usize,
        target: Box<LumoExpr>,
    },
}

impl WithId for LumoMatchArm {
    fn id(&self) -> usize {
        use LumoMatchArm::*;
        match self {
            Discard { id, .. }
            | LetName { id, .. }
            | Destructure { id, .. }
            | Equals { id, .. } => *id,
        }
    }
}
