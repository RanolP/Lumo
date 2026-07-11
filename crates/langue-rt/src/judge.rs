//! Relational judgment engine (M2 step 3, D-23). λProlog is the
//! implementation reference. The engine is generic — it knows nothing
//! about Lumo (D-08): generated judgment modules (M2 step 4) compile
//! `head := body` rules into [`Rule`] values and syntax/type nodes into
//! [`Term`]s; the engine solves goals, unifies (`=`), reads and extends
//! named context multimaps (D-16/D-23), and builds real derivation
//! trees.
//!
//! Discipline (D-23):
//! - **exactly one rule may succeed** per goal — zero is a proof
//!   failure (a generic bail, D-26), two or more is a hard definition
//!   error;
//! - **strictly decreasing recursion** (the D-28 measure): a nested
//!   call of the same judgment must have a strictly smaller subject
//!   (first argument, by term size) than the enclosing one — checked
//!   at runtime, so a non-decreasing rule bails instead of diverging.
//!
//! Context reads are newest-first: the most recent entry whose key
//! unifies is *the* binding (lexical shadowing); its value must unify
//! or the read fails.

use std::collections::HashMap;

pub type VarId = u32;

/// A first-order term. Judgment codegen encodes syntax nodes, types,
/// and literals as [`Term::Struct`]/[`Term::Atom`]; metavariables are
/// [`Term::Var`]. [`Term::Set`] is the capability-row value (D-25/
/// D-41): a hash-keyed set — entries dedup by structural key, order
/// is irrelevant — with an optional row tail (`rest`), so `{A | r}`
/// unifies against `{A, B}` binding `r = {B}`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Term {
    Var(VarId),
    Atom(String),
    Struct(String, Vec<Term>),
    Set { entries: Vec<Term>, rest: Option<Box<Term>> },
}

pub fn var(id: VarId) -> Term {
    Term::Var(id)
}

pub fn atom(text: impl Into<String>) -> Term {
    Term::Atom(text.into())
}

pub fn app(functor: impl Into<String>, args: Vec<Term>) -> Term {
    Term::Struct(functor.into(), args)
}

pub fn set(entries: Vec<Term>, rest: Option<Term>) -> Term {
    Term::Set { entries, rest: rest.map(Box::new) }
}

/// The structural key entries hash to (D-25: a map is a set when the
/// key is a hash). Stable for ground terms; variables key by id.
fn skey(term: &Term) -> String {
    match term {
        Term::Var(v) => format!("${v}"),
        Term::Atom(a) => format!("a:{a}"),
        Term::Struct(f, args) => {
            let args: Vec<String> = args.iter().map(skey).collect();
            format!("s:{f}({})", args.join(","))
        }
        Term::Set { entries, rest } => {
            let mut keys: Vec<String> = entries.iter().map(skey).collect();
            keys.sort();
            let rest = rest.as_ref().map(|r| skey(r)).unwrap_or_default();
            format!("set:{{{}|{rest}}}", keys.join(","))
        }
    }
}

impl Term {
    /// Node count — the D-28 strict-subtree measure. Unbound variables
    /// count 1 (the check is conservative on partly-unresolved
    /// subjects).
    fn size(&self) -> usize {
        match self {
            Term::Var(_) | Term::Atom(_) => 1,
            Term::Struct(_, args) => 1 + args.iter().map(Term::size).sum::<usize>(),
            Term::Set { entries, rest } => {
                1 + entries.iter().map(Term::size).sum::<usize>()
                    + rest.as_ref().map(|r| r.size()).unwrap_or(0)
            }
        }
    }

    /// Shift every variable by `offset` (rule instantiation: rules
    /// number their variables locally from 0).
    fn offset(&self, offset: VarId) -> Term {
        match self {
            Term::Var(v) => Term::Var(v + offset),
            Term::Atom(_) => self.clone(),
            Term::Struct(f, args) => {
                Term::Struct(f.clone(), args.iter().map(|a| a.offset(offset)).collect())
            }
            Term::Set { entries, rest } => Term::Set {
                entries: entries.iter().map(|e| e.offset(offset)).collect(),
                rest: rest.as_ref().map(|r| Box::new(r.offset(offset))),
            },
        }
    }

