import { AstId, IAstNode } from '../base';
import { Identifier } from '../construct';
import { FunctionCall } from './function-call';
import { InfixOperator, PrefixOperator } from './operator';

export * from './function-call';

export type Expression =
  | FunctionCall
  | NameExpression
  | PrefixOperator
  | InfixOperator
  | Block;

export class NameExpression implements IAstNode {
  constructor(readonly id: AstId, readonly ident: Identifier) {}

  toString(): string {
    return `Name{#${this.id.handle}}(ident=${this.ident})`;
  }
}

export class Block implements IAstNode {
  constructor(readonly id: AstId, readonly expressions: Expression[]) {}

  toString(): string {
    return `Block{#${this.id.handle}}(expressions=[\n${this.expressions.join(
      ',\n',
    )}\n])`;
  }
}
