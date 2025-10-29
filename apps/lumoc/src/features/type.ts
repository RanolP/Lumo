import { Handsum, handsum } from 'handsum';

export interface TTypeV {
  Sum(entries: Record<string, RefinedTypeV>): TypeV;
  Record(entries: Record<string, RefinedTypeV>): TypeV;
  Variant(tag: string, entries: Record<string, RefinedTypeV>): TypeV;
  Thunk(handle: TypeC): TypeV;
  Recursive(name: string, body: RefinedTypeV): TypeV;
  Variable(name: string): TypeV;
}
export interface ITypeV {
  freshRefined(this: TypeV): RefinedTypeV;

  display(this: TypeV): string;

  equals(this: TypeV, other: TypeV | null | undefined): boolean;
}
export type TypeV = Handsum<TTypeV, ITypeV>;
export const TypeV = handsum<TTypeV, ITypeV>({
  freshRefined(): RefinedTypeV {
    return new RefinedTypeV(this);
  },
  display() {
    return this.match({
      Sum(entries) {
        return `∑(${Object.entries(entries)
          .map(([key, value]) => `${key}: ${value.display()}`)
          .join(',')})`;
      },
      Record(entries) {
        return `record {${Object.entries(entries)
          .map(([key, value]) => `${key}: ${value.display()}`)
          .join(',')}}`;
      },
      Variant(tag, entries) {
        return `variant[${tag}] {${Object.entries(entries)
          .map(([key, value]) => `${key}: ${value.display()}`)
          .join(',')}}`;
      },
      Thunk(body) {
        return `thunk(${body.display()})`;
      },
      Recursive(name, body) {
        return `μ${name}. (${body.display()})`;
      },
      Variable(name) {
        return `var_${JSON.stringify(name)}`;
      },
    });
  },
  equals(other: TypeV | null | undefined): boolean {
    if (!other) {
      return false;
    }
    if (this.Sum && other.Sum) {
      const [thisEntries] = this.Sum;
      const [otherEntries] = other.Sum;
      const keys = new Set([
        ...Object.keys(thisEntries),
        ...Object.keys(otherEntries),
      ]);
      for (const key of keys) {
        const thisValue = thisEntries[key];
        const otherValue = otherEntries[key];
        if (!thisValue || !otherValue) {
          return false;
        }
        if (!thisValue.equals(otherValue)) {
          return false;
        }
      }
      return true;
    }
    if (this.Record && other.Record) {
      const [thisEntries] = this.Record;
      const [otherEntries] = other.Record;
      const keys = new Set([
        ...Object.keys(thisEntries),
        ...Object.keys(otherEntries),
      ]);
      for (const key of keys) {
        const thisFieldTy = thisEntries[key];
        const otherFieldTy = otherEntries[key];
        if (!thisFieldTy || !otherFieldTy) {
          return false;
        }
        if (!thisFieldTy.equals(otherFieldTy)) {
          return false;
        }
      }
      return true;
    }
    if (this.Variant && other.Variant) {
      const [thisTag, thisEntries] = this.Variant;
      const [otherTag, otherEntries] = other.Variant;
      if (thisTag !== otherTag) {
        return false;
      }
      const keys = new Set([
        ...Object.keys(thisEntries),
        ...Object.keys(otherEntries),
      ]);
      for (const key of keys) {
        const thisValue = thisEntries[key];
        const otherValue = otherEntries[key];
        if (!thisValue || !otherValue) {
          return false;
        }
        if (!thisValue.equals(otherValue)) {
          return false;
        }
      }
      return true;
    }
    if (this.Thunk && other.Thunk) {
      return this.Thunk[0].equals(other.Thunk[0]);
    }
    if (this.Recursive && other.Recursive) {
      return (
        this.Recursive[0] === other.Recursive[0] &&
        this.Recursive[1].equals(other.Recursive[1])
      );
    }
    if (this.Variable && other.Variable) {
      return this.Variable[0] === other.Variable[0];
    }
    return false;
  },
});

/**
 * { x : A | ψ }
 */
export class RefinedTypeV {
  constructor(public handle: TypeV) {}

  display(): string {
    return this.handle.display();
  }

  map(fn: (type: TypeV) => TypeV): RefinedTypeV {
    return new RefinedTypeV(fn(this.handle));
  }

