상태: verify

# J024 (디자인 3.0 D4): 도구 열 고고학 4칸

## 명세 (spec)
- 기준 커밋: `f1959cb` (J023 승인 대기 기록, 코드는 `a10f9f3`)
- 작업 브랜치: `claude/project-thread-ams9bp`
- 근거: 디자인 3.0 (https://claude.ai/artifact/GmVPKvYKq4ADAe5ZbmZLdc) 4장 시안 `.a-tools`, 5장 ④ 도구 열 표, 6장 「단면 표시」 「해발 기준점」 「토층」, 8장 5·7·8번, 9장 펜 막대, 10장 ④ 줄, 11장 D4.
- 정해진 것(사용자 2026-10-16 「진행시켜」, Q2 추천 답): 해발선 굵기는 편집 파일 값(해발선 0.3, 0.5 m 단위 0.5).

### 완료 기준
1. 도구 열은 시안과 같다: 너비 50, 칸 38×32, 칸 사이 2, 아이콘 18(단면선 20), 칸 오른쪽 아래 단축키 9 mono.
   - 순서: V A | P N \ M T | R S C K I | 「고고학」 1 2 3 4 | H Z. 가름줄은 28×1, 위아래 4.
   - 맨 아래 칠/선 상자와 「없애기」를 없앤다.
   - 길게 누르기·오른쪽 클릭의 펼침은 그대로다. 창이 낮으면 지금처럼 칸이 줄고, 그래도 모자라면 스크롤한다.
   - 확인: `tool_column_has_the_mockup_groups`, `small_windows_keep_every_tile_and_tool_in_reach`, `regions_match_the_mockup_at_1600_by_988`.
2. 단면선(1, 도구 `sectionCut`): 두 점을 누르면(또는 끌면) 두 끝에 A–A′ 표시가 생긴다.
   - 표시는 빨간 막대, 시선 화살표, 글자이고, 두 점을 잇는 방향으로 놓인다(비스듬한 선도).
   - 글자는 이 도면에서 아직 쓰지 않은 다음 글자다(A → B → C).
   - 움직이는 동안 미리 보기(점선과 「B–B′」 글자)가 보인다. Shift는 45° 단위, Esc는 그만두기다.
   - 되돌리기 한 번으로 없어진다.
   - 펼침에 「경사 표시」(도구 `slopeHachure`)가 있다. 선을 누르면 그 선을 따라 경사 표시를 넣는다(`trowel.slopeHachures`).
   - 확인: 엔진 `section_marker_between_two_points_takes_the_next_letter`, 도구 `section_cut_*`, 화면 `the_archaeology_tools_draw_the_eighth_chapter`.
3. 해발선(2, 도구 `levelLines`): 지표선 위에 포인터를 두면 그 선이 강조된다. 누르면 「기준 높이」 칸이 뜬다.
   - 높이를 넣고 Enter(또는 [넣기])를 누르면 누른 자리의 높이를 그 값으로 하여 해발선을 그린다.
   - 선은 10 cm마다, 50 cm마다 굵게(0.3 / 0.5 pt, Q2) 그리고, 지표선에서 멈춘다(`elevation.create`, 엔진 기본 간격 그대로).
   - 그 높이의 해발 기준점 「┣ 32.5m」를 지표선 오른쪽 끝에 함께 붙인다.
   - 되돌리기 한 번으로 둘 다 없어진다.
   - 빈 곳을 누르면 상태줄에 「지표선을 누르세요」가 뜬다.
   - 좌표계 원점이 정해진 도면(TM 좌표가 붙은 평면)에서는 그리지 않고 알린다. 지금 엔진은 그런 도면에서 넣은 높이를 쓰지 않고 북쪽 좌표를 높이로 쓰기 때문이다.
   - 확인: 엔진 `level_lines_put_the_clicked_point_at_the_height`, 도구 `level_lines_*`, 화면 시험.
4. 표고(3, 도구 `spotHeight`): 누르면 「높이」 칸이 뜬다. 넣으면 누른 자리에 ▽(꼭짓점이 그 점)와 「32.5m」가 생긴다.
   - 소수 자리: 0.1 m 단위면 한 자리(32.5m), 아니면 두 자리(32.45m). 해발선 글자와 같은 규칙이다.
   - 칸에는 마지막에 넣은 높이가 미리 들어 있다. 요소함(소스판)의 기준 해발과 같은 값을 쓴다.
   - 확인: 엔진 `spot_elevation_down_triangle_and_short_label`, 화면 시험.
5. 토층선(4, 도구 `strataLine`): 층 경계를 그린다.
   - 누를 때마다 점이 더해지고, 누른 채 끌면 손그림 점이 더해진다.
   - 끝내기: Enter, 두 번 누르기, 펜 막대 「선 끝내기」. Backspace는 마지막 점 지우기, Esc는 그만두기다.
   - 끝내면 한 번에 다음이 생긴다(되돌리기 한 번).
     - 지금 펜 모양(고른 범례 줄)의 선.
     - 선 가운데 위에 다음 층 번호(Ⅰ, Ⅱ …, 6 pt).
     - 토층 표의 새 줄.
     - 도면 아래 7 pt 토층 설명 칸의 새 줄 「Ⅰ층 : 」. 칸이 없으면 첫 선 아래에 만든다. 이미 친 설명 줄은 건드리지 않는다.
   - 설명은 그 줄을 두 번 눌러 바로 친다(지금 글자 고치기).
   - 확인: 엔진 `strata_boundary_numbers_and_adds_a_description_line`, 도구 `strata_line_*`, 화면 시험.
6. 숫자 키 1~4는 Trowel 창에서 네 도구를 고른다. 클래식 창에서는 숫자 키가 전처럼 아무 도구도 고르지 않는다.
   - 펜 막대는 펜·연필에 더해 단면선·토층선을 쓸 때도 뜬다. 예전 고고학 이름(`section-line`, `spot-height` 등)은 없앤다.
   - 확인: `digits_pick_the_archaeology_tools_in_trowel_only`, `test_canvas_penbar_adversarial_suite`.
7. 8장 5·7·8번이 끊기지 않는다: 단면선으로 A–A′ → 단면선 줄로 지표선 그리기 → 해발선으로 지표선을 누르고 32.5 → 토층선으로 경계 둘(Ⅰ, Ⅱ와 설명 줄 둘).
   - 확인 1: 화면 시험 `the_archaeology_tools_draw_the_eighth_chapter`(합성 포인터).
   - 확인 2: 새로 띄운 앱을 control 채널로 같은 순서로 눌러 캡처.
8. 새로 띄운 앱 화면의 도구 열을 4장 시안과 나란히 놓고 비교한 결과와 남은 차이 목록을 남긴다. 차이가 남아 있으면 「통과」라고 하지 않는다.
9. `cargo fmt --all -- --check`, `cargo clippy`(tools, engine, ui-egui, pathops, doc) `-D warnings`, 그 crate들의 시험, GitHub 「Trowel CI」(`cargo xtask ci`, wasm 포함) 통과.

### 변경 범위
- `crates/tools/src/trowel.rs`(새): 도구 다섯 개. `lib.rs`(만들기), `catalog.rs`(Trowel 도구 목록. 클래식 도구 막대 묶음에는 넣지 않음).
- `crates/engine/src/cmd/trowel_plan.rs`
  - `trowel.insertSectionMarker`에 `p0`·`p1`(두 점, 비스듬한 방향).
  - `label`을 주지 않으면 다음 글자. 지금처럼 `at`과 `label`을 주는 호출은 그대로다.
  - `trowel.spotElevation`에 `marker: 'triangleDown'`과 `format: 'auto'`.
  - 새 `trowel.levelLines`(해발선 + 기준점, 한 단계).
- `crates/engine/src/cmd/stratigraphy.rs`: 새 `stratigraphy.boundary`(선 + 번호 + 표 줄 + 설명 줄, 한 단계).
- `crates/pathops/src/plan.rs`: ▽ 표시 모양과 자동 소수 자리.
- `crates/doc/src`: 도면에서 쓰인 단면 글자를 찾아 다음 글자를 주는 함수(도구 미리 보기와 엔진이 함께 씀).
- `crates/ui-egui/src/trowel/{tools,mod,canvas_penbar,tests}.rs`, 새 `trowel/ask.rs`(높이 칸), `dialogs/tools.rs`(도구가 여는 칸), `shortcuts.rs`(숫자 키는 Trowel 창만).
- 아이콘: `assets/icons/tw-level.svg`(새), `tw-section-line.svg`·`tw-spot-height.svg`·`tw-strata.svg`를 시안 그림(디자인 3.0에서 그린 원작)으로 바꿈. `icon_data.rs`, `ASSETS.md`.
- `i18n/*.tsv`: 새 글자열 번역.

### 범위 밖
- 요소함(D5): 소스판의 단면 표시·해발 기준점·해발선·토층 번호 버튼은 그대로 둔다.
- 다음 글자·번호를 요소함 요소에도 주는 것, 평면의 A–A′와 단면도의 A–A′를 짝으로 잇는 것(D6).
- 해발선 간격·범위·자르기 규칙(D0 사람 검토 ◆ 그대로). 해발 기준점이 해발선 높이와 다르면 「!」(D6).
- 왼쪽 서랍 너비(236, D5). 도구 열이 56에서 50으로 줄어 서랍이 6 왼쪽으로 온다.

### 위험 (사람 검토)
- ◆ 해발 계산: 「누른 자리의 높이 = 넣은 값」으로 정했다(디자인 글 「지표선을 클릭하고 기준 높이를 넣으면」을 이렇게 읽음).
  - 엔진은 그 값을 지표선 가장 낮은 점의 높이로 바꿔 저장한다(`datumM` = 넣은 값 − (가장 낮은 점 y − 누른 y) ÷ 1 m의 pt 수). 축척이 없으면 엔진 기본 1:20을 쓴다.
  - 다른 뜻(예: 기준선을 따로 긋기)이면 사용자 결정.
- ◆ 표고 글자의 소수 자리를 「0.1 m 단위면 한 자리」로 바꾼다. 새 `format: 'auto'`를 줄 때만이고 기존 호출은 두 자리 그대로다.
- 단면 표시를 두 점 사이 아무 방향으로 놓는다. 가로·세로만 되던 기존 모양(`at`, `orientation`)은 그대로다. 두 점 사이를 잇는 선은 넣지 않는다(디자인 글이 「빨간 막대, 시선 화살표, 글자」만 적음). 넣을지는 사용자 결정.
- 토층 번호는 원 없이 글자만(6 pt)으로, 경계선 가운데 위에 둔다. 처음 만드는 토층 표의 번호 방식은 로마 숫자로 둔다.
- 기존 시험 기대값 변경: 도구 열 칸 크기(40×27 → 38×32)·너비(56 → 50)·칠/선 상자, 고고학 버튼 두 개(`arch`)를 확인하던 시험을 디자인 3.0에 맞게 바꾼다(근거: 승인된 디자인 3.0 4장 시안, 5장 ④).

## 빌드 (build) — 2026-10-16, 기준 `f1959cb`
- 손댄 파일
  - 도구: `crates/tools/src/trowel.rs`(새, 도구 다섯 개와 시험 9개), `catalog.rs`(`TROWEL_TOOLS`, `trowel_tools`, `is_trowel_tool`, `trowel_tool_for_shortcut`, `tool_info`가 Trowel 도구도 찾음), `lib.rs`(만들기).
  - 문서: `crates/doc/src/section_marks.rs`(새, 쓰인 단면 글자와 다음 글자), `lib.rs`.
  - 엔진: `cmd/trowel_plan.rs`(`trowel.insertSectionMarker`의 `p0`·`p1`과 다음 글자, `trowel.spotElevation`의 `triangleDown`·`auto`, 새 `trowel.levelLines`), `cmd/stratigraphy.rs`(새 `stratigraphy.boundary`), `cmd/library.rs`(`in_one_step`을 crate 안에서 씀), `tests_archaeology.rs`(새, 엔진 시험 4개), `lib.rs`.
  - 경로 계산: `crates/pathops/src/plan.rs`(▽ 모양 `TriangleDown`, `short_label`).
  - 화면: `trowel/tools.rs`(도구 열을 시안 묶음대로 다시 씀), `trowel/ask.rs`(새, 높이 칸), `trowel/mod.rs`(도구 열 너비 50, 높이 칸 그리기), `trowel/canvas_penbar.rs`(뜨는 도구, 「되돌리기」), `trowel/scale_dialog.rs`(버튼 함수를 함께 씀), `trowel/tests.rs`, `dialogs/tools.rs`(도구가 여는 칸), `shortcuts.rs`·`shortcut_editor.rs`(숫자 키), `state.rs`(`trowel_ask`, 저장 안 함), `theme.rs`(`section_mark` 토큰), `canvas.rs`(펜 옆 버튼이 그리기 도구로 보는 목록), `icon_data.rs`, `i18n/ko.tsv`(18줄).
  - 아이콘: `assets/icons/tw-level.svg`·`tw-strata-line.svg`(새), `tw-section-line.svg`·`tw-spot-height.svg`(시안 그림으로 다시 그림), `ASSETS.md`.
- 완료 기준별
  1. 도구 열: 묶음 다섯(V A | P N \ M T | R S C K I | 「고고학」 1 2 3 4 | H Z), 칸 38×32, 사이 2, 위아래 여백 8, 가름줄 1 px 위아래 4, 단축키 mono 9. 칠/선 상자와 「없애기」 없앰. 창이 낮으면 칸 22, 그래도 모자라면 스크롤.
  2. 단면선: 두 번 누르기 또는 끌기. 미리 보기는 점선, 두 끝 점, 「B–B′」 글자. Shift 45°, Esc 그만두기. 엔진이 두 점 방향으로 표시를 놓고 다음 글자를 줌. 펼침의 「경사 표시」는 누른 선으로 `trowel.slopeHachures`.
  3. 해발선: 포인터 밑 선을 2 px 강조, 누르면 「기준 높이」 칸, Enter·[넣기]로 `trowel.levelLines`(해발선과 「단면 해발 눈금 (32.5m)」을 한 단계로). 빈 곳은 상태줄 「지표선을 누르세요」. TM 원점이 있는 도면은 거절.
  4. 표고: 「높이」 칸(마지막 높이가 들어 있음) → ▽ + 「32.5m」(`format: 'auto'`).
  5. 토층선: 누르기·끌기로 점, Enter·마지막 점 다시 누르기·펜 막대 「선 끝내기」로 끝냄, Backspace·펜 막대 「되돌리기」는 마지막 점 지우기, Esc 그만두기. `stratigraphy.boundary`가 선·층 번호(6 pt)·토층 표 줄·설명 줄 「Ⅰ층 : 」을 한 단계로.
  6. 숫자 키 1–4는 Trowel 창에서만. 펜 막대는 단면선·토층선에서도 뜸. 예전 고고학 도구 이름은 펜 막대·펜 옆 버튼 목록에서 없앰.
- 명세에서 벗어난 점과 이유
  - 「두 번 누르기」로 끝내기는 「마지막 점을 다시 누르기(누르고 끌지 않고 놓기)」로 했다. egui는 두 번 누르기를 위치와 상관없이 세고, 빠른 세 번째 누르기는 두 번 누르기가 아니라서, 점 두 개를 빨리 찍으면 선이 끝나 버렸다. 시험 `strata_line_double_click_finishes_and_esc_stops`가 빠른 두 점은 끝내지 않는 것도 확인한다.
  - `tw-strata.svg`는 바꾸지 않고 `tw-strata-line.svg`를 새로 그렸다. 리본 「Strata 조사 자료…」가 `tw-strata`를 쓰기 때문이다.
  - 빈 곳을 누르면 명령을 실패시키지 않고 상태줄 안내를 띄운다(`HINT`).
  - 높이 칸은 누른 것이 끝난(포인터를 뗀) 다음 프레임에 글자를 받는다. egui는 같은 프레임의 누르기에 칸의 글자 받기를 빼앗기 때문이다.
- 정한 것(기록, 사용자 확인 대상)
  - 토층선을 그리는 중에 다른 도구로 바꾸면(Z 돋보기 포함) 그때까지의 선으로 끝낸다(펜과 같음).
  - 캔버스의 펜 옆 버튼(되돌리기)은 그리는 중에도 `edit.undo`를 한다. 펜 막대 「되돌리기」만 마지막 점을 지운다.
  - 도구 열이 56 → 50이 되어 캔버스와 서랍이 6 왼쪽으로 온다(서랍 너비는 D5).
  - 영어 화면에서는 「Archaeology」가 너비 50에 들어가게 글자를 줄인다(최소 7).
  - 단면 표시 빨간색은 새 theme 토큰 `section_mark`(#E8334F, 시안 색)로 그린다.

## 검증 (verify, 클라우드 컨테이너, Linux) — 2026-10-16
로그는 scratchpad `j024/`(git에 넣지 않음).

| 명령 | 종료 코드 | 결과 | 기록 |
|---|---|---|---|
| `cargo fmt --all -- --check` | 0 | 통과 | `j024/fmt.log` |
| `cargo clippy -p vectorcraft-tools -p vectorcraft-engine -p vectorcraft-ui-egui -p vectorcraft-pathops -p vectorcraft-doc --all-targets -- -D warnings` (1회) | 101 | 실패 1: `trowel_plan.rs` `!(bb.width() >= 1e-4)`(neg_cmp_op_on_partial_ord) → `!is_finite() \|\| < 1e-4`로 고침 | `j024/clippy.log` |
| 같은 명령 (2회) | 101 | 실패 1: 시험 `tests_archaeology.rs` 겹친 `if let`(collapsible_if) → 하나로 합침 | `j024/clippy2.log` |
| 같은 명령 (3회) | 0 | 경고 없음 | `j024/clippy3.log` |
| `cargo test -p vectorcraft-tools -p vectorcraft-doc -p vectorcraft-pathops` | 0 | 통과(도구 299 포함), 실패 0 | `j024/test-small.log` |
| `cargo test -p vectorcraft-engine --lib` | 0 | 1,700 통과 · 0 실패 | `j024/test-engine.log` |
| `cargo test -p vectorcraft-engine --test trowel_e2e --test command_sweep --test plan_adversarial --test cmd_stratigraphy` | 0 | 225 + 15 + 14 + 6 통과 · 0 실패 | `j024/test-engine-int.log` |
| `cargo test -p vectorcraft-ui-egui` | 0 | 1,060 + 6 통과 · 0 실패 | `j024/test-ui.log` |
| `cargo test -p vectorcraft-engine --lib tests_archaeology` (린트 고침 뒤) | 0 | 4 통과 | `j024/test-arch.log` |
| `cargo xtask assets` / `brands` / `layers` | 0 / 0 / 0 | 자산 359개 모두 출처 있음 · 다른 회사 이름 없음 · 계층 위반 없음 | `j024/xtask-*.log` |
| `twmissing.py` / `twunused.py` | 0 | 글자열 254개 중 번역 빠짐 0 / 안 쓰는 줄 7개(D4 전부터) | 터미널 |
| GitHub 「Trowel CI」(`cargo xtask ci`, wasm 포함) | - | 커밋 push 뒤 확인 | - |

- 처음 clippy 실패 두 건은 이번 변경에서 생긴 것이고 고쳤다. 기존 실패는 없었다.
- 린트 고침 두 건 뒤에 다시 돌린 것은 clippy 전체와 엔진 고고학 시험 4개다. 화면 시험 1,060은 고치기 전 코드에서 돌렸다(고친 두 곳은 엔진 안이고 동작이 같다: 너비 비교의 NaN 처리, 시험 안의 if 합치기).

## 검증 중 찾아 고친 것 — 2026-10-16
- GitHub 「Trowel CI」 run 38937920345(커밋 `8dff460`): **실패**. `cargo clippy --workspace --all-targets`에서 `apps/vectorcraft`(시험 빌드)가 `UiState { .., ..Default::default() }`로 만드는데 새 필드 `pub(crate) trowel_ask` 때문에 E0451(`window.rs:247`, `main.rs:691`). 로컬 clippy를 손댄 crate만 돌려서 놓쳤다. 이번 변경으로 생긴 실패다. → 높이 칸 상태를 `UiState`에서 빼서 `VectorcraftApp::trowel_ask`로 옮김. 윈도우 빌드는 통과.
- 새로 띄운 앱에서 높이 칸이 약 600 px로 넓게 그려짐(egui Area가 화면 폭만큼 열림). → 안쪽 폭 210으로 고정(`ask.rs` `INNER_W`). 시험: 넣기 버튼 오른쪽 − 입력 칸 왼쪽 ≤ 212.
- 새로 띄운 앱의 상태줄이 다섯 도구에서 「누르기 Ctrl+Shift+/ 모든 명령 검색」만 보임(도구 안내 없음). → `chrome.rs` `hint_for`에 다섯 도구 안내, `ko.tsv` 11줄. 다른 완전 언어(ja 등)에는 요구하지 않는다(클래식 도구 목록 `hint_strings`에 넣지 않음, 한국어만 새 시험 `the_archaeology_tools_have_korean_hints`). 시험: `digits_pick_the_archaeology_tools_in_trowel_only`가 안내가 「Click …」으로 시작하는지 확인.

| 명령 | 종료 코드 | 결과 | 기록 |
|---|---|---|---|
| `cargo fmt --all -- --check` | 0 | 통과 | `j024/fmt2.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 경고 없음(앱 crate 포함) | `j024/clippy-ws.log` |
| `cargo test -p vectorcraft-ui-egui` | 0 | 1,061 + 6 통과 · 0 실패 | `j024/test-ui2.log` |
| `cargo test -p vectorcraft` | 0 | 66 통과 | `j024/test-app.log` |
| GitHub 「Trowel CI」 | - | 이 커밋 push 뒤 확인 | - |