    fn max_var(&self) -> VarId {
        match self {
            Term::Var(v) => v + 1,
            Term::Atom(_) => 0,
            Term::Struct(_, args) => args.iter().map(Term::max_var).max().unwrap_or(0),
            Term::Set { entries, rest } => entries
                .iter()
                .map(Term::max_var)
                .chain(rest.iter().map(|r| r.max_var()))
                .max()
                .unwrap_or(0),
        }
    }
}

/// One goal of a rule body, solved left to right.
#[derive(Clone, Debug)]
pub enum Goal {
    /// A judgment call, optionally extending named contexts for the
    /// callee's derivation (`with Γ+{k: v}`, D-23). Extension entries
    /// are resolved when pushed.
    Call { judgment: String, args: Vec<Term>, extends: Vec<(String, Term, Term)> },
    /// `a = b`.
    Unify(Term, Term),
    /// `value = Γ.key` (D-16).
    CtxRead { ctx: String, key: Term, value: Term },
    /// `out = $e[$b := $a]` (D-24): `target` with every subterm
    /// structurally equal to `needle` replaced by `replacement`.
    /// Naively structural — the rule writer avoids capture until
    /// binders demand better.
    Subst { target: Term, needle: Term, replacement: Term, out: Term },
    /// `out = (hash input)` (D-25): a `#list` term as a hash-keyed
    /// set (idempotent on sets).
    Hash { input: Term, out: Term },
}

/// A `head := body` rule. `params`/`body` share one local variable
/// numbering `0..var_count`.
#[derive(Clone, Debug)]
pub struct Rule {
    pub judgment: String,
    pub params: Vec<Term>,
    pub var_count: VarId,
    pub body: Vec<Goal>,
}

/// A successful derivation: the proof tree the design promises (D-08).
/// `args` are fully resolved against the final substitution.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Derivation {
    pub judgment: String,
    pub args: Vec<Term>,
    pub children: Vec<Derivation>,
}

/// A failed judgment — a generic message for now (D-26). `hard` marks
/// definition errors (ambiguous rules, non-decreasing recursion) that
/// abort the whole run instead of failing one rule trial.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Bail {
    pub message: String,
    pub hard: bool,
}

impl Bail {
    fn soft(message: String) -> Bail {
        Bail { message, hard: false }
    }

    fn hard(message: String) -> Bail {
        Bail { message, hard: true }
    }
}

/// Named context multimaps (D-16), newest entry last. Seed this with
/// the global bindings (defs, externs) before solving.
pub type Contexts = HashMap<String, Vec<(Term, Term)>>;

type Subst = HashMap<VarId, Term>;

fn walk<'a>(subst: &'a Subst, mut term: &'a Term) -> &'a Term {
    while let Term::Var(v) = term {
        match subst.get(v) {
            Some(bound) => term = bound,
            None => break,
        }
    }
    term
}

/// Deep resolution. Sets canonicalize here: entries resolve and dedup
/// by [`skey`] in sorted order, a rest bound to another set is
/// absorbed (D-41), and an empty open set collapses to its rest.
fn resolve(subst: &Subst, term: &Term) -> Term {
    let term = walk(subst, term);
    match term {
        Term::Struct(f, args) => {
            Term::Struct(f.clone(), args.iter().map(|a| resolve(subst, a)).collect())
        }
        Term::Set { entries, rest } => {
            let mut out: Vec<Term> = entries.iter().map(|e| resolve(subst, e)).collect();
            let mut tail: Option<Term> = None;
            if let Some(rest) = rest {
                match resolve(subst, rest) {
                    Term::Set { entries, rest } => {
                        out.extend(entries);
                        tail = rest.map(|r| *r);
                    }
                    other => tail = Some(other),
                }
            }
            out.sort_by_key(skey);
            out.dedup_by_key(|e| skey(e));
            if out.is_empty() {
                if let Some(tail) = tail {
                    return tail;
                }
            }
            Term::Set { entries: out, rest: tail.map(Box::new) }
        }
        _ => term.clone(),
    }
}

