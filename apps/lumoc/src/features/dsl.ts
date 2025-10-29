import { Computation } from './syntax/computation';
import { Value } from './syntax/value';

export const dsl = {
  lambda(variable: string, body: (value: Value) => Computation): Computation {
    return Computation.Lambda(variable, body(Value.Variable(variable)));
  },
};
