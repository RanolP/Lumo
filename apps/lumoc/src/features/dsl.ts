import { freshName } from '../shared/name';
import { Computation } from './ast/computation';
import { Value } from './ast/value';
import { RefinedTypeV, TypeV } from './type';

export const dsl = {
  v: {
    forall(body: (value: RefinedTypeV) => Value): Value {
      const variable = freshName('ty');
      return Value.TyAbsV(
        variable,
        body(TypeV.Variable(variable).freshRefined()),
      );
    },
  },
  c: {
    lambda(body: (value: Value) => Computation): Computation {
      const variable = freshName('val');
      return Computation.Lambda(variable, body(Value.Variable(variable)));
    },
    bind(
      computation: Computation,
      body: (value: Value) => Computation,
    ): Computation {
      const name = freshName('val');
      return Computation.Sequence(
        computation,
        name,
        body(Value.Variable(name)),
      );
    },
  },
  t: {
    var(): RefinedTypeV {
      const variable = freshName('ty');
      return TypeV.Variable(variable).freshRefined();
    },
    recurse(body: (value: RefinedTypeV) => RefinedTypeV): RefinedTypeV {
      const variable = freshName('ty');
      return TypeV.Recursive(
        variable,
        body(TypeV.Variable(variable).freshRefined()),
      ).freshRefined();
    },
    forall(body: (value: RefinedTypeV) => RefinedTypeV): RefinedTypeV {
      const variable = freshName('ty');
      return TypeV.TyAbsV(
        variable,
        body(TypeV.Variable(variable).freshRefined()),
      ).freshRefined();
    },
  },
  f: {
    matchArm(f: (value: Value) => Computation): [string, Computation] {
      const variable = freshName('val');
      return [variable, f(Value.Variable(variable))];
    },
  },
};
