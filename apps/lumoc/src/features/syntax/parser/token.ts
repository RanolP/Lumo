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
    Type: expectToken(`'type'`, (token) => token.KeywordType).void(),
    Fn: expectToken(`'fn'`, (token) => token.KeywordFn).void(),
    Let: expectToken(`'let'`, (token) => token.KeywordLet).void(),
    Produce: expectToken(`'produce'`, (token) => token.KeywordProduce).void(),
    Thunk: expectToken(`'thunk'`, (token) => token.KeywordThunk).void(),
    Force: expectToken(`'force'`, (token) => token.KeywordForce).void(),
    Roll: expectToken(`'roll'`, (token) => token.KeywordRoll).void(),
    Unroll: expectToken(`'unroll'`, (token) => token.KeywordUnroll).void(),
    Bundle: expectToken(`'bundle'`, (token) => token.KeywordBundle).void(),
    Match: expectToken(`'match'`, (token) => token.KeywordMatch).void(),
    As: expectToken(`'as'`, (token) => token.KeywordAs).void(),
  },
  punct: {
    sum: expectToken(`'∑'`, (token) => token.PunctuationSum).void(),
    forall: expectToken(`'∀'`, (token) => token.PunctuationForall).void(),
    mu: expectToken(`'μ'`, (token) => token.PunctuationMu).void(),
    equals: expectToken(`'='`, (token) => token.PunctuationEquals).void(),
    comma: expectToken(`','`, (token) => token.PunctuationComma).void(),
    fullStop: expectToken(`'.'`, (token) => token.PunctuationFullStop).void(),
    colon: expectToken(`':'`, (token) => token.PunctuationColon).void(),
    underscore: expectToken(
      `'_'`,
      (token) => token.PunctuationUnderscore,
    ).void(),

    reverseSolidus: expectToken(
      `'\\'`,
      (token) => token.PunctuationReverseSolidus,
    ).void(),
    paren: {
      l: expectToken(`'('`, (token) => token.PunctuationLeftParenthesis).void(),
      r: expectToken(
        `')'`,
        (token) => token.PunctuationRightParenthesis,
      ).void(),
    },
    square: {
      l: expectToken(
        `'['`,
        (token) => token.PunctuationLeftSquareBracket,
      ).void(),
      r: expectToken(
        `']'`,
        (token) => token.PunctuationRightSquareBracket,
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
    leftArrow: expectToken(
      `'<-'`,
      (token) => token.PunctuationsLeftArrow,
    ).void(),
    arrow: expectToken(`'->'`, (token) => token.PunctuationsArrow).void(),
    fatArrow: expectToken(`'=>'`, (token) => token.PunctuationsFatArrow).void(),
    semicolon: expectToken(`';'`, (token) => token.PunctuationSemicolon).void(),
  },
  Ident: expectToken(`identifier`, (token) => token.Identifier?.[0]),
  Tag: expectToken(`tag`, (token) => token.Tag?.[0]),
};
