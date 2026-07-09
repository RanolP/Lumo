use crate::ast::*;
use crate::pass::flatten_iifes;

// ---------------------------------------------------------------------------
// Helper: run flatten_iifes (which internally runs dedup_const_names) on a
// single-function program and return the body stmts of that function.
// ---------------------------------------------------------------------------

fn run_dedup(body_stmts: Vec<Stmt>) -> Vec<Stmt> {
    let func = Stmt::Function(FunctionDecl {
        export: false,
        name: "f".to_owned(),
        type_params: vec![],
        params: vec![],
        return_type: None,
        body: FunctionBody::Block(Block::new(body_stmts)),
        inline_always: false,
    });
    let mut program = Program::new(vec![func]);
    flatten_iifes(&mut program);
    match program.body.remove(0) {
        Stmt::Function(f) => match f.body {
            FunctionBody::Block(b) => b.stmts,
            _ => panic!("expected block body"),
        },
        _ => panic!("expected function"),
    }
}

// ---------------------------------------------------------------------------
// Test: duplicate `let` declarations in the same block are renamed
// ---------------------------------------------------------------------------

#[test]
fn dedup_let_renames_second_declaration() {
    // Simulates the pattern produced by IIFE flattening:
    //   let s;          ← first declaration
    //   s = foo();
    //   let s;          ← duplicate — would be SyntaxError in JS
    //   s = bar(s);
    let stmts = vec![
        Stmt::Let { name: "s".to_owned(), export: false, type_ann: None, init: None },
        Stmt::Assign { name: "s".to_owned(), value: Expr::Call {
            callee: Box::new(Expr::Ident("foo".to_owned())),
            args: vec![],
        }},
        Stmt::Let { name: "s".to_owned(), export: false, type_ann: None, init: None },
        Stmt::Assign { name: "s".to_owned(), value: Expr::Call {
            callee: Box::new(Expr::Ident("bar".to_owned())),
            args: vec![Expr::Ident("s".to_owned())],
        }},
    ];

    let result = run_dedup(stmts);

    // First `let s` should remain unchanged
    assert!(
        matches!(&result[0], Stmt::Let { name, .. } if name == "s"),
        "first let should still be named 's', got: {:?}", &result[0]
    );

    // Second `let s` should be renamed to `s_0`
    assert!(
        matches!(&result[2], Stmt::Let { name, .. } if name == "s_0"),
        "second let should be renamed to 's_0', got: {:?}", &result[2]
    );

    // Assignment after second let: LHS should use renamed `s_0`
    assert!(
        matches!(&result[3], Stmt::Assign { name, .. } if name == "s_0"),
        "assign after second let should target 's_0', got: {:?}", &result[3]
    );

    // RHS of third assignment uses `s` (refers to second binding which was renamed):
    // `bar(s)` should now be `bar(s_0)`
    if let Stmt::Assign { value: Expr::Call { args, .. }, .. } = &result[3] {
        assert_eq!(args[0], Expr::Ident("s_0".to_owned()),
            "argument referencing second 's' should be rewritten to 's_0'");
    } else {
        panic!("expected assign with call, got: {:?}", &result[3]);
    }
}

// ---------------------------------------------------------------------------
// Test: mixing const and let with the same name are treated as the same namespace
// ---------------------------------------------------------------------------

#[test]
fn dedup_let_and_const_share_namespace() {
    // let x; const x = 1;  ← const shadows let — must rename
    let stmts = vec![
        Stmt::Let { name: "x".to_owned(), export: false, type_ann: None, init: None },
        Stmt::Const(ConstDecl {
            export: false,
            name: "x".to_owned(),
            type_ann: None,
            init: Expr::Number(1.0),
        }),
    ];

    let result = run_dedup(stmts);

    assert!(
        matches!(&result[0], Stmt::Let { name, .. } if name == "x"),
        "let x should remain 'x'"
    );
    assert!(
        matches!(&result[1], Stmt::Const(d) if d.name == "x_0"),
        "const x after let x should be renamed to 'x_0', got: {:?}", &result[1]
    );
}

// ---------------------------------------------------------------------------
// Test: non-duplicate lets are left unchanged
// ---------------------------------------------------------------------------

#[test]
fn dedup_non_duplicate_lets_unchanged() {
    let stmts = vec![
        Stmt::Let { name: "a".to_owned(), export: false, type_ann: None, init: None },
        Stmt::Let { name: "b".to_owned(), export: false, type_ann: None, init: None },
        Stmt::Assign { name: "a".to_owned(), value: Expr::Number(1.0) },
    ];

    let result = run_dedup(stmts);

    assert!(matches!(&result[0], Stmt::Let { name, .. } if name == "a"));
    assert!(matches!(&result[1], Stmt::Let { name, .. } if name == "b"));
    assert!(matches!(&result[2], Stmt::Assign { name, .. } if name == "a"));
}
