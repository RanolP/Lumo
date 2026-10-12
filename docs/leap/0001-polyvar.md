# LEAP 0001: polyvar

대상 층위는 LTIR이다. 1.0 스펙의 [합 타입](../ltir/enum.md)에 행 다형성을 더한다.

## 행 변수

행 변수 `T`는 `...T`로 `enum`에 전개한다. 행 변수 선언 표기는 미정이다.

```
enum { case .Io { .0 : std::IoError }, ...T }
```

## neg bound

`...T` 옆에 명시된 tag는 `T`의 neg bound(부재 제약)가 된다. 위 타입에서 `T`는 `.Io`를 가질 수 없다. 별도 표기는 없다.

neg bound는 선언 사이에 전파된다. 다른 선언의 행 매개변수 자리에 넘긴 행 변수는 그 매개변수가 가진 neg bound를 물려받는다.

적용 지점에서 넘긴 행의 tag가 neg bound와 겹치면 LTIR 검사 오류다.

## 빈 행

확장이 없는 빈 행은 `enum {}`로 쓴다.

## 미결 질문

- 행 변수 선언 표기
