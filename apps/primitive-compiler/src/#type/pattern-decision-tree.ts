import { AstId } from '@/#core/#ast/base.js';
import {
  DestructuringPattern,
  DiscardPattern,
  NameBindPattern,
  Pattern,
} from '@/#core/#ast/pattern.js';
import { TypingError } from '@/#core/#type/error.js';
import { Sum, Type } from '@/#core/#type/index.js';
import { Span } from '@/#syntax/index.js';
import { match, P } from 'ts-pattern';

type PatternCoverage = 'Generic' | `Destructor+${string}`;

export class PatternDecisionTree {
  private children: Map<PatternCoverage, PatternDecisionTree[]> = new Map();

  constructor(readonly id: AstId, readonly span: Span, branches: Pattern[]) {
    for (const branch of branches) {
      if (branch instanceof DestructuringPattern) {
        if (branch.matches.type === 'tuple') {
          const children = branch.matches.items.map(
            (child) => new PatternDecisionTree([child]),
          );
          this.children.set(`Destructor+${branch.destructor.display}`, [
            ...(this.children.get(`Destructor+${branch.destructor.display}]`) ??
              []),
            new PatternDecisionTree(
              branch.id,
              branch.span,
              branch.matches.items,
            ),
          ]);
        } else {
          throw new Error(
            'Destructor pattern other than tuple-style is unsupported yet.',
          );
        }
      } else {
        this.children.set('Generic', [
          ...(this.children.get('Generic') ?? []),
          new PatternDecisionTree(branch.id, branch.span, []),
        ]);
      }
    }
  }

  findMissingPattern(type: Type): PatternSeq[] {
    const uncoveredPatterns = createUncoveredCasesFromType(type);
  }
}

type PatternSeq = Pattern[];
