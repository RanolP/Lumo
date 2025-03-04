import { Span } from './span.js';
import { TokenKind } from './token-kind.js';

const Token$ = Symbol('Token');
export type Token = {
  __tag$: typeof Token$;
  toString(): string;

  kind: TokenKind;
  content: string;
  span: Span;
};
function make(data: Omit<Token, '__tag$' | typeof Symbol.toStringTag>): Token {
  return {
    ...data,
    __tag$: Token$,
    toString() {
      return `Token.${data.kind}(${JSON.stringify(data.content)}, ${
        data.span
      })`;
    },
  };
}
export const Token = { make };
