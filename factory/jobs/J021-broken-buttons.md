상태: 승인 대기

# J021 (디자인 3.0 D1): 눌러도 안 되던 버튼 고치기

## 명세 (spec)
- 기준 커밋: J020 커밋(`0900a12`, PC 변경을 올린 것) 위
- 작업 브랜치: `claude/project-thread-ams9bp`
- 근거: 디자인 3.0 (https://claude.ai/artifact/GmVPKvYKq4ADAe5ZbmZLdc) 2장 「눌러도 안 되는 것」, 11장 D1

### J020 이후 남은 고장 (코드에서 확인)
디자인 3.0이 찾은 고장 5건 중 PC 변경(J020)이 3건을 풀었다.
- 리본 단면선: 이제 팝업이 `trowel.insertSectionMarker`를 `at`과 함께 보냄 → 풀림.
- 리본 표고: 팝업이 `elevation.create`와 해발 기준점을 `at`과 함께 보냄 → 풀림.
- 한 번 눌러 넣기의 고정 좌표: 리본 팝업과 도면집 소스판이 `trowel::place_symbol_at`(화면 가운데 + 14 pt씩 비켜 놓기)을 씀 → 풀림.

남은 것:
1. 도구 열 「표고」가 `{x, y, elevation_m}`만 보내고 `at`을 안 보내 `trowel.spotElevation`이 「missing or invalid `at`」으로 실패 (`trowel/tools.rs:125`).
2. 도구 열 「유물 번호」 `trowel.artifactBadge`, 「주혈 깊이」 `trowel.postholeDepth`는 엔진에 없는 명령 (`trowel/tools.rs:126-127`).
3. 소스판 표준 선 버튼 5개의 이름이 엔진의 12종 이름과 달라(`이중굴광-가운데선` 대 `이중굴광시-가운데선`, `유물-외곽선`, `유물-내선`, `석곽·유물 깨진면`, `해발 0.5단위 해발선`) `legend.addFieldPresets`가 아무것도 안 하고 성공(`{"added": []}`)으로 끝남 (`trowel/drawings.rs`, 엔진 `cmd/legend.rs:771`).

### 완료 기준
1. 도구 열 「표고」를 누르면 화면 가운데 근처에 표고점이 생기고 선택 도구로 바뀐다. 높이는 리본 「표고」 팝업의 「기준 해발」 값(처음 93.0 m)을 쓴다. 확인: `tool_column_has_the_mockup_groups`, `every_one_click_button_draws_without_error`.
2. 도구 열에서 「유물 번호」·「주혈 깊이」가 빠진다(엔진 명령이 생기면 다시 넣는다. 디자인 3.0 D4의 도구 열에도 없다). 확인: `tool_column_has_the_mockup_groups`.
3. 소스판 표준 선 12개 버튼이 모두 엔진 이름과 같고, 누르면 그 줄이 생겨 그릴 줄이 된다. 이미 있으면 그 줄로 바뀐다(버튼 설명 「추가 및 선택」 그대로). 확인: `every_one_click_button_draws_without_error`.
4. `legend.addFieldPresets`에 12종에 없는 이름을 주면 오류를 낸다(조용히 성공하지 않음). 확인: 엔진 `test_trowel_plan_commands`.
5. 소스판·리본 팝업(축척자·단면·표고·방위·범례표)·도구 열의 한 번 넣기 버튼을 하나씩 모두 눌러, 버튼마다 도면에 무언가 생기고 상태줄에 오류가 뜨지 않는다. 확인: `every_one_click_button_draws_without_error`.
6. `cargo fmt --all -- --check`, `cargo clippy -p vectorcraft-engine -p vectorcraft-ui-egui --all-targets -- -D warnings`, `cargo test -p vectorcraft-engine`, `cargo test -p vectorcraft-ui-egui trowel` 통과. GitHub 「Trowel CI」 통과.

### 변경 범위
- `crates/engine/src/cmd/legend.rs`: `add_field_presets` 이름 확인, 이미 있는 줄 고르기
- `crates/engine/src/cmd/trowel_plan.rs`: 시험에 이름 경우 추가
- `crates/ui-egui/src/trowel/{tools,drawings,ribbon,mod,tests}.rs`

### 범위 밖
- 「경사 표시」는 그대로 둔다(위 선을 고르고 누르면 됨, 안 고르면 「give topId or select top boundary curve」 오류를 상태줄에 보임). 디자인 3.0 D4에서 도구 열을 다시 짠다.
- 버튼 모양·자리 정리(D2 이후).
- 쓰지 않게 된 아이콘 `tw-find-number`·`tw-posthole`과 번역 줄은 지우지 않는다(J009에서 다시 씀).

### 위험
- 기존 시험 기대값 변경 1건: `tool_column_has_the_mockup_groups`가 「표고를 눌러도 도구가 그대로」를 확인하던 것을 「표고점이 생기고 선택 도구로」로 바꾼다. 근거: 예전 기대값은 버튼이 고장 나 아무 일도 없던 상태를 적은 것이고, 다른 한 번 넣기 버튼(J020 `place_symbol_at`)과 같이 동작하게 하는 것이 D1의 목표다.
- 도구 열이 두 칸 짧아져 작은 창 시험(`small_windows_keep_every_tile_and_tool_in_reach`)의 「스크롤이 필요한 높이」가 바뀔 수 있다. 바뀌면 계산 근거를 적고 고친다.

## 빌드 (build)
- 2026-10-16. 기준 `e66bf96`(J020 재작업 1) 위, 같은 브랜치.
- 도구 열: 「표고」가 `{at, elevation}`을 보냄(`place_symbol_at`로 화면 가운데 근처, 높이는 `datum_elevation_m` = 리본 표고 팝업의 「기준 해발」, 처음 93.0 m). 「유물 번호」·「주혈 깊이」를 뺌(주혈 깊이는 엔진의 `trowel.depthMarker`로 D4에서 다시 넣음).
- 소스판: 표준 선 이름 5개를 엔진 12종 이름과 같게 고침. 소스판·리본의 해발선 버튼 2쌍도 `at`을 보냄(검토 9번, 빈 도면에서 기준선이 (300, 220) 고정 자리에 생기던 것).
- 엔진 `legend.addFieldPresets`: 12종에 없는 이름, 글자가 아닌 `name`은 오류. 이름으로 부른 줄이 이미 있으면 그 줄을 고름(`picked`). 잠긴 줄이면 고르기 오류를 그대로 냄. 매개변수 설명을 고침.
- 시험: 새 `every_one_click_button_draws_without_error`(소스판 버튼 전부, 표준 선 12개, 리본 팝업 5개의 버튼 전부, 도구 열 표고·경사 표시를 하나씩 눌러 노드가 늘고 상태줄에 오류가 없는지), 엔진 시험에 이름 경우 4개.
- 기대값 변경 3건과 근거
  - `tool_column_has_the_mockup_groups`: 「표고를 눌러도 도구가 그대로」 → 「표고점이 생기고 선택 도구로」. 예전 기대값은 고장 난 상태를 적은 것.
  - 아이콘 수 하한 37 → 35: 도구 열에서 아이콘 2개를 안 씀(검토자가 35개를 셈).
  - `small_windows_keep_every_tile_and_tool_in_reach`: 스크롤이 필요한 창 높이 728 기준. 1366×728에서 도구 열 500 − 칠·선 72 = 428, 필요한 높이 418(예전 464)이라 이제 다 들어감. 계산을 시험 주석에 적음.

## 검증 (verify)
모든 명령은 `CARGO_TARGET_DIR=/home/claude/target-trowel CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`(CI와 같음). 로그: 스크래치패드 `factory/d1-test.log`, `d1-test-2.log`, `d1-clippy.log`.

| 기준 | 명령 | 결과 |
|---|---|---|
| 1~5 | `cargo test -p vectorcraft-ui-egui trowel` (첫 실행) | **실패** 2개: 표준 선 키 순서를 잘못 가정한 새 시험, 작은 창 시험 높이(위 기대값 변경). 시험을 고침 |
| 1~6 | `cargo test -p vectorcraft-pathops -p vectorcraft-engine -p vectorcraft-ui-egui --no-fail-fast` | 종료 0, 3356 통과 0 실패 |
| 6 | `cargo fmt --all -- --check` | 종료 0 |
| 6 | `cargo clippy -p vectorcraft-pathops -p vectorcraft-engine -p vectorcraft-ui-egui --all-targets -- -D warnings` | 종료 0 |
| 6 | GitHub 「Trowel CI」 | 올린 뒤 확인 (PR #1에 적음) |

## 검토 (review) 1차
- 검토자: 독립 검토 에이전트, 읽기만. 범위: 작업 트리의 `crates/` 변경. 판정: **통과**(blocker·major 없음).
- 반영: minor 1(시험이 고른 줄의 이름과 잠긴 줄 경우를 확인), 2(기준 해발 95.5를 넣고 표고 이름에 95.50이 있는지 확인), 3(글자가 아닌 `name`은 오류), 4(매개변수 설명), nit 6(주혈 깊이 설명), 7(주석 범위), 8(높이 계산 주석), 참고 9(해발선 버튼도 `at`).
- 그대로 둔 것: nit 5 오류 머리말이 `legend.pick`으로 나오는 것. 오류는 상태줄에 보이고 뜻이 같아 고치지 않음.

## 사람이 볼 것
- 화면 모양은 바뀌지 않음(도구 열에서 버튼 2개가 빠진 것만). 디자인 3.0 시안과 나란히 보는 화면 비교는 D2부터.
- 남은 위험: 소스판의 다른 해발 기준점 버튼(dh/dv)과 리본 단면 팝업의 기준점 버튼은 아직 93.0 m 고정. D5 요소함에서 한 곳으로 모음.
- 되돌리기: `git revert <J021 커밋>`.
