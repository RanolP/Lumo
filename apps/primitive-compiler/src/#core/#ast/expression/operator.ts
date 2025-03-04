import { Expression } from './index.js';
import { AstId, IAstNode } from '../base.js';

export class PrefixOperator implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly op: PrefixOperatorKind,
    readonly expression: Expression,
  ) {}

  toString(): string {
    return `PrefixOp{#${this.id.handle}}(op=${this.op}, expr=${this.expression})`;
  }
}

export const PrefixOperatorKind = Object.freeze({
  Not: 'Not',
  Negate: 'Negate',
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

  toString(): string {
    return `InfixOp{#${this.id.handle}}(\nop=${this.op},\nlhs=${this.lhs},\nrhs=${this.rhs}\n)`;
  }
}

export const InfixOperatorKind = Object.freeze({
  Add: 'Add',
  Multiply: 'Multiply',
});
export type InfixOperatorKind =
  (typeof InfixOperatorKind)[keyof typeof InfixOperatorKind];
