import { handsum, type Handsum } from 'handsum';
import { RefinedTypeV, type TypeC } from '../type';
import { type ImplKind } from './common';
import { TypedValue, Value, type ValueF } from './value';

interface IComputationCommon<TImplKey extends ImplKind> {
  decorate?: (content: string, data: MetaOf<TImplKey>) => string;
  display(this: ComputationF<TImplKey>): string;
}
interface TComputationImplSet {
  untyped: {
    meta: void;
    impl: IComputationCommon<'untyped'> & {
      thunk(this: ComputationF<'untyped'>): ValueF<'untyped'>;
      annotate(
        this: ComputationF<'untyped'>,
        type: TypeC,
      ): ComputationF<'untyped'>;
      sub_v(
        this: ComputationF<'untyped'>,
        name: string,
        value: ValueF<'untyped'>,
      ): ComputationF<'untyped'>;
    };
  };
  typed: {
    meta: { type: TypeC };
    impl: IComputationCommon<'typed'> & {
      getType(this: ComputationF<'typed'>): TypeC;
      sub_v(
        this: ComputationF<'typed'>,
        name: string,
        value: ValueF<'typed'>,
      ): ComputationF<'typed'>;
    };
  };
}
type MetaOf<TImplKey extends ImplKind> = TComputationImplSet[TImplKey]['meta'];
interface TComputation<TImplKey extends ImplKind> {
  Annotate(
    target: ComputationF<TImplKey>,
    type: TypeC,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  Produce(
    value: ValueF<TImplKey>,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  Force(
    value: ValueF<TImplKey>,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  Apply(
    fn: ComputationF<TImplKey>,
    param: ValueF<TImplKey>,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  Resolve(
    bundle: ComputationF<TImplKey>,
    tag: string,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  Lambda(
    param: string,
    body: ComputationF<TImplKey>,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  With(
    bundle: Record<string, ComputationF<TImplKey>>,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  Sequence(
    left: ComputationF<TImplKey>,
    name: string,
    right: ComputationF<TImplKey>,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  Def(
    name: string,
    comput: ComputationF<TImplKey>,
    ty: TypeC,
    right: ComputationF<TImplKey>,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  TyAppV(
    body: ValueF<TImplKey>,
    ty: RefinedTypeV,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  TyAppC(
    body: ValueF<TImplKey>,
    ty: TypeC,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  Projection(
    value: ValueF<TImplKey>,
    key: string,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  Match(
    value: ValueF<TImplKey>,
    branches: Record<string, [string, ComputationF<TImplKey>]>,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
}
export type ComputationF<TImplKey extends ImplKind> = Handsum<
  TComputation<TImplKey>,
  TComputationImplSet[TImplKey]['impl']
>;
export type Computation = ComputationF<'untyped'>;
export const Computation = handsum<
  TComputation<'untyped'>,
  TComputationImplSet['untyped']['impl']
>({
  display(): string {
    const decorate = this.decorate ?? ((content, _) => content);
    return this.match({
      Annotate(_0, _1, meta) {
        return decorate(`(${_0.display()}) ⇐ ${_1.display()}`, meta);
      },
      Produce(_0, meta) {
        return decorate(`produce(${_0.display()})`, meta);
      },
      Force(_0, meta) {
        return decorate(`force(${_0.display()})`, meta);
      },
      Apply(_0, _1, meta) {
        return decorate(`(${_0.display()}).apply(${_1.display()})`, meta);
      },
      Resolve(_0, _1, meta) {
        return decorate(`(${_0.display()}) \`${_1}`, meta);
      },
      Lambda(name, _1, meta) {
        return decorate(`λ${name}. (${_1.display()})`, meta);
      },
      With(bundle, meta) {
        return decorate(
          `λ⟨${Object.entries(bundle)
            .map(([tag, body]) => `${tag}. ${body.display()}`)
            .join(', ')}⟩`,
          meta,
        );
      },
      Sequence(_0, _1, _2, meta) {
        return decorate(`${_1} <- (${_0.display()}); ${_2.display()}`, meta);
      },
      TyAppV(_0, _1, meta) {
        return decorate(`(${_0.display()})[${_1.display()}]`, meta);
      },
      TyAppC(_0, _1, meta) {
        return decorate(`(${_0.display()})[${_1.display()}: effect]`, meta);
      },
      Projection(_0, _1, meta) {
        return decorate(`(${_0.display()}).${_1}`, meta);
      },
      Match(_0, _1, meta) {
        return decorate(
          `match(${_0.display()}) {${Object.entries(_1)
            .map(([key, [v, body]]) => `${key} as ${v} => ${body.display()}`)
            .join(', ')}}`,
          meta,
        );
      },
      Def(_0, _1, _2, meta) {
        return decorate(
          `def rec ${_0} = (${_1.display()}) in ${_2.display()}`,
          meta,
        );
      },
    });
  },
  annotate(type: TypeC): Computation {
    return Computation.Annotate(this, type);
  },
  thunk() {
    return Value.Thunk(this);
  },
  sub_v(target: string, newValue: Value): Computation {
    return this.match({
      Annotate(_0, _1, meta) {
        return Computation.Annotate(_0.sub_v(target, newValue), _1, meta);
      },
      Produce(_0, meta) {
        return Computation.Produce(_0.sub_v(target, newValue), meta);
      },
      Force(_0, meta) {
        return Computation.Force(_0.sub_v(target, newValue), meta);
      },
      Apply(_0, _1, meta) {
        return Computation.Apply(
          _0.sub_v(target, newValue),
          _1.sub_v(target, newValue),
          meta,
        );
      },
      Resolve(_0, _1, meta) {
        return Computation.Resolve(_0.sub_v(target, newValue), _1, meta);
      },
      Lambda(_0, _1, meta) {
        return Computation.Lambda(_0, _1.sub_v(target, newValue), meta);
      },
      With(_0, meta) {
        return Computation.With(
          Object.fromEntries(
            Object.entries(_0).map(([k, v]) => [k, v.sub_v(target, newValue)]),
          ),
          meta,
        );
      },
      Sequence(_0, _1, _2, meta) {
        return Computation.Sequence(
          _0.sub_v(target, newValue),
          _1,
          _2.sub_v(target, newValue),
          meta,
        );
      },
      TyAppV(_0, _1, meta) {
        return Computation.TyAppV(_0.sub_v(target, newValue), _1, meta);
      },
      TyAppC(_0, _1, meta) {
        return Computation.TyAppC(_0.sub_v(target, newValue), _1, meta);
      },
      Projection(_0, _1, meta) {
        return Computation.Projection(_0.sub_v(target, newValue), _1, meta);
      },
      Match(_0, _1, meta) {
        return Computation.Match(
          _0.sub_v(target, newValue),
          Object.fromEntries(
            Object.entries(_1).map(([k, [tag, v]]) => [
              k,
              [tag, v.sub_v(target, newValue)],
            ]),
          ),
          meta,
        );
      },
      Def(_0, _1, _2, _3, meta) {
        return Computation.Def(
          _0,
          _1.sub_v(target, newValue),
          _2,
          _3.sub_v(target, newValue),
          meta,
        );
      },
    });
  },
});

export type TypedComputation = ComputationF<'typed'>;
export const TypedComputation = handsum<
  TComputation<'typed'>,
  TComputationImplSet['typed']['impl']
>({
  decorate(content, data) {
    return `(${content}): ${data.type.display()}`;
  },
  display(): string {
    const decorate = this.decorate ?? ((content, _) => content);
    return this.match({
      Annotate(_0, _1, meta) {
        return decorate(`(${_0.display()}) ⇐ ${_1.display()}`, meta);
      },
      Produce(_0, meta) {
        return decorate(`produce(${_0.display()})`, meta);
      },
      Force(_0, meta) {
        return decorate(`force(${_0.display()})`, meta);
      },
      Apply(_0, _1, meta) {
        return decorate(`(${_0.display()}).apply(${_1.display()})`, meta);
      },
      Resolve(_0, _1, meta) {
        return decorate(`(${_0.display()}) \`${_1}`, meta);
      },
      Lambda(name, _1, meta) {
        return decorate(`λ${name}. (${_1.display()})`, meta);
      },
      With(bundle, meta) {
        return decorate(
          `λ⟨${Object.entries(bundle)
            .map(([tag, body]) => `${tag}. ${body.display()}`)
            .join(', ')}⟩`,
          meta,
        );
      },
      Sequence(_0, _1, _2, meta) {
        return decorate(`${_1} <- (${_0.display()}); ${_2.display()}`, meta);
      },
      TyAppV(_0, _1, meta) {
        return decorate(`(${_0.display()})[${_1.display()}]`, meta);
      },
      TyAppC(_0, _1, meta) {
        return decorate(`(${_0.display()})[${_1.display()}: effect]`, meta);
      },
      Projection(_0, _1, meta) {
        return decorate(`(${_0.display()}).${_1}`, meta);
      },
      Match(_0, _1, meta) {
        return decorate(
          `match(${_0.display()}) {${Object.entries(_1)
            .map(([key, [v, body]]) => `${key} as ${v} => ${body.display()}`)
            .join(', ')}}`,
          meta,
        );
      },
      Def(_0, _1, _2, _3, meta) {
        return decorate(
          `def rec ${_0}: ${_2.display()} = (${_1.display()}) in ${_2.display()}`,
          meta,
        );
      },
    });
  },
  getType(): TypeC {
    return this.match({
      Annotate(_0, _1, meta) {
        return meta.type;
      },
      Produce(_0, meta) {
        return meta.type;
      },
      Force(_0, meta) {
        return meta.type;
      },
      Apply(_0, _1, meta) {
        return meta.type;
      },
      Resolve(_0, _1, meta) {
        return meta.type;
      },
      Lambda(_0, _1, meta) {
        return meta.type;
      },
      With(_0, meta) {
        return meta.type;
      },
      Sequence(_0, _1, _2, meta) {
        return meta.type;
      },
      TyAppV(_0, _1, meta) {
        return meta.type;
      },
      TyAppC(_0, _1, meta) {
        return meta.type;
      },
      Projection(_0, _1, meta) {
        return meta.type;
      },
      Match(_0, _1, meta) {
        return meta.type;
      },
      Def(_0, _1, _2, _3, meta) {
        return meta.type;
      },
    });
  },
  sub_v(name: string, value: TypedValue): TypedComputation {
    return this.match({
      Annotate(_0, _1, meta) {
        return TypedComputation.Annotate(_0.sub_v(name, value), _1, meta);
      },
      Produce(_0, meta) {
        return TypedComputation.Produce(_0.sub_v(name, value), meta);
      },
      Force(_0, meta) {
        return TypedComputation.Force(_0.sub_v(name, value), meta);
      },
      Apply(_0, _1, meta) {
        return TypedComputation.Apply(
          _0.sub_v(name, value),
          _1.sub_v(name, value),
          meta,
        );
      },
      Resolve(_0, _1, meta) {
        return TypedComputation.Resolve(_0.sub_v(name, value), _1, meta);
      },
      Lambda(_0, _1, meta) {
        return TypedComputation.Lambda(_0, _1.sub_v(name, value), meta);
      },
      With(_0, meta) {
        return TypedComputation.With(
          Object.fromEntries(
            Object.entries(_0).map(([key, target]) => [
              key,
              target.sub_v(name, value),
            ]),
          ),
          meta,
        );
      },
      Sequence(_0, _1, _2, meta) {
        return TypedComputation.Sequence(
          _0.sub_v(name, value),
          _1,
          _2.sub_v(name, value),
          meta,
        );
      },
      TyAppV(_0, _1, meta) {
        return TypedComputation.TyAppV(_0.sub_v(name, value), _1, meta);
      },
      TyAppC(_0, _1, meta) {
        return TypedComputation.TyAppC(_0.sub_v(name, value), _1, meta);
      },
      Projection(_0, _1, meta) {
        return TypedComputation.Projection(_0.sub_v(name, value), _1, meta);
      },
      Match(_0, _1, meta) {
        return TypedComputation.Match(_0.sub_v(name, value), _1, meta);
      },
      Def(_0, _1, _2, _3, meta) {
        return TypedComputation.Def(
          _0,
          _1.sub_v(name, value),
          _2,
          _3.sub_v(name, value),
          meta,
        );
      },
    });
  },
});
