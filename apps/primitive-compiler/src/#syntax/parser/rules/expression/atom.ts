import {
  Block,
  functionCallArgument,
  MutName,
  NameExpression,
} from '../../../../#core/#ast';
import { TokenKind } from '../../../common';
import {
  cut,
  oneof,
  repeat1,
  rule,
  separatedList1,
  seq,
  token,
} from '../../base';
import { astId, identifier, withNewlineAsSemi } from '../fragments';
import { expression } from '.';

export const atomExpression = rule(() =>
  oneof('Expression', expressions.name, expressions.block),
);

export const expressions = {
  name: rule(() =>
    seq(identifier, astId).map(([ident, id]) => new NameExpression(id, ident)),
  ),
  block: rule(() =>
    seq(
      token(TokenKind.PunctuationLeftCurlyBracket),
      cut(
        withNewlineAsSemi(
          seq(
            separatedList1(
              expression,
              repeat1(
                oneof(
                  'Separator',
                  token(TokenKind.SpaceVertical),
                  token(TokenKind.PunctuationSemicolon),
                ),
              ),
            ),
            token(TokenKind.PunctuationRightCurlyBracket),
          ),
        ),
      ),
      astId,
    ),
  ).map(([_, [expressions], id]) => new Block(id, expressions)),
};

const functionCallArgument = rule<functionCallArgument>(() =>
  oneof(
    'Function Argument',
    seq(token(TokenKind.KeywordMut), cut(seq(identifier, astId))).map(
      ([_0, [ident, id]]) => new MutName(id, ident),
    ),
  ),
);
