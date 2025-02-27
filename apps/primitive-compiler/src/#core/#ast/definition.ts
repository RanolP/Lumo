import { AstId, IAstNode } from './base';
import { Identifier } from './construct';

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
    },\nbranches=[\n${this.branches.join(', ')}\n]\n)`;
  }
}

export class EnumBranch {
  constructor(readonly name: Identifier) {}

  toString(): string {
    return `EnumBranch(name=${this.name})`;
  }
}
