상태: 승인 대기 (사람 결정 7개 포함)

# J020 (디자인 3.0 D0): PC에만 있던 변경을 저장소에 올리기

## 명세 (spec)
- 기준 커밋: `acf4e5d53dd129c2f1057aa37719efc682396c34` (origin/main, J014, 2026-10-15)
- 작업 브랜치: `claude/project-thread-ams9bp` (origin/main에서 새로 시작. 예전 끝 커밋 `dd3cfde`는 main에 다른 형태로 들어간 J003 시안이며 `origin/claude/project-thread-yam40j`에 그대로 남아 있음)
- 근거: 디자인 3.0 (https://claude.ai/artifact/GmVPKvYKq4ADAe5ZbmZLdc) 11장 D0, 사용자 「진행시켜」 2026-10-16 07:54Z

### 목표
사용자 PC(young, `D:\vector`, main 위)의 커밋하지 않은 변경 18개 파일을 그대로 작업 브랜치에 올려, 이후 작업(D1~D7)이 사용자가 실제로 보고 있는 앱과 같은 코드에서 시작하게 한다.

### 완료 기준
1. 저장소의 18개 파일이 PC 파일과 줄바꿈(CRLF→LF)만 빼고 같다. 확인: `git diff --ignore-cr-at-eol --stat`(PC) = `git diff --stat`(브랜치), 863줄 추가 · 322줄 삭제.
2. `cargo build -p vectorcraft`가 성공한다.
3. `cargo fmt --all -- --check`가 통과한다.
4. `cargo test -p vectorcraft-ui-egui trowel`과 `cargo test -p vectorcraft-engine`이 통과한다 (기존 실패가 있으면 그 목록을 따로 적는다).
5. GitHub 「Trowel CI」가 이 커밋에서 통과한다.

### 변경 범위
PC에서 받은 그대로(내용을 고치지 않음): `crates/engine/src/cmd/{buildcmds,elevation,hatch,trowel_plan}.rs`, `crates/engine/src/lib.rs`, `crates/pathops/src/{elevation,hatch}.rs`, `crates/text/src/fontdb.rs`, `crates/tools/src/builder.rs`, `crates/ui-egui/src/{chrome,widgets}.rs`, `crates/ui-egui/src/i18n/ko.tsv`, `crates/ui-egui/src/trowel/{drawings,mod,palette,props,ribbon,tests}.rs`.
PC 폴더에는 쓰지 않는다(읽기만).

### PC 변경의 내용 (읽은 것)
- 단면선·표고 명령이 리본이 보내던 인자 이름(`start/end`, `elevation_m`)도 받음 → 디자인 3.0이 찾은 고장 2건이 엔진 쪽에서 풀림.
- 한 번 눌러 넣는 요소가 화면 가운데 근처에 조금씩 비켜 놓임(`trowel::place_symbol_at`), 고정 좌표에 겹치던 고장이 줄어듦.
- 해발 기준점 모양이 「검은 보조선 + 붉은 막대 + 6pt 글자」로 바뀜, 단면 표시 화살표 모양 바뀜.
- `elevation.create`가 선택이 없을 때도 기준선을 만들고, 단면 윤곽을 한 줄로 정리함(`pathops::elevation`).
- 칠통으로 한 도형을 칠하면 라이브 페인트 묶음 없이 그 도형에 빗금(`tools::builder`, `cmd::hatch`), 세션에 기본 빗금 값.
- 굵기 칩 9개 한 줄(0.1~1.0, 0.4는 파선), 한글 대체 글꼴 목록에 윤고딕 이름 추가(글꼴 파일은 넣지 않음), 속성 크기 mm 표시, 조절줄 X·Y·W·H 좁은 칸.

### 위험·사람 검토 필요
- ◆ 수치 알고리즘: `pathops/src/elevation.rs` 해발선 범위·자르기 변경, `cmd/elevation.rs` 기준선 자동 생성. 사용자가 PC에서 만든 것이라 내용은 그대로 두고, 검토만 한다.
- 글꼴 이름(윤고딕)은 설치된 글꼴을 찾는 이름일 뿐 파일을 넣지 않으므로 자산 규칙에 걸리지 않는다.

### 범위 밖
디자인 3.0의 화면 정리(D2 이후). 여기서는 고치지 않는다.

## 빌드 (build)
- 2026-10-16. 기준 `acf4e5d`. PC 파일을 CRLF→LF로만 바꿔 옮김.
- 커밋
  - `0900a12` PC 변경 18개 파일 그대로
  - `09c2ff0` rustfmt 서식만(PC 파일 7개 + main에 이미 서식이 안 맞던 `cmd/trowel.rs`, `trowel/palette.rs`, `trowel/tests.rs`)
  - `63a624a` CI 고침 1: `elevation.create`가 「해발 기준선」을 따로 한 편집으로 만들어 되돌리기 한 번에 기준선이 남던 것을 한 편집으로 합침. 해발선 계산은 그대로
  - `4fce5b7` CI 고침 2: main부터 리눅스에서 실패하던 시험 4개(도면집 윈도우 경로 시험, 굵기 표의 같은 줄 판정, 「None (Delete)」 번역, 범례 상태 메시지 2개의 한국어 원문 → 영어 원문 + 번역)
- 명세에서 벗어난 점: 「내용을 고치지 않음」이었지만 CI를 통과시키려고 위 고침 2개를 더함(사용자 「CI 수정」 2026-10-16 08:52Z).

## 검증 (verify)
로그: 스크래치패드 `factory/` (`d0-build.log`, `d0-test-engine.log`, `d0-test-engine-2.log`, `d0-fix-sweep.log`, `d0-test-ui-all.log`). 모든 시험은 `CARGO_TARGET_DIR=/home/claude/target-trowel CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`(CI와 같음).

| 기준 | 명령 | 커밋 | 결과 |
|---|---|---|---|
| 1 | `git diff --ignore-cr-at-eol --stat`(PC) 대 `git diff --stat acf4e5d 0900a12` | 0900a12 | 같음 (18개 파일) |
| 2 | `cargo build -p vectorcraft` | 0900a12 | 종료 0 (17분 55초, 디버그 정보 켠 빌드) |
| 3 | `cargo fmt --all -- --check` | 0900a12 | **실패** 파일 10개(PC 7, main에 원래 있던 것 3) |
| 3 | 같은 명령 | 09c2ff0 이후 | 종료 0 |
| 4 | `cargo test -p vectorcraft-engine -p vectorcraft-pathops -p vectorcraft-text -p vectorcraft-tools` | 09c2ff0 | 첫 시도: 디스크가 차서 중단(환경). 두 번째: **실패** `command_sweep::every_command_undoes_exactly` (`elevation.create: children 6 vs 7`), 1781 통과 뒤 멈춤 |
| 4 | `cargo test -p vectorcraft-engine --test command_sweep --test cmd_elevation` | 63a624a(커밋 전) | 종료 0, 15·3 통과 |
| 4 | `cargo test -p vectorcraft-engine -p vectorcraft-pathops -p vectorcraft-text -p vectorcraft-tools --no-fail-fast` | 63a624a | 종료 0, 2801 통과 0 실패 |
| 4 | `cargo test -p vectorcraft-ui-egui --no-fail-fast trowel` | 63a624a | **실패** 2개(도면집 경로, 굵기 표) — main에 있던 시험, 이 PR의 변경과 무관 |
| 4 | `cargo test -p vectorcraft-ui-egui --no-fail-fast` | 4fce5b7(커밋 전) | 종료 0, 1050 통과 0 실패 |
| 5 | GitHub 「Trowel CI」 | 09c2ff0 | **실패**: fmt·clippy 통과, test에서 위 command_sweep 실패(run 38873673627). 윈도우 x64 빌드는 성공 |
| 5 | GitHub 「Trowel CI」 | 4fce5b7 | **통과**: `cargo xtask ci (Linux)` success, `Windows x64 release build` success (run 38879723907) |

## 검토 (review) 1차
- 검토자: 독립 검토 에이전트(코드·정확성·보안), 읽기만. 범위 `acf4e5d..4fce5b7`. 판정: **조건부 통과**(blocker 없음).
- major
  1. 해발선 줄 수 상한 없음: `pathops/src/elevation.rs` `count = (max-min)/minor`. 빈 도면에 `{"fillBelowProfile":true,"majorIntervalM":1e7}`이면 선분 약 2억 개 → 메모리 부족으로 앱이 꺼짐(guard로 못 잡음). acf4e5d에도 Explicit 범위로 있던 문제, PC 변경으로 닿는 길이 넓어짐. → **재작업 1에서 고침**.
  2. 닫힌 윤곽에서 고르는 사슬이 주석과 반대(`pathops/src/elevation.rs:150-154`): 사각형 구덩이를 위쪽 모드로 하면 선이 구덩이 안이 아니라 바깥 위에 생김. → 사람 결정(아래 4).
  3. 보이지 않는 「해발 기준선」(칠·선 없음, 해발선 묶음 밖)이 다음 실행 때 윤곽으로 다시 쓰여 같은 자리에 해발선이 한 벌 더 생기고, 묶음을 지워도 남음(`cmd/elevation.rs`). → 사람 결정(아래 5).
  4. 칠통의 빗금 기본값(`default_hatch`)이 세션에 계속 남아, 그 뒤 모든 문서의 칠에 빗금이 붙음(`cmd/buildcmds.rs:579-586`, `hatch.rs:114,149`). → 사람 결정(아래 7).
  5. 칠통으로 고르지 않은 도형을 누르면 바로 라이브 페인트 묶음이 됨(전에는 고른 것만). 마우스가 움직일 때마다 문서의 모든 경로를 복제(`tools/src/builder.rs:606-636, 661, 733-735`, 5만 경로 문서에서 느릴 것으로 추정, 재지 않음). → 사람 결정(아래 7).
- minor: 새 수치 동작에 시험 없음. 높이 1 pt 미만 윤곽은 위쪽 모드여도 아래로 채움. 아래쪽 채움 범위 주석과 실제(0.05~0.55 m)가 다름. 굵기 칩 0.15·0.6이 파선을 지우지 않음(0.4파 다음 0.6을 누르면 파선이 남음, `palette.rs:85`). 이름에 「단면」「해발」이 든 묶음은 자동 윤곽 후보에서 빠짐.
- 확인한 것: 63a624a(한 편집으로 합침) 맞음. 매개변수 이름 추가는 serde 숫자라 유한. `place_symbol_at`은 문서 좌표로 맞음. 글꼴 파일 추가 없음(윤고딕은 상용 글꼴이라 설치된 것을 PDF·SVG에 넣을 때 라이선스는 사용자 몫). 4fce5b7 시험 변경은 기대값을 약하게 하지 않음.

## 재작업 (rework) 1
- major 1: `pathops/src/elevation.rs`에 `MAX_ELEVATION_LINES = 10_000`(10 cm 간격으로 1,000 m). 넘거나 수가 아니면 줄을 만들지 않음. 정상 범위의 결과는 그대로. 시험 `engine/tests/cmd_elevation.rs::huge_intervals_draw_nothing_instead_of_running_out_of_memory`.
- 검증: `cargo test -p vectorcraft-pathops -p vectorcraft-engine --no-fail-fast` + `cargo test -p vectorcraft-ui-egui --no-fail-fast` 종료 0, 3356 통과 0 실패(J021 변경 포함, 로그 `d1-test-2.log`).
- major 2~5는 권 님이 PC에서 만든 동작이라 바꾸지 않고 아래에 결정을 여쭘.

## 사람이 정할 것 (◆ 해발 계산·칠통 동작)
1. 축척이 없는 문서의 아래쪽 모드·평평한 기준선에서 기준 해발을 윤곽 바닥이 아니라 맨 위에 맞춤. 이미 있는 해발선 묶음도 윤곽을 고치면 위치가 바뀜.
2. 높이 1 pt 미만 선은 위쪽 모드여도 기준선 아래로 주간격 2칸(기본 1 m)을 채움.
3. 아래쪽 채움을 바닥 아래로 얼마나 더 그릴지(지금 0.05~0.55 m로 들쭉날쭉).
4. 닫힌 윤곽(구덩이)에서 해발선을 도형 안에 그릴지 바깥 위에 그릴지.
5. 기준선 자동 생성: 고른 것이 없으면 맨 위 층 경로만 찾음, 흩어져 있으면(1.4배 판정) 문서 폭 기준선을 맨 아래에, 빈 캔버스는 화면 가운데 220 pt. 기준선을 해발선 묶음 안에 넣을지.
6. 축척 원점 y가 0일 때 기준 해발 0 m는 축척 원점, 0이 아니면 윤곽에 맞춤(0 m와 0.01 m의 결과가 크게 다름).
7. 칠통: 빗금 기본값이 세션에 남는 것, 고르지 않은 도형도 바로 묶음이 되는 것.

