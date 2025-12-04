import { handsum, type Handsum } from 'handsum';

interface TTsType {
  Variable(name: string): TsType;
  UntaggedUnion(types: TsType[]): TsType;
  Object(entries: { name: Key; type: TsType }[]): TsType;
  StringLiteral(value: string): TsType;
  Lambda(
    param: { name: string; type?: TsType | undefined }[],
    typeParams: string[],
    body: TsType,
  ): TsType;
  TypeApplication(body: TsType, typeParams: TsType[]): TsType;
}
export interface ITsType {
  sub(this: TsType, name: string, type: TsType): TsType;
}
export type TsType = Handsum<TTsType, ITsType>;
export const TsType = handsum<TTsType, ITsType>({
  sub(this: TsType, target: string, targetTy: TsType): TsType {
    return this.match({
      Variable(name) {
        return target === name ? targetTy : TsType.Variable(name);
      },
      UntaggedUnion(types) {
        return TsType.UntaggedUnion(types.map((t) => t.sub(target, targetTy)));
      },
      Object(entries) {
        return TsType.Object(
          entries.map(({ name, type }) => ({
            name,
            type: type.sub(target, targetTy),
          })),
        );
      },
      StringLiteral(value) {
        return TsType.StringLiteral(value);
      },
      Lambda(param, typeParams, body) {
        return TsType.Lambda(
          param.map(({ name, type }) => ({
            name,
            type: type?.sub(target, targetTy),
          })),
          typeParams,
          body.sub(target, targetTy),
        );
      },
      TypeApplication(body, typeParams) {
        return TsType.TypeApplication(
          body.sub(target, targetTy),
          typeParams.map((t) => t.sub(target, targetTy)),
        );
      },
      _() {
        throw new Error('never fails');
      },
    });
  },
});

interface TTsExpr {
  Variable(name: string): TsExpr;
  Lambda(
    params: { name: string; type?: TsType }[],
    typeParams: string[],
    ret: TsType | null,
    body:
      | { kind: 'block'; stmts: TsStatement[] }
      | { kind: 'expr'; expr: TsExpr },
  ): TsExpr;
  Apply(fn: TsExpr, param: TsExpr[]): TsExpr;
  Satisfies(expr: TsExpr, type: TsType): TsExpr;
  FieldAccess(object: TsExpr, field: Key): TsExpr;
  Object(entries: { name: Key; value: TsExpr }[]): TsExpr;
  StringLiteral(value: string): TsExpr;
  Ternary(condition: TsExpr, then: TsExpr, otherwise: TsExpr): TsExpr;
  Never(): TsExpr;
  Equals(left: TsExpr, right: TsExpr): TsExpr;
  TypeApplication(body: TsExpr, typeParams: TsType[]): TsExpr;
}
export interface ITsExpr {
  sub(this: TsExpr, target: string, targetExpr: TsExpr): TsExpr;
}
export type TsExpr = Handsum<TTsExpr, ITsExpr>;
export const TsExpr = handsum<TTsExpr, ITsExpr>({
  sub(this: TsExpr, target: string, targetExpr: TsExpr): TsExpr {
    return this.match({
      Variable(name) {
        return name === target ? targetExpr : TsExpr.Variable(name);
      },
      Lambda(params, typeParams, ret, body) {
        return TsExpr.Lambda(
          params,
          typeParams,
          ret,
          body.kind === 'block'
            ? {
                kind: 'block',
                stmts: body.stmts.map((s) => s.sub(target, targetExpr)),
              }
            : { kind: 'expr', expr: body.expr.sub(target, targetExpr) },
        );
      },
      Apply(fn, params) {
        return TsExpr.Apply(
          fn.sub(target, targetExpr),
          params.map((p) => p.sub(target, targetExpr)),
        );
      },
      Satisfies(expr, type) {
        return TsExpr.Satisfies(expr.sub(target, targetExpr), type);
      },
      FieldAccess(object, field) {
        return TsExpr.FieldAccess(object.sub(target, targetExpr), field);
      },
      Object(entries) {
        return TsExpr.Object(
          entries.map(({ name, value }) => ({
            name,
            value: value.sub(target, targetExpr),
          })),
        );
      },
      StringLiteral(value) {
        return TsExpr.StringLiteral(value);
      },
      Ternary(condition, then, otherwise) {
        return TsExpr.Ternary(
          condition.sub(target, targetExpr),
          then.sub(target, targetExpr),
          otherwise.sub(target, targetExpr),
        );
      },
      Never() {
        return TsExpr.Never();
      },
      Equals(left, right) {
        return TsExpr.Equals(
          left.sub(target, targetExpr),
          right.sub(target, targetExpr),
        );
      },
      TypeApplication(body, typeParams) {
        return TsExpr.TypeApplication(body.sub(target, targetExpr), typeParams);
      },
    });
  },
});

type SymbolKey = 'Lumo/tag';
export type Key =
  | { tag: 'string'; value: string }
  | { tag: 'symbol'; value: SymbolKey };

export const Key = {
  str(value: string): Key {
    return { tag: 'string', value };
  },
  sym(value: SymbolKey): Key {
    return { tag: 'symbol', value };
  },
};

interface TTsStatement {
  Const(name: string, value: TsExpr): TsStatement;
  Return(value: TsExpr): TsStatement;
}
interface ITsStatement {
  sub(this: TsStatement, target: string, targetExpr: TsExpr): TsStatement;
}
export type TsStatement = Handsum<TTsStatement, ITsStatement>;
export const TsStatement = handsum<TTsStatement, ITsStatement>({
  sub(this: TsStatement, target: string, targetExpr: TsExpr): TsStatement {
    return this.match({
      Const(name, value) {
        return TsStatement.Const(name, value.sub(target, targetExpr));
      },
      Return(value) {
        return TsStatement.Return(value.sub(target, targetExpr));
      },
    });
  },
});