fn occurs(subst: &Subst, var: VarId, term: &Term) -> bool {
    match walk(subst, term) {
        Term::Var(v) => *v == var,
        Term::Atom(_) => false,
        Term::Struct(_, args) => args.iter().any(|a| occurs(subst, var, a)),
        Term::Set { entries, rest } => {
            entries.iter().any(|e| occurs(subst, var, e))
                || rest.as_ref().is_some_and(|r| occurs(subst, var, r))
        }
    }
}

fn unify(subst: &mut Subst, next_var: &mut VarId, a: &Term, b: &Term) -> bool {
    let a = walk(subst, a).clone();
    let b = walk(subst, b).clone();
    match (&a, &b) {
        (Term::Var(x), Term::Var(y)) if x == y => true,
        (Term::Var(v), t) | (t, Term::Var(v)) => {
            if occurs(subst, *v, t) {
                return false;
            }
            subst.insert(*v, t.clone());
            true
        }
        (Term::Atom(x), Term::Atom(y)) => x == y,
        (Term::Struct(f, xs), Term::Struct(g, ys)) => {
            f == g
                && xs.len() == ys.len()
                && xs.iter().zip(ys).all(|(x, y)| unify(subst, next_var, x, y))
        }
        (Term::Set { .. }, _) | (_, Term::Set { .. }) => unify_sets(subst, next_var, &a, &b),
        _ => false,
    }
}

/// Row unification (D-25/D-41). Entries match greedily (first
/// unifiable, deterministic under canonical order — no backtracking
/// across matchings); leftovers flow into the other side's rest.
/// `{A | r1} = {B | r2}` gives `r1 = {B | ρ}`, `r2 = {A | ρ}` with a
/// shared fresh tail ρ.
fn unify_sets(subst: &mut Subst, next_var: &mut VarId, a: &Term, b: &Term) -> bool {
    let a = resolve(subst, a);
    let b = resolve(subst, b);
    let (Term::Set { entries: ea, rest: ta }, Term::Set { entries: eb, rest: tb }) = (&a, &b)
    else {
        // One side canonicalized away from a set (`{ | r}` → `r`).
        return unify(subst, next_var, &a, &b);
    };
    let mut left_b: Vec<Term> = eb.clone();
    let mut left_a: Vec<Term> = Vec::new();
    for entry in ea {
        let mut matched = false;
        for j in 0..left_b.len() {
            let saved = subst.clone();
            if unify(subst, next_var, entry, &left_b[j]) {
                left_b.remove(j);
                matched = true;
                break;
            }
            *subst = saved;
        }
        if !matched {
            left_a.push(entry.clone());
        }
    }
    match (ta, tb) {
        (None, None) => left_a.is_empty() && left_b.is_empty(),
        (Some(ta), None) => {
            left_a.is_empty() && unify(subst, next_var, ta, &set(left_b, None))
        }
        (None, Some(tb)) => {
            left_b.is_empty() && unify(subst, next_var, tb, &set(left_a, None))
        }
        (Some(ta), Some(tb)) => {
            if ta == tb {
                return left_a.is_empty() && left_b.is_empty();
            }
            let rho = Term::Var(*next_var);
            *next_var += 1;
            unify(subst, next_var, ta, &set(left_b, Some(rho.clone())))
                && unify(subst, next_var, tb, &set(left_a, Some(rho)))
        }
    }
}

/// `target` with every subterm structurally equal to `needle`
/// replaced by `replacement` (all three already resolved).
fn substitute(target: &Term, needle: &Term, replacement: &Term) -> Term {
    if target == needle {
        return replacement.clone();
    }
    match target {
        Term::Struct(f, args) => Term::Struct(
            f.clone(),
            args.iter().map(|a| substitute(a, needle, replacement)).collect(),
        ),
        Term::Set { entries, rest } => Term::Set {
            entries: entries.iter().map(|e| substitute(e, needle, replacement)).collect(),
            rest: rest.as_ref().map(|r| Box::new(substitute(r, needle, replacement))),
        },
        _ => target.clone(),
    }
}

