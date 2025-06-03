use std::{fmt::Debug, ops::Deref};

mod expression;
mod fragment;
mod item;
mod operator;

pub use expression::*;
pub use fragment::*;
pub use item::*;
pub use operator::*;

use crate::Spanned;

#[derive(Clone)]
pub struct WithId<T>(usize, T);

impl<T> WithId<Spanned<T>> {
    pub fn transpose(self) -> Spanned<WithId<T>> {
        self.1.map(|v| WithId(self.0, v))
    }
}

impl<T> WithId<T> {
    pub fn new(id: usize, value: T) -> Self {
        WithId(id, value)
    }

    pub fn id(&self) -> usize {
        self.0
    }

    pub fn map<U, F>(self, f: F) -> WithId<U>
    where
        F: FnOnce(T) -> U,
    {
        WithId(self.0, f(self.1))
    }

    pub fn inner(self) -> T {
        self.1
    }
}

impl<T> Debug for WithId<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.1)?;
        } else {
            write!(f, "{:?}", self.1)?;
        }
        write!(f, " <<{}>>", self.0)
    }
}

impl<T> WithId<Spanned<T>> {
    pub fn map_deep<U, F>(self, f: F) -> WithId<Spanned<U>>
    where
        F: FnOnce(T) -> U,
    {
        WithId(self.0, self.1.map(f))
    }
}

impl<T> Deref for WithId<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}
