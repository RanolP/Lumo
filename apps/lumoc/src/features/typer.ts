import { formatParens } from '../shared/fmt';
import { freshName } from '../shared/name';
import { Computation, TypedComputation } from './ast/computation';
import { TypedValue, Value } from './ast/value';
import { RefinedTypeV, TypeC, TypeV } from './type';
import { apply_c, apply_v, ProofObligations, unify_c, unify_v } from './unify';

export class Typer {
  readonly #parent: Typer | undefined;
  readonly #sub_v: Record<string, RefinedTypeV> = {};

  /**
   * @TODO
   *
   * Use Algorithm W instead of Algorithm J
   */
  #obligations: ProofObligations = { v: {}, c: {} };

  static create(): Typer {
    return new Typer();
  }

  private constructor(parent?: Typer) {
    this.#parent = parent;
  }

  makeSubscope(): Typer {
    return new Typer(this);
  }

  with_v(name: string, type: RefinedTypeV): this {
    if (name in this.#sub_v) {
      throw new NameConflictError(name);
    }
    this.#sub_v[name] = type;
    return this;
  }

  resolve_v(name: string): RefinedTypeV {
    if (name in this.#sub_v) {
      return this.#sub_v[name]!;
    }
    if (this.#parent) {
      return this.#parent.resolve_v(name);
    }
    throw new UnknownVariableError(name);
  }

  unify_v(a: RefinedTypeV, b: RefinedTypeV): this {
    if (this.#parent) {
      this.#parent.unify_v(a, b);
    } else {
      this.#obligations = unify_v(this.#obligations, a, b);
    }
    return this;
  }

  unify_c(a: TypeC, b: TypeC): this {
    if (this.#parent) {
      this.#parent.unify_c(a, b);
    } else {
      this.#obligations = unify_c(this.#obligations, a, b);
    }
    return this;
  }

