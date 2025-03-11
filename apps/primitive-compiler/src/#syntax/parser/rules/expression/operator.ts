import { atomExpression } from './atom.js';
import {
  InfixOperator,
  InfixOperatorKind,
  PrefixOperator,
  PrefixOperatorKind,
} from '@/#core/#ast/expression/operator.js';
import { Span, TokenKind } from '../../../common/index.js';
import {
  ctx,
  cut,
  failure,
  oneof,
  opt,
  Parser,
  repeat0,
  rule,
  separatedList1,
  seq,
  token,
  withCtxMod,
} from '../../base.js';
import { astId, identifier, spanning } from '../fragments.js';
import { Expression, FunctionCall, MutName } from '@/#core/#ast/index.js';

export const exprBp: Parser<Expression> = rule<Expression>(() =>
  seq(
    oneof(
      'Operator Expression Left Hand Side',
      spanning(prefixOperatorKind).flatMap(([op, span]) => {
        const [_, rightBp] = getPrefixBindingPower(op);
        return withCtxMod(
          'minimumBindingPower',
          rightBp,
        )(seq(exprBp, astId)).map(
          ([expr, id]) =>
            new PrefixOperator(id, Span.wrapping(span, expr.span), op, expr),
        );
      }),
      atomExpression,
    ),
    repeat0(
      oneof(
        'Infix/Postfix Operator',
        seq(
          postefixOperatorKind,
          ctx((ctx) => [ctx, ctx.minimumBindingPower]),
        ).flatMap(([op, bp]) => {
          const [leftBp, _] = getPostfixBindingPower(op);
          if (leftBp < bp) return failure('');

          return getPostfixProcessor(op);
        }),
        seq(
          infixOperatorKind,
          ctx((ctx) => [ctx, ctx.minimumBindingPower]),
        ).flatMap(([op, bp]) => {
          const [leftBp, rightBp] = getInfixBindingPower(op);
          if (leftBp < bp) return failure('');

          return withCtxMod(
            'minimumBindingPower',
            rightBp,
          )(spanning(seq(exprBp, astId))).map(
            ([[rhs, id], span]) =>
              (lhs: Expression) =>
                new InfixOperator(
                  id,
                  Span.wrapping(lhs.span, span),
                  lhs,
                  op,
                  rhs,
                ),
          );
        }),
      ),
    ),
  ).map(([lhs, rest]) =>
    rest.reduce<Expression>((currentLhs, f) => f(currentLhs), lhs),
  ),
);

const prefixOperatorKind = rule<PrefixOperatorKind>(() =>
  oneof(
    'Prefix Operator',
    token(TokenKind.PunctuationExclamationMark).map(
      () => PrefixOperatorKind.Not,
    ),
    token(TokenKind.PunctuationHyphenMinus).map(
      () => PrefixOperatorKind.Negate,
    ),
  ),
);

const infixOperatorKind = rule<InfixOperatorKind>(() =>
  oneof(
    'Infix Operator',
    token(TokenKind.PunctuationPlusSign).map(() => InfixOperatorKind.Add),
    token(TokenKind.PunctuationAsterisk).map(() => InfixOperatorKind.Multiply),
  ),
);

const postefixOperatorKind = rule<FullPostfixOperatorKind>(() =>
  oneof(
    'Postfix Operator',
    token(TokenKind.PunctuationLeftParenthesis).map<FullPostfixOperatorKind>(
      () => ({
        kind: 'FunctionCall',
      }),
    ),
    token(TokenKind.PunctuationLeftSquareBracket).map<FullPostfixOperatorKind>(
      () => ({
        kind: 'Index',
      }),
    ),
  ),
);

function getPrefixBindingPower(op: PrefixOperatorKind): [null, number] {
  switch (op) {
    case PrefixOperatorKind.Not:
    case PrefixOperatorKind.Negate:
      return [null, 5];
  }
}

function getInfixBindingPower(op: InfixOperatorKind): [number, number] {
  switch (op) {
    case InfixOperatorKind.Add:
      return [1, 2];
    case InfixOperatorKind.Multiply:
      return [3, 4];
  }
}

function getPostfixBindingPower(op: FullPostfixOperatorKind): [number, null] {
  switch (op.kind) {
    case 'FunctionCall':
    case 'Index':
      return [6, null];
  }
}

function getPostfixProcessor(
  op: FullPostfixOperatorKind,
): Parser<(expr: Expression) => Expression> {
  switch (op.kind) {
    case 'FunctionCall':
      return cut(
        withCtxMod(
          'newlineAsSemi',
          false,
        )(
          spanning(
            seq(
              opt(
                seq(
                  separatedList1(
                    withCtxMod(
                      'minimumBindingPower',
                      0,
                    )(oneof('mut name or value', mutName, exprBp)),
                    token(TokenKind.PunctuationComma),
                  ),
                  opt(token(TokenKind.PunctuationComma)),
                ),
              ),
              token(TokenKind.PunctuationRightParenthesis),
              astId,
            ),
          ).map(
            ([[args, _0, id], span]) =>
              (fn: Expression) =>
                new FunctionCall(
                  id,
                  Span.wrapping(fn.span, span),
                  fn,
                  args?.[0] ?? [],
                ),
          ),
        ),
      );
    case 'Index':
      return failure('Index is not implemented yet');
  }
}

const mutName = rule(() =>
  spanning(seq(token(TokenKind.KeywordMut), identifier, astId)),
).map(([[_, ident, id], span]) => new MutName(id, span, ident));

type FullPostfixOperatorKind = { kind: 'FunctionCall' } | { kind: 'Index' };
