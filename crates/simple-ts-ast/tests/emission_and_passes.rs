use simple_ts_ast::{
    expr_to_block, lower_expression_bodies, return_lifting, BinaryOp, Block, ConstDecl, EmitTarget,
    Emitter, Expr, FunctionBody, FunctionDecl, ObjectKey, ObjectProp, Param, Program, Stmt, TsType,
    UnaryOp,
};

#[test]
fn emits_ts_js_and_dts() {
    let mut add = FunctionDecl::new(
        "add",
        FunctionBody::Expr(Box::new(Expr::Binary {
            left: Box::new(Expr::Ident("a".into())),
            op: BinaryOp::Add,
            right: Box::new(Expr::Ident("b".into())),
        })),
    );
    add.export = true;
    add.params = vec![
        Param::new("a").with_type(TsType::Number),
        Param::new("b").with_type(TsType::Number),
    ];
    add.return_type = Some(TsType::Number);

    let program = Program::new(vec![
        Stmt::Function(add),
        Stmt::Const(ConstDecl {
            export: true,
            name: "label".into(),
            type_ann: Some(TsType::String),
            init: Expr::String("ok".into()),
        }),
    ]);

    let ts = Emitter::default().emit_program(&program, EmitTarget::TypeScript);
    let js = Emitter::default().emit_program(&program, EmitTarget::JavaScript);
    let dts = Emitter::default().emit_program(&program, EmitTarget::TypeScriptDefinition);

    assert!(ts.contains("function add(a: number, b: number): number"));
    assert!(js.contains("function add(a, b)"));
    assert!(!js.contains(": number"));
    assert!(dts.contains("export declare function add(a: number, b: number): number;"));
    assert!(dts.contains("export declare const label: string;"));
}

#[test]
fn lower_and_lift_for_readability() {
    let mut program = Program::new(vec![Stmt::Function(FunctionDecl {
        export: false,
        name: "pick".into(),
        type_params: Vec::new(),
        params: vec![Param::new("cond").with_type(TsType::Boolean)],
        return_type: Some(TsType::Number),
        body: FunctionBody::Expr(Box::new(Expr::IfElse {
            cond: Box::new(Expr::Ident("cond".into())),
            then_expr: Box::new(Expr::Number(1.0)),
            else_expr: Box::new(Expr::Number(2.0)),
        })),
        inline_always: false,
    })]);

    lower_expression_bodies(&mut program);
    return_lifting(&mut program);

    let js = Emitter::default().emit_program(&program, EmitTarget::JavaScript);
    assert!(js.contains("if (cond)"));
    assert!(js.contains("return 1;"));
    assert!(js.contains("return 2;"));
    assert!(!js.contains("?"));
}

#[test]
fn expr_to_block_wraps_return() {
    let block = expr_to_block(Expr::Ident("value".into()));
    assert_eq!(
        block,
        Block::new(vec![Stmt::Return(Some(Expr::Ident("value".into())))])
    );
}

#[test]
fn emits_object_array_and_index_expressions() {
    let program = Program::new(vec![Stmt::Const(ConstDecl {
        export: true,
        name: "Bool".into(),
        type_ann: None,
        init: Expr::Object(vec![
            ObjectProp {
                key: ObjectKey::String("true".into()),
                value: Expr::Arrow {
                    params: Vec::new(),
                    return_type: None,
                    body: Box::new(FunctionBody::Expr(Box::new(Expr::Object(vec![
                        ObjectProp {
                            key: ObjectKey::Computed(Box::new(Expr::Ident("LUMO_TAG".into()))),
                            value: Expr::String("true".into()),
                        },
                    ])))),
                },
            },
            ObjectProp {
                key: ObjectKey::String("list".into()),
                value: Expr::Array(vec![Expr::Number(1.0), Expr::Number(2.0)]),
            },
        ]),
    })]);
    let ts = Emitter::default().emit_program(&program, EmitTarget::TypeScript);
    assert!(
        ts.contains("\"true\": () => { [LUMO_TAG]: \"true\" }"),
        "{ts}"
    );
    assert!(ts.contains("\"list\": [1, 2]"), "{ts}");
}

#[test]
fn call_with_arrow_callee_is_parenthesized() {
    let program = Program::new(vec![Stmt::Expr(Expr::Call {
        callee: Box::new(Expr::Arrow {
            params: vec![Param::new("x")],
            return_type: None,
            body: Box::new(FunctionBody::Expr(Box::new(Expr::Ident("x".into())))),
        }),
        args: vec![Expr::String("ok".into())],
    })]);
    let js = Emitter::default().emit_program(&program, EmitTarget::JavaScript);
    assert!(js.contains("((x) => x)(\"ok\")"), "{js}");
}

#[test]
fn emits_unary_and_exponent_expressions() {
    let program = Program::new(vec![
        Stmt::Expr(Expr::Unary {
            op: UnaryOp::Not,
            expr: Box::new(Expr::Ident("flag".into())),
        }),
        Stmt::Expr(Expr::Binary {
            left: Box::new(Expr::Ident("lhs".into())),
            op: BinaryOp::Exp,
            right: Box::new(Expr::Ident("rhs".into())),
        }),
    ]);

    let js = Emitter::default().emit_program(&program, EmitTarget::JavaScript);
    assert!(js.contains("(!flag);"), "{js}");
    assert!(js.contains("(lhs ** rhs);"), "{js}");
}
