import { Expression } from './index.js';
import { AstId, IAstNode } from '../base.js';
import { Identifier } from '../construct.js';

export type functionCallArgument = Expression | MutName;

export class FunctionCall implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly fn: Expression,
    readonly args: functionCallArgument[],
  ) {}

  toString(): string {
    return `FunctionCall{#${this.id.handle}}(\nfn=${
      this.fn
    },\nargs=[\n${this.args.map((arg) => arg.toString()).join(',\n')}\n]\n)`;
  }
}

export class MutName implements IAstNode {
  constructor(readonly id: AstId, readonly ident: Identifier) {}

  toString() {
    return `MutName{#${this.id.handle}}(ident=${this.ident})`;
  }
}
