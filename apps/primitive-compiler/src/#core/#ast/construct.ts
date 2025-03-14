import { Span, Token } from '../../#syntax/index.js';

export class Identifier {
  constructor(readonly token: Token) {}

  get span(): Span {
    return this.token.span;
  }

  toString(): string {
    return `Ident(${JSON.stringify(this.token.content)}, ${this.token.span})`;
  }
}

export class Path {
  constructor(readonly segments: Identifier[]) {}

  get span(): Span {
    return Span.wrapping(...this.segments.map((segment) => segment.span));
  }

  get display(): string {
    return this.segments.map((ident) => ident.token.content).join('.');
  }

  toString(): string {
    return `Path([${this.display}])`;
  }
}
