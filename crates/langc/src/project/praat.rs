//! Praat row classification: the placement of `@bp` operands around token
//! groups draws the operator shape (design §2.3).
//!
//! Runtime convention (matches the plan): an infix/mixfix row binds while
//! `rbp > min_bp` and parses its right operand at `min_bp = lbp`, so
//! `lbp > rbp` is left-associative, `lbp < rbp` right-associative, and
//! `lbp == rbp` is rejected by check.

use crate::syntax::ast::{OpElem, OpRow};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RowKind {
    /// `'-' @100` — parsed in atom position.
    Prefix { toks: Vec<String>, rbp: u16 },
    /// `@80 '*' | '/' @79`
    Infix { lbp: u16, toks: Vec<String>, rbp: u16 },
    /// `@110 '!'` or, with a payload, `@110 '(' CallArgs ')'` — the tail
    /// is consumed in order after the lead tokens.
    Postfix { lbp: u16, tail: Vec<TailPart> },
    /// `@40 '?' @0 ':' @39` — after the head tokens, each `(bp, toks)`
    /// pair parses an inner operand then expects its tokens; the final
    /// operand parses at `lbp` (like an infix right side).
    Mixfix { lbp: u16, head: Vec<String>, inner: Vec<(u16, Vec<String>)>, rbp: u16 },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TailPart {
    Toks(Vec<String>),
    Node(String),
}

impl RowKind {
    /// The operator tokens that drive the Pratt loop (or atom dispatch
    /// for prefix rows).
    pub fn lead_toks(&self) -> &[String] {
        match self {
            RowKind::Prefix { toks, .. } | RowKind::Infix { toks, .. } => toks,
            RowKind::Postfix { tail, .. } => match tail.first() {
                Some(TailPart::Toks(toks)) => toks,
                _ => &[],
            },
            RowKind::Mixfix { head, .. } => head,
        }
    }
}

/// Classify one `operators` row by operand placement.
pub fn classify_row(row: &OpRow) -> Result<RowKind, String> {
    let e = &row.elems;
    match e.as_slice() {
        [OpElem::Toks(toks), OpElem::Operand(rbp)] => {
            Ok(RowKind::Prefix { toks: toks.clone(), rbp: *rbp })
        }
        // Postfix: `@lbp` then tokens/nodes with no further operand; must
        // lead with tokens so the Pratt loop can dispatch on them.
        [OpElem::Operand(lbp), OpElem::Toks(_), rest @ ..]
            if !rest.iter().any(|p| matches!(p, OpElem::Operand(_))) =>
        {
            let tail = e[1..]
                .iter()
                .map(|p| match p {
                    OpElem::Toks(toks) => TailPart::Toks(toks.clone()),
                    OpElem::Node(name) => TailPart::Node(name.clone()),
                    OpElem::Operand(_) => unreachable!("guarded above"),
                })
                .collect();
            Ok(RowKind::Postfix { lbp: *lbp, tail })
        }
        [OpElem::Operand(lbp), OpElem::Toks(toks), OpElem::Operand(rbp)] => {
            Ok(RowKind::Infix { lbp: *lbp, toks: toks.clone(), rbp: *rbp })
        }
        _ => {
            // Mixfix: Operand Toks (Operand Toks)+ Operand, strictly
            // alternating.
            if e.len() >= 5 && e.len() % 2 == 1 {
                let mut ok = true;
                for (i, elem) in e.iter().enumerate() {
                    ok &= if i % 2 == 0 {
                        matches!(elem, OpElem::Operand(_))
                    } else {
                        matches!(elem, OpElem::Toks(_))
                    };
                }
                if ok {
                    let OpElem::Operand(lbp) = e[0] else { unreachable!() };
                    let OpElem::Toks(head) = &e[1] else { unreachable!() };
                    let OpElem::Operand(rbp) = e[e.len() - 1] else { unreachable!() };
                    let mut inner = Vec::new();
                    let mut i = 2;
                    while i + 1 < e.len() {
                        let OpElem::Operand(bp) = e[i] else { unreachable!() };
                        let OpElem::Toks(toks) = &e[i + 1] else { unreachable!() };
                        inner.push((bp, toks.clone()));
                        i += 2;
                    }
                    return Ok(RowKind::Mixfix {
                        lbp,
                        head: head.clone(),
                        inner,
                        rbp,
                    });
                }
            }
            Err("operator row must be one of: prefix (`'-' @100`), infix (`@80 '*' @79`), \
                 postfix (`@110 '!'`), or mixfix (`@40 '?' @0 ':' @39`)"
                .to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use langue_rt::Span;

    fn row(elems: Vec<OpElem>) -> OpRow {
        OpRow { elems, span: Span::default() }
    }

    #[test]
    fn classifies_all_placements() {
        let toks = |s: &str| OpElem::Toks(vec![s.to_owned()]);
        assert_eq!(
            classify_row(&row(vec![toks("-"), OpElem::Operand(100)])),
            Ok(RowKind::Prefix { toks: vec!["-".into()], rbp: 100 })
        );
        assert_eq!(
            classify_row(&row(vec![OpElem::Operand(89), toks("**"), OpElem::Operand(90)])),
            Ok(RowKind::Infix { lbp: 89, toks: vec!["**".into()], rbp: 90 })
        );
        assert_eq!(
            classify_row(&row(vec![OpElem::Operand(110), toks("!")])),
            Ok(RowKind::Postfix { lbp: 110, tail: vec![TailPart::Toks(vec!["!".into()])] })
        );
        assert_eq!(
            classify_row(&row(vec![
                OpElem::Operand(110),
                toks("("),
                OpElem::Node("CallArgs".into()),
                toks(")"),
            ])),
            Ok(RowKind::Postfix {
                lbp: 110,
                tail: vec![
                    TailPart::Toks(vec!["(".into()]),
                    TailPart::Node("CallArgs".into()),
                    TailPart::Toks(vec![")".into()]),
                ],
            })
        );
        assert_eq!(
            classify_row(&row(vec![
                OpElem::Operand(40),
                toks("?"),
                OpElem::Operand(0),
                toks(":"),
                OpElem::Operand(39),
            ])),
            Ok(RowKind::Mixfix {
                lbp: 40,
                head: vec!["?".into()],
                inner: vec![(0, vec![":".into()])],
                rbp: 39,
            })
        );
        assert!(classify_row(&row(vec![toks("-")])).is_err());
        assert!(classify_row(&row(vec![toks("-"), toks("!"), OpElem::Operand(1)])).is_err());
    }
}
