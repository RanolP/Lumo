import { TsTy } from './ty.js';

export type TsExpr =
  | VarName
  | IfThenElse
  | Fn
  | typeof Unit
  | Seq
  | Assignment
  | typeof TodoExpr;

export class VarName {
  constructor(readonly name: string) {}
}

export class Seq {
  constructor(readonly exprs: TsExpr[]) {}
}

export class Assignment {
  constructor(readonly name: string, readonly value: TsExpr) {}
}

export class IfThenElse {
  constructor(
    readonly condition: TsExpr,
    readonly then: TsExpr,
    readonly otherwise: TsExpr,
  ) {}
}

export class Fn {
  constructor(
    readonly params: [string, TsTy][],
    readonly returnType: TsTy,
    readonly body: TsExpr,
  ) {}
}

export const Unit = Symbol('Unit');

export const TodoExpr = Symbol('TodoExpr');
