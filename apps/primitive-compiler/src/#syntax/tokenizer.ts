import { Span, Token, TokenKind } from './common/index.js';

export class Tokenizer {
  private index = 0n;
  constructor(private source: string) {}

  *[Symbol.iterator](): Iterator<Token, void> {
    while (true) {
      const token = this.next();
      if (token == null) break;
      yield token;
    }
  }

  next(): Token | null {
    return (
      this.nextVerticalSpace() ??
      this.nextHorizontalSpace() ??
      this.nextIdentifierOrKeyword() ??
      this.nextPunctuation()
    );
  }

  private peek(): string | null {
    return this.index < this.source.length
      ? this.source[this.index.toString() as unknown as number]
      : null;
  }

  HORIZONTAL_SPACE = [
    '\t', // CHARACTER TABULATION
    ' ', // SPACE
    '\u{00AD}', // SOFT HYPHEN
    '\u{00A0}', // NO-BREAK SPACE
    '\u{1680}', // OGHAM SPACE MARK
    '\u{2000}', // EN QUAD
    '\u{2001}', // EM QUAD
    '\u{2002}', // EN SPACE
    '\u{2003}', // EM SPACE
    '\u{2004}', // THREE-PER-EM SPACE
    '\u{2005}', // FOUR-PER-EM SPACE
    '\u{2006}', // SIX-PER-EM SPACE
    '\u{2007}', // FIGURE SPACE
    '\u{2008}', // PUNCTUATION SPACE
    '\u{2009}', // THIN SPACE
    '\u{200A}', // HAIR SPACE
    '\u{200B}', // ZERO WIDTH SPACE
    '\u{200E}', // LEFT-TO-RIGHT MARK
    '\u{200F}', // RIGHT-TO-LEFT MARK
    '\u{202F}', // NARROW NO-BREAK SPACE
    '\u{205F}', // MEDIUM MATHEMATICAL SPACE
    '\u{3000}', // IDEPGRAPHIC SPACE
    '\u{FEFF}', // ZERO WIDTH NO-BREAK SPACE
  ];
  private nextHorizontalSpace(): Token | null {
    const begin = this.index;

    let content = '';

    let current: string | null;
    do {
      current = this.peek();
      if (current == null || !this.HORIZONTAL_SPACE.includes(current)) break;

      content += current;
      this.index++;
    } while (true);

    if (content.length === 0) return null;

    return Token.make({
      content,
      kind: TokenKind.SpaceHorizotanl,
      span: Span.make({ begin, end: this.index }),
    });
  }

  VERTICAL_SPACE = [
    '\n', // LINE FEED
    '\u{000B}', // LINE TABULATION
    '\u{000C}', // FORM FEED
    '\r', // CARRIAGE RETURN
    '\u{0085}', // NEXT LINE
    '\u{2028}', // LINE SEPARATOR
    '\u{2029}', // PARAGRAPH SEPARATOR
  ];
  private nextVerticalSpace(): Token | null {
    const begin = this.index;

    let content = this.peek();

    if (content == null || !this.VERTICAL_SPACE.includes(content)) return null;
    this.index++;

    if (content === '\r' && this.peek() === '\n') {
      this.index++;

      return Token.make({
        content: '\r\n',
        kind: TokenKind.SpaceVertical,
        span: Span.make({ begin, end: this.index }),
      });
    } else {
      return Token.make({
        content,
        kind: TokenKind.SpaceVertical,
        span: Span.make({ begin, end: this.index }),
      });
    }
  }

  ID_Start = /^\p{ID_Start}$/u;
  ID_Continue = /^\p{ID_Continue}$/u;
  private nextIdentifierOrKeyword(): Token | null {
    const begin = this.index;

    let content = this.peek();
    if (content == null || !this.ID_Start.test(content)) return null;
    this.index++;

    let current: string | null;
    while ((current = this.peek()) != null && this.ID_Continue.test(current)) {
      content += current;
      this.index++;
    }

    let kind: TokenKind =
      {
        enum: TokenKind.KeywordEnum,
        fn: TokenKind.KeywordFn,
        mut: TokenKind.KeywordMut,
        struct: TokenKind.KeywordStruct,
      }[content] ?? TokenKind.Identifier;

    return Token.make({
      content,
      kind,
      span: Span.make({ begin, end: this.index }),
    });
  }

  PunctuationsTrieRoot: TrieNode = {
    children: {
      '!': { conclusion: TokenKind.PunctuationExclamationMark },
      '#': { conclusion: TokenKind.PunctuationNumberSign },
      $: { conclusion: TokenKind.PunctuationDollarSign },
      '%': { conclusion: TokenKind.PunctuationPercentSign },
      '&': { conclusion: TokenKind.PunctuationAmpersand },
      '*': { conclusion: TokenKind.PunctuationAsterisk },
      '+': { conclusion: TokenKind.PunctuationPlusSign },
      ',': { conclusion: TokenKind.PunctuationComma },
      '-': { conclusion: TokenKind.PunctuationHyphenMinus },
      '.': { conclusion: TokenKind.PunctuationFullStop },
      '/': { conclusion: TokenKind.PunctuationSolidus },
      ':': { conclusion: TokenKind.PunctuationColon },
      ';': { conclusion: TokenKind.PunctuationSemicolon },
      '<': { conclusion: TokenKind.PunctuationLessThanSign },
      '=': {
        conclusion: TokenKind.PunctuationEqualsSign,
        children: {
          '>': { conclusion: TokenKind.PunctuationsFatArrow },
        },
      },
      '>': { conclusion: TokenKind.PunctuationGreaterThanSign },
      '?': { conclusion: TokenKind.PunctuationQuestionMark },
      '@': { conclusion: TokenKind.PunctuationCommercialAt },
      '\\': { conclusion: TokenKind.PunctuationReverseSolidus },
      '^': { conclusion: TokenKind.PunctuationCircumflexAccent },
      '|': { conclusion: TokenKind.PunctuationVerticalLine },
      '~': { conclusion: TokenKind.PunctuationTilde },
      '(': { conclusion: TokenKind.PunctuationLeftParenthesis },
      '[': { conclusion: TokenKind.PunctuationLeftSquareBracket },
      '{': { conclusion: TokenKind.PunctuationLeftCurlyBracket },
      ')': { conclusion: TokenKind.PunctuationRightParenthesis },
      ']': { conclusion: TokenKind.PunctuationRightSquareBracket },
      '}': { conclusion: TokenKind.PunctuationRightCurlyBracket },
    },
  };
  nextPunctuation(): Token | null {
    const begin = this.index;

    let lastSuccess: Token | null = null;
    let node = this.PunctuationsTrieRoot;
    let content = '';
    let current: string | null;
    while (
      (current = this.peek()) != null &&
      node.children != null &&
      current in node.children
    ) {
      this.index++;
      node = node.children[current];
      content += current;
      if (node.conclusion != null) {
        lastSuccess = Token.make({
          kind: node.conclusion,
          content,
          span: Span.make({
            begin,
            end: this.index,
          }),
        });
      }
    }

    if (lastSuccess != null) {
      this.index = lastSuccess.span.end;
    }

    return lastSuccess;
  }
}

interface TrieNode {
  conclusion?: TokenKind;
  children?: Record<string, TrieNode>;
}
