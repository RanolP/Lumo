use lumo_compiler::lexer::{lex, Keyword, Symbol, TokenKind};

struct Case {
    name: &'static str,
    input: &'static str,
    tokens: &'static [&'static str],
    errors: &'static [&'static str],
}

#[test]
fn lexer_golden_cases() {
    let cases: &[Case] = &[
        Case {
            name: "kw_data",
            input: "data",
            tokens: &["kw(data)@0..4"],
            errors: &[],
        },
        Case {
            name: "kw_fn",
            input: "fn",
            tokens: &["kw(fn)@0..2"],
            errors: &[],
        },
        Case {
            name: "kw_let_in",
            input: "let x in y",
            tokens: &[
                "kw(let)@0..3",
                "ident(x)@4..5",
                "kw(in)@6..8",
                "ident(y)@9..10",
            ],
            errors: &[],
        },
        Case {
            name: "kw_produce",
            input: "produce a",
            tokens: &["kw(produce)@0..7", "ident(a)@8..9"],
            errors: &[],
        },
        Case {
            name: "kw_thunk_force_match",
            input: "thunk force match",
            tokens: &["kw(thunk)@0..5", "kw(force)@6..11", "kw(match)@12..17"],
            errors: &[],
        },
        Case {
            name: "brackets",
            input: "[](){}",
            tokens: &[
                "sym([)@0..1",
                "sym(])@1..2",
                "sym(()@2..3",
                "sym())@3..4",
                "sym({)@4..5",
                "sym(})@5..6",
            ],
            errors: &[],
        },
        Case {
            name: "punctuation",
            input: ":,=:=/*=>.",
            tokens: &[
                "sym(:)@0..1",
                "sym(,)@1..2",
                "sym(=)@2..3",
                "sym(:=)@3..5",
                "sym(/)@5..6",
                "sym(*)@6..7",
                "sym(=>)@7..9",
                "sym(.)@9..10",
            ],
            errors: &[],
        },
        Case {
            name: "identifier_ascii",
            input: "hello_world123",
            tokens: &["ident(hello_world123)@0..14"],
            errors: &[],
        },
        Case {
            name: "identifier_unicode",
            input: "데이터 함수",
            tokens: &["ident(데이터)@0..9", "ident(함수)@10..16"],
            errors: &[],
        },
        Case {
            name: "identifier_mixed",
            input: "αβ_12",
            tokens: &["ident(αβ_12)@0..7"],
            errors: &[],
        },
        Case {
            name: "data_decl_sample",
            input: "data Option[A] { .some(A), .none }",
            tokens: &[
                "kw(data)@0..4",
                "ident(Option)@5..11",
                "sym([)@11..12",
                "ident(A)@12..13",
                "sym(])@13..14",
                "sym({)@15..16",
                "sym(.)@17..18",
                "ident(some)@18..22",
                "sym(()@22..23",
                "ident(A)@23..24",
                "sym())@24..25",
                "sym(,)@25..26",
                "sym(.)@27..28",
                "ident(none)@28..32",
                "sym(})@33..34",
            ],
            errors: &[],
        },
        Case {
            name: "fn_decl_sample",
            input: "fn id[A](a: A): produce A / {} := produce a",
            tokens: &[
                "kw(fn)@0..2",
                "ident(id)@3..5",
                "sym([)@5..6",
                "ident(A)@6..7",
                "sym(])@7..8",
                "sym(()@8..9",
                "ident(a)@9..10",
                "sym(:)@10..11",
                "ident(A)@12..13",
                "sym())@13..14",
                "sym(:)@14..15",
                "kw(produce)@16..23",
                "ident(A)@24..25",
                "sym(/)@26..27",
                "sym({)@28..29",
                "sym(})@29..30",
                "sym(:=)@31..33",
                "kw(produce)@34..41",
                "ident(a)@42..43",
            ],
            errors: &[],
        },
        Case {
            name: "keywords_vs_identifiers",
            input: "database function letdown inbox producer",
            tokens: &[
                "ident(database)@0..8",
                "ident(function)@9..17",
                "ident(letdown)@18..25",
                "ident(inbox)@26..31",
                "ident(producer)@32..40",
            ],
            errors: &[],
        },
        Case {
            name: "colon_equals_spacing",
            input: ": = := :",
            tokens: &["sym(:)@0..1", "sym(=)@2..3", "sym(:=)@4..6", "sym(:)@7..8"],
            errors: &[],
        },
        Case {
            name: "multi_line",
            input: "data X\nfn y",
            tokens: &[
                "kw(data)@0..4",
                "ident(X)@5..6",
                "kw(fn)@7..9",
                "ident(y)@10..11",
            ],
            errors: &[],
        },
        Case {
            name: "tabs_and_spaces",
            input: "\tfn\tid\t",
            tokens: &["kw(fn)@1..3", "ident(id)@4..6"],
            errors: &[],
        },
        Case {
            name: "plus_operator",
            input: "a+b",
            tokens: &["ident(a)@0..1", "sym(+)@1..2", "ident(b)@2..3"],
            errors: &[],
        },
        Case {
            name: "unknown_unicode_symbol",
            input: "x→y",
            tokens: &["ident(x)@0..1", "ident(y)@4..5"],
            errors: &["unexpected character: '→'@1..4"],
        },
        Case {
            name: "only_whitespace",
            input: " \n\t ",
            tokens: &[],
            errors: &[],
        },
        Case {
            name: "empty",
            input: "",
            tokens: &[],
            errors: &[],
        },
        Case {
            name: "complex_mix",
            input: "let _x:=produce data/in",
            tokens: &[
                "kw(let)@0..3",
                "ident(_x)@4..6",
                "sym(:=)@6..8",
                "kw(produce)@8..15",
                "kw(data)@16..20",
                "sym(/)@20..21",
                "kw(in)@21..23",
            ],
            errors: &[],
        },
    ];

    assert!(cases.len() >= 20, "need at least 20 golden cases");

    for case in cases {
        let out = lex(case.input);
        let actual_tokens = out.tokens.iter().map(render_token).collect::<Vec<_>>();
        let actual_errors = out.errors.iter().map(render_error).collect::<Vec<_>>();

        assert_eq!(
            actual_tokens, case.tokens,
            "token mismatch in case {} with input {:?}",
            case.name, case.input
        );
        assert_eq!(
            actual_errors, case.errors,
            "error mismatch in case {} with input {:?}",
            case.name, case.input
        );
    }
}

