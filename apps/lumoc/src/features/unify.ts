import { RefinedTypeV, TypeC, TypeV } from './type';

function isBottomType(type: TypeV): boolean {
  return !!type.Sum && Object.keys(type.Sum[0]).length === 0;
}

export function unify_v(
  subs: Record<string, RefinedTypeV>,
  _a: RefinedTypeV,
  _b: RefinedTypeV,
  boundVariables: Record<string, string> = {},
): Record<string, RefinedTypeV> {
  const a = apply(subs, _a);
  const b = apply(subs, _b);
  if (isBottomType(a.handle) || isBottomType(b.handle)) {
    return subs;
  }
  if (
    a.handle.Variable &&
    a.handle.Variable[0] in boundVariables &&
    boundVariables[a.handle.Variable[0]] === b.handle.Variable?.[0]
  ) {
    return subs;
  }
  if (
    b.handle.Variable &&
    b.handle.Variable[0] in boundVariables &&
    boundVariables[b.handle.Variable[0]] === a.handle.Variable?.[0]
  ) {
    return subs;
  }
  if (a.handle.Variable) {
    return { ...subs, [a.handle.Variable[0]]: b };
  }
  if (b.handle.Variable) {
    return { ...subs, [b.handle.Variable[0]]: a };
  }
  return a.handle.match({
    Recursive(aname, aBody) {
      if (!b.handle.Recursive) throw new ValueUnificationFailureError(a, b);
      const [bName, bBody] = b.handle.Recursive;
      return unify_v(subs, aBody, bBody, {
        ...boundVariables,
        [aname]: bName,
        [bName]: aname,
      });
    },
    Sum(aEntries) {
      if (!b.handle.Sum) throw new ValueUnificationFailureError(a, b);
      const [bEntries] = b.handle.Sum;
      for (const key of new Set([
        ...Object.keys(aEntries),
        ...Object.keys(bEntries),
      ])) {
        const aEntry = aEntries[key];
        const bEntry = bEntries[key];
        if (!aEntry || !bEntry) {
          throw new ValueUnificationFailureError(a, b);
        }
        subs = unify_v(subs, aEntry, bEntry, boundVariables);
      }
      return subs;
    },
    Variant(tag, entries) {
      if (!b.handle.Variant) throw new ValueUnificationFailureError(a, b);
      const [bTag, bEntries] = b.handle.Variant;
      if (tag !== bTag) throw new ValueUnificationFailureError(a, b);
      for (const key of new Set([
        ...Object.keys(entries),
        ...Object.keys(bEntries),
      ])) {
        const aEntry = entries[key];
        const bEntry = bEntries[key];
        if (!aEntry || !bEntry) {
          throw new ValueUnificationFailureError(a, b);
        }
        subs = unify_v(subs, aEntry, bEntry, boundVariables);
      }
      return subs;
    },
    _() {
      throw new ValueUnificationFailureError(a, b);
    },
  });
}

export function unify_c(
  subs: Record<string, RefinedTypeV>,
  a: TypeC,
  b: TypeC,
  boundVariables: Record<string, string> = {},
): Record<string, RefinedTypeV> {
  return a.match({
    Produce(aHandle) {
      if (!b.Produce) throw new ComputationUnificationFailureError(a, b);
      const [bHandle] = b.Produce;
      return unify_v(subs, aHandle, bHandle, boundVariables);
    },
    _() {
      throw new ComputationUnificationFailureError(a, b);
    },
  });
}

export function apply(
  subs: Record<string, RefinedTypeV>,
  ty: RefinedTypeV,
): RefinedTypeV {
  const resolved = ty.handle.Variable && subs[ty.handle.Variable[0]];
  return resolved ?? ty;
}

export class ValueUnificationFailureError extends Error {
  constructor(a: RefinedTypeV, b: RefinedTypeV) {
    super(`Unification failure: ${a.display()} and ${b.display()}`);
    this.name = 'ValueUnificationFailureError';
  }
}

export class ComputationUnificationFailureError extends Error {
  constructor(a: TypeC, b: TypeC) {
    super(`Unification failure: ${a.display()} and ${b.display()}`);
    this.name = 'ComputationUnificationFailureError';
  }
}
