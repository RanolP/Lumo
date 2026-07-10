//! Elab engine runtime (M1 step 5). The generated per-pair modules do
//! the language-specific work (dispatch, matching, construction); this
//! module holds the language-agnostic pieces: target fragments with
//! pending sort-coercion binders (D-38), the elab context (fresh names,
//! errors), and the child-lookup helpers mirroring the M0 accessor
//! scheme (nth of a kind class, with a skip offset).

use crate::tree::{SyntaxNode, Token};

/// A rendered piece of the target language: its text, its root kind
/// when known (`None` for token splices), and coercion binders that
/// must be flushed at the nearest binder site (D-38).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Frag<K> {
    pub kind: Option<K>,
    pub text: String,
    /// `(fresh var, computation text)` pairs, outermost first.
    pub pending: Vec<(String, String)>,
}

impl<K> Frag<K> {
    pub fn node(kind: K, text: String) -> Frag<K> {
        Frag { kind: Some(kind), text, pending: Vec::new() }
    }

    pub fn token(text: String) -> Frag<K> {
        Frag { kind: None, text, pending: Vec::new() }
    }

    /// Move `other`'s pending binders onto this frag (child → parent).
    pub fn absorb<K2>(&mut self, other: &mut Frag<K2>) {
        self.pending.append(&mut other.pending);
    }
}

/// Per-invocation elaboration state: deterministic fresh names
/// (`__t1`, `__t2`, …) and accumulated errors.
#[derive(Default, Debug)]
pub struct ElabCtx {
    counter: u32,
    pub errors: Vec<String>,
}

impl ElabCtx {
    pub fn new() -> ElabCtx {
        ElabCtx::default()
    }

    pub fn fresh(&mut self) -> String {
        self.counter += 1;
        format!("__t{}", self.counter)
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
    }
}

/// Which side of an elab run an `extern pass` is offered (D-38): the
/// source text before parsing, or the target text after construction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PassPhase {
    PreSource,
    PostTarget,
}

/// What the registry hands the corpus harness for `:elab(A -> B)`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ElabReport {
    /// Canonical print of the reparsed target text.
    pub output: String,
    pub errors: Vec<String>,
}

/// `skip`-th child node whose kind is in `kinds` (the M0 accessor
/// occurrence scheme, generalized to transparent kind sets).
pub fn nth_node_in<'a, K: Copy + PartialEq>(
    node: &'a SyntaxNode<K>,
    kinds: &[K],
    skip: usize,
) -> Option<&'a SyntaxNode<K>> {
    node.child_nodes().filter(|n| kinds.contains(&n.kind)).nth(skip)
}

/// Child nodes in `kinds`, skipping the first `skip` (list captures).
pub fn nodes_in<'a, K: Copy + PartialEq>(
    node: &'a SyntaxNode<K>,
    kinds: &[K],
    skip: usize,
) -> Vec<&'a SyntaxNode<K>> {
    node.child_nodes().filter(|n| kinds.contains(&n.kind)).skip(skip).collect()
}

pub fn nth_token_of<'a, K: Copy + PartialEq>(
    node: &'a SyntaxNode<K>,
    kind: K,
    skip: usize,
) -> Option<&'a Token<K>> {
    node.child_tokens().filter(|t| t.kind == kind).nth(skip)
}

pub fn tokens_of<'a, K: Copy + PartialEq>(
    node: &'a SyntaxNode<K>,
    kind: K,
    skip: usize,
) -> Vec<&'a Token<K>> {
    node.child_tokens().filter(|t| t.kind == kind).skip(skip).collect()
}

/// The operator token of a praat row node: its first non-trivia token.
pub fn first_token<K: Copy>(
    node: &SyntaxNode<K>,
    is_trivia: fn(K) -> bool,
) -> Option<&Token<K>> {
    node.child_tokens().find(|t| !is_trivia(t.kind))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::SyntaxElement;
    use crate::Span;

    fn tok(kind: u8, text: &str) -> SyntaxElement<u8> {
        SyntaxElement::Token(Token { kind, text: text.to_owned(), span: Span::default() })
    }

    fn node(kind: u8, children: Vec<SyntaxElement<u8>>) -> SyntaxNode<u8> {
        SyntaxNode::from_children(kind, children)
    }

    #[test]
    fn nth_lookups_respect_class_and_skip() {
        let tree = node(
            0,
            vec![
                tok(1, "a"),
                SyntaxElement::Node(Box::new(node(2, vec![]))),
                tok(1, "b"),
                SyntaxElement::Node(Box::new(node(3, vec![]))),
                SyntaxElement::Node(Box::new(node(2, vec![]))),
            ],
        );
        assert_eq!(nth_node_in(&tree, &[2], 0).unwrap().kind, 2);
        assert_eq!(nth_node_in(&tree, &[2], 1).unwrap().kind, 2);
        assert!(nth_node_in(&tree, &[2], 2).is_none());
        assert_eq!(nth_node_in(&tree, &[2, 3], 1).unwrap().kind, 3);
        assert_eq!(nodes_in(&tree, &[2, 3], 1).len(), 2);
        assert_eq!(nth_token_of(&tree, 1, 1).unwrap().text, "b");
    }

    #[test]
    fn fresh_names_are_deterministic() {
        let mut ctx = ElabCtx::new();
        assert_eq!(ctx.fresh(), "__t1");
        assert_eq!(ctx.fresh(), "__t2");
        let mut ctx2 = ElabCtx::new();
        assert_eq!(ctx2.fresh(), "__t1");
    }

    #[test]
    fn frags_absorb_pending() {
        let mut child: Frag<u8> = Frag::node(1, "x".into());
        child.pending.push(("__t1".into(), "perform c".into()));
        let mut parent: Frag<u8> = Frag::node(2, "f x".into());
        parent.absorb(&mut child);
        assert_eq!(parent.pending.len(), 1);
        assert!(child.pending.is_empty());
    }
}
