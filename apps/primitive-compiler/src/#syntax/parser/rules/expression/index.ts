import { oneof, rule } from '../../base';
import { operator } from './operator';

export const expression = rule(() => oneof('Expression', operator));
