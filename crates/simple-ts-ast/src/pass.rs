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
    let params = std::mem::take(params);
    let body_stmts = std::mem::take(&mut block.stmts);
    let args = std::mem::take(args);
    Some((params, body_stmts, args))
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
            // Rewrite final `return e` → `const <outer_name> = e`
            let outer_name = std::mem::take(&mut decl.name);
            let outer_export = decl.export;
            let outer_type_ann = decl.type_ann.take();
            if let Some(last) = body_stmts.last_mut() {
                if let Stmt::Return(ret_expr) = last {
                    *last = Stmt::Const(ConstDecl {
                        export: outer_export,
                        name: outer_name,
                        type_ann: outer_type_ann,
                        init: ret_expr.take().unwrap_or(Expr::Undefined),
                    });
                }
            }
            let mut out = param_const_stmts(params, args);
            out.extend(body_stmts);
            Some(out)
        }
        _ => None,
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

        // If this is a const, check for duplicate name
        if let Stmt::Const(decl) = &mut block.stmts[i] {
            if declared.contains(&decl.name) {
                let orig = decl.name.clone();
                let fresh = loop {
                    let candidate = format!("{}_{}", orig, counter);
                    counter += 1;
                    if !declared.contains(&candidate) {
                        break candidate;
                    }
                };
                decl.name = fresh.clone();
                declared.insert(fresh.clone());
                rename_map.insert(orig, fresh);
            } else {
                declared.insert(decl.name.clone());
                // This declaration introduces a fresh binding that overrides any
                // prior rename for this name.
                rename_map.remove(&decl.name);
            }
        }
    }
}

fn rename_idents_in_stmt(stmt: &mut Stmt, map: &std::collections::HashMap<String, String>) {
    match stmt {
        Stmt::Expr(expr) | Stmt::Return(Some(expr)) => rename_idents_in_expr(expr, map),
        Stmt::Const(decl) => rename_idents_in_expr(&mut decl.init, map),
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
        Stmt::Expr(expr) | Stmt::Return(Some(expr)) => flatten_expr_arrows(expr),
        Stmt::Return(None) | Stmt::TypeAlias(_) | Stmt::Interface(_) => {}
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
