//! The DFA-walk lexer engine (D-09). Generated lexers hand this module a
//! pattern list (all literals first, escaped, then regexes) and get raw
//! tokens back; the walk records the longest match, breaking ties by
//! smallest pattern index — so a literal beats a regex of equal length.

use regex_automata::dfa::dense;
use regex_automata::dfa::Automaton;
use regex_automata::{Anchored, Input, MatchKind};

use crate::Span;

/// One lexed region. `pattern` indexes the pattern list; `None` is a
/// 1-byte UNKNOWN region.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RawToken {
    pub pattern: Option<u32>,
    pub span: Span,
}

/// One dense, anchored, all-matches DFA over every token pattern of a
/// language. Built once per process by the generated lexer.
pub struct LexDfa {
    dfa: dense::DFA<Vec<u32>>,
}

impl LexDfa {
    /// Panics on invalid patterns — `langc check` validates the whole
    /// pattern table with the same backend before code is ever generated.
    pub fn build(patterns: &[&str]) -> LexDfa {
        Self::try_build(patterns).expect("token patterns were validated by `langc check`")
    }

    pub fn try_build(patterns: &[&str]) -> Result<LexDfa, String> {
        let dfa = dense::Builder::new()
            .configure(
                dense::Config::new()
                    .match_kind(MatchKind::All)
                    .start_kind(regex_automata::dfa::StartKind::Anchored),
            )
            .build_many(patterns)
            .map_err(|e| e.to_string())?;
        Ok(LexDfa { dfa })
    }

    /// Tokenize the whole text. Every byte lands in exactly one token;
    /// unmatched bytes become 1-byte UNKNOWN tokens.
    pub fn lex(&self, text: &str) -> Vec<RawToken> {
        let bytes = text.as_bytes();
        let mut out = Vec::new();
        let mut pos = 0;
        while pos < bytes.len() {
            match self.longest_match_at(bytes, pos) {
                Some((end, pattern)) => {
                    out.push(RawToken {
                        pattern: Some(pattern),
                        span: Span::new(pos as u32, end as u32),
                    });
                    pos = end;
                }
                None => {
                    out.push(RawToken {
                        pattern: None,
                        span: Span::new(pos as u32, pos as u32 + 1),
                    });
                    pos += 1;
                }
            }
        }
        out
    }

    /// Walk the DFA from `pos`, recording `(len, smallest pattern)` at
    /// every accepting state; the last record is the longest match.
    /// Dense DFA match states are delayed by one byte, so a match seen
    /// after consuming byte `i` ends at `i`.
    fn longest_match_at(&self, bytes: &[u8], pos: usize) -> Option<(usize, u32)> {
        let input = Input::new(bytes).anchored(Anchored::Yes).range(pos..);
        let mut state = self
            .dfa
            .start_state_forward(&input)
            .expect("anchored start state exists");
        let mut best: Option<(usize, u32)> = None;

        for (i, &byte) in bytes.iter().enumerate().skip(pos) {
            state = self.dfa.next_state(state, byte);
            if self.dfa.is_special_state(state) {
                if self.dfa.is_match_state(state) {
                    best = Some((i, self.smallest_pattern(state)));
                } else if self.dfa.is_dead_state(state) {
                    return best.filter(|(end, _)| *end > pos);
                }
            }
        }
        state = self.dfa.next_eoi_state(state);
        if self.dfa.is_match_state(state) {
            best = Some((bytes.len(), self.smallest_pattern(state)));
        }
        best.filter(|(end, _)| *end > pos)
    }

    fn smallest_pattern(&self, state: regex_automata::util::primitives::StateID) -> u32 {
        (0..self.dfa.match_len(state))
            .map(|j| self.dfa.match_pattern(state, j).as_u32())
            .min()
            .expect("match state has at least one pattern")
    }
}

/// Escape a token literal into a regex that matches it verbatim, so
/// literals and regexes share one DFA. Only true metacharacters are
/// escaped — a backslash before other punctuation can change meaning
/// instead of quoting it (`\<` is a word-boundary assertion).
pub fn regex_escape(literal: &str) -> String {
    const META: &[char] = &['\\', '.', '+', '*', '?', '(', ')', '|', '[', ']', '{', '}', '^', '$'];
    let mut out = String::with_capacity(literal.len() * 2);
    for c in literal.chars() {
        if META.contains(&c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mirrors the seed token table's interesting rows: literals first.
    fn dfa() -> LexDfa {
        LexDfa::build(&[
            &regex_escape("fn"),           // 0 literal
            &regex_escape("*"),            // 1 literal
            &regex_escape("**"),           // 2 literal
            "[a-zA-Z_][a-zA-Z0-9_]*",      // 3 regex ident
            "[0-9]+(\\.[0-9]+)?",          // 4 regex number
            "[ \\t\\r\\n]+",               // 5 regex whitespace
        ])
    }

    fn patterns(text: &str) -> Vec<(Option<u32>, &str)> {
        dfa()
            .lex(text)
            .into_iter()
            .map(|t| (t.pattern, t.span.slice(text)))
            .collect()
    }

    #[test]
    fn literal_beats_regex_on_tie() {
        // `fn` matches both the literal (0) and ident (3) at length 2;
        // the literal has the smaller index.
        assert_eq!(patterns("fn"), vec![(Some(0), "fn")]);
    }

    #[test]
    fn longest_match_wins() {
        // `fnord` is a longer ident match than the `fn` literal.
        assert_eq!(patterns("fnord"), vec![(Some(3), "fnord")]);
        // `**` beats two `*`s.
        assert_eq!(patterns("***"), vec![(Some(2), "**"), (Some(1), "*")]);
        // Number with fraction lexes as one token.
        assert_eq!(patterns("1.25"), vec![(Some(4), "1.25")]);
    }

    #[test]
    fn unknown_bytes_are_one_byte_each() {
        assert_eq!(
            patterns("a##b"),
            vec![(Some(3), "a"), (None, "#"), (None, "#"), (Some(3), "b")]
        );
    }

    #[test]
    fn eoi_match_and_whitespace() {
        assert_eq!(
            patterns("fn x"),
            vec![(Some(0), "fn"), (Some(5), " "), (Some(3), "x")]
        );
    }
}
