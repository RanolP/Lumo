import { formatParens } from '../shared/fmt';
import { RefinedTypeV, TypeC, TypeV } from './type';

export interface ProofObligations {
  v: Record<string, RefinedTypeV>;
  c: Record<string, TypeC>;
}

function isBottomType(type: TypeV): boolean {
  return !!type.Sum && Object.keys(type.Sum[0]).length === 0;
}

export function unify_v(
  obligations: ProofObligations,
  _a: RefinedTypeV,
  _b: RefinedTypeV,
  boundVariables: Record<string, string> = {},
): ProofObligations {
  const a = apply_v(obligations, _a);
  const b = apply_v(obligations, _b);
  if (isBottomType(a.handle) || isBottomType(b.handle)) {
    return obligations;
  }
  if (
    a.handle.Variable &&
    a.handle.Variable[0] in boundVariables &&
    boundVariables[a.handle.Variable[0]] === b.handle.Variable?.[0]
  ) {
    return obligations;
  }
  if (
    b.handle.Variable &&
    b.handle.Variable[0] in boundVariables &&
    boundVariables[b.handle.Variable[0]] === a.handle.Variable?.[0]
  ) {
    return obligations;
  }
  if (a.handle.Variable) {
    return {
      ...obligations,
      v: { ...obligations.v, [a.handle.Variable[0]]: b },
    };
  }
  if (b.handle.Variable) {
    return {
      ...obligations,
      v: { ...obligations.v, [b.handle.Variable[0]]: a },
    };
  }
  return a.handle.match({
    Recursive(aName, aBody) {
      if (!b.handle.Recursive) throw new ValueUnificationFailureError(a, b);
      const [bName, bBody] = b.handle.Recursive;

      return unify_v(obligations, aBody, bBody, {
        ...boundVariables,
        [aName]: bName,
        [bName]: aName,
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
        obligations = unify_v(obligations, aEntry, bEntry, boundVariables);
      }
      return obligations;
    },
    Thunk(aHandle) {
      if (!b.handle.Thunk) throw new ValueUnificationFailureError(a, b);
      const [bHandle] = b.handle.Thunk;
      return unify_c(obligations, aHandle, bHandle, boundVariables);
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
        obligations = unify_v(obligations, aEntry, bEntry, boundVariables);
      }
      return obligations;
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
        obligations = unify_v(obligations, aEntry, bEntry, boundVariables);
      }
      return obligations;
    },
    _() {
      throw new ValueUnificationFailureError(a, b);
    },
  });
}

export function unify_c(
  obligations: ProofObligations,
  _a: TypeC,
  _b: TypeC,
  boundVariables: Record<string, string> = {},
): ProofObligations {
  const a = apply_c(obligations, _a);
  const b = apply_c(obligations, _b);

  if (
    a.Variable &&
    a.Variable[0] in boundVariables &&
    boundVariables[a.Variable[0]] === b.Variable?.[0]
  ) {
    return obligations;
  }
  if (
    b.Variable &&
    b.Variable[0] in boundVariables &&
    boundVariables[b.Variable[0]] === a.Variable?.[0]
  ) {
    return obligations;
  }
  if (a.Variable) {
    return { ...obligations, c: { ...obligations.c, [a.Variable[0]]: b } };
  }
  if (b.Variable) {
    return { ...obligations, c: { ...obligations.c, [b.Variable[0]]: a } };
  }

  return a.match({
    Arrow(_0, body) {
      if (!b.Arrow) throw new ComputationUnificationFailureError(a, b);
      const [_1, bBody] = b.Arrow;
      return unify_c(obligations, body, bBody, boundVariables);
    },
    Produce(aHandle) {
      if (!b.Produce) throw new ComputationUnificationFailureError(a, b);
      const [bHandle] = b.Produce;
      return unify_v(obligations, aHandle, bHandle, boundVariables);
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
        obligations = unify_c(obligations, aValue, bValue, boundVariables);
      }
      return obligations;
    },
  });
}

export function apply_v(
  obligations: ProofObligations,
  ty: RefinedTypeV,
): RefinedTypeV {
  const resolved = ty.handle.Variable && obligations.v[ty.handle.Variable[0]];
  return resolved ?? ty;
}

export function apply_c(obligations: ProofObligations, ty: TypeC): TypeC {
  const resolved = ty.Variable && obligations.c[ty.Variable[0]];
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
