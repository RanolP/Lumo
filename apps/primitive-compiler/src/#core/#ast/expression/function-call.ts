import { Expression } from '.';
import { AstId, IAstNode } from '../base';
import { Identifier } from '../construct';

export type functionCallArgument = Expression | MutName;

export class FunctionCall implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly fn: Expression,
    readonly args: functionCallArgument[],
  ) {}
}

export class MutName implements IAstNode {
  constructor(readonly id: AstId, readonly name: Identifier) {}
}
