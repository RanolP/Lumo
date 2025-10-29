import { handsum, Handsum } from 'handsum';
import { RefinedTypeV } from '../type';
import { ImplKind } from './common';
import { Computation, ComputationF } from './computation';

interface IValueCommon<TImplKey extends ImplKind> {
  decorate?: (content: string, data: MetaOf<TImplKey>) => string;
  display(this: ValueF<TImplKey>): string;
}
interface TValueImplSet {
  untyped: {
    meta: void;
    impl: IValueCommon<'untyped'> & {
      annotate(this: ValueF<'untyped'>, ty: RefinedTypeV): ValueF<'untyped'>;
      roll(this: ValueF<'untyped'>): ValueF<'untyped'>;
      inject(this: ValueF<'untyped'>, tag: string): ValueF<'untyped'>;
      ret(this: ValueF<'untyped'>): ComputationF<'untyped'>;
      force(this: ValueF<'untyped'>): ComputationF<'untyped'>;
    };
  };
  typed: {
    meta: { type: RefinedTypeV };
    impl: IValueCommon<'typed'> & {
      getType(this: ValueF<'typed'>): RefinedTypeV;
    };
  };
}
type MetaOf<TImplKey extends ImplKind> = TValueImplSet[TImplKey]['meta'];
interface TValue<TImplKey extends ImplKind> {
  Annotate(
    target: ValueF<TImplKey>,
    type: RefinedTypeV,
    meta: MetaOf<TImplKey>,
  ): ValueF<TImplKey>;
  Roll(value: ValueF<TImplKey>, meta: MetaOf<TImplKey>): ValueF<TImplKey>;
  Unroll(value: ValueF<TImplKey>, meta: MetaOf<TImplKey>): ValueF<TImplKey>;
  Injection(
    tag: string,
    value: ValueF<TImplKey>,
    meta: MetaOf<TImplKey>,
  ): ValueF<TImplKey>;
  Variable(name: string, meta: MetaOf<TImplKey>): ValueF<TImplKey>;
  Thunk(body: ComputationF<TImplKey>, meta: MetaOf<TImplKey>): ValueF<TImplKey>;
  TyAbsV(
    name: string,
    body: ValueF<TImplKey>,
    meta: MetaOf<TImplKey>,
  ): ValueF<TImplKey>;
  TyAbsC(
    name: string,
    body: ValueF<TImplKey>,
    meta: MetaOf<TImplKey>,
  ): ValueF<TImplKey>;
  Record(
    entries: Record<string, ValueF<TImplKey>>,
    meta: MetaOf<TImplKey>,
  ): ValueF<TImplKey>;
  Variant(
    tag: string,
    entries: Record<string, ValueF<TImplKey>>,
    meta: MetaOf<TImplKey>,
  ): ValueF<TImplKey>;
}
export type ValueF<TImplKey extends ImplKind> = Handsum<
  TValue<TImplKey>,
  TValueImplSet[TImplKey]['impl']
>;

export type Value = ValueF<'untyped'>;
export const Value = handsum<
  TValue<'untyped'>,
  TValueImplSet['untyped']['impl']
>({
  display() {
    const decorate = this.decorate ?? ((content, _) => content);
    return this.match({
      Annotate(expr, ty, meta) {
        return decorate(`(${expr.display()}) ⇐ ${ty.display()}`, meta);
      },
      Roll(expr, meta) {
        return decorate(`roll(${expr.display()})`, meta);
      },
      Unroll(expr, meta) {
        return decorate(`unroll(${expr.display()})`, meta);
      },
      Injection(tag, expr, meta) {
        return decorate(`inj_${JSON.stringify(tag)}(${expr.display()})`, meta);
      },
      Variable(name, meta) {
        return decorate(`var(${name})`, meta);
      },
      Thunk(body, meta) {
        return decorate(`thunk(${body.display()})`, meta);
      },
      TyAbsV(name, body, meta) {
        return decorate(`tyAbsV(${name}, ${body.display()})`, meta);
      },
      TyAbsC(name, body, meta) {
        return decorate(`tyAbsC(${name}, ${body.display()})`, meta);
      },
      Record(entries, meta) {
        return decorate(
          `record {${Object.entries(entries)
            .map(([key, value]) => `${key}: ${value.display()}`)
            .join(',')}}`,
          meta,
        );
      },
      Variant(tag, entries, meta) {
        return decorate(
          `variant[${tag}] {${Object.entries(entries)
            .map(([key, value]) => `${key}: ${value.display()}`)
            .join(',')}}`,
          meta,
        );
      },
    });
  },
  annotate(ty: RefinedTypeV): Value {
    return Value.Annotate(this, ty);
  },
  roll(): Value {
    return Value.Roll(this);
  },
  inject(tag: string): Value {
    return Value.Injection(tag, this);
  },
  ret(): Computation {
    return Computation.Return(this);
  },
  force(): Computation {
    return Computation.Force(this);
  },
});

export type TypedValue = ValueF<'typed'>;
export const TypedValue = handsum<
  TValue<'typed'>,
  TValueImplSet['typed']['impl']
>({
  display() {
    const decorate = this.decorate ?? ((content, _) => content);
    return this.match({
      Annotate(expr, ty, meta) {
        return decorate(`(${expr.display()}) ⇐ ${ty.display()}`, meta);
      },
      Roll(expr, meta) {
        return decorate(`roll(${expr.display()})`, meta);
      },
      Unroll(expr, meta) {
        return decorate(`unroll(${expr.display()})`, meta);
      },
      Injection(tag, expr, meta) {
        return decorate(`inj_${JSON.stringify(tag)}(${expr.display()})`, meta);
      },
      Variable(name, meta) {
        return decorate(`var(${name})`, meta);
      },
      Thunk(body, meta) {
        return decorate(`thunk(${body.display()})`, meta);
      },
      TyAbsV(name, body, meta) {
        return decorate(`tyAbsV(${name}, ${body.display()})`, meta);
      },
      TyAbsC(name, body, meta) {
        return decorate(`tyAbsC(${name}, ${body.display()})`, meta);
      },
      Record(entries, meta) {
        return decorate(
          `record {${Object.entries(entries)
            .map(([key, value]) => `${key}: ${value.display()}`)
            .join(',')}}`,
          meta,
        );
      },
      Variant(tag, entries, meta) {
        return decorate(
          `variant[${tag}] {${Object.entries(entries)
            .map(([key, value]) => `${key}: ${value.display()}`)
            .join(',')}}`,
          meta,
        );
      },
    });
  },
  decorate(content, data) {
    return `${content}: ${data.type.display()}`;
  },
  getType(): RefinedTypeV {
    return this.match({
      Annotate(_0, _1, meta) {
        return meta.type;
      },
      Roll(_, meta) {
        return meta.type;
      },
      Unroll(_, meta) {
        return meta.type;
      },
      Injection(_0, _1, meta) {
        return meta.type;
      },
      Variable(_, meta) {
        return meta.type;
      },
      Thunk(_, meta) {
        return meta.type;
      },
      TyAbsV(_0, _1, meta) {
        return meta.type;
      },
      TyAbsC(_0, _1, meta) {
        return meta.type;
      },
      Record(_, meta) {
        return meta.type;
      },
      Variant(_0, _1, meta) {
        return meta.type;
      },
    });
  },
});
