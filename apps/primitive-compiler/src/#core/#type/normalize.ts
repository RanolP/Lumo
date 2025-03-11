import { Type, TypeVar } from './variants.js';
import { TypeScope } from './scope.js';

export function normalizeType(scope: TypeScope, ty: Type): Type {
  if (ty instanceof TypeVar) {
    const got = scope.lookup(ty.path);
    if (got.id(scope) === ty.id(scope)) return got;
    return normalizeType(scope, got);
  } else {
    return ty;
  }
}
