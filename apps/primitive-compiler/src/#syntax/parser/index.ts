import { AstId } from '../../#core/#ast/base.js';
import { oneof, repeat0, rule } from './base.js';
import { definition } from './rules/definition.js';
import { expression } from './rules/expression/index.js';

export const program = rule(() =>
  repeat0(oneof('Item', definition, expression)),
).map((items) =>
  items.filter(
    (x): x is Exclude<typeof x, void> =>
      typeof x === 'object' && x.id instanceof AstId,
  ),
);