fn render_token(token: &lumo_compiler::lexer::Token) -> String {
    let head = match &token.kind {
        TokenKind::Keyword(Keyword::Data) => "kw(data)".to_owned(),
        TokenKind::Keyword(Keyword::Fn) => "kw(fn)".to_owned(),
        TokenKind::Keyword(Keyword::Extern) => "kw(extern)".to_owned(),
        TokenKind::Keyword(Keyword::Let) => "kw(let)".to_owned(),
        TokenKind::Keyword(Keyword::In) => "kw(in)".to_owned(),
        TokenKind::Keyword(Keyword::Produce) => "kw(produce)".to_owned(),
        TokenKind::Keyword(Keyword::Thunk) => "kw(thunk)".to_owned(),
        TokenKind::Keyword(Keyword::Force) => "kw(force)".to_owned(),
        TokenKind::Keyword(Keyword::Match) => "kw(match)".to_owned(),
        TokenKind::Keyword(Keyword::Cap) => "kw(cap)".to_owned(),
        TokenKind::Keyword(Keyword::Perform) => "kw(perform)".to_owned(),
        TokenKind::Keyword(Keyword::Handle) => "kw(handle)".to_owned(),
        TokenKind::Keyword(Keyword::Bundle) => "kw(bundle)".to_owned(),
        TokenKind::Keyword(Keyword::Use) => "kw(use)".to_owned(),
        TokenKind::Keyword(Keyword::Impl) => "kw(impl)".to_owned(),
        TokenKind::Keyword(Keyword::If) => "kw(if)".to_owned(),
        TokenKind::Keyword(Keyword::Else) => "kw(else)".to_owned(),
        TokenKind::Keyword(Keyword::Lambda) => "kw(lambda)".to_owned(),
        TokenKind::Keyword(Keyword::Roll) => "kw(roll)".to_owned(),
        TokenKind::Keyword(Keyword::Unroll) => "kw(unroll)".to_owned(),
        TokenKind::Keyword(Keyword::Ctor) => "kw(ctor)".to_owned(),
        TokenKind::Ident(s) => format!("ident({s})"),
        TokenKind::StringLit(s) => format!("string({s})"),
        TokenKind::Symbol(Symbol::Hash) => "sym(#)".to_owned(),
        TokenKind::Symbol(Symbol::LBracket) => "sym([)".to_owned(),
        TokenKind::Symbol(Symbol::RBracket) => "sym(])".to_owned(),
        TokenKind::Symbol(Symbol::LParen) => "sym(()".to_owned(),
        TokenKind::Symbol(Symbol::RParen) => "sym())".to_owned(),
        TokenKind::Symbol(Symbol::LBrace) => "sym({)".to_owned(),
        TokenKind::Symbol(Symbol::RBrace) => "sym(})".to_owned(),
        TokenKind::Symbol(Symbol::Semi) => "sym(;)".to_owned(),
        TokenKind::Symbol(Symbol::Colon) => "sym(:)".to_owned(),
        TokenKind::Symbol(Symbol::Comma) => "sym(,)".to_owned(),
        TokenKind::Symbol(Symbol::Equals) => "sym(=)".to_owned(),
        TokenKind::Symbol(Symbol::ColonEquals) => "sym(:=)".to_owned(),
        TokenKind::Symbol(Symbol::Slash) => "sym(/)".to_owned(),
        TokenKind::Symbol(Symbol::Star) => "sym(*)".to_owned(),
        TokenKind::Symbol(Symbol::FatArrow) => "sym(=>)".to_owned(),
        TokenKind::Symbol(Symbol::Dot) => "sym(.)".to_owned(),
        TokenKind::Symbol(Symbol::DotDot) => "sym(..)".to_owned(),
        TokenKind::Symbol(Symbol::Plus) => "sym(+)".to_owned(),
        TokenKind::Symbol(Symbol::Minus) => "sym(-)".to_owned(),
        TokenKind::Symbol(Symbol::Percent) => "sym(%)".to_owned(),
        TokenKind::Symbol(Symbol::Bang) => "sym(!)".to_owned(),
        TokenKind::Symbol(Symbol::Lt) => "sym(<)".to_owned(),
        TokenKind::Symbol(Symbol::Gt) => "sym(>)".to_owned(),
        TokenKind::Symbol(Symbol::LtEq) => "sym(<=)".to_owned(),
        TokenKind::Symbol(Symbol::GtEq) => "sym(>=)".to_owned(),
        TokenKind::Symbol(Symbol::EqEq) => "sym(==)".to_owned(),
        TokenKind::Symbol(Symbol::BangEq) => "sym(!=)".to_owned(),
        TokenKind::Symbol(Symbol::AmpAmp) => "sym(&&)".to_owned(),
        TokenKind::Symbol(Symbol::PipePipe) => "sym(||)".to_owned(),
        TokenKind::NumberLit(s) => format!("number({s})"),
    };
    format!("{head}@{}..{}", token.span.start, token.span.end)
}

fn render_error(error: &lumo_compiler::lexer::LexError) -> String {
    format!("{}@{}..{}", error.message, error.span.start, error.span.end)
}
