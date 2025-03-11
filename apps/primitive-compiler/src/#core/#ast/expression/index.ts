import { Span } from '@/#syntax/index.js';
import { AstId, IAstNode } from '../base.js';
import { Path } from '../construct.js';
import { FunctionCall } from './function-call.js';
import { InfixOperator, PrefixOperator } from './operator.js';

export * from './function-call.js';

export type Expression =
  | FunctionCall
  | NameExpression
  | PrefixOperator
  | InfixOperator
  | Block;

export class NameExpression implements IAstNode {
  constructor(readonly id: AstId, readonly span: Span, readonly path: Path) {}

  toString(): string {
    return `Name{#${this.id.handle}}(path=${this.path})`;
  }
}

export class Block implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly span: Span,
    readonly expressions: Expression[],
  ) {}

  toString(): string {
    return `Block{#${this.id.handle}}(expressions=[\n${this.expressions.join(
      ',\n',
    )}\n])`;
  }
}
