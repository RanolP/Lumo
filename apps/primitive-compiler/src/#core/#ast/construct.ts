import { Token } from '../../#syntax';

export class Identifier {
  constructor(readonly token: Token) {}

  toString(): string {
    return `Ident(${JSON.stringify(this.token.content)}, ${this.token.span})`;
  }
}

export class Type {}
