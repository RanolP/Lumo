import { AstId } from '@/#core/#ast/base.js';
import {
  DestructuringPattern,
  DiscardPattern,
  NameBindPattern,
  Pattern,
} from '@/#core/#ast/pattern.js';
import { TypingError } from '@/#core/#type/error.js';
import {
  Constructor,
  normalizeType,
  Quantification,
  Recursion,
  Sum,
  Type,
  TypeScope,
  TypeVar,
} from '@/#core/#type/index.js';
import { Span } from '@/#syntax/index.js';
import { Result, Unit } from 'true-myth';
import { match, P } from 'ts-pattern';

type PatternCoverage = 'Skip+1' | `Destructor+${string}`;

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
          this.children.get('Skip+1') ??
          new PatternDecisionTree(pattern.id, pattern.span);
        this.children.set('Skip+1', tree);
        return tree;
      })
      .with(P.instanceOf(DiscardPattern), (pattern) => {
        const tree =
          this.children.get('Skip+1') ??
          new PatternDecisionTree(pattern.id, pattern.span);
        this.children.set('Skip+1', tree);
        return tree;
      })
      .exhaustive();
  }

  debug(root: boolean = true): string {
    return `${root ? '#root ' : ''}{\n${Array.from(this.children.entries())
      .map(([key, value]) => `${key} => ${value.debug(false)}`)
      .join('\n')
      .split('\n')
      .map((x) => `    ${x}`)
      .join('\n')
      .trimEnd()}${this.children.size > 0 ? '\n' : ''}}`;
  }

  findMissingPattern(
    scope: TypeScope,
    type: Type | { kind: 'tuple'; items: Type[] },
    visitChild: (node: PatternDecisionTree) => MissingPatternResult = () =>
      Result.ok(),
  ): MissingPatternResult {
    return match(type)
      .with({ kind: 'tuple' }, ({ items }): MissingPatternResult => {
        if (items.length === 0) return Result.ok();
        const [head, ...tails] = items;

        return this.findMissingPattern(scope, head, (node) =>
          node.findMissingPattern(
            scope,
            { kind: 'tuple', items: tails },
            visitChild,
          ),
        );
      })
      .with(P.instanceOf(Quantification), (q) =>
        this.findMissingPattern(scope, q.then, visitChild),
      )
      .with(P.instanceOf(Recursion), (r) =>
        this.findMissingPattern(scope, r.then, visitChild),
      )
      .with(
        P.instanceOf(Sum),
        (s): MissingPatternResult =>
          Array.from(s.items).reduce<MissingPatternResult>(
            (acc, curr): MissingPatternResult =>
              acc.and(this.findMissingPattern(scope, curr, visitChild)),
            Result.ok(),
          ),
      )
      .with(P.instanceOf(Constructor), (c): MissingPatternResult => {
        const skip1Node = this.children.get('Skip+1');
        if (skip1Node)
          return visitChild(skip1Node).mapErr((e) => [
            { kind: 'simple', value: '_' },
            ...e,
          ]);

        const node = this.children.get(`Destructor+${c.folded}.${c.tag}`);
        if (!node)
          return Result.err([
            {
              kind: 'tag',
              tag: `${c.folded}.${c.tag}`,
              paramsCount: c.items.types.length,
            },
            ...Array.from({ length: c.items.types.length }).map(
              (): PatternItem => ({
                kind: 'simple',
                value: '_',
              }),
            ),
          ]);
        return match(c.items)
          .with(
            { kind: 'positional' },
            ({ types }): MissingPatternResult =>
              node
                .findMissingPattern(
                  scope,
                  {
                    kind: 'tuple',
                    items: types.map(({ type }) => type),
                  },
                  visitChild,
                )
                .mapErr((e) => [
                  {
                    kind: 'tag',
                    tag: `${c.folded}.${c.tag}`,
                    paramsCount: types.length,
                  },
                  ...e,
                ]),
          )
          .with({ kind: 'named' }, () => {
            throw new TypingError(
              `Match on named constructor is not supported yet`,
              null,
            );
          })
          .exhaustive();
      })
      .with(
        P.instanceOf(TypeVar),
        (v): MissingPatternResult =>
          this.findMissingPattern(scope, normalizeType(scope, v), visitChild),
      )
      .otherwise((type): MissingPatternResult => {
        const skip1Node = this.children.get('Skip+1');
        if (skip1Node)
          return visitChild(skip1Node).mapErr((e) => [
            { kind: 'simple', value: '_' },
            ...e,
          ]);

        return Result.err([{ kind: 'simple', value: type.id(scope) }]);
      });
  }
}

type PatternPath = PatternItem[];
type PatternItem =
  | { kind: 'tag'; tag: string; paramsCount: number }
  | { kind: 'simple'; value: string };
type MissingPatternResult = Result<Unit, PatternPath>;

export function formatPatternPath(path: PatternPath): string {
  let result = '';
  const ctorStack: number[] = [];
  for (const item of path) {
    switch (item.kind) {
      case 'tag':
        result += item.tag;
        result += '(';
        if (item.paramsCount > 0) {
          ctorStack.push(item.paramsCount);
        }
        break;
      case 'simple':
        result += item.value;
        break;
    }

    const last = ctorStack.pop();
    if (last != null) {
      if (last === 0) {
        result += ')';
      } else {
        ctorStack.push(last - 1);
      }
    }
  }
  if (ctorStack.pop() == 0) {
    result += ')';
  }

  return result;
}
