import { Span } from '@/#syntax/index.js';

export interface IAstNode {
  readonly id: AstId;
  readonly span: Span;
}

export class AstId {
  constructor(readonly handle: number) {}
}
