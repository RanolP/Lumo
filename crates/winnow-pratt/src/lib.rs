use std::{fmt::Debug, marker::PhantomData};

use winnow::{
    Parser, Stateful,
    combinator::{cut_err, fail},
    error::{AddContext, ErrMode, ParserError, StrContext, StrContextValue},
    stream::Stream,
};

pub enum BindingPower {}

impl BindingPower {
    pub const fn prefix(right: usize) -> PrefixBindingPower {
        PrefixBindingPower(right)
    }
    pub const fn infix(left: usize, right: usize) -> InfixBindingPower {
        InfixBindingPower(left, right)
    }
    pub const fn postfix(left: usize) -> PostfixBindingPower {
        PostfixBindingPower(left)
    }
}

/**
 * the right binding power of a prefix operator
 */
pub struct PrefixBindingPower(usize);
pub trait PrefixOperatorSet<I: Stream, TExpr, E: ParserError<I>>: Sized {
    fn binding_power(&self) -> PrefixBindingPower;
    fn apply(self, input: &mut I, expr: TExpr) -> TExpr;
    fn parser() -> impl Parser<I, Self, E>;
}

/**
 * the left and right binding power of an infix operator
 */
pub struct InfixBindingPower(usize, usize);
pub trait InfixOperatorSet<I: Stream, TExpr, E: ParserError<I>>: Sized {
    fn binding_power(&self) -> InfixBindingPower;
    fn apply(self, input: &mut I, lhs: TExpr, rhs: TExpr) -> TExpr;
    fn parser() -> impl Parser<I, Self, E>;
}

/**
 * the left binding power of a postfix operator
 */
pub struct PostfixBindingPower(usize);
pub trait PostfixOperatorSet<I: Stream, TExpr, E: ParserError<I>>: Sized {
    fn binding_power(&self) -> PostfixBindingPower;
    fn apply(self, input: &mut I, expr: TExpr) -> TExpr;
    fn parser() -> impl Parser<I, Self, E>;
}

pub enum NoParser {}
impl<I: Stream, TExpr, E: ParserError<I>> PrefixOperatorSet<I, TExpr, E> for NoParser {
    fn binding_power(&self) -> PrefixBindingPower {
        match *self {}
    }
    fn apply(self, _input: &mut I, _expr: TExpr) -> TExpr {
        match self {}
    }
    fn parser() -> impl Parser<I, Self, E> {
        fail
    }
}
impl<I: Stream, TExpr, E: ParserError<I>> InfixOperatorSet<I, TExpr, E> for NoParser {
    fn binding_power(&self) -> InfixBindingPower {
        match *self {}
    }
    fn apply(self, _input: &mut I, _lhs: TExpr, _rhs: TExpr) -> TExpr {
        match self {}
    }
    fn parser() -> impl Parser<I, Self, E> {
        fail
    }
}
impl<I: Stream, TExpr, E: ParserError<I>> PostfixOperatorSet<I, TExpr, E> for NoParser {
    fn binding_power(&self) -> PostfixBindingPower {
        match *self {}
    }
    fn apply(self, _input: &mut I, _expr: TExpr) -> TExpr {
        match self {}
    }
    fn parser() -> impl Parser<I, Self, E> {
        fail
    }
}

pub trait PrattState {
    fn min_binding_power(&self) -> usize;

    fn set_min_binding_power(&mut self, binding_power: usize);
}

impl<I, S: PrattState> PrattState for Stateful<I, S> {
    fn min_binding_power(&self) -> usize {
        self.state.min_binding_power()
    }

    fn set_min_binding_power(&mut self, binding_power: usize) {
        self.state.set_min_binding_power(binding_power);
    }
}

pub struct PrattParser<TExpr, TModal, TPrefix, TInfix, TPostfix, TBasicExprParser> {
    phantom: PhantomData<(TExpr, TModal, TPrefix, TInfix, TPostfix)>,
    basic_expr_parser: TBasicExprParser,
    rhs_ctx_label: Option<&'static str>,
    rhs_ctx_expected_description: Option<&'static str>,
    prefix_ctx_label: Option<&'static str>,
    prefix_ctx_expected_description: Option<&'static str>,
}

pub enum NonModal {}
pub enum Modal {}

