# Type Annotations

TLDR: We consistently use `value-ish: Ty` for type annotations.

## Unified Language for Annotating Types

Following declaration states that the `x` has type `s32` and the `y` has type `u32`.

```
let x: s32 = 3
let S(S(y: u32)) = 3
```

Similarly, following states that `f(x)` has type `s32` (where `x` is `s32`).

```
fn f(x: s32): s32 => 2 * x
```

Furthermore, we can use the same syntax for declaring GADTs in enum declaration.
Think (G)ADT constructor as collection of functions making values.

```
enum Expr<A> {
    IntVal(s32): Expr<s32>,
    BoolVal(bool): Expr<bool>,
    Add(Expr<s32>, Expr<s32>): Expr<s32>,
    Equiv(Expr<A>, Expr<A>): Expr<bool> where A: Eq,
}
```

And of coarse in struct fields, we annotates. (In tuple-like definition, there's skipped name `0:`, `1:`, and so on)

```
struct Pos2d(s32, s32)
struct Config {
    verbose: bool,
    long: bool,
}
```

## No `->` on Return Types

We talked about why unified language is good but someone can tackle some arguments like `Rust uses ->`!

Here's another view for why not `->`.

Since we use `=>` for simple expression body for function declaration.
`->` arrow symbol can be confusing. Consider this:

```
fn add1(x: s32) -> s32 => x + 1
```

Therefore, it is not a good idea to use `->` on return types.
