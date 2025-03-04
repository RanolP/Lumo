export type Type =
  | Quantification
  | TypeVar
  | TypeApplication
  | Lambda
  | Sum
  | Prod
  | Recursion;

/**
 * forall A.
 */
export class Quantification {
  constructor(readonly name: string, readonly then: Type) {}
}

/**
 * Just "T"
 */
export class TypeVar {
  constructor(readonly name: string) {}
}

/**
 * Given `F = forall A. A -> A` and `Arg = Int`.
 * `F(Arg) = Int -> Int`
 *
 * Similarly you can extend this with HKT in future.
 */
export class TypeApplication {
  constructor(readonly type: Type, readonly argument: Type) {}
}

/**
 * (A, B, C, ...Y) -> Z
 */
export class Lambda {
  constructor(readonly parameters: Type[], readonly returning: Type) {}
}

/**
 * A | B | C | ... | Z
 */
export class Sum {
  constructor(readonly items: Type[]) {}
}

/**
 * (A, B, C, ..., Z)
 */
export class Prod {
  constructor(readonly items: Type[]) {}
}

/**
 * mu X.
 */
export class Recursion {
  constructor(readonly name: string, readonly then: Type) {}
}
