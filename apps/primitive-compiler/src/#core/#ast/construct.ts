import { Token } from '../../#syntax/index.js';

export class Identifier {
  constructor(readonly token: Token) {}

  toString(): string {
    return `Ident(${JSON.stringify(this.token.content)}, ${this.token.span})`;
  }
}

export class Path {
  constructor(readonly segments: Identifier[]) {}

  toString(): string {
    return `Path([${this.segments
      .map((segment) => segment.toString())
      .join('.')}])`;
  }
}
