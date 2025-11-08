import { freshName } from '../shared/name';
import { Computation } from './ast/computation';
import { Value } from './ast/value';

export const dsl = {
  lambda(body: (value: Value) => Computation): Computation {
    const variable = freshName();
    return Computation.Lambda(variable, body(Value.Variable(variable)));
  },
  bind(
    computation: Computation,
    name: string,
    body: (value: Value) => Computation,
  ): Computation {
    return Computation.Sequence(computation, name, body(Value.Variable(name)));
  },
  matchArm(f: (value: Value) => Computation): [string, Computation] {
    const variable = freshName();
    return [variable, f(Value.Variable(variable))];
  },
};
