# Lumo 3층위 구조

Lumo는 세 층위의 언어로 구성된다. 각 층위는 자기 스펙을 가지며, 위 층위는 코드로 작성된 번역을 거쳐 아래 층위로 lowering된다.

| 층위 | 확장자 | 역할 |
| --- | --- | --- |
| Lumo | `*.lumo` | 사용자가 직접 사용하는 언어 |
| Lumo MIR (LMIR) | `*.lmir` | 타입 추론이 종료되어 추론 코드가 필요 없어진 코드 |
| Lumo Target IR (LTIR) | `*.ltir` | 끝까지 번역된, 타겟 코드가 되기 직전의 최소 IR |

## Lumo

사용자가 직접 사용하는 언어다. Lumo에도 타입 스펙이 있고, 타입 추론은 Lumo에서 종료된다.

## LMIR

Lumo를 한 단계 lowering한 코드다. 타입 추론이 끝났으므로 추론 코드가 필요 없고, 모호한 문법은 desugar되어 있다.

사용자 오류를 검사하는 검사기는 LMIR에만 있다. 검사 중 발생한 오류는 Lumo AST에서 lowering할 때 내려온 range 정보로 raise된다.

## LTIR

Lumo를 끝까지 번역했고, 타겟 코드가 되기 직전의 최소 IR이다. 타입 선언을 포함하므로 LTIR만 배포해도 중간 언어로 기능한다.

LTIR은 자체 타입 검사기를 가진다. LMIR → LTIR은 Lumo → LMIR과 비슷한 과정이다. LTIR 타입에 알맞은 값이 오지 않으면 LMIR → LTIR lowering 실패이며, Internal Compiler Error다.

Target IR → Target Language 과정에서는 패턴에 따라 unlowering할 수 있다.

## 층위 간 번역

번역은 코드로 작성한다. 번역 코드를 정적 분석해 스펙을 생성하는 프레임워크는 long term 골이다.

번역을 기술하는 DSL은 open question이다.

## 메모리 모델

Linear Lambda Calculus를 확장해서 사용하고, Perceus를 채택한다. 따라서 refcount 의미론이 생긴다. 순환 참조는 허용하지 않는다.

GC인지 구분할 필요가 없다. refcount 스펙을 쓰고, GC 타겟에서는 free()가 실제 메모리 해제를 하지 않는다. refcount 변경은 그대로 한다. GC 등은 주입된 기능이다.

dup/drop은 LMIR → LTIR 단계에서 삽입한다.
