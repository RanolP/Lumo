use std::{
    fmt::{Debug, Display},
    ops::{Deref, Range},
};

use crate::WithId;

#[derive(Clone, Eq, Ord)]
pub struct Offset {
    pub offset: usize,
    pub line: usize,
    pub col: usize,
}

impl PartialEq for Offset {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
    }
}

impl PartialOrd for Offset {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.offset.partial_cmp(&other.offset)
    }
}

impl Debug for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(L{}:{})", self.offset, self.line, self.col)
    }
}

#[derive(Clone)]
pub struct Span(pub Range<Offset>);

impl Span {
    pub fn new(start: Offset, end: Offset) -> Self {
        Span(start..end)
    }

    pub fn start(&self) -> &Offset {
        &self.0.start
    }

    pub fn end(&self) -> &Offset {
        &self.0.end
    }

    pub fn merge(&self, other: &Self) -> Self {
        let start = self.0.start.clone().min(other.0.start.clone());
        let end = self.0.end.clone().max(other.0.end.clone());
        Span::new(start, end)
    }
}

#[derive(Clone)]
pub struct Spanned<T>(Span, T);

impl<T> Spanned<T> {
    pub fn new(span: Span, value: T) -> Self {
        Spanned(span, value)
    }

    pub fn span(&self) -> &Span {
        &self.0
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned(self.0, f(self.1))
    }
}

impl<T> Spanned<WithId<T>> {
    pub fn transpose(self) -> WithId<Spanned<T>> {
        self.1.map(|v| Spanned(self.0, v))
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<T> Debug for Spanned<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.1)?;
        } else {
            write!(f, "{:?}", self.1)?;
        }
        write!(f, " @ {}..{}", self.0.0.start.offset, self.0.0.end.offset)
    }
}

impl<T> Display for Spanned<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} @ {}..{}",
            self.1, self.0.0.start.offset, self.0.0.end.offset
        )
    }
}
