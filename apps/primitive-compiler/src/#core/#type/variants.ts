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
    readonly then: Type,
  ) {}

  id(): string {
    return `∀${this.name}.`;
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
}

/**
 * A | B | C | ... | Z
 */
export class Sum implements IType {
  readonly items: Set<Type>;
  constructor(readonly origin: AstId, items: Type[]) {
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
}

/**
 * (A, B, C, ..., Z)
 */
export class Prod implements IType {
  static unit = new Prod(null, []);

  constructor(readonly origin: AstId | null, readonly types: Type[]) {}

  id(scope: TypeScope): string {
    return this.types.map((ty) => ty.id(scope)).join(' × ');
  }

  toString(): string {
    return this.types.length <= 3
      ? `(${this.types.join(', ')})`
      : `(\n${this.types.join(',\n')}\n)`;
  }
}

/**
 * mu X.
 */
export class Recursion implements IType {
  constructor(
    readonly origin: AstId,
    readonly name: Path,
    readonly then: Type,
  ) {}

  id(scope: TypeScope): string {
    return `μ ${this.name.display}, ${this.then.id(scope)}`;
  }

  toString(): string {
    return `μ ${this.name.display}, ${this.then.toString()}`;
  }
}

export class Constructor implements IType {
  constructor(
    readonly origin: AstId | null,
    readonly folded: string,
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
}

interface IType {
  readonly origin: AstId | null;
  id(scope: TypeScope): string;
}
