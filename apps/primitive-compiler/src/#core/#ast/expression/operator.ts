import { Expression } from '.';
import { AstId, IAstNode } from '../base';

export class PrefixOperator implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly operators: PrefixOperatorKind[],
    readonly expression: Expression,
  ) {}
}

export const PrefixOperatorKind = Object.freeze({
  Not: 'Not',
  Minus: 'Minus',
});
export type PrefixOperatorKind =
  (typeof PrefixOperatorKind)[keyof typeof PrefixOperatorKind];

export class InfixOperator implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly lhs: Expression,
    readonly op: InfixOperatorKind,
    readonly rhs: Expression,
  ) {}
}

export const InfixOperatorKind = Object.freeze({
  Add: 'Add',
  Multiply: 'Multiply',
});
export type InfixOperatorKind =
  (typeof InfixOperatorKind)[keyof typeof InfixOperatorKind];
