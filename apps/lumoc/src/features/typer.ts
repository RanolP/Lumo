import { formatParens } from '../shared/fmt';
import { Computation, TypedComputation } from './syntax/computation';
import { TypedValue, Value } from './syntax/value';
import { RefinedTypeV, TypeC } from './type';

export class Typer {
  readonly #parent: Typer | undefined;
  readonly #scope: Record<string, RefinedTypeV> = {};

  static create(): Typer {
    return new Typer();
  }

  private constructor(parent?: Typer) {
    this.#parent = parent;
  }

  makeSubscope(): Typer {
    return new Typer(this);
  }

  with(name: string, type: RefinedTypeV): this {
    if (name in this.#scope) {
      throw new NameConflictError(name);
    }
    this.#scope[name] = type;
    return this;
  }

  resolve(ty: RefinedTypeV): RefinedTypeV {
    if (ty.handle.Variable) {
      const [name] = ty.handle.Variable;
      if (this.#scope[name]) {
        return ty.sub(name, this.resolve(this.#scope[name]));
      }
      if (this.#parent) {
        return this.#parent.resolve(ty);
      }
      throw new UndefinedTypeError(name);
    }
    return ty;
  }

  infer_v(value: Value): TypedValue {
    const that = this;
    return value.match({
      Annotate(target, type) {
        return that.check_v(target, type);
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

    const inferred = this.infer_v(value);
    const resolved = this.resolve(type);
    if (!inferred.getType().equals(resolved)) {
      throw new ValueTypeMismatchError(inferred, resolved);
    }
    return inferred;
  }

  infer_c(computation: Computation): TypedComputation {
    return computation.match({
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
        this.makeSubscope().with(name, paramTy).check_c(body, bodyTy),
        { type },
      );
    }
    if (computation.Return && type.Produce) {
      const [value] = computation.Return;
      const [handle] = type.Produce;
      return TypedComputation.Return(this.check_v(value, handle), { type });
    }

    const inferred = this.infer_c(computation);
    if (!inferred.getType().equals(type)) {
      throw new ComputationTypeMismatchError(inferred, type);
    }
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

export class UndefinedTypeError extends Error {
  constructor(name: string) {
    super(`Undefined type: ${name}`);
    this.name = 'UndefinedTypeError';
  }
}
