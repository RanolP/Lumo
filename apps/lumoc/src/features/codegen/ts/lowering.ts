import { Key, TsExpr, TsType } from '../../../lib/simple-ts-ast';
import type { Computation, TypedComputation } from '../../ast/computation';
import { TypedValue, Value } from '../../ast/value';
import { TypeV, type RefinedTypeV, type TypeC } from '../../type';
import { formatParens } from '../../../shared/fmt';
import { emitType } from '../../../lib/simple-ts-ast/emit';

export class TsLoweringContext {
  #id: number = 0;
  #fixedTypes: Record<string, { typeParams: string[]; body: TsType }> = {};

  emitTsTypes() {
    return Object.entries(this.#fixedTypes)
      .map(
        ([name, { typeParams, body }]) =>
          `type ${name}${
            typeParams.length > 0 ? `<${typeParams.join(', ')}>` : ''
          } = ${emitType(body)};`,
      )
      .join('\n');
  }

  defineTsType(name: string, type: TsType) {
    this.#fixedTypes[name] = {
      typeParams: [],
      body: type,
    };
  }

  lower_t_v(
    type: RefinedTypeV,
    captures: string[],
    name_hint?: {
      text: string;
      onRaw: () => void;
      onNumbered: () => void;
    },
  ): TsType {
    const that = this;
    let fixedName: string | undefined;
    if (name_hint) {
      if (name_hint.text in that.#fixedTypes) {
        name_hint.onNumbered();
        fixedName = `${name_hint.text}_${that.#id++}`;
      } else {
        name_hint.onRaw();
        fixedName = name_hint.text;
      }
    }

    return type.handle.match({
      Recursive(name, body) {
        fixedName ??= `Rec_${that.#id++}`;
        that.#fixedTypes[fixedName] = {
          typeParams: Array.from(captures),
          body: that.lower_t_v(body, [...captures, name]).sub(
            name,
            captures.length > 0
              ? TsType.TypeApplication(
                  TsType.Variable(fixedName),
                  Array.from(captures).map((name) => TsType.Variable(name)),
                )
              : TsType.Variable(fixedName),
          ),
        };
        return TsType.TypeApplication(
          TsType.Variable(fixedName),
          Array.from(captures).map((name) => TsType.Variable(name)),
        );
      },
      Sum(entries) {
        return TsType.UntaggedUnion(
          Object.values(entries).map((entry) =>
            that.lower_t_v(entry, captures),
          ),
        );
      },
      Variant(tag, entries) {
        return TsType.Object([
          { name: Key.sym('Lumo/tag'), type: TsType.StringLiteral(tag) },
          ...Object.entries(entries).map(([name, entry]) => ({
            name: Key.str(name),
            type: that.lower_t_v(entry, captures),
          })),
        ]);
      },
      Thunk(handle) {
        return that.lower_t_c(handle, captures);
      },
      Variable(name) {
        return TsType.Variable(name);
      },
      TyAbsV(name, body) {
        const ty = TsType.Lambda(
          [],
          [name],
          that.lower_t_v(body, [...captures, name]),
        );
        if (fixedName) {
          that.defineTsType(fixedName, ty);
        }
        return ty;
      },
      _() {
        throw new TodoError(
          'lower_t_v not implemented for type: ' + type.display(),
        );
      },
    });
  }

  lower_t_c(type: TypeC, captures: string[]): TsType {
    const that = this;
    return type.match({
      /**
       * @TODO
       * suspicious
       */
      Arrow(param, body) {
        return TsType.Lambda(
          [{ name: `_${that.#id++}`, type: that.lower_t_v(param, captures) }],
          [],
          that.lower_t_c(body, captures),
        );
      },
      Produce(handle, effects) {
        /**
         * @TODO
         * effects
         */
        return that.lower_t_v(handle, captures);
      },
      With(bundle) {
        return TsType.Object(
          Object.entries(bundle).map(([name, type]) => ({
            name: Key.str(name),
            type: that.lower_t_c(type, captures),
          })),
        );
      },
      _() {
        throw new TodoError(
          'lower_t_c not implemented for type: ' + formatParens(type.display()),
        );
      },
    });
  }

  lower_v(value: TypedValue, typeCaptures: string[]): TsExpr {
    const that = this;
    return value.match({
      Thunk(body) {
        return that.lower_c(body, typeCaptures);
      },
      Variable(name) {
        return TsExpr.Variable(name);
      },
      Unroll(value, meta) {
        return that.lower_v(value, typeCaptures);
      },
      Roll(value, meta) {
        return that.lower_v(value, typeCaptures);
      },
      Injection(tag, value, meta) {
        return that.lower_v(value, typeCaptures);
      },
      Variant(tag, entries, meta) {
        return TsExpr.Object([
          { name: Key.sym('Lumo/tag'), value: TsExpr.StringLiteral(tag) },

          ...Object.entries(entries).map(([name, value]) => ({
            name: Key.str(name),
            value: that.lower_v(value, typeCaptures),
          })),
        ]);
      },
      TyAbsV(name, body) {
        return TsExpr.Lambda(
          [],
          [name],
          null,
          that.lower_v(body, [...typeCaptures, name]),
        );
      },
      _() {
        throw new TodoError(
          'lower_v not implemented for value: ' + formatParens(value.display()),
        );
      },
    });
  }

  lower_c(computation: TypedComputation, typeCaptures: string[]): TsExpr {
    const that = this;
    return computation.match({
      Annotate(target, type, meta) {
        return TsExpr.Satisfies(
          that.lower_c(target, typeCaptures),
          that.lower_t_c(type, typeCaptures),
        );
      },
      Sequence(left, name, right, meta) {
        const newName = `_${that.#id++}`;
        const paramType = left.getType().Produce?.[0];
        if (!paramType) {
          throw new Error('never fails');
        }
        return TsExpr.Apply(
          TsExpr.Lambda(
            [{ name: newName, type: that.lower_t_c(meta.type, typeCaptures) }],
            [],
            null,
            that.lower_c(
              right.sub_v(
                name,
                TypedValue.Variable(newName, { type: paramType }),
              ),
              typeCaptures,
            ),
          ),
          [that.lower_c(left, typeCaptures)],
        );
      },
      Resolve(bundle, tag) {
        return TsExpr.FieldAccess(
          that.lower_c(bundle, typeCaptures),
          Key.str(tag),
        );
      },
      Apply(fn, param) {
        return TsExpr.Apply(that.lower_c(fn, typeCaptures), [
          that.lower_v(param, typeCaptures),
        ]);
      },
      With(bundle) {
        return TsExpr.Object(
          Object.entries(bundle).map(([key, value]) => ({
            name: Key.str(key),
            value: that.lower_c(value, typeCaptures),
          })),
        );
      },
      Lambda(param, body, meta) {
        const newName = `_${that.#id++}`;
        if (!meta.type.Arrow) {
          throw new Error('never fails');
        }
        const [paramType] = meta.type.Arrow;
        return TsExpr.Satisfies(
          TsExpr.Lambda(
            [{ name: newName }],
            [],
            that.lower_t_c(body.getType(), typeCaptures),
            that.lower_c(
              /**
               * @TODO
               * suspicious
               */
              body.sub_v(
                param,
                TypedValue.Variable(newName, { type: paramType }),
              ),
              typeCaptures,
            ),
          ),
          that.lower_t_c(meta.type, typeCaptures),
        );
      },
      Force(value) {
        return that.lower_v(value, typeCaptures);
      },
      Produce(value) {
        return that.lower_v(value, typeCaptures);
      },
      Projection(value, key) {
        return TsExpr.FieldAccess(
          that.lower_v(value, typeCaptures),
          Key.str(key),
        );
      },
      Match(value, branches) {
        const cbvName = `_${that.#id++}`;
        const ty = value.getType().handle.Sum?.[0];
        if (!ty) {
          throw new Error('never fails');
        }
        return TsExpr.Apply(
          TsExpr.Lambda(
            [{ name: cbvName }],
            [],
            null,
            Object.entries(branches).reduce((result, [key, [name, body]]) => {
              const newName = `_${that.#id++}`;
              return TsExpr.Ternary(
                TsExpr.Equals(
                  TsExpr.FieldAccess(
                    TsExpr.Variable(cbvName),
                    Key.sym('Lumo/tag'),
                  ),
                  TsExpr.StringLiteral(key),
                ),
                TsExpr.Apply(
                  TsExpr.Lambda(
                    [
                      {
                        name: newName,
                        type: that.lower_t_v(ty[key]!, typeCaptures),
                      },
                    ],
                    [],
                    null,
                    that.lower_c(
                      body.sub_v(
                        name,
                        TypedValue.Variable(newName, {
                          type: TypeV.Variable(newName).freshRefined(),
                        }),
                      ),
                      typeCaptures,
                    ),
                  ),
                  [TsExpr.Variable(cbvName)],
                ),
                result,
              );
            }, TsExpr.Never()),
          ),
          [that.lower_v(value, typeCaptures)],
        );
      },
      TyAppV(body, ty, meta) {
        return TsExpr.Apply(
          TsExpr.TypeApplication(that.lower_v(body, typeCaptures), [
            that.lower_t_v(ty, typeCaptures),
          ]),
          [],
        );
      },
      _() {
        throw new TodoError(
          'lower_c not implemented for computation: ' +
            formatParens(computation.display()),
        );
      },
    });
  }
}

export class TodoError extends Error {
  name = 'TodoError';
  constructor(public message: string) {
    super(message);
  }
}
