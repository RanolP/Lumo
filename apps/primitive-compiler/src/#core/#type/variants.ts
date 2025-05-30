import { TypeScope } from './scope.js';
import { AstId } from '../#ast/base.js';
import { Identifier, Path } from '../#ast/construct.js';
import { normalizeType } from './normalize.js';
import { match, P } from 'ts-pattern';

export type Type =
  | TypeVar
  | Quantification
  | TypeApplication
  | Lambda
  | Sum
  | Prod
  | Constructor
  | Recursion;

/**
 * forall A.
 */
export class Quantification implements IType {
  constructor(
    readonly origin: AstId,
    readonly name: string,
    readonly then: any, // Type
  ) {}

  id(): string {
    return `∀${this.name}.`;
  }

  replace(path: Path, withTy: Type): Type {
    return new Quantification(
      this.origin,
      this.name,
      this.then.replace(path, withTy),
    );
  }

  equals(other: Type): boolean {
    return (
      other instanceof Quantification &&
      this.name === other.name &&
      this.then.equals(other.then)
    );
  }
}

/**
 * Path to type like "a.b.c"
 */
export class TypeVar implements IType {
  constructor(readonly origin: AstId, readonly path: Path) {}

  toString(): string {
    return `${this.path.display}`;
  }

  id(scope: TypeScope): string {
    return `${this.toString()}`;
  }

  replace(path: Path, withTy: Type): Type {
    return this.path.equals(path) ? withTy : this;
  }

  equals(other: Type): boolean {
    return other instanceof TypeVar && this.path.equals(other.path);
  }
}

/**
 * Given `F = forall A. A -> A` and `Arg = Int`.
 * `F(Arg) = Int -> Int`
 *
 * Similarly you can extend this with HKT in future.
 */
export class TypeApplication implements IType {
  constructor(
    readonly origin: AstId,
    readonly type: Type,
    readonly argument: Type,
  ) {}

  id(scope: TypeScope): string {
    return `${this.type.id(scope)}(${this.argument.id(scope)})`;
  }

  replace(path: Path, withTy: Type): Type {
    return new TypeApplication(
      this.origin,
      this.type.replace(path, withTy),
      this.argument.replace(path, withTy),
    );
  }

  equals(other: Type): boolean {
    if (!(other instanceof TypeApplication)) return false;
    return this.type.equals(other.type) && this.argument.equals(other.argument);
  }
}

/**
 * (A, B, C, ...Y) -> Z
 */
export class Lambda implements IType {
  constructor(
    readonly origin: AstId,
    readonly parameters: Type[],
    readonly returning: Type,
  ) {}

  id(scope: TypeScope): string {
    return `(${this.parameters
      .map((ty) => ty.id(scope))
      .join('×')}) → ${this.returning.id(scope)}`;
  }

  toString(): string {
    return `(${this.parameters.join(', ')}) => ${this.returning}`;
  }

  replace(path: Path, withTy: Type): Type {
    return new Lambda(
      this.origin,
      this.parameters.map((ty) => ty.replace(path, withTy)),
      this.returning.replace(path, withTy),
    );
  }

  equals(other: Type): boolean {
    if (!(other instanceof Lambda)) return false;
    if (this.parameters.length !== other.parameters.length) return false;
    if (!this.returning.equals(other.returning)) return false;
    for (let i = 0; i < this.parameters.length; i++) {
      if (!this.parameters[i].equals(other.parameters[i])) return false;
    }
    return true;
  }
}

/**
 * A | B | C | ... | Z
 */
export class Sum implements IType {
  static never: Type = new Sum(null, []);

  readonly items: Set<Type>;
  constructor(readonly origin: AstId | null, items: Type[]) {
    this.items = new Set(items);
  }

  id(scope: TypeScope): string {
    return `Σ{${Array.from(this.items)
      .map((ty) => ty.id(scope))
      .join(',')}}`;
  }

  toString(): string {
    return this.items.size <= 3
      ? `{${Array.from(this.items).join(' | ')}}`
      : `Σ{\n${Array.from(this.items)
          .map((ty) => `| ${ty}`)
          .join('\n')}\n}`;
  }

  replace(path: Path, withTy: Type): Type {
    return new Sum(
      this.origin,
      Array.from(this.items).map((ty) => ty.replace(path, withTy)),
    );
  }

  equals(other: Type): boolean {
    if (!(other instanceof Sum)) return false;
    if (this.items.size !== other.items.size) return false;
    const otherItems = Array.from(other.items);
    for (const ty of this.items) {
      if (otherItems.every((otherTy) => !ty.equals(otherTy))) {
        return false;
      }
    }
    return true;
  }
}

