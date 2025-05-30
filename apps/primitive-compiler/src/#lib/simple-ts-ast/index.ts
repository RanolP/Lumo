import { TsExpr } from './expr.js';

export type TsAst = TsAst[] | Define | ExprStmt;

export class Define {
  constructor(readonly name: string, readonly expr: TsExpr) {}
}

export class ExprStmt {
  constructor(readonly expr: TsExpr) {}
}
