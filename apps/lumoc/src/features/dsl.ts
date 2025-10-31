import { Computation } from './ast/computation';
import { Value } from './ast/value';

export const dsl = {
  lambda(variable: string, body: (value: Value) => Computation): Computation {
    return Computation.Lambda(variable, body(Value.Variable(variable)));
  },
};
