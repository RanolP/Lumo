import { oneof, rule } from '../base';
import { identifier } from './fragments';

export const type = rule(() => oneof('type', types.ident));

const types = {
  ident: rule(() => identifier),
};
