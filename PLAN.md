1. 언어는 Rust로 한다
2. 그 어떤 라이브러리도 사용하지 않는다 (부트스트래핑에 용이)
3. Query-based Compiler를 짜서 compiler-language server 간 코드를 공유한다
4. Lossless Syntax Tree를 사용한다
5. Tree Sitter 논문에서 error-recovering parser 접근을 취한다
6. IR 전략
   6.1. IR는 우선 분리한다 (LST→HIR→Core IR)
   6.2. 추적성은 노드 Unique ID로 보장한다 (ID=정체성)
   6.3. 성능/증분 처리를 위해 Structural Hash를 병행한다 (Hash=동등성/캐시 키)
7. MVP 범위
   7.1. LSP 목표는 우선 하이라이팅이 가능한 상태로 한다
   7.2. 컴파일러 스펙은 우선 data 정의와 함수까지로 한정한다
8. 문법 MVP 예시
   8.1. data 예시
      data Option[A: *] { Some(A), None }
   8.2. fn 예시
      fn id[A: *](a: A): produce A / {} := produce a
9. 어휘(lexer) 규칙
   9.1. 기호는 아스키 토큰으로 처리한다 ([ ] ( ) { } : , = := / * 등)
   9.2. 식별자는 Unicode Ident_Start + Ident_Continue 규칙을 따른다
10. 구현 마일스톤
   10.1. M1 Lexer
      - 키워드/식별자/기호 토큰화 및 span 산출
   10.2. M2 Parser (Error Recovery)
      - data, fn, let ... in ..., produce 파싱
      - 깨진 입력에서도 트리 + 에러 생성
   10.3. M3 IR 기초
      - LST 생성
      - HIR lowering
      - 노드 Unique ID / Structural Hash 부여
   10.4. M4 Query 엔진 최소셋
      - parse(file), lower(file), diagnostics(file)
   10.5. M5 LSP 하이라이팅
      - 토큰 기반 하이라이팅 제공
11. 완료(Definition of Done) 기준
   11.1. M1 Done
      - 지정 키워드(data, fn, let, in, produce)와 아스키 기호 토큰이 모두 분리된다
      - 모든 토큰에 시작/끝 span이 붙는다
      - 대표 입력 20개 이상 골든 테스트 통과
   11.2. M2 Done
      - data/fn/let-in/produce 구문 파싱 성공
      - 문법 오류 입력에서도 AST(LST) 생성이 중단되지 않는다
      - 파서 에러가 span과 함께 보고된다
   11.3. M3 Done
      - LST→HIR lowering 경로가 data/fn에 대해 동작한다
      - 모든 핵심 노드에 Unique ID가 부여된다
      - Structural Hash가 재계산 가능하며 동일 입력에서 결정적으로 일치한다
   11.4. M4 Done
      - parse/lower/diagnostics query가 파일 단위로 호출 가능
      - 변경 없는 재호출 시 캐시 재사용이 확인된다
   11.5. M5 Done
      - LSP에서 키워드/식별자/기호 하이라이팅이 응답된다
      - 문법 오류가 있어도 가능한 범위의 하이라이팅이 유지된다

12. 디렉터리 구조(초안)
   12.1. crates/compiler
      - src/lexer
      - src/parser
      - src/lst
      - src/hir
      - src/query
      - src/diagnostics
   12.2. crates/lsp
      - src/server
      - src/highlight
   12.3. tests
      - lexer/
      - parser/
      - recovery/
      - query/

13. 우선순위/진행 원칙
   13.1. 순서는 M1 → M2 → M3 → M4 → M5로 고정한다
   13.2. 선행 마일스톤 Done 전에는 다음 마일스톤 구현을 시작하지 않는다
   13.3. 공통 로직은 compiler에 두고 lsp는 query 호출만 담당한다

14. 첫 주 실행 계획
   14.1. Day 1-2: Lexer 토큰 enum, span 타입, 키워드/기호 분기 구현
   14.2. Day 3: Unicode Ident_Start/Ident_Continue 식별자 처리 구현
   14.3. Day 4: Lexer 골든 테스트 20개 작성 및 고정
   14.4. Day 5: Parser 뼈대(data/fn 헤더) + 기본 에러 리포트 시작