/**
 * (A, B, C, ..., Z)
 */
export class Prod implements IType {
  static unit: Type = new Prod(null, []);

  constructor(readonly origin: AstId | null, readonly types: Type[]) {}

  id(scope: TypeScope): string {
    return this.types.map((ty) => ty.id(scope)).join(' × ');
  }

  toString(): string {
    return this.types.length <= 3
      ? `(${this.types.join(', ')})`
      : `(\n${this.types.join(',\n')}\n)`;
  }

  replace(path: Path, withTy: Type): Type {
    return new Prod(
      this.origin,
      this.types.map((ty) => ty.replace(path, withTy)),
    );
  }

  equals(other: Type): boolean {
    if (!(other instanceof Prod)) return false;
    if (this.types.length !== other.types.length) return false;
    for (let i = 0; i < this.types.length; i++) {
      if (!this.types[i].equals(other.types[i])) return false;
    }
    return true;
  }
}

/**
 * mu X.
 */
export class Recursion implements IType {
  constructor(
    readonly origin: AstId,
    readonly name: Path,
    readonly then: any, // Type
  ) {}

  id(scope: TypeScope): string {
    return `μ ${this.name.display}, ${this.then.id(scope)}`;
  }

  toString(): string {
    return `μ ${this.name.display}, ${this.then.toString()}`;
  }

  unfold(): Type {
    return (this.then as IType).replace(this.name, this.then);
  }

  replace(path: Path, withTy: Type): Type {
    return new Recursion(
      this.origin,
      this.name,
      this.then.replace(path, withTy),
    );
  }

  equals(other: Type): boolean {
    return (
      other instanceof Recursion &&
      this.name.equals(other.name) &&
      this.then.equals(other.then)
    );
  }
}

export class Constructor implements IType {
  constructor(
    readonly origin: AstId,
    readonly folded: Identifier,
    readonly tag: string,
    readonly items:
      | { kind: 'positional'; types: { type: Type }[] }
      | { kind: 'named'; types: { name: string; type: Type }[] },
  ) {}

  id(scope: TypeScope): string {
    switch (this.items.kind) {
      case 'positional':
        return this.items.types.length <= 3
          ? `${this.tag}+ctor(${this.items.types
              .map(({ type }) => type.id(scope))
              .join(', ')})`
          : `${this.tag}+ctor(\n${this.items.types
              .map(({ type }) => type.id(scope))
              .join(',\n')}\n)`;
      case 'named':
        return this.items.types.length === 0
          ? `${this.tag}+ctor {}`
          : `${this.tag}+ctor {\n${this.items.types
              .map(({ name, type }) => `${name}: ${type.id(scope)}`)
              .join(',\n')}\n}`;
    }
  }

  toString(): string {
    switch (this.items.kind) {
      case 'positional':
        return this.items.types.length <= 3
          ? `${this.tag}+ctor(${this.items.types
              .map(({ type }) => type)
              .join(', ')})`
          : `${this.tag}+ctor(\n${this.items.types
              .map(({ type }) => type)
              .join(',\n')}\n)`;
      case 'named':
        return this.items.types.length === 0
          ? `${this.tag}+ctor {}`
          : `${this.tag}+ctor {\n${this.items.types
              .map(({ name, type }) => `${name}: ${type}`)
              .join(',\n')}\n}`;
    }
  }

  replace(path: Path, withTy: Type): Constructor {
    return new Constructor(
      this.origin,
      this.folded,
      this.tag,
      match(this.items)
        .with({ kind: 'named' }, ({ types }) => ({
          kind: 'named' as const,
          types: types.map(({ name, type }) => ({
            name,
            type: type.replace(path, withTy),
          })),
        }))
        .with({ kind: 'positional' }, ({ types }) => ({
          kind: 'positional' as const,
          types: types.map(
            ({ type }) =>
              ({
                type: type.replace(path, withTy),
              } as const),
          ),
        }))
        .exhaustive(),
    );
  }

  equals(other: Type): boolean {
    if (!(other instanceof Constructor)) return false;
    if (!this.folded.equals(other.folded)) return false;
    if (this.tag !== other.tag) return false;
    return true;
  }
}

interface IType {
  readonly origin: AstId | null;
  id(scope: TypeScope): string;
  replace(path: Path, withTy: Type): Type;
  equals(other: Type): boolean;
}
