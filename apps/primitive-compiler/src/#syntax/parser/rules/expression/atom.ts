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
import {
  astId,
  identifier,
  path,
  spanning,
  withNewlineAsSemi,
} from '../fragments.js';
import { expression } from './index.js';

export const atomExpression = rule(() =>
  oneof('Expression', expressions.name, expressions.block),
);

export const expressions = {
  name: rule(() =>
    spanning(seq(path, astId)).map(
      ([[path, id], span]) => new NameExpression(id, span, path),
    ),
  ),
  block: rule(() =>
    spanning(
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
    ),
  ).map(
    ([[_, expressions, id], span]) =>
      new Block(id, span, expressions?.[1] ?? []),
  ),
};

const functionCallArgument = rule<functionCallArgument>(() =>
  oneof(
    'Function Argument',
    spanning(seq(token(TokenKind.KeywordMut), cut(seq(identifier, astId)))).map(
      ([[_0, [ident, id]], span]) => new MutName(id, span, ident),
    ),
  ),
);