impl PrattParser<(), (), (), (), (), ()> {
    pub fn non_modal<I, TExpr, E, TPrefix, TInfix, TPostfix, TBasicExprParser>(
        basic_expr_parser: TBasicExprParser,
    ) -> PrattParser<TExpr, NonModal, TPrefix, TInfix, TPostfix, TBasicExprParser>
    where
        I: Stream,
        E: ParserError<I>,
        TPrefix: PrefixOperatorSet<I, TExpr, E>,
        TInfix: InfixOperatorSet<I, TExpr, E>,
        TPostfix: PostfixOperatorSet<I, TExpr, E>,
    {
        PrattParser {
            phantom: PhantomData,
            basic_expr_parser,
            rhs_ctx_label: None,
            rhs_ctx_expected_description: None,
            prefix_ctx_label: None,
            prefix_ctx_expected_description: None,
        }
    }
    pub fn modal<I, TExpr, E, TPrefix, TInfix, TPostfix, TBasicExprParser>(
        basic_expr_parser: TBasicExprParser,
    ) -> PrattParser<TExpr, Modal, TPrefix, TInfix, TPostfix, TBasicExprParser>
    where
        I: Stream,
        E: ParserError<I>,
        TPrefix: PrefixOperatorSet<I, TExpr, E>,
        TInfix: InfixOperatorSet<I, TExpr, E>,
        TPostfix: PostfixOperatorSet<I, TExpr, E>,
    {
        PrattParser {
            phantom: PhantomData,
            basic_expr_parser,
            rhs_ctx_label: None,
            rhs_ctx_expected_description: None,
            prefix_ctx_label: None,
            prefix_ctx_expected_description: None,
        }
    }
}

impl<TExpr, TPrefix, TInfix, TPostfix, TBasicExprParser>
    PrattParser<TExpr, Modal, TPrefix, TInfix, TPostfix, TBasicExprParser>
{
    pub fn rhs_ctx_label(mut self, label: &'static str) -> Self {
        self.rhs_ctx_label = Some(label);
        self
    }
    pub fn rhs_ctx_expected_description(mut self, description: &'static str) -> Self {
        self.rhs_ctx_expected_description = Some(description);
        self
    }
    pub fn prefix_ctx_label(mut self, label: &'static str) -> Self {
        self.prefix_ctx_label = Some(label);
        self
    }

    pub fn prefix_ctx_expected_description(mut self, description: &'static str) -> Self {
        self.prefix_ctx_expected_description = Some(description);
        self
    }
}

impl<TExpr, TPrefix, TInfix, TPostfix, TBasicExprParser>
    PrattParser<TExpr, NonModal, TPrefix, TInfix, TPostfix, TBasicExprParser>
{
    pub fn rhs_ctx_label(mut self, label: &'static str) -> Self {
        self.rhs_ctx_label = Some(label);
        self
    }
    pub fn rhs_ctx_expected_description(mut self, description: &'static str) -> Self {
        self.rhs_ctx_expected_description = Some(description);
        self
    }
    pub fn prefix_ctx_label(mut self, label: &'static str) -> Self {
        self.prefix_ctx_label = Some(label);
        self
    }

    pub fn prefix_ctx_expected_description(mut self, description: &'static str) -> Self {
        self.prefix_ctx_expected_description = Some(description);
        self
    }
}

impl<TExpr, TPrefix, TInfix, TPostfix, TBasicExprParser, I, E> Parser<I, TExpr, ErrMode<E>>
    for PrattParser<TExpr, Modal, TPrefix, TInfix, TPostfix, TBasicExprParser>
where
    TExpr: Debug,
    I: Stream,
    I: PrattState,
    E: ParserError<I>,
    E: AddContext<I, StrContext>,
    TPrefix: PrefixOperatorSet<I, TExpr, ErrMode<E>>,
    TInfix: InfixOperatorSet<I, TExpr, ErrMode<E>>,
    TPostfix: PostfixOperatorSet<I, TExpr, ErrMode<E>>,
    TBasicExprParser: Parser<I, TExpr, ErrMode<E>>,
{
    fn parse_next(&mut self, input: &mut I) -> winnow::Result<TExpr, ErrMode<E>> {
        let mut prefix_parser = TPrefix::parser();
        let mut infix_parser = TInfix::parser();
        let mut postfix_parser = TPostfix::parser();

        let mut result = match prefix_parser.parse_next(input) {
            Ok(prefix) => {
                let start = input.checkpoint();
                let old = input.min_binding_power();
                input.set_min_binding_power(prefix.binding_power().0);
                let expr = self.parse_next(input).map_err(|err| {
                    if !err.is_backtrack() {
                        return err;
                    }
                    let err = if let Some(label) = self.prefix_ctx_label {
                        err.add_context(input, &start, StrContext::Label(label))
                    } else {
                        err
                    };
                    let err =
                        if let Some(expected_description) = self.prefix_ctx_expected_description {
                            err.add_context(
                                input,
                                &start,
                                StrContext::Expected(StrContextValue::Description(
                                    expected_description,
                                )),
                            )
                        } else {
                            err
                        };
                    err.cut()
                });
                input.set_min_binding_power(old);
                prefix.apply(input, expr?)
            }
            Err(e) if e.is_backtrack() => self.basic_expr_parser.parse_next(input)?,
            Err(e) => return Err(e),
        };
        loop {
            let after_postfix = match postfix_parser.parse_next(input) {
                Ok(postfix) if postfix.binding_power().0 >= input.min_binding_power() => {
                    postfix.apply(input, result)
                }
                otherwise => {
                    if let Err(e) = otherwise {
                        if !e.is_backtrack() {
                            break Err(e);
                        }
                    }
                    let start = input.checkpoint();
                    match infix_parser.parse_next(input) {
                        Ok(infix) if infix.binding_power().0 >= input.min_binding_power() => {
                            let old = input.min_binding_power();
                            input.set_min_binding_power(infix.binding_power().1);
                            let rhs = self.parse_next(input).map_err(|err| {
                                if !err.is_backtrack() {
                                    return err;
                                }
                                let err = if let Some(label) = self.rhs_ctx_label {
                                    err.add_context(input, &start, StrContext::Label(label))
                                } else {
                                    err
                                };
                                let err = if let Some(expected_description) =
                                    self.rhs_ctx_expected_description
                                {
                                    err.add_context(
                                        input,
                                        &start,
                                        StrContext::Expected(StrContextValue::Description(
                                            expected_description,
                                        )),
                                    )
                                } else {
                                    err
                                };
                                err.cut()
                            });
                            input.set_min_binding_power(old);

                            infix.apply(input, result, rhs?)
                        }
                        Ok(_) => {
                            input.reset(&start);
                            break Ok(result);
                        }
                        Err(e) if e.is_backtrack() => {
                            input.reset(&start);
                            break Ok(result);
                        }
                        Err(e) => break Err(e),
                    }
                }
            };
            result = after_postfix;
        }
    }
}

