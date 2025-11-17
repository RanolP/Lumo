import { handsum, Handsum } from 'handsum';
import { lexer } from '../../vendors/malssi/lexer';
import escape from 'regexp.escape';

const Keywords = {
  Enum: 'enum',
  Type: 'type',
  Fn: 'fn',
  Let: 'let',
  Produce: 'produce',
  Thunk: 'thunk',
  Force: 'force',
  Roll: 'roll',
  Unroll: 'unroll',
  Bundle: 'bundle',
  Match: 'match',
  As: 'as',
} as const;

const Punctuations = {
  single: {
    Comma: ',',
    FullStop: '.',
    Colon: ':',
    LeftParenthesis: '(',
    RightParenthesis: ')',
    LeftSquareBracket: '[',
    RightSquareBracket: ']',
    LeftCurlyBracket: '{',
    RightCurlyBracket: '}',
    ReverseSolidus: '\\',
    Equals: '=',
    Sum: '∑',
    Mu: 'μ',
    Forall: '∀',
    Semicolon: ';',
    Underscore: '_',
  },
  sequence: {
    LeftArrow: '<-',
    Arrow: '->',
    FatArrow: '=>',
  },
} as const;

type TToken = {
  HorizontalSpace(content: string): Token;
  VerticalSpace(content: string): Token;
  Identifier(content: string): Token;
  Tag(content: string): Token;

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
interface IToken {
  display(this: Token): string;
}
export type Token = Handsum<TToken, IToken>;
export const Token = handsum<TToken, IToken>({
  display() {
    const kw = Object.keys(Keywords)
      .map((key) => this[`Keyword${key as keyof typeof Keywords}`])
      .find((key) => key !== undefined);
    if (kw) return kw[0];
    const punctSingle = Object.keys(Punctuations.single)
      .map(
        (key) => this[`Punctuation${key as keyof typeof Punctuations.single}`],
      )
      .find((key) => key !== undefined);
    if (punctSingle) return punctSingle[0];
    const punctSequence = Object.keys(Punctuations.sequence)
      .map(
        (key) =>
          this[`Punctuations${key as keyof typeof Punctuations.sequence}`],
      )
      .find((key) => key !== undefined);
    if (punctSequence) return punctSequence[0];
    if (this.HorizontalSpace) return this.HorizontalSpace[0];
    if (this.VerticalSpace) return this.VerticalSpace[0];
    if (this.Identifier) return this.Identifier[0];
    if (this.Tag) return `\`${this.Tag[0]}`;

    throw new Error('Invalid token');
  },
});

export const IrLexer = lexer<Token>()
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
  .rule(/(?<content>\p{ID_Start}\p{ID_Continue}*)/u, ({ content }) =>
    Token.Identifier(content!),
  )
  .rule(
    /`(?<content>[A-Za-z][A-Za-z0-9_]*(?:\/[A-Za-z][A-Za-z0-9_]*)?)/u,
    ({ content }) => Token.Tag(content!),
  );
