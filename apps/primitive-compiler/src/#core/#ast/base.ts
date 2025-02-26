export interface IAstNode {
  readonly id: AstId;
}

export class AstId {
  constructor(readonly handle: number) {}
}
