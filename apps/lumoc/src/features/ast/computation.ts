import { handsum, Handsum } from 'handsum';
import { RefinedTypeV, TypeC } from '../type';
import { ImplKind } from './common';
import { Value, ValueF } from './value';

interface IComputationCommon<TImplKey extends ImplKind> {
  display(this: ComputationF<TImplKey>): string;
}
interface TComputationImplSet {
  untyped: {
    meta: void;
    impl: IComputationCommon<'untyped'> & {
      thunk(this: ComputationF<'untyped'>): ValueF<'untyped'>;
    };
  };
  typed: {
    meta: { type: TypeC };
    impl: IComputationCommon<'typed'> & {
      getType(this: ComputationF<'typed'>): TypeC;
    };
  };
}
type MetaOf<TImplKey extends ImplKind> = TComputationImplSet[TImplKey]['meta'];
interface TComputation<TImplKey extends ImplKind> {
  Return(
    value: ValueF<TImplKey>,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  Force(
    value: ValueF<TImplKey>,
    meta: MetaOf<TImplKey>,
  ): ComputationF<TImplKey>;
  Apply(
    param: ValueF<TImplKey>,
    fn: ValueF<TImplKey>,
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
    branches: Record<string, ComputationF<TImplKey>>,
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
    return this.match({
      Return(_0, meta) {
        return _0.display();
      },
      Force(_0, meta) {
        return _0.display();
      },
      Apply(_0, _1, meta) {
        return _0.display();
      },
      Lambda(name, _1, meta) {
        return `λ${name}.${_1.display()}`;
      },
      With(bundle, meta) {
        return `λ⟨${Object.entries(bundle)
          .map(([tag, body]) => `${tag}. ${body.display()}`)
          .join(',')}⟩`;
      },
      Sequence(_0, _1, _2, meta) {
        return _0.display();
      },
      TyAppV(_0, _1, meta) {
        return _0.display();
      },
      TyAppC(_0, _1, meta) {
        return _0.display();
      },
      Projection(_0, _1, meta) {
        return _0.display();
      },
      Match(_0, _1, meta) {
        return _0.display();
      },
    });
  },
  thunk() {
    return Value.Thunk(this);
  },
});

export type TypedComputation = ComputationF<'typed'>;
export const TypedComputation = handsum<
  TComputation<'typed'>,
  TComputationImplSet['typed']['impl']
>({
  display(): string {
    return this.match({
      Return(_0, meta) {
        return _0.display();
      },
      Force(_0, meta) {
        return _0.display();
      },
      Apply(_0, _1, meta) {
        return _0.display();
      },
      Lambda(name, _1, meta) {
        return `λ${name}.${_1.display()}`;
      },
      With(bundle, meta) {
        return `λ⟨${Object.entries(bundle)
          .map(([tag, body]) => `${tag}. ${body.display()}`)
          .join(',')}⟩`;
      },
      Sequence(_0, _1, _2, meta) {
        return _0.display();
      },
      TyAppV(_0, _1, meta) {
        return _0.display();
      },
      TyAppC(_0, _1, meta) {
        return _0.display();
      },
      Projection(_0, _1, meta) {
        return _0.display();
      },
      Match(_0, _1, meta) {
        return _0.display();
      },
    });
  },
  getType(): TypeC {
    return this.match({
      Return(_0, meta) {
        return meta.type;
      },
      Force(_0, meta) {
        return meta.type;
      },
      Apply(_0, _1, meta) {
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
    });
  },
});
