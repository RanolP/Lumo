/// Half-open byte range `[start, end)` into a source text.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        debug_assert!(start <= end);
        Span { start, end }
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// The smallest span covering both `self` and `other`.
    pub fn cover(&self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn slice<'t>(&self, text: &'t str) -> &'t str {
        &text[self.start as usize..self.end as usize]
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cover_expands_both_ends() {
        let a = Span::new(3, 5);
        let b = Span::new(1, 4);
        assert_eq!(a.cover(b), Span::new(1, 5));
    }

    #[test]
    fn slice_indexes_bytes() {
        let s = Span::new(3, 6);
        assert_eq!(s.slice("fn add()"), "add");
    }
}
