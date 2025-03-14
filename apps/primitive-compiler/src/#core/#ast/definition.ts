import { Span } from '@/#syntax/index.js';
import { AstId, IAstNode } from './base.js';
import { Identifier, Expression } from './index.js';
import { Pattern } from './pattern.js';
import { AstType } from './type.js';

export type DefinitionNode = EnumDefinition | FunctionDefinition;

export class EnumDefinition implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly span: Span,
    readonly name: Identifier,
    readonly branches: EnumBranch[],
  ) {}

  toString(): string {
    return `Enum{#${this.id.handle}}(\nname=${
      this.name
    },\nbranches=[\n${this.branches.join(',\n')}\n],\n)`;
  }
}

export class EnumBranch implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly span: Span,
    readonly name: Identifier,
    readonly body:
      | { readonly kind: 'tuple'; readonly types: { readonly type: AstType }[] }
      | {
          readonly kind: 'struct';
          readonly types: { readonly name: string; readonly type: AstType }[];
        }
      | null = null,
  ) {}

  toString(): string {
    return `EnumBranch(name=${this.name})`;
  }
}

export class FunctionDefinition implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly span: Span,
    readonly name: Identifier,
    readonly parameters: FunctionParameter[],
    readonly returnType: AstType | null,
    readonly body: Expression,
  ) {}

  toString(): string {
    return `Function{#${this.id.handle}}(\nname=${
      this.name
    },\nparameters=[\n${this.parameters.join(',\n')}\n],\nreturn=${
      this.returnType
    },\nbody=${this.body}\n)`;
  }
}

export class FunctionParameter implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly span: Span,
    readonly pattern: Pattern | Identifier,
    readonly type?: AstType,
  ) {}

  toString(): string {
    return `FunctionParameter{#${this.id.handle}}(\npattern=${this.pattern},\ntype=${this.type}\n)`;
  }
}
