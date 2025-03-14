import { Span } from '@/#syntax/index.js';
import { AstId, IAstNode } from '../base.js';
import { Expression } from './index.js';
import { Pattern } from '../pattern.js';

export class Match implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly span: Span,
    readonly expr: Expression,
    readonly arms: MatchArm[],
  ) {}
}

export class MatchArm implements IAstNode {
  constructor(
    readonly id: AstId,
    readonly span: Span,
    readonly pattern: Pattern,
    readonly body: Expression,
  ) {}
}
