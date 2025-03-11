import { Span } from '@/#syntax/index.js';
import { AstId, IAstNode } from './base.js';
import { Path } from './construct.js';

export type AstType = AstTupleType | AstPathType;

export class AstTupleType implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly span: Span,
    readonly elements: AstType[],
  ) {}
}

export class AstPathType implements IAstNode {
  constructor(readonly id: AstId, readonly span: Span, readonly path: Path) {}
}
