use std::collections::HashMap;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum LumoType {
    Fn(LumoFnType),
    Product(LumoProductType),

    TypeVar(String),
}

impl LumoType {
    pub fn substitute(&self, name: &str, ty: &LumoType) -> Self {
        match self {
            Self::Fn(fn_type) => Self::Fn(fn_type.substitute(name, ty)),
            Self::Product(prod_type) => Self::Product(prod_type.substitute(name, ty)),
            Self::TypeVar(s) if s == name => ty.clone(),
            _ => self.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct LumoFnType {
    pub parameter_types: Vec<LumoType>,
    pub return_type: Box<LumoType>,
}

impl LumoFnType {
    pub fn substitute(&self, name: &str, ty: &LumoType) -> Self {
        let parameter_types = self
            .parameter_types
            .iter()
            .map(|t| t.substitute(name, ty))
            .collect();
        let return_type = Box::new(self.return_type.substitute(name, ty));
        Self {
            parameter_types,
            return_type,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum LumoProductType {
    Index(Vec<LumoType>),
    Named(HashMap<String, LumoType>),
}

impl LumoProductType {
    pub fn substitute(&self, name: &str, ty: &LumoType) -> Self {
        match self {
            Self::Index(types) => {
                Self::Index(types.iter().map(|t| t.substitute(name, ty)).collect())
            }
            Self::Named(fields) => Self::Named(
                fields
                    .iter()
                    .map(|(k, v)| (k.clone(), v.substitute(name, ty)))
                    .collect(),
            ),
        }
    }
}
