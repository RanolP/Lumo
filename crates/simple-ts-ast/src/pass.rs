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
fn param_const_stmts(params: Vec<Param>, args: Vec<Expr>) -> Vec<Stmt> {
    params
        .into_iter()
        .zip(args)
        .map(|(p, a)| {
            Stmt::Const(ConstDecl {
                export: false,
                name: p.name,
                type_ann: p.type_ann,
                init: a,
            })
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

    // Phase 2: recurse into sub-structures
    for stmt in &mut block.stmts {
        flatten_stmt(stmt);
    }
}

fn flatten_stmt(stmt: &mut Stmt) {
    match stmt {
        Stmt::Function(f) => flatten_function_body(&mut f.body),
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => {
            flatten_block(then_branch);
            if let Some(eb) = else_branch {
                flatten_block(eb);
            }
        }
        Stmt::Block(block) => flatten_block(block),
        Stmt::Const(decl) => flatten_expr_arrows(&mut decl.init),
        Stmt::Expr(expr) | Stmt::Return(Some(expr)) => flatten_expr_arrows(expr),
        Stmt::Return(None) | Stmt::TypeAlias(_) | Stmt::Interface(_) => {}
    }
}

fn flatten_function_body(body: &mut FunctionBody) {
    match body {
        FunctionBody::Expr(expr) => flatten_expr_arrows(expr),
        FunctionBody::Block(block) => flatten_block(block),
    }
}

/// Recurse into sub-expressions to flatten IIFEs inside nested arrow bodies.
fn flatten_expr_arrows(expr: &mut Expr) {
    match expr {
        Expr::Arrow { body, .. } => flatten_function_body(body),
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
