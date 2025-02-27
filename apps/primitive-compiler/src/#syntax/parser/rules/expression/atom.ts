import {
  functionCallArgument,
  MutName,
  NameExpression,
} from '../../../../#core/#ast';
import { TokenKind } from '../../../common';
import { cut, oneof, rule, seq, token } from '../../base';
import { astId, identifier } from '../fragments';

export const atomExpression = rule(() => oneof('Expression', expressions.name));

export const expressions = {
  name: rule(() =>
    seq(identifier, astId).map(([ident, id]) => new NameExpression(id, ident)),
  ),
};

const functionCallArgument = rule<functionCallArgument>(() =>
  oneof(
    'Function Argument',
    seq(token(TokenKind.KeywordMut), cut(seq(identifier, astId))).map(
      ([_0, [ident, id]]) => new MutName(id, ident),
    ),
  ),
);
