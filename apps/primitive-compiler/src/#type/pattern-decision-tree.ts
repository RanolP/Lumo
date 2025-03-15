import { AstId } from '@/#core/#ast/base.js';
import {
  DestructuringPattern,
  DiscardPattern,
  NameBindPattern,
  Pattern,
} from '@/#core/#ast/pattern.js';
import { Type, TypeScope } from '@/#core/#type/index.js';
import { Span } from '@/#syntax/index.js';
import { match, P } from 'ts-pattern';

type PatternCoverage = 'Generic' | `Destructor+${string}`;

export class PatternDecisionTree {
  private children: Map<PatternCoverage, PatternDecisionTree> = new Map();

  constructor(readonly id: AstId, readonly span: Span) {}

  addMatchArm(
    pattern: Pattern | { kind: 'tuple'; items: Pattern[] },
  ): PatternDecisionTree {
    return match(pattern)
      .with(P.instanceOf(DestructuringPattern), (pattern) => {
        const tag: PatternCoverage = `Destructor+${pattern.destructor.display}`;
        const tree =
          this.children.get(tag) ??
          new PatternDecisionTree(pattern.id, pattern.span);
        this.children.set(tag, tree);

        return match(pattern.matches)
          .with({ type: 'tuple' }, ({ items }) =>
            tree.addMatchArm({ kind: 'tuple', items }),
          )
          .exhaustive();
      })
      .with({ kind: 'tuple' }, ({ items }) => {
        if (items.length === 0) return this;
        const [head, ...tails] = items;
        return this.addMatchArm(head).addMatchArm({
          kind: 'tuple',
          items: tails,
        });
      })
      .with(P.instanceOf(NameBindPattern), (pattern) => {
        const tree =
          this.children.get('Generic') ??
          new PatternDecisionTree(pattern.id, pattern.span);
        this.children.set('Generic', tree);
        return tree;
      })
      .with(P.instanceOf(DiscardPattern), (pattern) => {
        const tree =
          this.children.get('Generic') ??
          new PatternDecisionTree(pattern.id, pattern.span);
        this.children.set('Generic', tree);
        return tree;
      })
      .exhaustive();
  }

  findMissingPattern(
    scope: TypeScope,
    type: Type | { kind: 'tuple'; items: Type[] },
    prefix: PatternPath,
  ): MissingPatternResult {
    return match(type)
      .with({ kind: 'tuple' }, ({ items }): MissingPatternResult => {})
      .otherwise((type): MissingPatternResult => {
        const generic = this.children.get('Generic');
        if (generic) {
          return {
            kind: 'continue',
            toVisit: [generic],
            prefix: [...prefix, '_'],
          };
        } else {
          return { kind: 'error', case: [...prefix, type.id(scope)] };
        }
      });
  }
}

type PatternPath = string[];
type MissingPatternResult =
  | { kind: 'error'; case: PatternPath }
  | { kind: 'continue'; toVisit: PatternDecisionTree[]; prefix: PatternPath };
