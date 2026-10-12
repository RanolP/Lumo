# 합 타입

합 타입은 `enum`으로만 쓴다. `enum { ... }` 안에 `case`를 콤마로 구분해 나열한다.

```
type Either[A, B] = enum { case .left { .0 : A }, case .right { .0 : B } }
```

한 `enum` 안에서 tag 중복은 금지한다.

`case` 이름은 tag다.

`case`는 `case .name { ...field }` 형식으로 쓴다. `...field`는 `.name: TypeExpr`를 콤마로 구분한 목록이다.
