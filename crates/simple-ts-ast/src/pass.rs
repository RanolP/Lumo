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
