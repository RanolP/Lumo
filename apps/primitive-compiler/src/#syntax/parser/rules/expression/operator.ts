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
  failure,
  oneof,
  opt,
  Parser,
  repeat0,
  rule,
  seq,
  token,
  withCtxMod,
} from '../../base';
import { astId } from '../fragments';
import { Expression } from '../../../../#core/#ast';

export const operator: Parser<Expression> = rule<Expression>(() =>
  seq(
    oneof(
      'Operator Expression Left Hand Side',
      prefixOperatorKind.flatMap((op) => {
        const [_, rightBp] = getPrefixBindingPower(op);
        return withCtxMod(
          'minimumBindingPower',
          rightBp,
        )(seq(operator, astId)).map(
          ([expr, astId]) => new PrefixOperator(astId, op, expr),
        );
      }),
      atomExpression,
    ),
    repeat0(
      seq(
        infixOperatorKind,
        ctx((ctx) => [ctx, ctx.minimumBindingPower]),
      ).flatMap(([op, bp]) => {
        const [leftBp, rightBp] = getInfixBindingPower(op);
        if (leftBp < bp) return failure('');

        return withCtxMod(
          'minimumBindingPower',
          rightBp,
        )(seq(operator, astId)).map(
          ([rhs, astId]) => [op, rhs, astId] as const,
        );
      }),
    ),
  ).map(([lhs, rest]) =>
    rest.reduce<Expression>(
      (currentLhs, [op, rhs, astId]) =>
        new InfixOperator(astId, currentLhs, op, rhs),
      lhs,
    ),
  ),
);

const prefixOperatorKind = rule<PrefixOperatorKind>(() =>
  oneof(
    'Prefix Operator',
    token(TokenKind.PunctuationExclamationMark).map(
      () => PrefixOperatorKind.Not,
    ),
    token(TokenKind.PunctuationHyphenMinus).map(() => PrefixOperatorKind.Minus),
  ),
);

const infixOperatorKind = rule<InfixOperatorKind>(() =>
  oneof(
    'Infix Operator',
    token(TokenKind.PunctuationPlusSign).map(() => InfixOperatorKind.Add),
    token(TokenKind.PunctuationAsterisk).map(() => InfixOperatorKind.Multiply),
  ),
);

function getPrefixBindingPower(op: PrefixOperatorKind): [null, number] {
  switch (op) {
    case PrefixOperatorKind.Not:
    case PrefixOperatorKind.Minus:
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
