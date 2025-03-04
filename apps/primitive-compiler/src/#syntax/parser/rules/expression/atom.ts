import {
  Block,
  functionCallArgument,
  MutName,
  NameExpression,
} from '@/#core/#ast/index.js';
import { TokenKind } from '@/#syntax/common/index.js';
import {
  cut,
  oneof,
  opt,
  repeat0,
  repeat1,
  rule,
  separatedList1,
  seq,
  token,
} from '../../base.js';
import { astId, identifier, path, withNewlineAsSemi } from '../fragments.js';
import { expression } from './index.js';

export const atomExpression = rule(() =>
  oneof('Expression', expressions.name, expressions.block),
);

export const expressions = {
  name: rule(() =>
    seq(path, astId).map(([path, id]) => new NameExpression(id, path)),
  ),
  block: rule(() =>
    seq(
      token(TokenKind.PunctuationLeftCurlyBracket),
      cut(
        withNewlineAsSemi(
          seq(
            repeat0(token(TokenKind.SpaceVertical)),
            opt(
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
            ),
            repeat0(token(TokenKind.SpaceVertical)),
            token(TokenKind.PunctuationRightCurlyBracket),
          ),
        ),
      ),
      astId,
    ),
  ).map(([_, expressions, id]) => new Block(id, expressions?.[1] ?? [])),
};

const functionCallArgument = rule<functionCallArgument>(() =>
  oneof(
    'Function Argument',
    seq(token(TokenKind.KeywordMut), cut(seq(identifier, astId))).map(
      ([_0, [ident, id]]) => new MutName(id, ident),
    ),
  ),
);
