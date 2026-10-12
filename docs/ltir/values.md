# 값 선언

`let name = e` 형식으로 쓴다. 값 선언은 무조건 constexpr다.

constexpr인 값은 다음과 같다.

- function: 가장 바깥이 `fn`이면 body가 무엇이든 constexpr다. 인자를 주기 전까지는 함수 레퍼런스 자체인 const 값이다.
- bundle
- forall

## function

`fn(x : A / B): C \ E -> e` 형식으로 쓴다. 다인자 함수만 구현한다. 함수 타입은 [types.md](types.md#함수-타입)에 있다.

`A/B` 타입 값을 받는 인자에는 `~`를 붙인다.

```
fn(~a : A/B, ~b : B/A) -> ...
```

함수 타입을 반환하려면 반환 위치에 괄호를 친다.

## bundle

`bundle { .name: ..., .name2: ..., }` 형식으로 쓴다. tag로 읽을 수 있는 함수 묶음이다.

## tag

`.{id}` 형태를 모두 tag라고 부른다. `.1`, `.2`, `.name`처럼 쓴다.

tag는 런타임에 존재하지 않는 이급값이다. 값 선언의 우변에 올 수 없다.

tag의 사용처는 다음과 같다.

- bundle에 apply해서 함수를 뽑아낸다.
- 값에 apply해서 field를 읽는다.
- 이름으로써 enum case match에 쓴다.

## forall

`forall T. e` 형식으로 쓰는 TyAbs다. kind를 지정하지 않으면 `*`다.

## 타입 적용

`x[T]` 형식으로 쓰는 TyApp이다.
