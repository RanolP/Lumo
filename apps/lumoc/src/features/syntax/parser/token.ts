import { ExpectedError } from '../../../vendors/malssi/parser/errors';
import { Token } from '../lexer';
import { parser } from './base';

const expectToken = <R>(label: string, fn: (token: Token) => R | undefined) =>
  parser.raw((i) => {
    let token: Token | null;
    do {
      token = i.next();
      if (token?.HorizontalSpace) continue;
      if (!i.context.isBlock && token?.VerticalSpace) continue;

      break;
    } while (token);
    if (!token) throw new ExpectedError(i, label, 'end of input');
    const result = fn(token);
    if (result !== undefined) return result;
    throw new ExpectedError(i, label, JSON.stringify(token));
  });

export const tok = {
  kw: {
    Enum: expectToken(`'enum'`, (token) => token.KeywordEnum).void(),
  },
  punct: {
    comma: expectToken(`','`, (token) => token.PunctuationComma).void(),
    colon: expectToken(`':'`, (token) => token.PunctuationColon).void(),
    paren: {
      l: expectToken(`'('`, (token) => token.PunctuationLeftParenthesis).void(),
      r: expectToken(
        `')'`,
        (token) => token.PunctuationRightParenthesis,
      ).void(),
    },
    curly: {
      l: expectToken(
        `'{'`,
        (token) => token.PunctuationLeftCurlyBracket,
      ).void(),
      r: expectToken(
        `'}'`,
        (token) => token.PunctuationRightCurlyBracket,
      ).void(),
    },
  },
  Ident: expectToken(`identifier`, (token) => token.Identifier?.[0]),
};
