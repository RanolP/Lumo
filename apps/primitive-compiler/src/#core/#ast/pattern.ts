import { Span } from '@/#syntax/index.js';
import { AstId, IAstNode } from './base.js';
import { Identifier, Path } from './construct.js';

export type Pattern = DestructuringPattern | DiscardPattern | NameBindPattern;

export class DestructuringPattern implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly span: Span,
    readonly destructor: Path,
    readonly matches: { type: 'tuple'; items: Pattern[] },
  ) {}
}

export class DiscardPattern implements IAstNode {
  constructor(readonly id: AstId, readonly span: Span) {}
}

export class NameBindPattern implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly span: Span,
    readonly name: Identifier,
  ) {}
}