  apply_v(ty: RefinedTypeV): RefinedTypeV {
    if (this.#parent) {
      return this.#parent.apply_v(ty);
    } else {
      return apply_v(this.#obligations, ty);
    }
  }

  apply_c(ty: TypeC): TypeC {
    if (this.#parent) {
      return this.#parent.apply_c(ty);
    } else {
      return apply_c(this.#obligations, ty);
    }
  }

  infer_v(value: Value): TypedValue {
    const that = this;
    return value.match({
      Annotate(target, type) {
        return that.check_v(target, type);
      },
      Variable(name) {
        const ty = that.resolve_v(name);
        return TypedValue.Variable(name, { type: ty });
      },
      Unroll(value) {
        const typedValue = that.infer_v(value);
        const ty = typedValue.getType();
        if (!ty.handle.Recursive) {
          throw new UnrollOnWrongTypeError(ty);
        }
        return TypedValue.Unroll(typedValue, {
          type: ty.unroll(),
        });
      },
      _() {
        throw new ValueInferenceFailureError(value);
      },
    });
  }

  check_v(value: Value, type: RefinedTypeV): TypedValue {
    if (value.Record && type.handle.Record) {
      const [valueEntries] = value.Record;
      const [typeEntries] = type.handle.Record;

      return TypedValue.Record(
        Object.fromEntries(
          Array.from(
            new Set([
              ...Object.keys(valueEntries),
              ...Object.keys(typeEntries),
            ]),
          ).map((key) => {
            const valueEntry = valueEntries[key];
            const typeEntry = typeEntries[key];
            if (!valueEntry) {
              throw new VariantMissingKeyError(key);
            }
            if (!typeEntry) {
              throw new VariantExtraKeyError(key);
            }
            return [key, this.check_v(valueEntry, typeEntry)];
          }),
        ),
        { type },
      );
    }
    if (value.Variant && type.handle.Variant) {
      const [valueTag, valueEntries] = value.Variant;
      const [tyTag, typeEntries] = type.handle.Variant;
      if (valueTag !== tyTag) {
        throw new VariantTagMismatchError(valueTag, tyTag);
      }
      return TypedValue.Variant(
        valueTag,
        Object.fromEntries(
          Array.from(
            new Set([
              ...Object.keys(valueEntries),
              ...Object.keys(typeEntries),
            ]),
          ).map((key) => {
            const valueEntry = valueEntries[key];
            const typeEntry = typeEntries[key];
            if (!valueEntry) {
              throw new VariantMissingKeyError(key);
            }
            if (!typeEntry) {
              throw new VariantExtraKeyError(key);
            }
            return [key, this.check_v(valueEntry, typeEntry)];
          }),
        ),
        { type },
      );
    }

    if (value.Roll && type.handle.Recursive) {
      const [valueRoll] = value.Roll;
      return TypedValue.Roll(this.check_v(valueRoll, type.unroll()), { type });
    }

    if (value.Injection && type.handle.Sum) {
      const [valueTag, valueExpr] = value.Injection;
      const [tyEntries] = type.handle.Sum;
      const expected = tyEntries[valueTag];
      if (!expected) {
        throw new SumMissingTagError(valueTag);
      }
      return TypedValue.Injection(valueTag, this.check_v(valueExpr, expected), {
        type,
      });
    }

    if (value.Thunk && type.handle.Thunk) {
      return TypedValue.Thunk(
        this.check_c(value.Thunk[0], type.handle.Thunk[0]),
        { type },
      );
    }

    if (value.TyAbsV && type.handle.TyAbsV) {
      const [name, body] = value.TyAbsV;
      const [nameTy, bodyTy] = type.handle.TyAbsV;

      const typedBody = this.makeSubscope().check_v(
        body,
        bodyTy.sub(nameTy, TypeV.Variable(name).freshRefined()),
      );
      return TypedValue.TyAbsV(name, typedBody, { type });
    }

    const inferred = this.infer_v(value);
    this.unify_v(inferred.getType(), type);
    return inferred;
  }

  infer_c(computation: Computation): TypedComputation {
    const that = this;
    return computation.match({
      Annotate(target, type) {
        return that.check_c(target, type);
      },
      Sequence(left, name, right) {
        const typedLeft = that.infer_c(left);
        const tyLeft = typedLeft.getType();
        if (!tyLeft.Produce) {
          throw new SequenceOnWrongTypeError(tyLeft);
        }
        const typedRight = that
          .makeSubscope()
          .with_v(name, tyLeft.Produce[0])
          .infer_c(right);
        return TypedComputation.Sequence(typedLeft, name, typedRight, {
          type: typedRight.getType(),
        });
      },
      Projection(value, key) {
        const typedValue = that.infer_v(value);
        return typedValue.getType().handle.match({
          Record(entries) {
            if (!(key in entries)) {
              throw new RecordMissingKeyError(key);
            }
            return TypedComputation.Projection(typedValue, key, {
              type: entries[key]!.comput(),
            });
          },
          Variant(_0, entries) {
            if (!(key in entries)) {
              throw new VariantMissingKeyError(key);
            }
            return TypedComputation.Projection(typedValue, key, {
              type: entries[key]!.comput(),
            });
          },
          _() {
            throw new ComputationInferenceFailureError(computation);
          },
        });
      },
      Match(value, branches) {
        const typedValue = that.infer_v(value);
        const ty = typedValue.getType();
        if (!ty.handle.Sum) {
          throw new MatchOnWrongTypeError(ty);
        }
        const [entries] = ty.handle.Sum;
        const resultingType = TypeV.Variable(freshName('ty')).freshRefined();
        const typedArms: Record<string, [string, TypedComputation]> = {};
        for (const [key, [name, body]] of Object.entries(branches)) {
          if (!entries[key]) {
            throw new UnknownMatchArmError(key);
          }
          const comput = that
            .makeSubscope()
            .with_v(name, entries[key])
            .infer_c(body);
          that.unify_c(resultingType.comput(), comput.getType());
          typedArms[key] = [name, comput];
        }
        return TypedComputation.Match(typedValue, typedArms, {
          type: that.apply_v(resultingType).comput(),
        });
      },
      Apply(fn, param) {
        const typedFn = that.infer_c(fn);
        const ty = that.apply_c(typedFn.getType());
        if (!ty.Arrow) {
          throw new ApplyOnWrongTypeError(ty);
        }
        const typedParam = that.check_v(param, ty.Arrow[0]);
        return TypedComputation.Apply(typedFn, typedParam, {
          type: ty.Arrow[1],
        });
      },
      Resolve(bundle, tag) {
        const typedBundle = that.infer_c(bundle);
        const ty = typedBundle.getType();
        if (!ty.With) {
          throw new ResolveOnWrongTypeError(ty);
        }
        const [bundleEntries] = ty.With;
        if (!(tag in bundleEntries)) {
          throw new ResolveMissingTagError(tag);
        }
        return TypedComputation.Resolve(typedBundle, tag, {
          type: bundleEntries[tag]!,
        });
      },
      Force(value) {
        const typedValue = that.infer_v(value);
        const ty = that.apply_v(typedValue.getType());
        if (!ty.handle.Thunk) {
          throw new ForceOnWrongTypeError(ty);
        }
        return TypedComputation.Force(typedValue, {
          type: ty.handle.Thunk[0],
        });
      },
      TyAppV(body, ty) {
        const typedBody = that.infer_v(body);
        const bodyTy = that.apply_v(typedBody.getType());
        if (!bodyTy.handle.TyAbsV) throw new TyAppVOnWrongTypeError(body);
        const [name, inner] = bodyTy.handle.TyAbsV;
        return TypedComputation.TyAppV(typedBody, ty, {
          type: that.apply_v(inner).sub(name, ty).comput(),
        });
      },
      _() {
        throw new ComputationInferenceFailureError(computation);
      },
    });
  }

  check_c(computation: Computation, type: TypeC): TypedComputation {
    if (computation.Lambda && type.Arrow) {
      const [name, body] = computation.Lambda;
      const [paramTy, bodyTy] = type.Arrow;
      return TypedComputation.Lambda(
        name,
        this.makeSubscope().with_v(name, paramTy).check_c(body, bodyTy),
        { type },
      );
    }
    if (computation.Produce && type.Produce) {
      const [value] = computation.Produce;
      const [handle] = type.Produce;
      return TypedComputation.Produce(this.check_v(value, handle), { type });
    }

    if (computation.With && type.With) {
      const [bundle] = computation.With;
      const [typeBundle] = type.With;
      let result: Record<string, TypedComputation> = {};
      for (const key of new Set([
        ...Object.keys(bundle),
        ...Object.keys(typeBundle),
      ])) {
        const bundleValue = bundle[key];
        const typeValue = typeBundle[key];
        if (!bundleValue) {
          throw new WithExtraKeyError(key);
        }
        if (!typeValue) {
          throw new WithMissingKeyError(key);
        }
        result[key] = this.check_c(bundleValue, typeValue);
      }
      return TypedComputation.With(result, { type });
    }

    const inferred = this.infer_c(computation);
    this.unify_c(inferred.getType(), type);
    return inferred;
  }
}

export class ValueInferenceFailureError extends Error {
  constructor(value: Value) {
    super(`Failed to infer type: ${formatParens(value.display())}`);
    this.name = 'ValueInferenceFailureError';
  }
}

export class ComputationInferenceFailureError extends Error {
  constructor(computation: Computation) {
    super(`Failed to infer type: ${formatParens(computation.display())}`);
    this.name = 'ComputationInferenceFailureError';
  }
}

export class ValueTypeMismatchError extends Error {
  constructor(inferred: TypedValue, expected: RefinedTypeV) {
    super(
      `Type mismatch:\n${formatParens(inferred.display())}\n\n${formatParens(
        expected.display(),
      )}`,
    );
    this.name = 'TypeMismatchError';
  }
}

export class ComputationTypeMismatchError extends Error {
  constructor(inferred: TypedComputation, expected: TypeC) {
    super(
      `Type mismatch:\n${formatParens(inferred.display())}\n\n${formatParens(
        expected.display(),
      )}`,
    );
    this.name = 'ComputationTypeMismatchError';
  }
}

export class RecordExtraKeyError extends Error {
  constructor(key: string) {
    super(`Record extra key: ${key}`);
    this.name = 'RecordExtraKeyError';
  }
}

export class RecordMissingKeyError extends Error {
  constructor(key: string) {
    super(`Record missing key: ${key}`);
    this.name = 'RecordMissingKeyError';
  }
}

export class WithExtraKeyError extends Error {
  constructor(key: string) {
    super(`With extra key: ${key}`);
    this.name = 'WithExtraKeyError';
  }
}

export class WithMissingKeyError extends Error {
  constructor(key: string) {
    super(`With missing key: ${key}`);
    this.name = 'WithMissingKeyError';
  }
}

export class VariantTagMismatchError extends Error {
  constructor(valueTag: string, typeTag: string) {
    super(`Variant tag mismatch: ${valueTag} !== ${typeTag}`);
    this.name = 'VariantTagMismatchError';
  }
}

export class VariantMissingKeyError extends Error {
  constructor(key: string) {
    super(`Variant missing key: ${key}`);
    this.name = 'VariantMissingKeyError';
  }
}

export class VariantExtraKeyError extends Error {
  constructor(key: string) {
    super(`Variant extra key: ${key}`);
    this.name = 'VariantExtraKeyError';
  }
}

export class NameConflictError extends Error {
  constructor(name: string) {
    super(`Name conflict: ${name}`);
    this.name = 'NameConflictError';
  }
}

export class SumMissingTagError extends Error {
  constructor(tag: string) {
    super(`Sum missing tag: ${tag}`);
    this.name = 'SumMissingTagError';
  }
}

export class UnknownVariableError extends Error {
  constructor(name: string) {
    super(`Unknown variable: ${name}`);
    this.name = 'UnknownVariableError';
  }
}

export class UnrollOnWrongTypeError extends Error {
  constructor(type: RefinedTypeV) {
    super(`Unroll on wrong type: ${type.display()}`);
    this.name = 'UnrollOnWrongTypeError';
  }
}

export class MatchOnWrongTypeError extends Error {
  constructor(type: RefinedTypeV) {
    super(`Match on wrong type: ${type.display()}`);
    this.name = 'MatchOnWrongTypeError';
  }
}

export class SequenceOnWrongTypeError extends Error {
  constructor(type: TypeC) {
    super(`Sequence on wrong type: ${type.display()}`);
    this.name = 'SequenceOnWrongTypeError';
  }
}
export class UnknownMatchArmError extends Error {
  constructor(arm: string) {
    super(`Unknown match arm: ${arm}`);
    this.name = 'UnknownMatchArmError';
  }
}

export class ApplyOnWrongTypeError extends Error {
  constructor(type: TypeC) {
    super(`Apply on wrong type: ${type.display()}`);
    this.name = 'ApplyOnWrongTypeError';
  }
}

export class ForceOnWrongTypeError extends Error {
  constructor(type: RefinedTypeV) {
    super(`Force on wrong type: ${type.display()}`);
    this.name = 'ForceOnWrongTypeError';
  }
}

export class TyAppVOnWrongTypeError extends Error {
  constructor(body: Value) {
    super(`TyAppV on wrong type: ${body.display()}`);
    this.name = 'TyAppVOnWrongTypeError';
  }
}

export class ResolveOnWrongTypeError extends Error {
  constructor(type: TypeC) {
    super(`Resolve on wrong type: ${type.display()}`);
    this.name = 'ResolveOnWrongTypeError';
  }
}

export class ResolveMissingTagError extends Error {
  constructor(tag: string) {
    super(`Resolve missing tag: ${tag}`);
    this.name = 'ResolveMissingTagError';
  }
}
