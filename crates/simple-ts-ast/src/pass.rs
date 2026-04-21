use crate::ast::*;

pub fn expr_to_block(expr: Expr) -> Block {
    Block::new(vec![Stmt::Return(Some(expr))])
}

pub fn lower_expression_bodies(program: &mut Program) {
    for stmt in &mut program.body {
        lower_stmt_expr_bodies(stmt);
    }
}

pub fn return_lifting(program: &mut Program) {
    for stmt in &mut program.body {
        lift_stmt(stmt);
    }
}

// ---------------------------------------------------------------------------
// Pass: inline_always_calls
//
// For every `function f(p1, p2) { body }` marked `inline_always`,
// replace each `f(arg1, arg2)` call elsewhere with an IIFE-substituted body:
//   `((p1, p2) => body)(arg1, arg2)`
// Then drop the inline declarations themselves from the program.
// The follow-up `flatten_iifes` pass collapses the IIFE into the surrounding
// expression where possible.
// ---------------------------------------------------------------------------

pub fn inline_always_calls(program: &mut Program) {
    let inline_table: std::collections::HashMap<String, FunctionDecl> = program
        .body
        .iter()
        .filter_map(|stmt| {
            if let Stmt::Function(decl) = stmt {
                if decl.inline_always {
                    return Some((decl.name.clone(), decl.clone()));
                }
            }
            None
        })
        .collect();
    if inline_table.is_empty() {
        return;
    }
    for stmt in &mut program.body {
        inline_calls_in_stmt(stmt, &inline_table);
    }
    program.body.retain(|stmt| match stmt {
        Stmt::Function(decl) => !decl.inline_always,
        _ => true,
    });
}

fn inline_calls_in_stmt(
    stmt: &mut Stmt,
    table: &std::collections::HashMap<String, FunctionDecl>,
) {
    match stmt {
        Stmt::Expr(expr) | Stmt::Return(Some(expr)) => inline_calls_in_expr(expr, table),
        Stmt::Const(decl) => inline_calls_in_expr(&mut decl.init, table),
        Stmt::Let { init: Some(init), .. } => inline_calls_in_expr(init, table),
        Stmt::Assign { value, .. } => inline_calls_in_expr(value, table),
        Stmt::If { cond, then_branch, else_branch } => {
            inline_calls_in_expr(cond, table);
            for s in &mut then_branch.stmts { inline_calls_in_stmt(s, table); }
            if let Some(eb) = else_branch {
                for s in &mut eb.stmts { inline_calls_in_stmt(s, table); }
            }
        }
        Stmt::Block(b) => {
            for s in &mut b.stmts { inline_calls_in_stmt(s, table); }
        }
        Stmt::Function(f) => inline_calls_in_function_body(&mut f.body, table),
        _ => {}
    }
}

fn inline_calls_in_function_body(
    body: &mut FunctionBody,
    table: &std::collections::HashMap<String, FunctionDecl>,
) {
    match body {
        FunctionBody::Expr(e) => inline_calls_in_expr(e, table),
        FunctionBody::Block(b) => {
            for s in &mut b.stmts { inline_calls_in_stmt(s, table); }
        }
    }
}

