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
  ): MissingPatternResult {
    return match(type)
      .with({ kind: 'tuple' }, ({ items }): MissingPatternResult => {
        if (items.length === 0) return Result.ok({ continue: (f) => f(this) });
        const [head, ...tails] = items;

        return this.findMissingPattern(scope, head).match({
          Ok(result): MissingPatternResult {
            return result.continue((child) =>
              child
                .findMissingPattern(scope, {
                  kind: 'tuple',
                  items: tails,
                })
                .mapErr((e) => [{ kind: 'simple', value: '_' }, ...e]),
            );
          },
          Err(error): MissingPatternResult {
            return Result.err([
              ...error,
              ...tails.map(
                (ty): PatternItem => ({
                  kind: 'simple',
                  value: ty.id(scope),
                }),
              ),
            ]);
          },
        });
      })
      .with(P.instanceOf(Quantification), (q) =>
        this.findMissingPattern(scope, q.then),
      )
      .with(P.instanceOf(Recursion), (r) =>
        this.findMissingPattern(scope, r.then),
      )
      .with(P.instanceOf(Sum), (s): MissingPatternResult => {
        const [head, ...tails] = Array.from(s.items).map((ty) =>
          this.findMissingPattern(scope, ty),
        );

        return tails.reduce<MissingPatternResult>(
          (acc, curr): MissingPatternResult =>
            acc.andThen((f) =>
              curr.andThen((g) =>
                Result.ok({
                  continue: (visitChild) =>
                    f.continue(visitChild).and(g.continue(visitChild)),
                }),
              ),
            ),
          head,
        );
      })
      .with(P.instanceOf(Constructor), (c): MissingPatternResult => {
        const skip1Node = this.children.get('Skip+1');
        if (skip1Node)
          return Result.ok({
            continue: (visitChild) =>
              visitChild(skip1Node).mapErr((e) => [
                { kind: 'simple', value: '_' },
                ...e,
              ]),
          });

        const node = this.children.get(
          `Destructor+${c.folded.token.content}.${c.tag}`,
        );
        if (!node)
          return Result.err([
            {
              kind: 'tag',
              tag: `${c.folded.token.content}.${c.tag}`,
              paramsCount: c.items.types.length,
            },
            ...c.items.types.map(
              ({ type }): PatternItem => ({
                kind: 'simple',
                value: type.id(scope),
              }),
            ),
          ]);
        return match(c.items)
          .with(
            { kind: 'positional' },
            ({ types }): MissingPatternResult =>
              node
                .findMissingPattern(scope, {
                  kind: 'tuple',
                  items: types.map(({ type }) => type),
                })
                .mapErr((e) => [
                  {
                    kind: 'tag',
                    tag: `${c.folded.token.content}.${c.tag}`,
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
          this.findMissingPattern(scope, normalizeType(scope, v)),
      )
      .otherwise((type): MissingPatternResult => {
        const skip1Node = this.children.get('Skip+1');
        if (skip1Node)
          return Result.ok({
            continue: (visitChild) =>
              visitChild(skip1Node).mapErr((e) => [
                { kind: 'simple', value: '_' },
                ...e,
              ]),
          });

        return Result.err([{ kind: 'simple', value: type.id(scope) }]);
      });
  }
}

type PatternPath = PatternItem[];
type PatternItem =
  | { kind: 'tag'; tag: string; paramsCount: number }
  | { kind: 'simple'; value: string };
type MissingPatternResult = Result<
  {
    continue(
      visitChild: (tree: PatternDecisionTree) => MissingPatternResult,
    ): MissingPatternResult;
  },
  PatternPath
>;

export function formatPatternPath(path: PatternPath): string {
  let result = '';
  const ctorStack: number[] = [];
  let skipComma = true;
  for (const item of path) {
    while (true) {
      const last = ctorStack.pop();
      if (last == null) break;

      if (last === 0) {
        result += ')';
      } else {
        ctorStack.push(last - 1);
        break;
      }
    }
    if (!skipComma) result += ', ';
    skipComma = false;
    switch (item.kind) {
      case 'tag':
        result += item.tag;
        if (item.paramsCount > 0) {
          result += '(';
        }
        skipComma = true;
        if (item.paramsCount > 0) {
          ctorStack.push(item.paramsCount);
        }
        break;
      case 'simple':
        result += item.value;
        break;
    }
  }
  if (ctorStack.pop() == 0) {
    result += ')';
  }

  return result;
}
