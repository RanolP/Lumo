import { Path } from '@/#core/#ast/construct.js';
import { normalizeType } from '@/#core/#type/normalize.js';
import { TypeScope } from '@/#core/#type/scope.js';
import { Constructor, Sum, Type, TypeVar } from '@/#core/#type/variants.js';
import { match, P } from 'ts-pattern';

export function unify(scope: TypeScope, a: Type, b: Type): Type {
  const l = normalizeType(scope, a);
  const r = normalizeType(scope, b);
  return match([l, r])
    .with([P.instanceOf(Constructor), P.instanceOf(Constructor)], ([l, r]) => {
      if (l.folded === r.folded)
        return new TypeVar(l.origin, new Path([l.folded]));

      throw new Error(`Cannot unify ${l} with ${r}`);
    })
    .with([Sum.never, P.any], ([_, r]) => r)
    .with([P.any, Sum.never], ([l, _]) => l)
    .otherwise(() => {
      if (l.equals(r)) return l;
      throw new Error(`Cannot unify ${l} with ${r}`);
    });
}
