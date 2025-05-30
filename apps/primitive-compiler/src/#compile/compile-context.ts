import { IAstNode } from '@/#core/#ast/base.js';
import { TypeScope } from '@/#core/#type/scope.js';

export class CompileContext {
  private varIndex = 0;

  constructor(
    readonly scope: TypeScope,
    readonly root: CompileContext | null = null,
  ) {}

  of(node: IAstNode): CompileContext {
    return new CompileContext(this.scope.of(node), this.root ?? this);
  }

  generateUniqueVariable(): string {
    return this.root?.generateUniqueVariable() ?? `_LumoTmp${this.varIndex++}`;
  }
}