/// The rule database plus the entry point.
pub struct Engine {
    rules: Vec<Rule>,
}

impl Engine {
    pub fn new(rules: Vec<Rule>) -> Engine {
        Engine { rules }
    }

    /// Solve `(judgment args…)` under seeded contexts. `args` may use
    /// variables numbered from 0 (they name the inout positions and
    /// come back resolved in the derivation).
    pub fn solve(
        &self,
        judgment: &str,
        args: Vec<Term>,
        ctxs: Contexts,
    ) -> Result<Derivation, Bail> {
        let next_var = args.iter().map(Term::max_var).max().unwrap_or(0);
        let mut solver = Solver {
            engine: self,
            subst: Subst::new(),
            next_var,
            ctxs,
            active: HashMap::new(),
        };
        let derivation = solver.solve_call(judgment, &args)?;
        Ok(solver.finish(derivation))
    }
}

struct Solver<'e> {
    engine: &'e Engine,
    subst: Subst,
    next_var: VarId,
    ctxs: Contexts,
    /// Per-judgment stack of active subject sizes (the D-28 guard).
    active: HashMap<String, Vec<usize>>,
}

impl Solver<'_> {
    fn solve_call(&mut self, judgment: &str, args: &[Term]) -> Result<Derivation, Bail> {
        let subject_size =
            args.first().map(|a| resolve(&self.subst, a).size()).unwrap_or(0);
        if let Some(&enclosing) = self.active.get(judgment).and_then(|s| s.last()) {
            if subject_size >= enclosing {
                return Err(Bail::hard(format!(
                    "recursion on `{judgment}` does not strictly decrease (D-23/D-28)"
                )));
            }
        }
        self.active.entry(judgment.to_owned()).or_default().push(subject_size);
        let result = self.trial_rules(judgment, args);
        self.active.get_mut(judgment).expect("pushed above").pop();
        result
    }

    /// Try every rule of the judgment under a snapshot; adopt the
    /// bindings of the single success (D-23: exactly one).
    fn trial_rules(&mut self, judgment: &str, args: &[Term]) -> Result<Derivation, Bail> {
        let mut winner: Option<(Subst, VarId, Vec<Derivation>)> = None;
        for rule in self.engine.rules.iter().filter(|r| r.judgment == judgment) {
            let saved_subst = self.subst.clone();
            let saved_next = self.next_var;
            match self.try_rule(rule, args) {
                Ok(children) => {
                    if winner.is_some() {
                        return Err(Bail::hard(format!(
                            "more than one rule succeeds for `{judgment}` (D-23)"
                        )));
                    }
                    winner = Some((
                        std::mem::replace(&mut self.subst, saved_subst),
                        std::mem::replace(&mut self.next_var, saved_next),
                        children,
                    ));
                }
                Err(bail) if bail.hard => return Err(bail),
                Err(_) => {
                    self.subst = saved_subst;
                    self.next_var = saved_next;
                }
            }
        }
        match winner {
            Some((subst, next_var, children)) => {
                self.subst = subst;
                self.next_var = next_var;
                Ok(Derivation {
                    judgment: judgment.to_owned(),
                    args: args.to_vec(),
                    children,
                })
            }
            None => Err(Bail::soft(format!("no rule succeeds for `{judgment}` (D-26)"))),
        }
    }

    fn try_rule(&mut self, rule: &Rule, args: &[Term]) -> Result<Vec<Derivation>, Bail> {
        let fail = || Bail::soft(String::new());
        if rule.params.len() != args.len() {
            return Err(fail());
        }
        let offset = self.next_var;
        self.next_var += rule.var_count;
        for (param, arg) in rule.params.iter().zip(args) {
            if !unify(&mut self.subst, &mut self.next_var, &param.offset(offset), arg) {
                return Err(fail());
            }
        }
        let mut children = Vec::new();
        for goal in &rule.body {
            match goal {
                Goal::Unify(a, b) => {
                    let (a, b) = (a.offset(offset), b.offset(offset));
                    if !unify(&mut self.subst, &mut self.next_var, &a, &b) {
                        return Err(fail());
                    }
                }
                Goal::Subst { target, needle, replacement, out } => {
                    let result = substitute(
                        &resolve(&self.subst, &target.offset(offset)),
                        &resolve(&self.subst, &needle.offset(offset)),
                        &resolve(&self.subst, &replacement.offset(offset)),
                    );
                    let out = out.offset(offset);
                    if !unify(&mut self.subst, &mut self.next_var, &out, &result) {
                        return Err(fail());
                    }
                }
                Goal::Hash { input, out } => {
                    let hashed = match resolve(&self.subst, &input.offset(offset)) {
                        Term::Struct(f, items) if f == "#list" => {
                            resolve(&self.subst, &set(items, None))
                        }
                        s @ Term::Set { .. } => s,
                        _ => return Err(fail()),
                    };
                    let out = out.offset(offset);
                    if !unify(&mut self.subst, &mut self.next_var, &out, &hashed) {
                        return Err(fail());
                    }
                }
                Goal::CtxRead { ctx, key, value } => {
                    self.ctx_read(ctx, &key.offset(offset), &value.offset(offset))?;
                }
                Goal::Call { judgment, args, extends } => {
                    for (ctx, k, v) in extends {
                        let entry = (
                            resolve(&self.subst, &k.offset(offset)),
                            resolve(&self.subst, &v.offset(offset)),
                        );
                        self.ctxs.entry(ctx.clone()).or_default().push(entry);
                    }
                    let call_args: Vec<Term> =
                        args.iter().map(|a| a.offset(offset)).collect();
                    let result = self.solve_call(judgment, &call_args);
                    for (ctx, _, _) in extends {
                        self.ctxs.get_mut(ctx).expect("pushed above").pop();
                    }
                    children.push(result?);
                }
            }
        }
        Ok(children)
    }

    /// `value = Γ.key`: the newest entry whose key unifies is the
    /// binding (shadowing); its value must unify too.
    fn ctx_read(&mut self, ctx: &str, key: &Term, value: &Term) -> Result<(), Bail> {
        let entries = self.ctxs.get(ctx).cloned().unwrap_or_default();
        for (k, v) in entries.iter().rev() {
            let saved = self.subst.clone();
            if unify(&mut self.subst, &mut self.next_var, key, k) {
                if unify(&mut self.subst, &mut self.next_var, value, v) {
                    return Ok(());
                }
                self.subst = saved;
                return Err(Bail::soft(String::new()));
            }
            self.subst = saved;
        }
        Err(Bail::soft(String::new()))
    }

    /// Assignment happens at the bottom of a derivation and propagates
    /// up (D-17) — so args resolve only once the whole tree succeeded.
    fn finish(&self, derivation: Derivation) -> Derivation {
        Derivation {
            judgment: derivation.judgment,
            args: derivation.args.iter().map(|a| resolve(&self.subst, a)).collect(),
            children: derivation.children.into_iter().map(|c| self.finish(c)).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A miniature inference judgment over `num(_)`, `var(name)`, and
    /// `let(name, value, body)` — enough to exercise unification,
    /// context read/write, shadowing, and inout result propagation.
    fn mini_rules() -> Vec<Rule> {
        vec![
            // infer num(n) -> number
            Rule {
                judgment: "infer".into(),
                params: vec![app("num", vec![var(0)]), var(1)],
                var_count: 2,
                body: vec![Goal::Unify(var(1), atom("number"))],
            },
            // infer var(x) -> Γ.x
            Rule {
                judgment: "infer".into(),
                params: vec![app("var", vec![var(0)]), var(1)],
                var_count: 2,
                body: vec![Goal::CtxRead {
                    ctx: "Γ".into(),
                    key: var(0),
                    value: var(1),
                }],
            },
            // infer let(x, v, b) -> T  :=  infer v -> Tv,
            //                             infer b -> T with Γ+{x: Tv}
            Rule {
                judgment: "infer".into(),
                params: vec![app("let", vec![var(0), var(1), var(2)]), var(3)],
                var_count: 5,
                body: vec![
                    Goal::Call {
                        judgment: "infer".into(),
                        args: vec![var(1), var(4)],
                        extends: vec![],
                    },
                    Goal::Call {
                        judgment: "infer".into(),
                        args: vec![var(2), var(3)],
                        extends: vec![("Γ".into(), var(0), var(4))],
                    },
                ],
            },
        ]
    }

    fn num() -> Term {
        app("num", vec![atom("1")])
    }

    #[test]
    fn infers_through_context_write_and_read() {
        let engine = Engine::new(mini_rules());
        // let x = 1 in x  :  number
        let goal = app("let", vec![atom("x"), num(), app("var", vec![atom("x")])]);
        let derivation =
            engine.solve("infer", vec![goal, var(0)], Contexts::new()).unwrap();
        assert_eq!(derivation.args[1], atom("number"));
        // Derivation shape: let → [value, body].
        assert_eq!(derivation.children.len(), 2);
        assert_eq!(derivation.children[1].args[1], atom("number"));
    }

    #[test]
    fn newest_context_entry_shadows() {
        let engine = Engine::new(mini_rules());
        // let x = 1 in let x = var(y) in x — inner x has y's type.
        let inner = app("let", vec![
            atom("x"),
            app("var", vec![atom("y")]),
            app("var", vec![atom("x")]),
        ]);
        let goal = app("let", vec![atom("x"), num(), inner]);
        let mut ctxs = Contexts::new();
        ctxs.insert("Γ".into(), vec![(atom("y"), atom("string"))]);
        let derivation = engine.solve("infer", vec![goal, var(0)], ctxs).unwrap();
        assert_eq!(derivation.args[1], atom("string"));
    }

    #[test]
    fn zero_rules_bail_softly_and_unknown_var_fails_ctx_read() {
        let engine = Engine::new(mini_rules());
        let unknown = app("var", vec![atom("nope")]);
        let bail = engine.solve("infer", vec![unknown, var(0)], Contexts::new());
        assert!(!bail.unwrap_err().hard);
        let no_judgment = engine.solve("check", vec![num()], Contexts::new());
        assert!(!no_judgment.unwrap_err().hard);
    }

    #[test]
    fn two_successful_rules_are_a_hard_error() {
        let mut rules = mini_rules();
        rules.push(Rule {
            judgment: "infer".into(),
            params: vec![app("num", vec![var(0)]), var(1)],
            var_count: 2,
            body: vec![Goal::Unify(var(1), atom("float"))],
        });
        let engine = Engine::new(rules);
        let bail = engine.solve("infer", vec![num(), var(0)], Contexts::new());
        assert!(bail.unwrap_err().hard);
    }

    #[test]
    fn non_decreasing_recursion_bails_instead_of_diverging() {
        let engine = Engine::new(vec![Rule {
            judgment: "spin".into(),
            params: vec![var(0)],
            var_count: 1,
            body: vec![Goal::Call {
                judgment: "spin".into(),
                args: vec![var(0)],
                extends: vec![],
            }],
        }]);
        let bail = engine.solve("spin", vec![num()], Contexts::new());
        assert!(bail.unwrap_err().hard);
    }

    #[test]
    fn occurs_check_rejects_infinite_terms() {
        let mut subst = Subst::new();
        let mut next = 1;
        assert!(!unify(&mut subst, &mut next, &var(0), &app("f", vec![var(0)])));
    }

    #[test]
    fn sets_unify_regardless_of_order_and_duplicates() {
        let mut subst = Subst::new();
        let mut next = 0;
        let ab = set(vec![atom("A"), atom("B")], None);
        let baa = set(vec![atom("B"), atom("A"), atom("A")], None);
        assert!(unify(&mut subst, &mut next, &ab, &baa));
        // Closed rows with different entries do not unify.
        let a = set(vec![atom("A")], None);
        assert!(!unify(&mut subst, &mut next, &a, &ab));
    }

    #[test]
    fn open_row_binds_its_rest_to_the_leftovers() {
        let mut subst = Subst::new();
        let mut next = 1;
        let open = set(vec![atom("A")], Some(var(0)));
        let full = set(vec![atom("A"), atom("B"), atom("C")], None);
        assert!(unify(&mut subst, &mut next, &open, &full));
        assert_eq!(
            resolve(&subst, &var(0)),
            set(vec![atom("B"), atom("C")], None)
        );
        // The bound-rest row now resolves equal to the full row.
        assert_eq!(resolve(&subst, &open), resolve(&subst, &full));
    }

    #[test]
    fn two_open_rows_share_a_fresh_tail() {
        let mut subst = Subst::new();
        let mut next = 2;
        let left = set(vec![atom("A")], Some(var(0)));
        let right = set(vec![atom("B")], Some(var(1)));
        assert!(unify(&mut subst, &mut next, &left, &right));
        // Both sides now contain A and B plus the shared tail.
        let l = resolve(&subst, &left);
        let r = resolve(&subst, &right);
        assert_eq!(l, r);
        let Term::Set { entries, rest } = l else { panic!("{l:?}") };
        assert_eq!(entries, vec![atom("A"), atom("B")]);
        assert!(rest.is_some());
    }

    #[test]
    fn empty_open_set_collapses_to_its_rest() {
        let mut subst = Subst::new();
        let mut next = 1;
        let hollow = set(vec![], Some(var(0)));
        let full = set(vec![atom("A")], None);
        assert!(unify(&mut subst, &mut next, &hollow, &full));
        assert_eq!(resolve(&subst, &var(0)), full);
    }

    #[test]
    fn subst_and_hash_goals() {
        // subst: rewrite occurrences of a inside f(a, g(a), b).
        let engine = Engine::new(vec![Rule {
            judgment: "inst".into(),
            params: vec![var(0), var(1)],
            var_count: 2,
            body: vec![Goal::Subst {
                target: var(0),
                needle: atom("a"),
                replacement: atom("x"),
                out: var(1),
            }],
        }]);
        let target = app("f", vec![atom("a"), app("g", vec![atom("a")]), atom("b")]);
        let d = engine.solve("inst", vec![target, var(0)], Contexts::new()).unwrap();
        assert_eq!(
            d.args[1],
            app("f", vec![atom("x"), app("g", vec![atom("x")]), atom("b")])
        );
        // hash: a #list becomes a deduped set; non-lists fail softly.
        let engine = Engine::new(vec![Rule {
            judgment: "rowify".into(),
            params: vec![var(0), var(1)],
            var_count: 2,
            body: vec![Goal::Hash { input: var(0), out: var(1) }],
        }]);
        let list = app("#list", vec![atom("B"), atom("A"), atom("B")]);
        let d = engine.solve("rowify", vec![list, var(0)], Contexts::new()).unwrap();
        assert_eq!(d.args[1], set(vec![atom("A"), atom("B")], None));
        let bail = engine.solve("rowify", vec![atom("nope"), var(0)], Contexts::new());
        assert!(!bail.unwrap_err().hard);
    }

    #[test]
    fn context_extensions_pop_after_failed_calls() {
        let engine = Engine::new(mini_rules());
        // The body `var(z)` fails (z unbound in Γ) inside a `let`
        // extension — a later read of Γ must not see the extension.
        let goal = app("let", vec![atom("x"), num(), app("var", vec![atom("z")])]);
        let mut ctxs = Contexts::new();
        ctxs.insert("Γ".into(), vec![]);
        let bail = engine.solve("infer", vec![goal, var(0)], ctxs);
        assert!(!bail.unwrap_err().hard);
    }
}
