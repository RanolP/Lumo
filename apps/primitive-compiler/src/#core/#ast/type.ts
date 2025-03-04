import { AstId, IAstNode } from './base.js';
import { Path } from './construct.js';

export type Type = TupleType | PathType;

export class TupleType implements IAstNode {
  constructor(readonly id: AstId, readonly elements: Type[]) {}
}

export class PathType implements IAstNode {
  constructor(readonly id: AstId, readonly path: Path) {}
}
