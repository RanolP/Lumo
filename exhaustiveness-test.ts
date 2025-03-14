const NAME_BINDING = Symbol.for('NAME_BINDING');
type ExprType =
  | { kind: 'Sum'; items: Extract<ExprType, { kind: 'Constructor' }>[] }
  | { kind: 'Constructor'; tag: string; arguments: ExprType[] }
  | { kind: 'Tuple'; items: ExprType[] }
  | { kind: 'TypeVar'; name: string };
type Pattern =
  | { kind: 'Destructor'; tag: string; arguments: Pattern[] }
  | { kind: 'Tuple'; items: Pattern[] }
  | { kind: 'NameBinding'; name: string };
type DecisionTreeNode = {
  [tag: string]: DecisionTreeNode;
  [NAME_BINDING]?: DecisionTreeNode;
};
function stringifyType(ty: ExprType): string {
  switch (ty.kind) {
    case 'Constructor':
      return ty.tag;
    case 'TypeVar':
      return ty.name;
    case 'Sum':
      return `(${ty.items.map(stringifyType).join('+')})`;
    case 'Tuple':
      return `${ty.items.map(stringifyType).join('×')}`;
  }
}
function appendPatternToTree(
  root: DecisionTreeNode,
  pattern: Pattern,
): DecisionTreeNode {
  switch (pattern.kind) {
    case 'Destructor': {
      if (!(pattern.tag in root)) {
        root[pattern.tag] = {};
      }
      return appendPatternToTree(root[pattern.tag], {
        kind: 'Tuple',
        items: pattern.arguments,
      });
    }
    case 'Tuple': {
      if (pattern.items.length === 0) return root;
      const [head, ...tail] = pattern.items;
      const afterHead = appendPatternToTree(root, head);

      return appendPatternToTree(afterHead, {
        kind: 'Tuple',
        items: tail,
      });
    }
    case 'NameBinding': {
      root[NAME_BINDING] = {};
      return root[NAME_BINDING];
    }
  }
}
function findUncoveredCase(
  tree: DecisionTreeNode,
  exprType: ExprType,
): [DecisionTreeNode[], string | null] {
  if (exprType.kind === 'Tuple') {
    if (exprType.items.length === 0) {
      return [[tree], null];
    }
    const [head, ...tail] = exprType.items;
    const [next, err] = findUncoveredCase(tree, head);
    if (err != null) return [[], err];
    return next.reduce<[DecisionTreeNode[], string | null]>(
      ([oldNodes, err], node) => {
        if (err) return [[], err];
        const [newNodes, newErr] = findUncoveredCase(node, {
          kind: 'Tuple',
          items: tail,
        });
        if (newErr != null) return [[], stringifyType(head) + '×' + newErr];
        return [[...oldNodes, ...newNodes], null];
      },
      [[], null],
    );
  }
  if (NAME_BINDING in tree) return [[tree[NAME_BINDING]!], null];
  switch (exprType.kind) {
    case 'Sum': {
      return exprType.items.reduce<[DecisionTreeNode[], string | null]>(
        ([oldNodes, err], ty) => {
          if (err) return [[], err];
          if (!(ty.tag in tree)) return [[], ty.tag];
          const [newNodes, newErr] = findUncoveredCase(tree, ty);
          if (newErr) return [[], newErr];
          return [[...oldNodes, ...newNodes], null];
        },
        [[], null],
      );
    }
    case 'Constructor': {
      if (!(exprType.tag in tree)) {
        return [[], stringifyType(exprType)];
      }
      const [nodes, err] = findUncoveredCase(tree[exprType.tag], {
        kind: 'Tuple',
        items: exprType.arguments,
      });
      if (err) return [[], exprType.tag + '×' + err];
      return [nodes, null];
    }
    case 'TypeVar': {
      return [[], exprType.name];
    }
  }
}

// #region Helpers
const TypeRef = (name: string): ExprType => ({ kind: 'TypeVar', name });
const Let = (name: string): Pattern => ({ kind: 'NameBinding', name });
const Tuple = {
  of: (...arguments: ExprType[]): ExprType => ({
    kind: 'Constructor',
    tag: `Tuple${arguments.length}`,
    arguments,
  }),
  match: (...arguments: Pattern[]): Pattern => ({
    kind: 'Destructor',
    tag: `Tuple${arguments.length}`,
    arguments,
  }),
};
const Maybe = {
  of: (value: ExprType): ExprType => ({
    kind: 'Sum',
    items: [
      { kind: 'Constructor', tag: 'Maybe.nothing', arguments: [] },
      { kind: 'Constructor', tag: 'Maybe.just', arguments: [value] },
    ],
  }),
  just: (value: Pattern): Pattern => ({
    kind: 'Destructor',
    tag: 'Maybe.just',
    arguments: [value],
  }),
  nothing: {
    kind: 'Destructor',
    tag: 'Maybe.nothing',
    arguments: [],
  } satisfies Pattern,
};
const List = {
  of: (value: ExprType): ExprType => {
    const list = {
      kind: 'Sum',
      items: [
        { kind: 'Constructor', tag: 'List.nil', arguments: [] },
        {
          kind: 'Constructor',
          tag: 'List.cons',
          arguments: [value],
        },
      ],
    } satisfies ExprType;
    list.items[1].arguments.push(list);
    return list;
  },
  cons: (value: Pattern, tail: Pattern): Pattern => ({
    kind: 'Destructor',
    tag: 'List.cons',
    arguments: [value, tail],
  }),
  nil: { kind: 'Destructor', tag: 'List.nil', arguments: [] } satisfies Pattern,
};

const root: DecisionTreeNode = {};
appendPatternToTree(
  root,
  Tuple.match(Maybe.just(Let('a')), Maybe.just(Let('b'))),
);
appendPatternToTree(root, Tuple.match(Maybe.nothing, Maybe.just(Let('b'))));
appendPatternToTree(root, Tuple.match(Maybe.just(Let('a')), Maybe.nothing));
appendPatternToTree(root, Tuple.match(Maybe.nothing, Maybe.nothing));

// console.log('root {\n', debugTree(root, 4), '}');

console.log(
  findUncoveredCase(
    root,
    Tuple.of(Maybe.of(TypeRef('Int')), Maybe.of(TypeRef('Int'))),
  )[1],
);