impl<TExpr, TPrefix, TInfix, TPostfix, TBasicExprParser, I, E> Parser<I, TExpr, E>
    for PrattParser<TExpr, NonModal, TPrefix, TInfix, TPostfix, TBasicExprParser>
where
    I: Stream,
    I: PrattState,
    E: ParserError<I>,
    E: AddContext<I, StrContext>,
    TPrefix: PrefixOperatorSet<I, TExpr, E>,
    TInfix: InfixOperatorSet<I, TExpr, E>,
    TPostfix: PostfixOperatorSet<I, TExpr, E>,
    TBasicExprParser: Parser<I, TExpr, E>,
{
    fn parse_next(&mut self, input: &mut I) -> winnow::Result<TExpr, E> {
        let mut prefix_parser = TPrefix::parser();
        let mut infix_parser = TInfix::parser();
        let mut postfix_parser = TPostfix::parser();

        let mut result = match prefix_parser.parse_next(input) {
            Ok(prefix) => {
                let start = input.checkpoint();
                let old = input.min_binding_power();
                input.set_min_binding_power(prefix.binding_power().0);
                let expr = self.parse_next(input).map_err(|err| {
                    if !err.is_backtrack() {
                        return err;
                    }
                    let err = if let Some(label) = self.prefix_ctx_label {
                        err.add_context(input, &start, StrContext::Label(label))
                    } else {
                        err
                    };
                    let err =
                        if let Some(expected_description) = self.prefix_ctx_expected_description {
                            err.add_context(
                                input,
                                &start,
                                StrContext::Expected(StrContextValue::Description(
                                    expected_description,
                                )),
                            )
                        } else {
                            err
                        };
                    err
                });
                input.set_min_binding_power(old);
                prefix.apply(input, expr?)
            }
            Err(e) if e.is_backtrack() => self.basic_expr_parser.parse_next(input)?,
            Err(e) => return Err(e),
        };
        loop {
            let after_postfix = match postfix_parser.parse_next(input) {
                Ok(postfix) if postfix.binding_power().0 >= input.min_binding_power() => {
                    postfix.apply(input, result)
                }
                otherwise => {
                    if let Err(e) = otherwise {
                        if !e.is_backtrack() {
                            break Err(e);
                        }
                    }
                    let start = input.checkpoint();
                    match infix_parser.parse_next(input) {
                        Ok(infix) if infix.binding_power().0 >= input.min_binding_power() => {
                            let old = input.min_binding_power();
                            input.set_min_binding_power(infix.binding_power().1);
                            let rhs = self.parse_next(input).map_err(|err| {
                                if !err.is_backtrack() {
                                    return err;
                                }
                                let err = if let Some(label) = self.rhs_ctx_label {
                                    err.add_context(input, &start, StrContext::Label(label))
                                } else {
                                    err
                                };
                                let err = if let Some(expected_description) =
                                    self.rhs_ctx_expected_description
                                {
                                    err.add_context(
                                        input,
                                        &start,
                                        StrContext::Expected(StrContextValue::Description(
                                            expected_description,
                                        )),
                                    )
                                } else {
                                    err
                                };
                                err
                            });
                            input.set_min_binding_power(old);

                            infix.apply(input, result, rhs?)
                        }
                        Ok(_) => {
                            input.reset(&start);
                            break Ok(result);
                        }
                        Err(e) if e.is_backtrack() => {
                            input.reset(&start);
                            break Ok(result);
                        }
                        Err(e) => break Err(e),
                    }
                }
            };
            result = after_postfix;
        }
    }
}
