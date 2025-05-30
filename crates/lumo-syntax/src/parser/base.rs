use lumo_core::{Spanned, Token, TokenKind};
use winnow::{
    Parser, Stateful,
    combinator::{alt, repeat},
    error::{AddContext, ContextError, ErrMode, ModalError, ParserError, StrContext},
    stream::{Stream, StreamIsPartial},
    token::any,
};
use winnow_pratt::PrattState;

#[derive(Clone, Debug)]
pub struct State {
    pub node_id: usize,
    pub is_block: bool,
    pub min_binding_power: usize,
}

impl PrattState for State {
    fn min_binding_power(&self) -> usize {
        self.min_binding_power
    }

    fn set_min_binding_power(&mut self, binding_power: usize) {
        self.min_binding_power = binding_power;
    }
}

pub type Input<'a> = Stateful<&'a [Spanned<Token>], State>;
pub type Error = ErrMode<ContextError>;
pub type Result<T> = winnow::ModalResult<T>;

pub(super) fn raw_token<I, E>(kind: TokenKind) -> impl Parser<I, Spanned<Token>, E>
where
    I: Stream<Token = Spanned<Token>>,
    I: StreamIsPartial,
    E: ParserError<I> + ModalError,
{
    move |i: &mut I| -> winnow::Result<Spanned<Token>, E> {
        any::<I, E>.verify(|token| token.kind == kind).parse_next(i)
    }
}

pub fn token<E>(kind: TokenKind) -> impl for<'a> Parser<Input<'a>, Spanned<Token>, E>
where
    E: for<'a> ParserError<Input<'a>> + ModalError,
    E: for<'a> AddContext<Input<'a>, StrContext>,
{
    move |i: &mut Input| -> winnow::Result<Spanned<Token>, E> {
        if !matches!(
            kind,
            TokenKind::SpaceVertical | TokenKind::SpaceHorizontal | TokenKind::Eof
        ) {
            if !i.state.is_block {
                let _: () = repeat(
                    0..,
                    alt((
                        raw_token(TokenKind::SpaceHorizontal),
                        raw_token(TokenKind::SpaceVertical),
                    )),
                )
                .parse_next(i)?;
            } else {
                let _: () = repeat(0.., raw_token(TokenKind::SpaceHorizontal)).parse_next(i)?;
            }
        }
        any.verify(|token: &Spanned<Token>| token.kind == kind)
            .context(StrContext::Label("token"))
            .parse_next(i)
    }
}

#[macro_export]
macro_rules! with_span {
    ($parser:expr) => {
        (|input: &mut Input| {
            #[allow(unused_imports)]
            use ::winnow::{
                Parser,
                error::{Needed, ParserError},
                stream::{Offset, Stream, StreamIsPartial},
            };

            let start = input
                .peek_token()
                .ok_or_else(|| {
                    if input.is_partial() {
                        ParserError::incomplete(input, Needed::new(1))
                    } else {
                        ParserError::from_input(input)
                    }
                })?
                .span()
                .start()
                .clone();

            let checkpoint = input.checkpoint();
            let result = $parser.parse_next(input)?;
            let offset = input.offset_from(&checkpoint);
            input.reset(&checkpoint);
            let end = input
                .next_slice(offset)
                .last()
                .map(|token| token.span().end().clone())
                .unwrap_or(start.clone());

            Ok(Spanned::new(::lumo_core::Span::new(start, end), result))
        })
    };
}

#[macro_export]
macro_rules! with_node_id {
    ($parser:expr) => {
        (move |input: &mut Input| {
            #[allow(unused_imports)]
            use ::winnow::Parser;

            input.state.node_id += 1;
            let node_id = input.state.node_id;

            let result = $parser.parse_next(input)?;

            Ok(WithId::new(node_id, result))
        })
    };
}

#[macro_export]
macro_rules! with_block {
    ($parser:expr) => {
        (|input: &mut Input| {
            let old = input.state.is_block;
            input.state.is_block = true;

            let result = $parser.parse_next(input);
            input.state.is_block = old;
            result
        })
    };
}

#[macro_export]
macro_rules! without_block {
    ($parser:expr) => {
        (|input: &mut Input| {
            let old = input.state.is_block;
            input.state.is_block = false;

            let result = $parser.parse_next(input);
            input.state.is_block = old;
            result
        })
    };
}

#[macro_export]
macro_rules! with_binding_power {
    ($bp:expr, $parser:expr) => {
        (|input: &mut Input| {
            let old = input.state.min_binding_power;
            input.state.min_binding_power = $bp;

            let result = $parser.parse_next(input);
            input.state.min_binding_power = old;
            result
        })
    };
}
