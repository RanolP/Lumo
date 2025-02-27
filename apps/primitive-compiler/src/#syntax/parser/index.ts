import { AstId } from '../../#core/#ast/base';
import { oneof, repeat0, rule } from './base';
import { definition } from './rules/definition';
import { expression } from './rules/expression';

export const program = rule(() =>
  repeat0(oneof('Item', definition, expression)),
).map((items) =>
  items.filter(
    (x): x is Exclude<typeof x, void> =>
      typeof x === 'object' && x.id instanceof AstId,
  ),
);
