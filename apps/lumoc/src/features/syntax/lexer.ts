import { handsum, Handsum } from 'handsum';
import { lexer } from '../../vendors/malssi/lexer';
import escape from 'regexp.escape';

const Keywords = {
  Enum: 'enum',
  Fn: 'fn',
} as const;

const Punctuations = {
  single: {
    Comma: ',',
    Colon: ':',
    LeftParenthesis: '(',
    RightParenthesis: ')',
    LeftCurlyBracket: '{',
    RightCurlyBracket: '}',
  },
  sequence: {
    FatArrow: '=>',
  },
} as const;

type TToken = {
  HorizontalSpace(content: string): Token;
  VerticalSpace(content: string): Token;
  Identifier(content: string): Token;

  // == keywords ==
} & {
  [K in keyof typeof Keywords as `Keyword${K}`]: (content: string) => Token;
} & {
  [K in keyof typeof Punctuations.single as `Punctuation${K}`]: (
    content: string,
  ) => Token;
} & {
  [K in keyof typeof Punctuations.sequence as `Punctuations${K}`]: (
    content: string,
  ) => Token;
};
export type Token = Handsum<TToken>;
export const Token = handsum<TToken>({});

export const Lexer = lexer<Token>()
  .rule(/(?<content>[ \t]+)/, ({ content }) => Token.HorizontalSpace(content!))
  .rule(/(?<content>\r\n|[\r\n])+/, ({ content }) =>
    Token.VerticalSpace(content!),
  )
  .ruleset(
    Object.entries(Keywords).map(
      ([key, value]) =>
        [
          new RegExp(`(?<content>${escape(value)})`),
          ({ content }) =>
            Token[`Keyword${key as keyof typeof Keywords}`](content!),
        ] as const,
    ),
  )
  .ruleset(
    Object.entries(Punctuations.single).map(
      ([key, value]) =>
        [
          new RegExp(`(?<content>${escape(value)})`),
          ({ content }) =>
            Token[`Punctuation${key as keyof typeof Punctuations.single}`](
              content!,
            ),
        ] as const,
    ),
  )
  .ruleset(
    Object.entries(Punctuations.sequence).map(
      ([key, value]) =>
        [
          new RegExp(`(?<content>${escape(value)})`),
          ({ content }) =>
            Token[`Punctuations${key as keyof typeof Punctuations.sequence}`](
              content!,
            ),
        ] as const,
    ),
  )
  .rule(/(?<content>\p{ID_Start}\p{ID_Continue}*)/u, ({ content }) =>
    Token.Identifier(content!),
  );
