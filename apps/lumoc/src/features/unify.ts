import { formatParens } from '../shared/fmt';
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
    Record(entries) {
      if (!b.handle.Record) throw new ValueUnificationFailureError(a, b);
      const [bEntries] = b.handle.Record;
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
    Thunk(handle) {
      if (!b.handle.Thunk) throw new ValueUnificationFailureError(a, b);
      const [bHandle] = b.handle.Thunk;
      return unify_c(subs, handle, bHandle, boundVariables);
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
    Arrow(_0, body) {
      if (!b.Arrow) throw new ComputationUnificationFailureError(a, b);
      const [_1, bBody] = b.Arrow;
      return unify_c(subs, body, bBody, boundVariables);
    },
    Produce(aHandle) {
      if (!b.Produce) throw new ComputationUnificationFailureError(a, b);
      const [bHandle] = b.Produce;
      return unify_v(subs, aHandle, bHandle, boundVariables);
    },
    With(bundle) {
      if (!b.With) throw new ComputationUnificationFailureError(a, b);
      const [bBundle] = b.With;
      for (const key of new Set([
        ...Object.keys(bundle),
        ...Object.keys(bBundle),
      ])) {
        const aValue = bundle[key];
        const bValue = bBundle[key];
        if (!aValue || !bValue) {
          throw new ComputationUnificationFailureError(a, b);
        }
        subs = unify_c(subs, aValue, bValue, boundVariables);
      }
      return subs;
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
    super(
      `Unification failure: ${formatParens(a.display())}\nand\n${formatParens(
        b.display(),
      )}`,
    );
    this.name = 'ValueUnificationFailureError';
  }
}

export class ComputationUnificationFailureError extends Error {
  constructor(a: TypeC, b: TypeC) {
    super(
      `Unification failure: ${formatParens(a.display())}\nand\n${formatParens(
        b.display(),
      )}`,
    );
    this.name = 'ComputationUnificationFailureError';
  }
}
