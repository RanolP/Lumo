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
    body: TsExpr,
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
export type TsExpr = Handsum<TTsExpr>;
export const TsExpr = handsum<TTsExpr>({});

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