  sub(name: string, type: RefinedTypeV): RefinedTypeV {
    return this.map((handle) =>
      handle.match({
        Sum(entries) {
          return TypeV.Sum(
            Object.fromEntries(
              Object.entries(entries).map(([key, value]) => [
                key,
                value.sub(name, type),
              ]),
            ),
          );
        },
        Record(entries) {
          return TypeV.Record(
            Object.fromEntries(
              Object.entries(entries).map(([key, value]) => [
                key,
                value.sub(name, type),
              ]),
            ),
          );
        },
        Variant(tag, entries) {
          return TypeV.Variant(
            tag,
            Object.fromEntries(
              Object.entries(entries).map(([key, value]) => [
                key,
                value.sub(name, type),
              ]),
            ),
          );
        },
        Thunk(body) {
          return TypeV.Thunk(body.sub(name, type));
        },
        Recursive(name, body) {
          return TypeV.Recursive(name, body.sub(name, type));
        },
        Variable(innerName) {
          return innerName === name ? type.handle : TypeV.Variable(innerName);
        },
      }),
    );
  }

  unroll(): RefinedTypeV {
    if (!this.handle.Recursive) {
      return this;
    }
    const [name, body] = this.handle.Recursive;
    return body.sub(name, this);
  }

  equals(other: RefinedTypeV | null | undefined): boolean {
    return this.handle.equals(other?.handle);
  }

  comput(): TypeC {
    return TypeC.Produce(this, {});
  }
}

interface TTypeC {
  Produce(handle: RefinedTypeV, effects: Record<string, TypeC>): TypeC;
  With(bundle: Record<string, TypeC>): TypeC;
  Arrow(param: RefinedTypeV, body: TypeC): TypeC;
}
interface ITypeC {
  display(this: TypeC): string;
  sub(this: TypeC, name: string, type: RefinedTypeV): TypeC;
  equals(this: TypeC, other: TypeC | null | undefined): boolean;
  thunk(this: TypeC): TypeV;
}
export type TypeC = Handsum<TTypeC, ITypeC>;
export const TypeC = handsum<TTypeC, ITypeC>({
  display(): string {
    return this.match({
      Produce(handle, effects) {
        return `produce(${handle.display()}, ${Object.entries(effects)
          .map(([key, value]) => `${key}: ${value.display()}`)
          .join(',')})`;
      },
      With(bundle) {
        return `with(${Object.entries(bundle)
          .map(([key, value]) => `${key}: ${value.display()}`)
          .join(',')})`;
      },
      Arrow(param, body) {
        return `arrow(${param.display()}, ${body.display()})`;
      },
    });
  },

  sub(name: string, type: RefinedTypeV): TypeC {
    return this.match({
      Produce(handle, effects) {
        return TypeC.Produce(handle.sub(name, type), effects);
      },
      With(bundle) {
        return TypeC.With(
          Object.fromEntries(
            Object.entries(bundle).map(([key, value]) => [
              key,
              value.sub(name, type),
            ]),
          ),
        );
      },
      Arrow(param, body) {
        return TypeC.Arrow(param.sub(name, type), body.sub(name, type));
      },
    });
  },

  equals(other: TypeC | null | undefined): boolean {
    return this.match({
      Produce(handle, effects) {
        if (!other?.Produce) {
          return false;
        }
        const [otherHandle, otherEffects] = other.Produce;
        if (!handle.equals(otherHandle)) {
          return false;
        }
        return Array.from(
          new Set([...Object.keys(effects), ...Object.keys(otherEffects)]),
        ).every((key) => {
          const thisValue = effects[key];
          const otherValue = otherEffects[key];
          return thisValue && thisValue.equals(otherValue);
        });
      },
      With(bundle) {
        if (!other?.With) {
          return false;
        }
        const [otherBundle] = other.With;
        return Object.keys(bundle).every((key) => {
          const thisValue = bundle[key];
          const otherValue = otherBundle[key];
          return thisValue && thisValue.equals(otherValue);
        });
      },
      Arrow(param, body) {
        if (!other?.Arrow) {
          return false;
        }
        const [otherParam, otherBody] = other.Arrow;
        return param.equals(otherParam) && body.equals(otherBody);
      },
    });
  },

  thunk(): TypeV {
    return TypeV.Thunk(this);
  },
});
