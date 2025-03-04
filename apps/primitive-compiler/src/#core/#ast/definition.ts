import { AstId, IAstNode } from './base.js';
import { Identifier, Expression } from './index.js';
import { BasePattern } from './pattern.js';
import { Type } from './type.js';

export type DefinitionNode = EnumDefinition;

export class EnumDefinition implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly name: Identifier,
    readonly branches: EnumBranch[],
  ) {}

  toString(): string {
    return `Enum{#${this.id.handle}}(\nname=${
      this.name
    },\nbranches=[\n${this.branches.join(',\n')}\n],\n)`;
  }
}

export class EnumBranch {
  constructor(readonly name: Identifier) {}

  toString(): string {
    return `EnumBranch(name=${this.name})`;
  }
}

export class FunctionDefinition implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly name: Identifier,
    readonly parameters: FunctionParameter[],
    readonly returnType: Type | null,
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
    readonly pattern: BasePattern | Identifier,
    readonly type?: Type,
  ) {}

  toString(): string {
    return `FunctionParameter{#${this.id.handle}}(\npattern=${this.pattern},\ntype=${this.type}\n)`;
  }
}
