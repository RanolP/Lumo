import { atomExpression } from './atom';
import {
  InfixOperator,
  InfixOperatorKind,
  PrefixOperator,
  PrefixOperatorKind,
} from '../../../../#core/#ast/expression/operator';
import { TokenKind } from '../../../common';
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
} from '../../base';
import { astId, identifier } from '../fragments';
import { Expression, FunctionCall, MutName } from '../../../../#core/#ast';

export const exprBp: Parser<Expression> = rule<Expression>(() =>
  seq(
    oneof(
      'Operator Expression Left Hand Side',
      prefixOperatorKind.flatMap((op) => {
        const [_, rightBp] = getPrefixBindingPower(op);
        return withCtxMod(
          'minimumBindingPower',
          rightBp,
        )(seq(exprBp, astId)).map(
          ([expr, id]) => new PrefixOperator(id, op, expr),
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
          )(seq(exprBp, astId)).map(
            ([rhs, id]) =>
              (lhs: Expression) =>
                new InfixOperator(id, lhs, op, rhs),
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
          seq(
            separatedList1(
              withCtxMod(
                'minimumBindingPower',
                0,
              )(oneof('mut name or value', mutName, exprBp)),
              token(TokenKind.PunctuationComma),
            ),
            opt(token(TokenKind.PunctuationComma)),
            token(TokenKind.PunctuationRightParenthesis),
            astId,
          ).map(
            ([args, _0, _1, id]) =>
              (fn: Expression) =>
                new FunctionCall(id, fn, args),
          ),
        ),
      );
    case 'Index':
      return failure('Index is not implemented yet');
  }
}

const mutName = rule(() =>
  seq(token(TokenKind.KeywordMut), identifier, astId),
).map(([_, ident, id]) => new MutName(id, ident));

type FullPostfixOperatorKind = { kind: 'FunctionCall' } | { kind: 'Index' };