fn inline_calls_in_expr(
    expr: &mut Expr,
    table: &std::collections::HashMap<String, FunctionDecl>,
) {
    // Recurse first so nested calls get inlined too.
    match expr {
        Expr::Call { callee, args } => {
            inline_calls_in_expr(callee, table);
            for a in args.iter_mut() { inline_calls_in_expr(a, table); }
        }
        Expr::Member { object, .. } => inline_calls_in_expr(object, table),
        Expr::Index { object, index } => {
            inline_calls_in_expr(object, table);
            inline_calls_in_expr(index, table);
        }
        Expr::Unary { expr: e, .. } | Expr::Void(e) => inline_calls_in_expr(e, table),
        Expr::Binary { left, right, .. } => {
            inline_calls_in_expr(left, table);
            inline_calls_in_expr(right, table);
        }
        Expr::Array(items) => {
            for item in items { inline_calls_in_expr(item, table); }
        }
        Expr::Object(props) => {
            for prop in props {
                if let ObjectKey::Computed(e) = &mut prop.key {
                    inline_calls_in_expr(e, table);
                }
                inline_calls_in_expr(&mut prop.value, table);
            }
        }
        Expr::IfElse { cond, then_expr, else_expr } => {
            inline_calls_in_expr(cond, table);
            inline_calls_in_expr(then_expr, table);
            inline_calls_in_expr(else_expr, table);
        }
        Expr::Arrow { body, .. } => inline_calls_in_function_body(body, table),
        _ => {}
    }
    // Now check if this is a Call to an inline fn.
    if let Expr::Call { callee, args } = expr {
        if let Expr::Ident(name) = callee.as_ref() {
            if let Some(decl) = table.get(name) {
                if decl.params.len() == args.len() {
                    let arrow = Expr::Arrow {
                        params: decl.params.clone(),
                        return_type: decl.return_type.clone(),
                        body: Box::new(decl.body.clone()),
                    };
                    let new_call = Expr::Call {
                        callee: Box::new(arrow),
                        args: std::mem::take(args),
                    };
                    *expr = new_call;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Pass: collapse_let_to_const
//
// Flattening IIFEs that sit in `const x = ...` or `Stmt::Assign` position
// emits `let x;` at the top of a block, then later assigns `x = e;`. When
// `x` is written exactly once at the top level of the same block AND not
// read between the declaration and the assignment, the two can be fused
// into a single `const x = e;` at the assignment site.
// ---------------------------------------------------------------------------

pub fn collapse_let_to_const(program: &mut Program) {
    for stmt in &mut program.body {
        collapse_in_stmt(stmt);
    }
}

fn collapse_in_stmt(stmt: &mut Stmt) {
    match stmt {
        Stmt::Function(f) => match &mut f.body {
            FunctionBody::Block(b) => collapse_in_block(b),
            FunctionBody::Expr(e) => collapse_in_expr(e),
        },
        Stmt::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collapse_in_expr(cond);
            collapse_in_block(then_branch);
            if let Some(eb) = else_branch {
                collapse_in_block(eb);
            }
        }
        Stmt::Block(b) => collapse_in_block(b),
        Stmt::Const(d) => collapse_in_expr(&mut d.init),
        Stmt::Let { init: Some(init), .. } => collapse_in_expr(init),
        Stmt::Assign { value, .. } => collapse_in_expr(value),
        Stmt::Return(Some(e)) | Stmt::Expr(e) => collapse_in_expr(e),
        _ => {}
    }
}

fn collapse_in_expr(expr: &mut Expr) {
    match expr {
        Expr::Arrow { body, .. } => match body.as_mut() {
            FunctionBody::Block(b) => collapse_in_block(b),
            FunctionBody::Expr(e) => collapse_in_expr(e),
        },
        Expr::Call { callee, args } => {
            collapse_in_expr(callee);
            for a in args {
                collapse_in_expr(a);
            }
        }
        Expr::Member { object, .. } => collapse_in_expr(object),
        Expr::Index { object, index } => {
            collapse_in_expr(object);
            collapse_in_expr(index);
        }
        Expr::Unary { expr: e, .. } | Expr::Void(e) => collapse_in_expr(e),
        Expr::Binary { left, right, .. } => {
            collapse_in_expr(left);
            collapse_in_expr(right);
        }
        Expr::Array(items) => {
            for i in items {
                collapse_in_expr(i);
            }
        }
        Expr::Object(props) => {
            for p in props {
                if let ObjectKey::Computed(e) = &mut p.key {
                    collapse_in_expr(e);
                }
                collapse_in_expr(&mut p.value);
            }
        }
        Expr::IfElse { cond, then_expr, else_expr } => {
            collapse_in_expr(cond);
            collapse_in_expr(then_expr);
            collapse_in_expr(else_expr);
        }
        _ => {}
    }
}

fn collapse_in_block(block: &mut Block) {
    for stmt in block.stmts.iter_mut() {
        collapse_in_stmt(stmt);
    }

    let mut i = 0;
    while i < block.stmts.len() {
        let (name, type_ann) = match &block.stmts[i] {
            Stmt::Let { name, init: None, type_ann, .. } => (name.clone(), type_ann.clone()),
            _ => {
                i += 1;
                continue;
            }
        };

        let mut assign_idx: Option<usize> = None;
        let mut safe = true;
        for j in (i + 1)..block.stmts.len() {
            if let Stmt::Assign { name: n, value } = &block.stmts[j] {
                if n == &name {
                    if expr_references_name(value, &name) {
                        safe = false;
                        break;
                    }
                    if assign_idx.is_some() {
                        safe = false;
                        break;
                    }
                    assign_idx = Some(j);
                    continue;
                }
            }
            if assign_idx.is_none() && stmt_references_name(&block.stmts[j], &name) {
                safe = false;
                break;
            }
        }

        if let (true, Some(a_idx)) = (safe, assign_idx) {
            let (n, v) = match std::mem::replace(&mut block.stmts[a_idx], Stmt::Return(None)) {
                Stmt::Assign { name, value } => (name, value),
                _ => unreachable!(),
            };
            block.stmts[a_idx] = Stmt::Const(ConstDecl {
                export: false,
                name: n,
                type_ann,
                init: v,
            });
            block.stmts.remove(i);
            continue;
        }

        i += 1;
    }
}

// ---------------------------------------------------------------------------
// Pass: inline_trivial_consts
//
// Eliminates `const x = y;` aliases by substituting `y` for every later
// reference to `x` in the same scope, then dropping the declaration.
// Only inlines when the RHS is a single identifier — function calls,
// member accesses, and other expressions are left alone (they may have side
// effects or perform work that shouldn't be duplicated).
// ---------------------------------------------------------------------------

pub fn inline_trivial_consts(program: &mut Program) {
    for stmt in &mut program.body {
        inline_in_stmt(stmt);
    }
}

fn inline_in_stmt(stmt: &mut Stmt) {
    match stmt {
        Stmt::Function(func) => match &mut func.body {
            FunctionBody::Expr(expr) => inline_in_expr(expr),
            FunctionBody::Block(block) => inline_in_block(block),
        },
        Stmt::If { cond, then_branch, else_branch } => {
            inline_in_expr(cond);
            inline_in_block(then_branch);
            if let Some(eb) = else_branch {
                inline_in_block(eb);
            }
        }
        Stmt::Block(block) => inline_in_block(block),
        // Expression positions can carry arrow bodies whose blocks contain
        // their own aliases — we must descend so those aliases get detected.
        Stmt::Return(Some(expr)) | Stmt::Expr(expr) => inline_in_expr(expr),
        Stmt::Const(decl) => inline_in_expr(&mut decl.init),
        Stmt::Let { init: Some(init), .. } => inline_in_expr(init),
        Stmt::Assign { value, .. } => inline_in_expr(value),
        _ => {}
    }
}

/// Walk an expression, descending into arrow bodies so that aliases declared
/// inside them are detected by `inline_in_block`.
fn inline_in_expr(expr: &mut Expr) {
    match expr {
        Expr::Arrow { body, .. } => match body.as_mut() {
            FunctionBody::Expr(inner) => inline_in_expr(inner),
            FunctionBody::Block(block) => inline_in_block(block),
        },
        Expr::Call { callee, args } => {
            inline_in_expr(callee);
            for a in args {
                inline_in_expr(a);
            }
        }
        Expr::Member { object, .. } => inline_in_expr(object),
        Expr::Index { object, index } => {
            inline_in_expr(object);
            inline_in_expr(index);
        }
        Expr::Unary { expr: e, .. } | Expr::Void(e) => inline_in_expr(e),
        Expr::Binary { left, right, .. } => {
            inline_in_expr(left);
            inline_in_expr(right);
        }
        Expr::Array(items) => {
            for item in items {
                inline_in_expr(item);
            }
        }
        Expr::Object(props) => {
            for prop in props {
                if let ObjectKey::Computed(e) = &mut prop.key {
                    inline_in_expr(e);
                }
                inline_in_expr(&mut prop.value);
            }
        }
        Expr::IfElse { cond, then_expr, else_expr } => {
            inline_in_expr(cond);
            inline_in_expr(then_expr);
            inline_in_expr(else_expr);
        }
        Expr::Ident(_)
        | Expr::String(_)
        | Expr::Number(_)
        | Expr::Bool(_)
        | Expr::Null
        | Expr::Undefined => {}
    }
}

fn inline_in_block(block: &mut Block) {
    let mut subs: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut new_stmts: Vec<Stmt> = Vec::with_capacity(block.stmts.len());
    for mut stmt in std::mem::take(&mut block.stmts) {
        // Apply current substitutions before inspecting/recursing.
        if !subs.is_empty() {
            inline_subst_stmt(&mut stmt, &subs);
        }
        // After substitution, see if this is a trivial alias.
        // A type annotation on the alias is redundant — the target ident
        // already carries its own type — so we drop the alias regardless.
        if let Stmt::Const(decl) = &stmt {
            if let Expr::Ident(rhs) = &decl.init {
                if !decl.export {
                    subs.insert(decl.name.clone(), rhs.clone());
                    continue; // drop the alias
                }
            }
        }
        // Recurse into nested control flow / functions.
        inline_in_stmt(&mut stmt);
        new_stmts.push(stmt);
    }
    block.stmts = new_stmts;
}

/// Substitute identifiers throughout a statement, INCLUDING inside arrow
/// bodies (they capture from outer scope). Stops at param shadows so we
/// don't substitute params that happen to share a name with a substituted ident.
fn inline_subst_stmt(stmt: &mut Stmt, subs: &std::collections::HashMap<String, String>) {
    match stmt {
        Stmt::Expr(expr) | Stmt::Return(Some(expr)) => inline_subst_expr(expr, subs),
        Stmt::Const(decl) => inline_subst_expr(&mut decl.init, subs),
        Stmt::Let { init: Some(init), .. } => inline_subst_expr(init, subs),
        Stmt::Assign { value, .. } => inline_subst_expr(value, subs),
        Stmt::If { cond, then_branch, else_branch } => {
            inline_subst_expr(cond, subs);
            for s in &mut then_branch.stmts { inline_subst_stmt(s, subs); }
            if let Some(eb) = else_branch {
                for s in &mut eb.stmts { inline_subst_stmt(s, subs); }
            }
        }
        Stmt::Block(b) => {
            for s in &mut b.stmts { inline_subst_stmt(s, subs); }
        }
        _ => {}
    }
}

fn inline_subst_expr(expr: &mut Expr, subs: &std::collections::HashMap<String, String>) {
    match expr {
        Expr::Ident(name) => {
            if let Some(new) = subs.get(name.as_str()) {
                *name = new.clone();
            }
        }
        Expr::Call { callee, args } => {
            inline_subst_expr(callee, subs);
            for a in args { inline_subst_expr(a, subs); }
        }
        Expr::Member { object, .. } => inline_subst_expr(object, subs),
        Expr::Index { object, index } => {
            inline_subst_expr(object, subs);
            inline_subst_expr(index, subs);
        }
        Expr::Unary { expr: e, .. } | Expr::Void(e) => inline_subst_expr(e, subs),
        Expr::Binary { left, right, .. } => {
            inline_subst_expr(left, subs);
            inline_subst_expr(right, subs);
        }
        Expr::Array(items) => {
            for item in items { inline_subst_expr(item, subs); }
        }
        Expr::Object(props) => {
            for prop in props {
                if let ObjectKey::Computed(e) = &mut prop.key {
                    inline_subst_expr(e, subs);
                }
                inline_subst_expr(&mut prop.value, subs);
            }
        }
        Expr::IfElse { cond, then_expr, else_expr } => {
            inline_subst_expr(cond, subs);
            inline_subst_expr(then_expr, subs);
            inline_subst_expr(else_expr, subs);
        }
        Expr::Arrow { params, body, .. } => {
            // Arrow body captures from outer scope, but params shadow.
            // Build a filtered sub map that excludes shadowed names.
            let shadows: std::collections::HashSet<&str> =
                params.iter().map(|p| p.name.as_str()).collect();
            if shadows.is_empty() {
                inline_subst_function_body(body, subs);
            } else {
                let filtered: std::collections::HashMap<String, String> = subs
                    .iter()
                    .filter(|(k, _)| !shadows.contains(k.as_str()))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                inline_subst_function_body(body, &filtered);
            }
        }
        Expr::String(_) | Expr::Number(_) | Expr::Bool(_)
        | Expr::Null | Expr::Undefined => {}
    }
}

fn inline_subst_function_body(body: &mut FunctionBody, subs: &std::collections::HashMap<String, String>) {
    match body {
        FunctionBody::Expr(expr) => inline_subst_expr(expr, subs),
        FunctionBody::Block(block) => {
            for s in &mut block.stmts { inline_subst_stmt(s, subs); }
        }
    }
}

fn lower_stmt_expr_bodies(stmt: &mut Stmt) {
    match stmt {
        Stmt::Function(func) => {
            lower_function_body(&mut func.body);
        }
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => {
            lower_block_expr_bodies(then_branch);
            if let Some(else_branch) = else_branch {
                lower_block_expr_bodies(else_branch);
            }
        }
        Stmt::Block(block) => lower_block_expr_bodies(block),
        Stmt::Const(decl) => lower_expr(&mut decl.init),
        Stmt::Expr(expr) => lower_expr(expr),
        Stmt::Return(expr) => {
            if let Some(expr) = expr {
                lower_expr(expr);
            }
        }
        Stmt::Let { init, .. } => {
            if let Some(init) = init {
                lower_expr(init);
            }
        }
        Stmt::Assign { value, .. } => lower_expr(value),
        Stmt::TypeAlias(_) | Stmt::Interface(_) => {}
    }
}

fn lower_block_expr_bodies(block: &mut Block) {
    for stmt in &mut block.stmts {
        lower_stmt_expr_bodies(stmt);
    }
}

fn lower_function_body(body: &mut FunctionBody) {
    match body {
        FunctionBody::Expr(expr) => {
            lower_expr(expr);
            let expr = std::mem::replace(expr, Box::new(Expr::Undefined));
            *body = FunctionBody::Block(expr_to_block(*expr));
        }
        FunctionBody::Block(block) => lower_block_expr_bodies(block),
    }
}

fn lower_expr(expr: &mut Expr) {
    match expr {
        Expr::Unary { expr, .. } => {
            lower_expr(expr);
        }
        Expr::Binary { left, right, .. } => {
            lower_expr(left);
            lower_expr(right);
        }
        Expr::Call { callee, args } => {
            lower_expr(callee);
            for arg in args {
                lower_expr(arg);
            }
        }
        Expr::Member { object, .. } => lower_expr(object),
        Expr::Index { object, index } => {
            lower_expr(object);
            lower_expr(index);
        }
        Expr::Array(items) => {
            for item in items {
                lower_expr(item);
            }
        }
        Expr::Object(props) => {
            for prop in props {
                if let ObjectKey::Computed(key) = &mut prop.key {
                    lower_expr(key);
                }
                lower_expr(&mut prop.value);
            }
        }
        Expr::Arrow { body, .. } => lower_function_body(body.as_mut()),
        Expr::Void(expr) => lower_expr(expr),
        Expr::IfElse {
            cond,
            then_expr,
            else_expr,
        } => {
            lower_expr(cond);
            lower_expr(then_expr);
            lower_expr(else_expr);
        }
        Expr::Ident(_)
        | Expr::String(_)
        | Expr::Number(_)
        | Expr::Bool(_)
        | Expr::Null
        | Expr::Undefined => {}
    }
}

fn lift_stmt(stmt: &mut Stmt) {
    match stmt {
        Stmt::Function(f) => lift_function_body(&mut f.body),
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => {
            lift_block(then_branch);
            if let Some(else_branch) = else_branch {
                lift_block(else_branch);
            }
        }
        Stmt::Block(block) => lift_block(block),
        Stmt::Const(decl) => lift_expr(&mut decl.init),
        Stmt::Expr(expr) => lift_expr(expr),
        Stmt::Return(expr) => {
            if let Some(expr) = expr {
                lift_expr(expr);
            }
        }
        Stmt::Let { init, .. } => {
            if let Some(init) = init {
                lift_expr(init);
            }
        }
        Stmt::Assign { value, .. } => lift_expr(value),
        Stmt::TypeAlias(_) | Stmt::Interface(_) => {}
    }
}

fn lift_function_body(body: &mut FunctionBody) {
    match body {
        FunctionBody::Expr(expr) => lift_expr(expr),
        FunctionBody::Block(block) => lift_block(block),
    }
}

fn lift_block(block: &mut Block) {
    for stmt in &mut block.stmts {
        lift_stmt(stmt);
    }

    let mut new_stmts = Vec::with_capacity(block.stmts.len());
    for stmt in std::mem::take(&mut block.stmts) {
        match stmt {
            Stmt::Return(Some(Expr::IfElse {
                cond,
                then_expr,
                else_expr,
            })) => {
                new_stmts.push(Stmt::If {
                    cond: *cond,
                    then_branch: expr_to_block(*then_expr),
                    else_branch: Some(expr_to_block(*else_expr)),
                });
            }
            other => new_stmts.push(other),
        }
    }
    block.stmts = new_stmts;
}

// ---------------------------------------------------------------------------
// Pass: flatten_iifes
//
// Rewrites `((x) => { ...stmts; return e; })(v)` → `const x = v; ...stmts; e`
// when the IIFE appears as a statement in a block.  This is a pure JS/TS
// equivalence — the pass knows nothing about Lumo semantics.
// ---------------------------------------------------------------------------

pub fn flatten_iifes(program: &mut Program) {
    for stmt in &mut program.body {
        flatten_stmt(stmt);
    }
}

/// Try to decompose `expr` as an IIFE: `Call { callee: Arrow { params, Block(stmts) }, args }`.
/// Returns `(params, body_stmts, args)` on success, consuming the pieces out of `expr`.
///
/// If a param name appears free in any arg, the param is alpha-renamed to a
/// fresh name (and its references in the body are updated) before extraction.
/// Without this, flattening would lift `const x = <expr using x>` into the
/// outer scope and hit JS's TDZ. Example:
/// `((__caps) => body)(Object.assign({}, __caps, ...))` — naively flattened
/// would shadow the outer `__caps` with an uninitialized `const __caps`.
fn take_iife(expr: &mut Expr) -> Option<(Vec<Param>, Vec<Stmt>, Vec<Expr>)> {
    let Expr::Call { callee, args } = expr else {
        return None;
    };
    let Expr::Arrow { params, body, .. } = callee.as_mut() else {
        return None;
    };
    let FunctionBody::Block(block) = body.as_mut() else {
        return None;
    };
    if params.len() != args.len() {
        return None;
    }
    for param in params.iter_mut() {
        let conflicts = args.iter().any(|a| expr_references_name(a, &param.name));
        if !conflicts {
            continue;
        }
        let fresh = fresh_iife_param(&param.name);
        let old = std::mem::replace(&mut param.name, fresh.clone());
        for stmt in block.stmts.iter_mut() {
            rename_free_in_stmt(stmt, &old, &fresh);
        }
    }
    let params = std::mem::take(params);
    let body_stmts = std::mem::take(&mut block.stmts);
    let args = std::mem::take(args);
    Some((params, body_stmts, args))
}

fn fresh_iife_param(base: &str) -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{base}__iife_{n}")
}

/// Rename every free reference to `old` in `stmt` to `new`. A reference is
/// "free" if it is not shadowed by a nested binding (Arrow param, Function
/// param, or same-block `let`/`const` with the same name).
fn rename_free_in_stmt(stmt: &mut Stmt, old: &str, new: &str) {
    match stmt {
        Stmt::Expr(e) | Stmt::Return(Some(e)) => rename_free_in_expr(e, old, new),
        Stmt::Const(decl) => rename_free_in_expr(&mut decl.init, old, new),
        Stmt::Let { init: Some(init), .. } => rename_free_in_expr(init, old, new),
        Stmt::Assign { name, value } => {
            if name == old {
                *name = new.to_owned();
            }
            rename_free_in_expr(value, old, new);
        }
        Stmt::If { cond, then_branch, else_branch } => {
            rename_free_in_expr(cond, old, new);
            rename_free_in_block(then_branch, old, new);
            if let Some(eb) = else_branch {
                rename_free_in_block(eb, old, new);
            }
        }
        Stmt::Block(b) => rename_free_in_block(b, old, new),
        Stmt::Function(f) => {
            if f.params.iter().any(|p| p.name == old) {
                return;
            }
            match &mut f.body {
                FunctionBody::Expr(e) => rename_free_in_expr(e, old, new),
                FunctionBody::Block(b) => rename_free_in_block(b, old, new),
            }
        }
        _ => {}
    }
}

/// Walk `block`'s statements in order; stop renaming once a same-block
/// `let`/`const` binds `old` (it shadows from declaration onward).
fn rename_free_in_block(block: &mut Block, old: &str, new: &str) {
    for stmt in block.stmts.iter_mut() {
        rename_free_in_stmt(stmt, old, new);
        let shadowed = match stmt {
            Stmt::Const(decl) => decl.name == old,
            Stmt::Let { name, .. } => name == old,
            _ => false,
        };
        if shadowed {
            return;
        }
    }
}

fn rename_free_in_expr(expr: &mut Expr, old: &str, new: &str) {
    match expr {
        Expr::Ident(name) => {
            if name == old {
                *name = new.to_owned();
            }
        }
        Expr::Call { callee, args } => {
            rename_free_in_expr(callee, old, new);
            for a in args {
                rename_free_in_expr(a, old, new);
            }
        }
        Expr::Member { object, .. } => rename_free_in_expr(object, old, new),
        Expr::Index { object, index } => {
            rename_free_in_expr(object, old, new);
            rename_free_in_expr(index, old, new);
        }
        Expr::Unary { expr: e, .. } | Expr::Void(e) => rename_free_in_expr(e, old, new),
        Expr::Binary { left, right, .. } => {
            rename_free_in_expr(left, old, new);
            rename_free_in_expr(right, old, new);
        }
        Expr::Array(items) => {
            for item in items {
                rename_free_in_expr(item, old, new);
            }
        }
        Expr::Object(props) => {
            for prop in props {
                if let ObjectKey::Computed(e) = &mut prop.key {
                    rename_free_in_expr(e, old, new);
                }
                rename_free_in_expr(&mut prop.value, old, new);
            }
        }
        Expr::IfElse { cond, then_expr, else_expr } => {
            rename_free_in_expr(cond, old, new);
            rename_free_in_expr(then_expr, old, new);
            rename_free_in_expr(else_expr, old, new);
        }
        Expr::Arrow { params, body, .. } => {
            if params.iter().any(|p| p.name == old) {
                return;
            }
            match body.as_mut() {
                FunctionBody::Expr(e) => rename_free_in_expr(e, old, new),
                FunctionBody::Block(b) => rename_free_in_block(b, old, new),
            }
        }
        Expr::String(_) | Expr::Number(_) | Expr::Bool(_) | Expr::Null | Expr::Undefined => {}
    }
}

/// Returns `true` if `expr` references the given identifier as a free name.
fn expr_references_name(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Ident(n) => n == name,
        Expr::Call { callee, args } => {
            expr_references_name(callee, name)
                || args.iter().any(|a| expr_references_name(a, name))
        }
        Expr::Member { object, .. } => expr_references_name(object, name),
        Expr::Index { object, index } => {
            expr_references_name(object, name) || expr_references_name(index, name)
        }
        Expr::Unary { expr, .. } | Expr::Void(expr) => expr_references_name(expr, name),
        Expr::Binary { left, right, .. } => {
            expr_references_name(left, name) || expr_references_name(right, name)
        }
        Expr::Array(items) => items.iter().any(|i| expr_references_name(i, name)),
        Expr::Object(props) => props.iter().any(|p| {
            let key_refs = matches!(&p.key, ObjectKey::Computed(e) if expr_references_name(e, name));
            key_refs || expr_references_name(&p.value, name)
        }),
        Expr::IfElse { cond, then_expr, else_expr } => {
            expr_references_name(cond, name)
                || expr_references_name(then_expr, name)
                || expr_references_name(else_expr, name)
        }
        Expr::Arrow { params, body, .. } => {
            // Name is shadowed if bound by params
            if params.iter().any(|p| p.name == name) {
                return false;
            }
            match body.as_ref() {
                FunctionBody::Expr(e) => expr_references_name(e, name),
                FunctionBody::Block(b) => b.stmts.iter().any(|s| stmt_references_name(s, name)),
            }
        }
        Expr::String(_) | Expr::Number(_) | Expr::Bool(_)
        | Expr::Null | Expr::Undefined => false,
    }
}

fn stmt_references_name(stmt: &Stmt, name: &str) -> bool {
    match stmt {
        Stmt::Expr(e) | Stmt::Return(Some(e)) => expr_references_name(e, name),
        Stmt::Const(decl) => expr_references_name(&decl.init, name),
        Stmt::Let { init: Some(init), .. } => expr_references_name(init, name),
        Stmt::Assign { value, .. } => expr_references_name(value, name),
        Stmt::If { cond, then_branch, else_branch } => {
            expr_references_name(cond, name)
                || then_branch.stmts.iter().any(|s| stmt_references_name(s, name))
                || else_branch
                    .as_ref()
                    .map_or(false, |eb| eb.stmts.iter().any(|s| stmt_references_name(s, name)))
        }
        Stmt::Block(b) => b.stmts.iter().any(|s| stmt_references_name(s, name)),
        _ => false,
    }
}

/// Build `const` bindings from IIFE params + args.
/// Skips identity bindings (`const x = x`) which would cause a TDZ error.
fn param_const_stmts(params: Vec<Param>, args: Vec<Expr>) -> Vec<Stmt> {
    params
        .into_iter()
        .zip(args)
        .filter_map(|(p, a)| {
            // Skip `const x = x` — it's a no-op and causes TDZ in JS
            if let Expr::Ident(ref name) = a {
                if name == &p.name {
                    return None;
                }
            }
            Some(Stmt::Const(ConstDecl {
                export: false,
                name: p.name,
                type_ann: p.type_ann,
                init: a,
            }))
        })
        .collect()
}

/// Try to flatten an IIFE in the given statement.
/// Returns `Some(replacement_stmts)` if flattened, `None` otherwise.
fn try_flatten(stmt: &mut Stmt) -> Option<Vec<Stmt>> {
    match stmt {
        Stmt::Return(Some(expr)) => {
            let (params, body_stmts, args) = take_iife(expr)?;
            let mut out = param_const_stmts(params, args);
            out.extend(body_stmts);
            Some(out)
        }
        Stmt::Expr(expr) => {
            let (params, mut body_stmts, args) = take_iife(expr)?;
            // Rewrite final `return e` → `Stmt::Expr(e)`, drop bare `return`
            if let Some(last) = body_stmts.last_mut() {
                if let Stmt::Return(ret_expr) = last {
                    *last = match ret_expr.take() {
                        Some(e) => Stmt::Expr(e),
                        None => Stmt::Expr(Expr::Undefined),
                    };
                }
            }
            let mut out = param_const_stmts(params, args);
            out.extend(body_stmts);
            Some(out)
        }
        Stmt::Const(decl) => {
            let (params, mut body_stmts, args) = take_iife(&mut decl.init)?;
            let outer_name = std::mem::take(&mut decl.name);
            let outer_export = decl.export;
            let outer_type_ann = decl.type_ann.take();
            // Rewrite ALL leaf `return e` → `<outer_name> = e` in the body.
            // This handles both simple final returns and returns inside if/else branches.
            if !rewrite_returns_to_assign(&mut body_stmts, &outer_name) {
                return None; // bail if we can't safely rewrite
            }
            // Insert `let outer_name;` at the top
            let mut out = vec![Stmt::Let {
                name: outer_name.clone(),
                export: outer_export,
                type_ann: outer_type_ann,
                init: None,
            }];
            out.extend(param_const_stmts(params, args));
            out.extend(body_stmts);
            Some(out)
        }
        Stmt::Assign { name, value } => {
            // `outer = ((x) => { ...; return e; })(v)` → `const x = v; ...; outer = e`
            // `outer` is already declared elsewhere (caller's `let outer;`), so no
            // declaration is inserted — just rewrite leaf returns to assignments.
            let (params, mut body_stmts, args) = take_iife(value)?;
            let outer_name = std::mem::take(name);
            if !rewrite_returns_to_assign(&mut body_stmts, &outer_name) {
                return None;
            }
            let mut out = param_const_stmts(params, args);
            out.extend(body_stmts);
            Some(out)
        }
        _ => None,
    }
}

/// Rewrite all leaf `return expr` in a statement list to `name = expr`.
/// Returns true if successful, false if there's a `return` that can't be safely rewritten.
fn rewrite_returns_to_assign(stmts: &mut Vec<Stmt>, name: &str) -> bool {
    if stmts.is_empty() {
        return false;
    }
    let last_idx = stmts.len() - 1;
    let last = &mut stmts[last_idx];
    match last {
        Stmt::Return(ret_expr) => {
            *last = Stmt::Assign {
                name: name.to_owned(),
                value: ret_expr.take().unwrap_or(Expr::Undefined),
            };
            true
        }
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => {
            let ok_then = rewrite_returns_to_assign(&mut then_branch.stmts, name);
            let ok_else = if let Some(eb) = else_branch {
                rewrite_returns_to_assign(&mut eb.stmts, name)
            } else {
                true // no else branch is OK
            };
            ok_then && ok_else
        }
        _ => false, // last stmt is not a return or if/else — bail
    }
}

fn flatten_block(block: &mut Block) {
    flatten_block_with_params(block, &[]);
}

fn flatten_block_with_params(block: &mut Block, enclosing_names: &[String]) {
    // Phase 1: inline IIFEs at statement level
    let mut i = 0;
    while i < block.stmts.len() {
        if let Some(replacement) = try_flatten(&mut block.stmts[i]) {
            let _ = block.stmts.remove(i);
            for (j, s) in replacement.into_iter().enumerate() {
                block.stmts.insert(i + j, s);
            }
            // Don't advance — re-check for nested IIFEs in newly inserted stmts
            continue;
        }
        i += 1;
    }

    // Phase 2: deduplicate shadowed const names (seeded with enclosing params)
    dedup_const_names_with_params(block, enclosing_names);

    // Phase 3: recurse into sub-structures, threading enclosing names for
    // nested blocks (e.g. if-branches) that share the same scope for TDZ.
    for stmt in &mut block.stmts {
        flatten_stmt_with_enclosing(stmt, enclosing_names);
    }
}

/// After IIFE flattening, the same `const` name may appear multiple times in a
/// block (from Lumo `let s = … in let s = … in …` shadowing). JS forbids
/// duplicate `const` in a single scope, so we rename later occurrences and
/// rewrite all subsequent references.
fn dedup_const_names(block: &mut Block) {
    dedup_const_names_with_params(block, &[]);
}

fn dedup_const_names_with_params(block: &mut Block, param_names: &[String]) {
    use std::collections::{HashMap, HashSet};

    let mut declared: HashSet<String> = HashSet::new();
    // Pre-seed with function/arrow parameter names so `const s = ...` is
    // treated as a duplicate when `s` is already a param.
    for p in param_names {
        declared.insert(p.clone());
    }
    let mut rename_map: HashMap<String, String> = HashMap::new();
    let mut counter: usize = 0;

    for i in 0..block.stmts.len() {
        // Apply pending renames to expressions in this statement
        if !rename_map.is_empty() {
            rename_idents_in_stmt(&mut block.stmts[i], &rename_map);
        }

        // If this is a const or let, check for duplicate name
        let decl_name: Option<String> = match &block.stmts[i] {
            Stmt::Const(decl) => Some(decl.name.clone()),
            Stmt::Let { name, .. } => Some(name.clone()),
            _ => None,
        };
        if let Some(orig) = decl_name {
            if declared.contains(&orig) {
                let fresh = loop {
                    let candidate = format!("{}_{}", orig, counter);
                    counter += 1;
                    if !declared.contains(&candidate) {
                        break candidate;
                    }
                };
                // Rename the declaration itself
                match &mut block.stmts[i] {
                    Stmt::Const(decl) => decl.name = fresh.clone(),
                    Stmt::Let { name, .. } => *name = fresh.clone(),
                    _ => {}
                }
                declared.insert(fresh.clone());
                rename_map.insert(orig, fresh);
            } else {
                declared.insert(orig.clone());
                // This declaration introduces a fresh binding that overrides any
                // prior rename for this name.
                rename_map.remove(&orig);
            }
        }
    }
}

fn rename_idents_in_stmt(stmt: &mut Stmt, map: &std::collections::HashMap<String, String>) {
    match stmt {
        Stmt::Expr(expr) | Stmt::Return(Some(expr)) => rename_idents_in_expr(expr, map),
        Stmt::Const(decl) => rename_idents_in_expr(&mut decl.init, map),
        Stmt::Let { init: Some(init), .. } => rename_idents_in_expr(init, map),
        Stmt::Assign { name, value } => {
            // Rename the LHS target if it has been remapped (e.g. `let s` → `let s_0`
            // means subsequent `s = ...` must become `s_0 = ...`).
            if let Some(new_name) = map.get(name.as_str()) {
                *name = new_name.clone();
            }
            rename_idents_in_expr(value, map);
        }
        Stmt::If { cond, then_branch, else_branch } => {
            rename_idents_in_expr(cond, map);
            for s in &mut then_branch.stmts { rename_idents_in_stmt(s, map); }
            if let Some(eb) = else_branch {
                for s in &mut eb.stmts { rename_idents_in_stmt(s, map); }
            }
        }
        Stmt::Block(b) => {
            for s in &mut b.stmts { rename_idents_in_stmt(s, map); }
        }
        _ => {}
    }
}

fn rename_idents_in_expr(expr: &mut Expr, map: &std::collections::HashMap<String, String>) {
    match expr {
        Expr::Ident(name) => {
            if let Some(new) = map.get(name.as_str()) {
                *name = new.clone();
            }
        }
        Expr::Call { callee, args } => {
            rename_idents_in_expr(callee, map);
            for a in args { rename_idents_in_expr(a, map); }
        }
        Expr::Member { object, .. } => rename_idents_in_expr(object, map),
        Expr::Index { object, index } => {
            rename_idents_in_expr(object, map);
            rename_idents_in_expr(index, map);
        }
        Expr::Unary { expr: e, .. } | Expr::Void(e) => rename_idents_in_expr(e, map),
        Expr::Binary { left, right, .. } => {
            rename_idents_in_expr(left, map);
            rename_idents_in_expr(right, map);
        }
        Expr::Array(items) => {
            for item in items { rename_idents_in_expr(item, map); }
        }
        Expr::Object(props) => {
            for prop in props {
                if let ObjectKey::Computed(e) = &mut prop.key {
                    rename_idents_in_expr(e, map);
                }
                rename_idents_in_expr(&mut prop.value, map);
            }
        }
        Expr::IfElse { cond, then_expr, else_expr } => {
            rename_idents_in_expr(cond, map);
            rename_idents_in_expr(then_expr, map);
            rename_idents_in_expr(else_expr, map);
        }
        // Don't recurse into Arrow bodies — they have their own scope
        Expr::Arrow { .. } => {}
        Expr::String(_) | Expr::Number(_) | Expr::Bool(_)
        | Expr::Null | Expr::Undefined => {}
    }
}

fn flatten_stmt(stmt: &mut Stmt) {
    flatten_stmt_with_enclosing(stmt, &[]);
}

/// Flatten a statement, propagating `enclosing_names` (function/arrow params
/// plus already-declared consts) into nested blocks so the dedup pass can
/// detect TDZ-inducing shadowing in `if`/`else` branches.
fn flatten_stmt_with_enclosing(stmt: &mut Stmt, enclosing_names: &[String]) {
    match stmt {
        Stmt::Function(f) => {
            // New function scope — only its own params matter, not the parent's.
            let param_names: Vec<String> = f.params.iter().map(|p| p.name.clone()).collect();
            flatten_function_body_with_params(&mut f.body, &param_names);
        }
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => {
            // if/else branches share the enclosing function's scope for TDZ
            flatten_block_with_params(then_branch, enclosing_names);
            if let Some(eb) = else_branch {
                flatten_block_with_params(eb, enclosing_names);
            }
        }
        Stmt::Block(block) => flatten_block_with_params(block, enclosing_names),
        Stmt::Const(decl) => flatten_expr_arrows(&mut decl.init),
        Stmt::Let { init: Some(init), .. } => flatten_expr_arrows(init),
        Stmt::Assign { value, .. } => flatten_expr_arrows(value),
        Stmt::Expr(expr) | Stmt::Return(Some(expr)) => flatten_expr_arrows(expr),
        Stmt::Return(None) | Stmt::Let { .. } | Stmt::TypeAlias(_) | Stmt::Interface(_) => {}
    }
}

fn flatten_function_body(body: &mut FunctionBody) {
    flatten_function_body_with_params(body, &[]);
}

fn flatten_function_body_with_params(body: &mut FunctionBody, param_names: &[String]) {
    match body {
        FunctionBody::Expr(expr) => flatten_expr_arrows(expr),
        FunctionBody::Block(block) => flatten_block_with_params(block, param_names),
    }
}

/// Recurse into sub-expressions to flatten IIFEs inside nested arrow bodies.
fn flatten_expr_arrows(expr: &mut Expr) {
    match expr {
        Expr::Arrow { params, body, .. } => {
            let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            flatten_function_body_with_params(body, &param_names);
        }
        Expr::Call { callee, args } => {
            flatten_expr_arrows(callee);
            for arg in args {
                flatten_expr_arrows(arg);
            }
        }
        Expr::Unary { expr, .. } | Expr::Void(expr) => flatten_expr_arrows(expr),
        Expr::Binary { left, right, .. } => {
            flatten_expr_arrows(left);
            flatten_expr_arrows(right);
        }
        Expr::Member { object, .. } => flatten_expr_arrows(object),
        Expr::Index { object, index } => {
            flatten_expr_arrows(object);
            flatten_expr_arrows(index);
        }
        Expr::Array(items) => {
            for item in items {
                flatten_expr_arrows(item);
            }
        }
        Expr::Object(props) => {
            for prop in props {
                if let ObjectKey::Computed(key) = &mut prop.key {
                    flatten_expr_arrows(key);
                }
                flatten_expr_arrows(&mut prop.value);
            }
        }
        Expr::IfElse {
            cond,
            then_expr,
            else_expr,
        } => {
            flatten_expr_arrows(cond);
            flatten_expr_arrows(then_expr);
            flatten_expr_arrows(else_expr);
        }
        Expr::Ident(_)
        | Expr::String(_)
        | Expr::Number(_)
        | Expr::Bool(_)
        | Expr::Null
        | Expr::Undefined => {}
    }
}

fn lift_expr(expr: &mut Expr) {
    match expr {
        Expr::Unary { expr, .. } => {
            lift_expr(expr);
        }
        Expr::Binary { left, right, .. } => {
            lift_expr(left);
            lift_expr(right);
        }
        Expr::Call { callee, args } => {
            lift_expr(callee);
            for arg in args {
                lift_expr(arg);
            }
        }
        Expr::Member { object, .. } => lift_expr(object),
        Expr::Index { object, index } => {
            lift_expr(object);
            lift_expr(index);
        }
        Expr::Array(items) => {
            for item in items {
                lift_expr(item);
            }
        }
        Expr::Object(props) => {
            for prop in props {
                if let ObjectKey::Computed(key) = &mut prop.key {
                    lift_expr(key);
                }
                lift_expr(&mut prop.value);
            }
        }
        Expr::Arrow { body, .. } => lift_function_body(body.as_mut()),
        Expr::Void(expr) => lift_expr(expr),
        Expr::IfElse {
            cond,
            then_expr,
            else_expr,
        } => {
            lift_expr(cond);
            lift_expr(then_expr);
            lift_expr(else_expr);
        }
        Expr::Ident(_)
        | Expr::String(_)
        | Expr::Number(_)
        | Expr::Bool(_)
        | Expr::Null
        | Expr::Undefined => {}
    }
}
