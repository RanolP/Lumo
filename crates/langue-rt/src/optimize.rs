//! egglog execution for `between` groups (M3, D-42): load the compiled
//! program, define a root, saturate (`(run N)`), extract the lowest-cost
//! term, and loop through a caller-supplied host-side tactic reduction
//! (`subst` — egglog has no capture-avoiding substitution) until the
//! extracted term is tactic-free.

use egglog::ast::Literal;
use egglog::{CommandOutput, EGraph, Term, TermDag, TermId};

/// Owned term walked out of egglog's `TermDag`. Grammar-datatype fields
/// are strings (tokens) or nested applications (nodes / `vec-of` lists),
/// so those are the only two shapes.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EggTerm {
    Str(String),
    App(String, Vec<EggTerm>),
}

impl EggTerm {
    pub fn app(head: impl Into<String>, args: Vec<EggTerm>) -> EggTerm {
        EggTerm::App(head.into(), args)
    }

    pub fn str(text: impl Into<String>) -> EggTerm {
        EggTerm::Str(text.into())
    }

    /// Render back to egglog expression text.
    pub fn to_sexpr(&self) -> String {
        match self {
            EggTerm::Str(s) => format!("{s:?}"),
            EggTerm::App(head, args) if args.is_empty() => format!("({head})"),
            EggTerm::App(head, args) => {
                let args: Vec<String> = args.iter().map(EggTerm::to_sexpr).collect();
                format!("({head} {})", args.join(" "))
            }
        }
    }

    /// Does any node in this term have the given head?
    pub fn contains_app(&self, name: &str) -> bool {
        match self {
            EggTerm::Str(_) => false,
            EggTerm::App(head, args) => {
                head == name || args.iter().any(|a| a.contains_app(name))
            }
        }
    }
}

/// An e-graph loaded with one compiled `between` program and a single
/// root under optimization.
pub struct Optimizer {
    egraph: EGraph,
}

impl Optimizer {
    pub fn new(program: &str) -> Result<Optimizer, String> {
        let mut egraph = EGraph::default();
        egraph
            .parse_and_run_program(None, program)
            .map_err(|e| format!("egglog: program rejected: {e}"))?;
        Ok(Optimizer { egraph })
    }

    fn run_text(&mut self, text: &str) -> Result<Vec<CommandOutput>, String> {
        self.egraph
            .parse_and_run_program(None, text)
            .map_err(|e| format!("egglog: {e}"))
    }

    pub fn define_root(&mut self, expr: &str) -> Result<(), String> {
        self.run_text(&format!("(let root {expr})")).map(drop)
    }

    /// Saturate: one bounded `(run N)`.
    pub fn run(&mut self, iterations: usize) -> Result<(), String> {
        self.run_text(&format!("(run {iterations})")).map(drop)
    }

    /// Lowest-cost term of the root's e-class.
    pub fn extract_root(&mut self) -> Result<EggTerm, String> {
        let outputs = self.run_text("(extract root)")?;
        for output in outputs {
            if let CommandOutput::ExtractBest(dag, _cost, id) = output {
                return walk(&dag, id);
            }
        }
        Err("egglog: (extract root) produced no ExtractBest output".to_owned())
    }

    /// Merge a host-computed equal term into the root's e-class.
    pub fn union_root(&mut self, term: &EggTerm) -> Result<(), String> {
        self.run_text(&format!("(union root {})", term.to_sexpr())).map(drop)
    }
}

fn walk(dag: &TermDag, id: TermId) -> Result<EggTerm, String> {
    match dag.get(id) {
        Term::Lit(Literal::String(s)) => Ok(EggTerm::Str(s.clone())),
        Term::Lit(other) => Err(format!("egglog: non-string literal {other:?} in extraction")),
        Term::Var(v) => Err(format!("egglog: free variable `{v}` in extraction")),
        Term::App(head, children) => {
            let args = children
                .iter()
                .map(|c| walk(dag, *c))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(EggTerm::App(head.clone(), args))
        }
    }
}

/// The D-42 loop: saturate, extract, reduce host-side tactics, union the
/// reduction back, repeat. `reduce` returns `None` when the term has no
/// tactic nodes left (it is the final answer) and `Some(reduced)` after
/// rewriting at least one. Leftover tactics past `max_rounds` error.
pub fn optimize_loop(
    program: &str,
    root_expr: &str,
    run_iterations: usize,
    max_rounds: usize,
    reduce: impl Fn(&EggTerm) -> Result<Option<EggTerm>, String>,
) -> Result<EggTerm, String> {
    let mut opt = Optimizer::new(program)?;
    opt.define_root(root_expr)?;
    for _ in 0..max_rounds {
        opt.run(run_iterations)?;
        let term = opt.extract_root()?;
        match reduce(&term)? {
            None => return Ok(term),
            Some(reduced) => opt.union_root(&reduced)?,
        }
    }
    Err(format!("optimize: tactic reduction did not converge in {max_rounds} rounds"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOY: &str = r#"
; toy grammar: strings wrapped in high-cost shells
(datatype Expr
  (Leaf String :cost 1)
  (Wrap Expr :cost 1000)
  (Host Expr :cost 1000))
(rewrite (Wrap e) e)
"#;

    #[test]
    fn saturate_and_extract_picks_cheapest() {
        let mut opt = Optimizer::new(TOY).unwrap();
        opt.define_root(r#"(Wrap (Wrap (Leaf "x")))"#).unwrap();
        opt.run(10).unwrap();
        let term = opt.extract_root().unwrap();
        assert_eq!(term, EggTerm::app("Leaf", vec![EggTerm::str("x")]));
    }

    #[test]
    fn loop_reduces_host_nodes_via_union() {
        // (Host e) has no egglog rule; the host callback strips it, the
        // union makes the stripped form extractable by cost.
        let reduce = |term: &EggTerm| -> Result<Option<EggTerm>, String> {
            if !term.contains_app("Host") {
                return Ok(None);
            }
            fn strip(t: &EggTerm) -> EggTerm {
                match t {
                    EggTerm::Str(_) => t.clone(),
                    EggTerm::App(head, args) => {
                        let args: Vec<EggTerm> = args.iter().map(strip).collect();
                        if head == "Host" {
                            match args.into_iter().next() {
                                Some(inner) => inner,
                                None => unreachable!("Host is unary"),
                            }
                        } else {
                            EggTerm::App(head.clone(), args)
                        }
                    }
                }
            }
            Ok(Some(strip(term)))
        };
        let term =
            optimize_loop(TOY, r#"(Host (Wrap (Leaf "x")))"#, 10, 20, reduce).unwrap();
        assert_eq!(term, EggTerm::app("Leaf", vec![EggTerm::str("x")]));
    }

    #[test]
    fn non_converging_reduction_errors() {
        let reduce = |term: &EggTerm| Ok(Some(term.clone()));
        let err = optimize_loop(TOY, r#"(Host (Leaf "x"))"#, 1, 3, reduce).unwrap_err();
        assert!(err.contains("did not converge"), "{err}");
    }

    #[test]
    fn sexpr_round_trip_escapes_strings() {
        let term = EggTerm::app("Leaf", vec![EggTerm::str("say \"hi\"")]);
        assert_eq!(term.to_sexpr(), r#"(Leaf "say \"hi\"")"#);
    }
}
