import { type Handsum, handsum } from 'handsum';
import { Hasher } from '../vendors/malssi/hasher';

export interface TTypeV {
  Sum(entries: Record<string, RefinedTypeV>): TypeV;
  Record(entries: Record<string, RefinedTypeV>): TypeV;
  Variant(tag: string, entries: Record<string, RefinedTypeV>): TypeV;
  Thunk(handle: TypeC): TypeV;
  Recursive(name: string, body: RefinedTypeV): TypeV;
  Variable(name: string): TypeV;
  TyAbsV(name: string, body: RefinedTypeV): TypeV;
  TyAppV(body: RefinedTypeV, type: RefinedTypeV): TypeV;
}
export interface ITypeV {
  freshRefined(this: TypeV): RefinedTypeV;

  display(this: TypeV): string;

  vars(this: TypeV): Set<string>;

  hashCode(this: TypeV, idx?: number): number;
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
          .join(', ')})`;
      },
      Record(entries) {
        return `record {${Object.entries(entries)
          .map(([key, value]) => `${key}: ${value.display()}`)
          .join(', ')}}`;
      },
      Variant(tag, entries) {
        return `variant[${tag}] {${Object.entries(entries)
          .map(([key, value]) => `${key}: ${value.display()}`)
          .join(', ')}}`;
      },
      Thunk(body) {
        return `thunk(${body.display()})`;
      },
      Recursive(name, body) {
        return `μ${name}. (${body.display()})`;
      },
      Variable(name) {
        return name;
      },
      TyAbsV(name, body) {
        return `forall ${name}. (${body.display()})`;
      },
      TyAppV(body, type) {
        return `${body.display()}[${type.display()}]`;
      },
    });
  },
  vars() {
    return this.match({
      Sum(entries) {
        return new Set(
          ...Object.values(entries).map((entry) => entry.handle.vars()),
        );
      },
      Record(entries) {
        return new Set(
          ...Object.values(entries).map((entry) => entry.handle.vars()),
        );
      },
      Variant(tag, entries) {
        return new Set(
          ...Object.values(entries).map((entry) => entry.handle.vars()),
        );
      },
      Thunk(body) {
        return body.vars();
      },
      Recursive(name, body) {
        return new Set([name, ...body.handle.vars()]);
      },
      Variable(name) {
        return new Set([name]);
      },
      TyAbsV(name, body) {
        return new Set([name, ...body.handle.vars()]);
      },
      TyAppV(body, type) {
        return new Set([...body.handle.vars(), ...type.handle.vars()]);
      },
    });
  },
  hashCode(this: TypeV, idx: number = 0): number {
    return this.match({
      Sum(entries) {
        const hasher = new Hasher().with('TypeV.Sum');
        for (const [key, value] of Object.entries(entries)) {
          hasher.with(key).with(value.hashCode(idx));
        }
        return hasher.result();
      },
      Record(entries) {
        const hasher = new Hasher().with('TypeV.Record');
        for (const [key, value] of Object.entries(entries)) {
          hasher.with(key).with(value.hashCode(idx));
        }
        return hasher.result();
      },
      Variant(tag, entries) {
        const hasher = new Hasher().with('TypeV.Variant').with(tag);
        for (const [key, value] of Object.entries(entries)) {
          hasher.with(key).with(value.hashCode(idx));
        }
        return hasher.result();
      },
      Thunk(body) {
        return new Hasher()
          .with('TypeV.Thunk')
          .with(body.hashCode(idx))
          .result();
      },
      Recursive(name, body) {
        return new Hasher()
          .with('TypeV.Recursive')
          .with(
            body
              .sub_v(name, TypeV.Variable(`#${idx}`).freshRefined())
              .hashCode(idx + 1),
          )
          .result();
      },
      Variable(name) {
        return new Hasher().with('TypeV.Variable').with(name).result();
      },
      TyAbsV(name, body) {
        return new Hasher()
          .with('TypeV.TyAbsV')
          .with(
            body
              .sub_v(name, TypeV.Variable(`#${idx}`).freshRefined())
              .hashCode(idx + 1),
          )
          .result();
      },
      TyAppV(body, type) {
        return new Hasher()
          .with('TypeV.TyAppV')
          .with(body.hashCode(idx))
          .with(type.hashCode(idx))
          .result();
      },
    });
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

  sub_v(name: string, type: RefinedTypeV): RefinedTypeV {
    return this.map((handle) =>
      handle.match({
        Sum(entries) {
          return TypeV.Sum(
            Object.fromEntries(
              Object.entries(entries).map(([key, value]) => [
                key,
                value.sub_v(name, type),
              ]),
            ),
          );
        },
        Record(entries) {
          return TypeV.Record(
            Object.fromEntries(
              Object.entries(entries).map(([key, value]) => [
                key,
                value.sub_v(name, type),
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
                value.sub_v(name, type),
              ]),
            ),
          );
        },
        Thunk(body) {
          return TypeV.Thunk(body.sub(name, type));
        },
        Recursive(innerName, body) {
          return TypeV.Recursive(innerName, body.sub_v(name, type));
        },
        Variable(innerName) {
          return innerName === name ? type.handle : TypeV.Variable(innerName);
        },
        TyAbsV(innerName, body) {
          return TypeV.TyAbsV(innerName, body.sub_v(name, type));
        },
        TyAppV(body, type) {
          return TypeV.TyAppV(body.sub_v(name, type), type.sub_v(name, type));
        },
      }),
    );
  }

  unroll(): RefinedTypeV {
    if (!this.handle.Recursive) {
      return this;
    }
    const [name, body] = this.handle.Recursive;
    return body.sub_v(name, this);
  }

  comput(): TypeC {
    return TypeC.Produce(this, {});
  }

  hashCode(this: RefinedTypeV, idx: number = 0): number {
    return this.handle.hashCode(idx);
  }
}

