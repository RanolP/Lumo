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

    /// Merge two host-computed equal terms.
    pub fn union_terms(&mut self, a: &EggTerm, b: &EggTerm) -> Result<(), String> {
        self.run_text(&format!("(union {} {})", a.to_sexpr(), b.to_sexpr())).map(drop)
    }

    /// Every row of table `name` as a call term — arguments are
    /// min-cost extractions of the row's argument e-classes. This is
    /// how host-side tactics see their pending work: a high-cost
    /// tactic constructor never wins extraction, so its calls are read
    /// off the table instead (D-42).
    pub fn table_calls(&mut self, name: &str) -> Result<Vec<EggTerm>, String> {
        let outputs = self.run_text(&format!("(print-function {name} {})", u32::MAX))?;
        for output in outputs {
            if let CommandOutput::PrintFunction(_, dag, rows, _) = output {
                return rows.iter().map(|(call, _)| walk(&dag, *call)).collect();
            }
        }
        Err(format!("egglog: (print-function {name}) produced no output"))
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

/// The D-42 loop: saturate, read the pending `tactic` calls off its
/// table, reduce each host-side, union the reductions back (making the
/// reduced forms extractable by true cost), and repeat until a round
/// adds nothing new. `reduce` maps one call term (`(subst t b v)`) to
/// its tactic-free reduction. No convergence within `max_rounds`, or a
/// tactic node surviving into the final extraction, is an error.
pub fn optimize_loop(
    program: &str,
    root_expr: &str,
    run_iterations: usize,
    max_rounds: usize,
    tactic: &str,
    reduce: impl Fn(&EggTerm) -> Result<EggTerm, String>,
) -> Result<EggTerm, String> {
    let mut opt = Optimizer::new(program)?;
    opt.define_root(root_expr)?;
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..max_rounds {
        opt.run(run_iterations)?;
        let mut progressed = false;
        for call in opt.table_calls(tactic)? {
            if !seen.insert(call.to_sexpr()) {
                continue;
            }
            let reduced = reduce(&call)?;
            opt.union_terms(&call, &reduced)?;
            progressed = true;
        }
        if !progressed {
            let term = opt.extract_root()?;
            if term.contains_app(tactic) {
                return Err(format!("optimize: `{tactic}` leaked into extraction"));
            }
            return Ok(term);
        }
    }
    Err(format!("optimize: tactic reduction did not converge in {max_rounds} rounds"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Toy grammar shaped like the real compiled programs: a rewrite
    // introduces a high-cost host tactic (`host` strips one Shell).
    const TOY: &str = r#"
(datatype Expr
  (Leaf String :cost 1)
  (Wrap Expr :cost 1000)
  (Shell Expr :cost 1))
(constructor host (Expr) Expr :cost 1000)
(rewrite (Wrap e) e)
(rewrite (Shell e) (host e))
"#;

    /// `(host e)` → `e` (the toy tactic's host-side reduction).
    fn reduce_host(call: &EggTerm) -> Result<EggTerm, String> {
        match call {
            EggTerm::App(head, args) if head == "host" && args.len() == 1 => {
                Ok(args[0].clone())
            }
            other => Err(format!("not a host call: {other:?}")),
        }
    }

    #[test]
    fn saturate_and_extract_picks_cheapest() {
        let mut opt = Optimizer::new(TOY).unwrap();
        opt.define_root(r#"(Wrap (Wrap (Leaf "x")))"#).unwrap();
        opt.run(10).unwrap();
        let term = opt.extract_root().unwrap();
        assert_eq!(term, EggTerm::app("Leaf", vec![EggTerm::str("x")]));
    }

    #[test]
    fn table_calls_sees_pending_tactic_work() {
        let mut opt = Optimizer::new(TOY).unwrap();
        opt.define_root(r#"(Shell (Leaf "x"))"#).unwrap();
        opt.run(10).unwrap();
        let calls = opt.table_calls("host").unwrap();
        assert_eq!(
            calls,
            vec![EggTerm::app("host", vec![EggTerm::app("Leaf", vec![EggTerm::str("x")])])]
        );
    }

    #[test]
    fn loop_reduces_tactic_calls_until_dry() {
        // Nested shells: each round's reduction can enable the next.
        let term = optimize_loop(
            TOY,
            r#"(Shell (Wrap (Shell (Leaf "x"))))"#,
            10,
            20,
            "host",
            reduce_host,
        )
        .unwrap();
        assert_eq!(term, EggTerm::app("Leaf", vec![EggTerm::str("x")]));
    }

    #[test]
    fn non_converging_reduction_errors() {
        // A reduction that mints a fresh Shell each call keeps the
        // tactic table growing — the loop has to give up.
        let n = std::cell::Cell::new(0u32);
        let reduce = |_: &EggTerm| {
            n.set(n.get() + 1);
            Ok(EggTerm::app(
                "Shell",
                vec![EggTerm::app("Leaf", vec![EggTerm::str(format!("x{}", n.get()))])],
            ))
        };
        let err = optimize_loop(TOY, r#"(Shell (Leaf "x"))"#, 1, 3, "host", reduce)
            .unwrap_err();
        assert!(err.contains("did not converge"), "{err}");
    }

    #[test]
    fn sexpr_round_trip_escapes_strings() {
        let term = EggTerm::app("Leaf", vec![EggTerm::str("say \"hi\"")]);
        assert_eq!(term.to_sexpr(), r#"(Leaf "say \"hi\"")"#);
    }
}
