# LEAP 0002: record

대상 층위는 LTIR이다.

## record 타입

`record { ... }` 안에 필드를 콤마로 구분해 나열한다. 필드 이름은 tag다.

```
record { .x : T, .y : U }
```

## 행 확장

[LEAP 0001](0001-polyvar.md)과 함께 채택하면 record에도 `enum`과 같은 행 다형성 규칙을 적용한다. 행 변수 `R`은 `...R`로 전개한다.

```
record { .x : T, .y : U, ...R }
```

확장이 없는 빈 행은 `record {}`로 쓴다.