interface TTypeC {
  Produce(handle: RefinedTypeV, effects: Record<string, TypeC>): TypeC;
  With(bundle: Record<string, TypeC>): TypeC;
  Arrow(param: RefinedTypeV, body: TypeC): TypeC;
  Variable(name: string): TypeC;
}
interface ITypeC {
  display(this: TypeC): string;
  sub(this: TypeC, name: string, type: RefinedTypeV): TypeC;
  thunk(this: TypeC): TypeV;
  vars(this: TypeC): Set<string>;
  hashCode(this: TypeC, idx?: number): number;
}
export type TypeC = Handsum<TTypeC, ITypeC>;
export const TypeC = handsum<TTypeC, ITypeC>({
  display(): string {
    return this.match({
      Produce(handle, effects) {
        return `produce(${handle.display()}${
          Object.entries(effects).length > 0 ? ', ' : ''
        }${Object.entries(effects)
          .map(([key, value]) => `${key}: ${value.display()}`)
          .join(', ')})`;
      },
      With(bundle) {
        return `with(${Object.entries(bundle)
          .map(([key, value]) => `${key}: ${value.display()}`)
          .join(', ')})`;
      },
      Arrow(param, body) {
        return `(${param.display()}) -> ${body.display()}`;
      },
      Variable(name) {
        return name;
      },
    });
  },

  sub(name: string, type: RefinedTypeV): TypeC {
    return this.match({
      Produce(handle, effects) {
        return TypeC.Produce(handle.sub_v(name, type), effects);
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
        return TypeC.Arrow(param.sub_v(name, type), body.sub(name, type));
      },
      Variable(name) {
        return TypeC.Variable(name);
      },
    });
  },

  thunk(): TypeV {
    return TypeV.Thunk(this);
  },

  vars(): Set<string> {
    return this.match({
      Produce(handle, effects) {
        return new Set(
          ...handle.handle.vars(),
          ...Object.values(effects).map((value) => value.vars()),
        );
      },
      With(bundle) {
        return new Set(...Object.values(bundle).map((value) => value.vars()));
      },
      Arrow(param, body) {
        return new Set([...param.handle.vars(), ...body.vars()]);
      },
      Variable(name) {
        return new Set([name]);
      },
    });
  },

  hashCode(this: TypeC, idx: number = 0): number {
    return this.match({
      Produce(handle, effects) {
        const hasher = new Hasher().with('TypeC.Produce');
        hasher.with(handle);
        for (const [key, value] of Object.entries(effects)) {
          hasher.with(key).with(value.hashCode(idx));
        }
        return hasher.result();
      },
      With(bundle) {
        const hasher = new Hasher().with('TypeC.With');
        for (const [key, value] of Object.entries(bundle)) {
          hasher.with(key).with(value.hashCode(idx));
        }
        return hasher.result();
      },
      Arrow(param, body) {
        return new Hasher()
          .with('TypeC.Arrow')
          .with(param.hashCode(idx))
          .with(body.hashCode(idx))
          .result();
      },
      Variable(name) {
        return new Hasher().with('TypeC.Variable').with(name).result();
      },
    });
  },
});
