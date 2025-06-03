use std::fmt::{self, Debug};

use crate::{IdentifierNode, Spanned, WithId};

pub enum RepresentationalType {
    Top,
    Bot,
    Union(Box<RepresentationalType>, Box<RepresentationalType>),
    Inter(Box<RepresentationalType>, Box<RepresentationalType>),
    Function(Vec<RepresentationalType>, Box<RepresentationalType>),
    Recursive(usize, Box<RepresentationalType>),
    Variable(usize),
    Primitive(String),
    VariantTag {
        root: WithId<Spanned<IdentifierNode>>,
        variant: WithId<Spanned<IdentifierNode>>,
    },
    Tuple(Vec<RepresentationalType>),
}

impl Debug for RepresentationalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Top => write!(f, "⊤"),
            Self::Bot => write!(f, "⊥"),
            Self::Union(l, r) => write!(f, "{l:?} ∪ {r:?}"),
            Self::Inter(l, r) => write!(f, "{l:?} ∩ {r:?}"),
            Self::Function(args, ret) => {
                let mut list = f.debug_tuple("fn");
                for arg in args {
                    list.field(arg);
                }
                list.finish()?;
                if args.is_empty() {
                    write!(f, "()")?;
                }
                write!(f, " -> {ret:?}")
            }
            Self::Recursive(id, ty) => {
                write!(f, "μ<#{id}>. ({ty:?})")
            }
            Self::Variable(id) => write!(f, "<#{id}>"),
            Self::Primitive(arg0) => f.debug_tuple("Primitive").field(arg0).finish(),
            Self::VariantTag { root, variant } => {
                write!(f, "{}.{}", root.0.content, variant.0.content)
            }
            Self::Tuple(types) => match &types[..] {
                [] => write!(f, "()"),
                [ty] => {
                    if f.alternate() {
                        write!(f, "({ty:#?},)")
                    } else {
                        write!(f, "({ty:?},)")
                    }
                }
                types => {
                    let mut tuple = f.debug_tuple("");
                    for ty in types {
                        tuple.field(ty);
                    }
                    tuple.finish()
                }
            },
        }
    }
}
