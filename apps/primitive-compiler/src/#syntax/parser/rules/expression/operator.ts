import { atomExpression } from './atom';
import { InfixOperatorKind } from '../../../../#core/#ast/expression/operator';
import { TokenKind } from '../../../common';
import { oneof, rule, seq, token } from '../../base';

const infixOperatorBp = rule(() => seq(atomExpression));

const infixOperatorKind = rule<InfixOperatorKind>(() =>
  oneof(
    'Infix Operator',
    token(TokenKind.PunctuationPlusSign).map(() => InfixOperatorKind.Add),
    token(TokenKind.PunctuationAsterisk).map(() => InfixOperatorKind.Multiply),
  ),
);

function getInfixBindingPower(op: InfixOperatorKind): [number, number] {
  switch (op) {
    case 'Add':
      return [1, 2];
    case 'Multiply':
      return [3, 4];
  }
}
