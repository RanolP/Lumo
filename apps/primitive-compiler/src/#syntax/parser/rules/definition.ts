import { cut, oneof, opt, rule, separatedList1, seq, token } from '../base';
import { TokenKind } from '../../common';
import {
  astId,
  identifier,
  tupleTypeBody,
  withoutNewlineAsSemi,
} from './fragments';
import { EnumBranch, EnumDefinition } from '../../../#core/#ast';

export const definition = rule(() => oneof('Definition', definitions.enum));

export const definitions = {
  enum: rule(() =>
    seq(
      token(TokenKind.KeywordEnum),
      cut(
        seq(
          identifier,
          token(TokenKind.PunctuationLeftCurlyBracket),
          withoutNewlineAsSemi(
            seq(
              opt(
                seq(
                  separatedList1(enumBranch, token(TokenKind.PunctuationComma)),
                  opt(token(TokenKind.PunctuationComma)),
                ),
              ),
              token(TokenKind.PunctuationRightCurlyBracket),
            ),
          ),
        ),
      ),
      astId,
    ),
  ).map(
    ([_0, [name, _1, [body, _2]], astId]) =>
      new EnumDefinition(astId, name, body?.[0] ?? []),
  ),
};

const enumBranch = rule(() =>
  seq(identifier, opt(oneof('enum branch body', tupleTypeBody))),
).map(([name]) => new EnumBranch(name));
