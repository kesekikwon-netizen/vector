상태: 승인 대기

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
| GitHub 「Trowel CI」 run 38937920345 (커밋 `8dff460`, `cargo xtask ci`, wasm 포함) | - | **실패**: 앱 crate E0451(아래 절에서 고침) | GitHub Actions |

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
| GitHub 「Trowel CI」 run 38941924987 (커밋 `0bd0e41`) | - | **성공**(Linux `cargo xtask ci` + Windows 빌드) | GitHub Actions |

### 완료 기준 7 확인 2 · 기준 8 (새로 띄운 앱, `0bd0e41` 릴리스 빌드)
- 방법: 새 HOME으로 앱을 띄우고(1440×880, 한국어) control 채널로 키·누르기만 보냈다(`ui.key`/`ui.click`/`ui.text`, 지표선 하나만 `path.create`). 스크립트 scratchpad `shots/scene_d4.py`, 결과 `shots/d4r0/`.
- 결과(`d4-scene.json`)
  - 키 1/2/4/3 → `sectionCut`/`levelLines`/`strataLine`/`spotHeight`.
  - 빈 곳 누르기 → 상태줄 「지표선을 누르세요」.
  - 생긴 것: 「단면 기호 A–A′」(화살표·붉은 막대·A/A′), 해발선 묶음과 「단면 해발 눈금 (32.5m)」, 「토층선 Ⅰ」·「토층선 Ⅱ」와 「층위 번호 Ⅰ/Ⅱ」, 「토층설명 7p」 칸 안 「토층설명 Ⅰ/Ⅱ」, 「표고 33.2m」.
  - 높이 칸은 Enter로 닫힘. 표고 칸에는 소스판 기준 해발(93.0)이 미리 들어 있었다(화면 `d4-spot-ask.png`).
- 기준 8 비교(`shots/d4r0/d4-compare-tools.png`, 앱 도구 열과 4장 시안 `.a-tools` 2배)
  - 가름줄 y: 앱 80/261/442/606, 시안 81/262/443/607(시안 맨 위 1 px 테두리 때문).
  - 칸 위치·크기는 1–3 px 안. 「고고학」 글자 같음.
  - **남은 차이**: (1) 클래식 도구 아이콘(V A P N \ M T R S C K I H Z)은 VectorCraft 그림이라 시안 그림과 다르다(D4 범위 밖). (2) 앱 아이콘 선이 조금 굵어 보인다. 그래서 「통과」가 아니라 「차이 2건 남음」이다.

## 검토 (review) — 2026-10-16
### 코드 검토 (code-reviewer, 대상 `f1959cb..0bd0e41`) — 판정 **수정 필요**
기록 scratchpad `review/j024-code.md`, 재현 `review/j024-repro/`.

| # | 지적 | 무게 | 처리 |
|---|---|---|---|
| 0 | `8dff460`이 앱 crate 빌드를 깸(E0451) | 막는 것 | `0bd0e41`에서 고침(검토자 확인) |
| 1 | 토층선이 점을 가진 동안 `busy()`라서 키보드 Backspace/Delete/Enter/Esc가 도구에 안 감 | 막는 것 | 재작업 1 |
| 2 | `stratigraphy.boundary`가 격자 표·9 pt 인라인 칸에도 7 pt 줄을 끼워 넣음 | 고칠 것 | 재작업 1 |
| 3 | 로마 숫자가 Ⅹ 뒤 「11」 | 고칠 것 | 재작업 1 |
| 4 | 해발선 도구가 해발선·표시도 지표선으로 집음 | 고칠 것 | 재작업 1 |
| 5 | 기준 7(확인 2)·8·9 증거 없음 | 고칠 것 | 위 절에 기록 |
| 6 | 해발 기준점 글자 형식이 모든 호출자에서 바뀜 | 사소 | 재작업 1(`format:'auto'`일 때만) |
| 7 | 좌표 인자 유한성·범위 검사 없음 | 사소 | 재작업 1 |
| 8 | 단면선 최소 간격 도구 4 px / 엔진 1 pt | 사소 | 재작업 1 |
| 9 | 두 번 누르기 판정 3 px | 사소 | 재작업 1(6 px) |
| 10 | TM 도면 거절을 높이 입력 뒤 영어로 알림 | 사소 | 재작업 1 |
| 11 | 미리 보기 매 프레임 문서 전체 훑기 | 사소 | 재작업 1 |
| 12 | 느슨한·빈 단언 | 사소 | 재작업 1 |

- 사고: 검토자가 재현 시험을 같은 작업 트리의 `trowel/tests.rs`에 잠깐 붙였다가 지웠다. `0bd0e41`에 `zz_review`가 없고 구현자 편집이 남아 있는 것을 구현자가 확인했다(diff 대조). 앞으로 검토자는 작업 트리를 건드리지 않는다.

### 화면 검토 (ui-reviewer, 새로 띄운 `0bd0e41` 릴리스 앱) — 판정 **수정 필요**
기록 scratchpad `review/j024-ui.md`, 그림 `review/j024-ui/out/`. 도구 열은 시안과 칸·가름줄·글자 자리 0 px 차이(단축키 글자만 2–2.5 px 위). 8장 5·7·8번은 끊기지 않음.

| # | 지적 | 무게 | 처리 |
|---|---|---|---|
| 1 | 표고 글자가 안 보임(글자 위에 흰 테두리 2.5 pt) | 막는 것 | 재작업 1 |
| 2 | 토층선 Backspace가 점을 안 지움 | 막는 것 | 재작업 1(코드 검토 1과 같음) |
| 3 | 펜 막대 「닫기」가 단면선·토층선에서 오류만 냄 | 고칠 것 | 재작업 1 |
| 4 | 설명 줄 두 번 누르기가 묶음 격리만 함 | 고칠 것 | 재작업 1 |
| 5 | 해발선이 지금 고른 범례 줄에 들어가 「따로 16」 | 고칠 것 | **사람 결정 H9** |
| 6 | TM 도면 거절이 반 영어, 입력 뒤에야 | 고칠 것 | 재작업 1(코드 검토 10과 같음) |
| 7 | 「지표선을 누르세요」가 도구를 바꿔도 남음 | 고칠 것 | 재작업 1 |
| 8 | 단축키 글자 2–2.5 px 위, 색이 진함 | 사소 | 재작업 1 |
| 9 | 고고학 밖 칸 아이콘이 시안 그림과 다름 | 사소 | 남은 차이(D4 범위 밖) |
| 10 | 펼침의 단면선 아이콘에 붉은 표시 없음 | 사소 | 재작업 1 |
| 11 | 창이 조금 낮아도 칸이 32 → 22 | 사소 | 재작업 1 |
| 12 | 한국어 안내 문구 어색, 풍선의 ▽가 작게 보임 | 사소 | 재작업 1 |
| 13 | 해발선을 다시 누르면 해발선이 잡힘 | 사소 | 재작업 1(코드 검토 4와 같음) |
| 14 | A–A′가 지금 줄에 들어가 「따로 1」, [범례대로]가 화살표 굵기를 바꿈(J024 전부터) | 사소 | **사람 결정 H9** |
| 15 | 단면선 첫 점 뒤 「선 끝내기」가 아무 일 안 함 | 사소 | 재작업 1 |
| 16 | 토층선 그리는 중 숫자·V 키로 도구가 안 바뀜 | 사소 | 재작업 1(busy 뜻 바꿈) |
| 17 | `d4-scene.json`의 높이 칸 값이 확인 불가(`ui.inspect`에 없음) | 증거 | 재작업 1(`trowelAsk`), 다시 찍음 |

## 재작업 1 (rework) — 2026-10-16, 기준 `0bd0e41`
- 1 키보드: 단면선·토층선의 `busy()`를 「단추를 누르고 있는 동안」으로 바꿨다(펜과 같은 뜻). 점 사이 대기는 `claims_key`로 나타낸다. 그래서 `shortcuts.rs`의 `if busy { … return; }`에 키가 막히지 않고, 점 사이에는 한 글자 단축키(1–4, Z)도 된다.
  - 시험 기대값 변경: 도구 시험이 「점이 있으면 `busy()`」를 보던 곳을 `claims_key`로 바꿨다. 근거: 검토 1(실제 키 경로에서 Backspace가 버려짐)과 `Tool::busy`의 뜻(누르고 끄는 중). 새 화면 시험 `the_keyboard_reaches_the_strata_line_while_it_is_drawn`(세 점 → Backspace → Enter면 점 2개, Esc는 아무것도 안 만듦, 점 사이 Z는 돋보기로 바꾸고 그때까지의 선을 남김).
- 2 설명 칸: 토층 표가 인라인일 때만 그 칸을 쓰고, 크기는 표의 `font_size_pt`(2–72, 아니면 7), 간격은 `max(크기×1.65, 11.55)`, 다음 줄은 마지막 글자 줄 아래에 둔다. 격자 표가 있으면 표 배치는 그대로 두고 「토층설명 7p」 칸을 찾거나 새로 만든다. 시험 `strata_boundaries_past_ten_and_with_a_table_already_there`.
- 3 번호: 10을 넘으면 `XI`, `XII` …(`stratigraphy.number`와 같은 `to_ascii_roman`). 같은 시험.
- 4 표시 건너뛰기: `vectorcraft_doc::section_marks::is_mark`(해발선 묶음, 단면 기호, 해발 눈금, 표고, 토층선 번호·설명, 축척 막대, 방위표, 비탈면 빗금)를 도구의 선 집기(`hit_test_skipping`, 이번에 `pub`)가 건너뛰고, 엔진 `trowel.levelLines`도 그런 `profileId`를 거절한다. 시험 `marks_are_not_lines_to_click`, `level_lines_refuse_marks_and_points_off_the_canvas`, `marks_are_told_from_drawn_lines`.
- 6 기준점 글자: `trowel.insertSectionMarker`에 `format?: 'fixed'|'auto'`를 더했다. 기본은 예전처럼 소수 한 자리(「32.5m」), `trowel.levelLines`만 `auto`를 준다. 요소함의 기준점 버튼은 예전 그대로다. 시험 `datum_ticks_keep_one_decimal_unless_asked`.
- 7 범위: `p0`·`p1`, `at`, `points`가 유한하고 `MAX_COORD` 안인지 바로 보고, 아니면 「off the canvas」 오류. 같은 시험.
- 8 최소 간격: 도구가 `max(화면 4 px, 1 pt)`. 시험 `section_cut_points_are_a_point_apart_on_paper`(800 %에서 0.6 pt는 기다리고 1 pt는 놓음).
- 9 끝내기 거리: 마지막 점에서 6 px(`FINISH_PX`). 도구 시험이 5 px 어긋난 두 번째 누르기로 끝나는지 본다.
- 10 TM 도면: 높이 칸을 열기 전에(`ask::open`) 도면 원점의 북쪽 좌표를 보고, 한국어 문구로 상태줄에 알린다(`ko.tsv` 1줄). 시험 `level_lines_on_a_map_drawing_are_refused_at_the_click`.
- 11 미리 보기: 다음 글자를 첫 점을 놓을 때 한 번 찾아 둔다.
- 12 단언: 표고 이름은 정확히 「표고 95.5m」, 경사 표시는 누른 선의 가로 범위 안.

### 재작업 1 (화면 검토 반영) — 같은 날, 기준 `5943506`
- UI 1 표고 글자: 흰 테두리를 글자 자체의 선이 아니라 글자 아래에 칠하는 모양 항목으로 옮겼다(`trowel.spotElevation`, `contents_index` 1, 둥근 모서리). 요소함의 표고 버튼도 같은 명령이라 함께 보이게 된다. 시험: 엔진 `spot_elevation_down_triangle_and_short_label`이 4배로 그린 그림에서 글자 자리의 검은 점을 센다(고치기 전 0, 뒤 40 넘음).
  - 같은 「글자 위 흰 선」이 요소함의 다른 버튼(평면 단면 기호 글자 10 pt, 주혈 깊이, 토층 번호 원)에도 있다. D4 길에는 없어서 이번에 고치지 않고 남은 위험으로 적는다(D5).
- UI 3·15 펜 막대: 토층선은 [되돌리기][선 끝내기], 단면선은 [되돌리기][그만두기 Esc]만 보인다. 단면선은 Backspace·[되돌리기]·[그만두기]가 첫 점을 지운다. 시험 `the_pen_bar_has_what_each_tool_can_do`, 도구 시험 `section_cut_waits_…`.
- UI 4 설명 줄: 선택 도구로 「토층설명 7p」 묶음의 글자 줄을 두 번 누르면 격리하지 않고 바로 문자 도구로 그 줄을 친다(`section_marks::is_note_block`, 엔진과 같은 이름 상수). 다른 묶음은 예전처럼 격리한다. 시험 `a_description_line_takes_typing_at_a_double_click`.
- UI 7 안내: 상태줄 메시지는 그 메시지가 나온 도구일 때만 보인다(오류는 5초 그대로). 높이 칸이 열리면 앞의 「지표선을 누르세요」를 지운다. 시험 `the_level_lines_hint_goes_with_the_tool`.
- UI 8 단축키 글자: 2 px 아래로, 색은 새 토큰 `tile_key` #9A988F / `tile_key_on` #CFCFC8(Quiet Plate, 시안 값). 다른 테마는 예전 색(`text_disabled`, `icon_on`).
- UI 10 펼침: 펼침 목록과 떼어 낸 도구 띠에서도 단면선 아이콘에 붉은 표시를 그린다.
- UI 11 칸 높이: 창이 낮으면 칸을 필요한 만큼만 줄인다(정수 pt, 22–32). 시험 기대값 변경: `small_windows_keep_every_tile_and_tool_in_reach`가 「900 미만이면 22」 대신 「(열 높이 − 111) ÷ 18의 내림, 22–32」(1366×728은 24)를 본다. 근거: 화면 검토 11과 시안 칸 32(완료 기준 1).
- UI 12 문구: 「클릭 두 점에 A–A′ 표시」, 「클릭 지표선 → 그 자리 높이 넣기」, 「클릭 점 → 높이 넣기」. 표고 풍선은 「역삼각형 표시와 「32.5m」」(글꼴이 ▽를 작게 그림).
- UI 17 증거: `ui.inspect`에 `trowelAsk`(`{for, text}` 또는 null)를 더했고, `scene_d4.py`가 그것을 본다. Backspace 단계(점 셋 → Backspace → Enter)도 더했다.

### 재작업 1 검증 (클라우드 컨테이너, Linux) — 2026-10-16
로그는 scratchpad `j024/r1-*.log`(git에 넣지 않음).

| 명령 | 종료 코드 | 결과 | 기록 |
|---|---|---|---|
| `cargo test -p vectorcraft-engine --lib` | 0 | 1,703 통과 · 0 실패(새 시험 3) | `r1-engine.log` |
| `cargo test -p vectorcraft-engine --test cmd_stratigraphy --test command_sweep --test plan_adversarial --test trowel_e2e --test m4_adversarial_stress --test trowel_adversarial_oracles` | 0 | 6 + 15 + 12 + 14 + 13 + 225 통과 · 0 실패 | `r1-engine-int.log` |
| `cargo test -p vectorcraft-tools` | 0 | 301 통과(새 시험 2) | `r1-tools.log` |
| `cargo test -p vectorcraft-doc` | 0 | 177 통과(새 시험 1) | `r1-doc.log` |
| `cargo test -p vectorcraft-ui-egui` | 0 | 1,063 + 6 통과 · 0 실패(새 시험 2) | `r1-ui.log` |
| `cargo test -p vectorcraft` | 0 | 66 통과 | `r1-app.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 경고 없음(앱·웹 crate 포함) | `r1-clippy.log` |
| `cargo fmt --all -- --check` | 0 | 통과 | `r1-fmt.log` |
| `twmissing.py` | 0 | 글자열 255개 중 번역 빠짐 0 | 터미널 |
| GitHub 「Trowel CI」 run 38947212723 (커밋 `5943506`) | - | **성공** | GitHub Actions |

### 재작업 1 (화면 검토 반영) 검증 — 2026-10-16
로그는 scratchpad `j024/r1b-*.log`.

| 명령 | 종료 코드 | 결과 | 기록 |
|---|---|---|---|
| `cargo test -p vectorcraft-engine --lib` | 0 | 1,703 통과 · 0 실패 | `r1b-engine.log` |
| 엔진 통합 시험 6개(위와 같은 명령) | 0 | 6 + 15 + 12 + 14 + 13 + 225 통과 · 0 실패 | `r1b-engine-int.log` |
| `cargo test -p vectorcraft-tools` | 0 | 301 통과 | `r1b-tools.log` |
| `cargo test -p vectorcraft-doc` | 0 | 177 통과 | `r1b-doc.log` |
| `cargo test -p vectorcraft-ui-egui` | 0 | 1,066 + 6 통과 · 0 실패(새 시험 3) | `r1b-ui.log` |
| `cargo test -p vectorcraft` | 0 | 66 통과 | `r1b-app.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 경고 없음 | `r1b-clippy.log` |
| `cargo fmt --all -- --check` | 0 | 통과 | `r1b-fmt.log` |
| `twmissing.py` | 0 | 글자열 255개 중 번역 빠짐 0 | 터미널 |
| GitHub 「Trowel CI」 run 38952781487 (커밋 `a33d898`) | - | **성공** | GitHub Actions |

### 완료 기준 7 확인 2 · 기준 8 다시 (새로 띄운 앱, `a33d898` 릴리스 빌드 sha256 `800b83dc…`)
- 같은 `scene_d4.py`에 Backspace 단계를 더해 돌렸다. 결과 scratchpad `shots/d4r1/`.
  - 키 1/2/4/3 → `sectionCut`/`levelLines`/`strataLine`/`spotHeight`. 빈 곳 → 「지표선을 누르세요」.
  - 높이 칸: Enter 뒤 닫힘(`trowelAsk` null), 표고 칸은 마지막 높이 「32.5」로 열림(`trowelAsk {for: spot, text: "32.5"}`).
  - 토층선 셋째 줄: 점 셋 → 키보드 Backspace → 마지막 점이 지워지고 미리 보기 점선이 그 자리로(`d4-strata-3pts.png`, `d4-strata-backspace.png`) → Enter로 「토층선 Ⅲ」·「층위 번호 Ⅲ」·「토층설명 Ⅲ」.
  - 표고: `ui.render` 4배에서 ▽와 「32.5m」가 보인다(`spot-render-crop.png`). 고치기 전 그림은 흰 사각형만(`spot-before-after.png` 위).
  - 상태줄은 표고 도구에서 「클릭 점 → 높이 넣기」.
- 기준 8 비교(`shots/d4r1/d4-compare-tools.png`)
  - 가름줄 y 226/407/588/752(열 기준 80/261/442/606), 칸·글자 자리는 앞 측정과 같다.
  - 단축키 글자: 대문자 23.0–30.0(시안 23.5–30.0), 색 토큰 #9A988F(시안 값, 그려진 가장 진한 점 약 (161,159,150), 시안 (154,152,143)).
  - **남은 차이**: (1) 고고학 밖 칸(P·R·S·K·I·H·Z 등) 아이콘이 VectorCraft 그림이라 시안 그림과 모양이 다르고 1–3 px 크다(D4 범위 밖). (2) 해발선·단면 기호가 지금 고른 범례 줄에 들어가 「따로 N」으로 세어진다(사람 결정 H9). (3) 길게 누르기 펼침은 control 채널로 누름을 유지할 수 없어 새로 띄운 앱에서 확인하지 못했다(합성 시험만).

## 확인 검토 (재작업 1 뒤) — 2026-10-16
### 코드 확인 검토 (code-reviewer, 대상 `0bd0e41..a33d898`) — 판정 **수정 필요**
기록 scratchpad `review/j024-code-r1.md`, 재현 `review/j024-r1-code/`. 앞 지적 0–12와 화면 지적 1–17의 처리는 「해결」로 확인(H9 빼고).

| # | 지적 | 무게 | 처리 |
|---|---|---|---|
| N1 | 해발선·경사 표시 도구의 선 집기가 객체 수의 제곱만큼 일함(5만 객체에서 포인터 한 번 8,917 ms) | 막는 것 | 재작업 2 |
| N2 | 토층선·단면선을 그리는 중 Ctrl+Z가 앞서 끝낸 선(번호·설명 줄째)을 지우고, 다시 하기로도 못 되살림 | 고칠 것 | 재작업 2 |
| N3 | 손으로 그린 「토층선 Ⅰ」을 지표선으로 못 집음 | 고칠 것(또는 H10) | 재작업 2(집을 수 있게, H10 추천대로) |
| N4 | 작업 기록에 H9 본문·CI 결과·다시 찍은 결과가 없음 | 고칠 것 | 이 절과 「사람 결정」 절 |
| N5 | 단면선 미리 보기 글자가 문서가 바뀌면 낡음 | 사소 | N2를 고쳐 그리는 중 되돌리기로는 안 생김. 메뉴 밖 다른 패널에서 지울 때는 남음(남은 위험) |
| N6 | 설명 칸 줄 간격 11.55가 인라인 표(11.0)와 다름, 72 pt 넘는 표에 7 pt 줄 | 사소 | 재작업 2 |
| N7 | 격자 토층 표(「토층 설명표」·「표 격자」)가 표시 목록에 없음 | 사소 | 재작업 2 |
| N8 | 높이 칸이 열릴 때 오류 아닌 상태줄 메시지를 모두 지움 | 사소 | 재작업 2 |

### 화면 확인 검토 (ui-reviewer, 새로 띄운 `a33d898` 릴리스 앱) — 판정 **수정 필요**
기록 scratchpad `review/j024-ui-r1.md`, 그림 `review/j024-r1-ui/out/`. 앞 지적 1–4, 6–8, 10–17은 앱에서 직접 눌러 「해결」. 진짜 길게 누르기(XTest)로 펼침이 열리고 경사 표시를 고를 수 있음을 처음 확인했다(앞의 남은 차이 3 해소).

| # | 지적 | 무게 | 처리 |
|---|---|---|---|
| N1 | 그리는 중 Ctrl+Z가 앞 선을 지움(코드 N2와 같음) | 고칠 것 | 재작업 2 |
| N2 | 길게 누르기로 펼침이 열려도 칸 설명 풍선이 남아 「경사 표시」 줄을 가림 | 사소 | 재작업 2 |

## 재작업 2 (rework, 마지막 루프) — 2026-10-16, 기준 `a33d898`
- 코드 N1 선 집기: `hit_test_skipping`의 건너뛰기 함수가 id 대신 노드를 받는다(`&dyn Fn(&Node) -> bool`). 걷다가 만난 노드를 그대로 보므로 `Document::node` 찾기가 없다. `objects_at`도 같이 바꿨다.
  - 시험 `picking_a_line_is_quick_in_a_big_drawing`(도구): 한 그룹에 짧은 선 2만 개, 그 사이를 움직이고 누르기 → 안내 대화, 1초 안. 고치기 전 코드로 돌리면 2.74초로 실패, 고친 뒤 통과(전체 12개 0.06초).
- 코드 N2·화면 N1 그리는 중 되돌리기: `menus::invoke`에서 Trowel 도구가 Backspace를 가져가는 동안(점이 있을 때) `edit.undo`는 마지막 점을 지운다. Ctrl+Z, 편집 메뉴, 펜 막대 「되돌리기」가 모두 이 길로 온다. 점이 없으면 예전처럼 문서를 되돌린다.
  - 시험 `the_keyboard_reaches_the_strata_line_while_it_is_drawn`에 Ctrl+Z 단계를 더했다(점 셋 → Ctrl+Z → 「토층선 Ⅰ」 그대로 → Enter면 「토층선 Ⅱ」 2점). 고치기 전 코드에서는 4438행에서 실패했다.
  - 펜 옆 단추(캔버스 오른쪽 누르기, J017 「펜 단추 1 = 되돌리기」)도 같은 `menus::invoke`로 보낸다. D4 빌드 때 단면선·토층선을 이 목록에 넣으면서 「그리는 중에도 `edit.undo`」로 정했던 것(빌드 절 「정한 것」)을 이것으로 바꾼다. 같은 시험에 오른쪽 누르기 단계를 더했고, 고치기 전 코드에서는 4459행에서 실패했다.
- 코드 N3 토층선: 「토층선 」을 표시 목록에서 뺐다. 번호(「층위 번호」·「토층 번호」)·설명(「토층설명」)은 그대로 건너뛴다.
  - 시험 기대값 변경: `marks_are_told_from_drawn_lines`가 「토층선 Ⅰ」을 표시가 **아님**으로 본다. 근거: 검토 N3과 `is_mark`의 뜻(「손으로 그린 선이 아님」), 사람 결정 H10의 추천.
  - 새 엔진 시험 `level_lines_take_a_strata_line_but_not_a_table`: 「토층선 Ⅰ」로 해발선이 생긴다.
- 코드 N7 격자 표: 「토층 설명표」를 목록에 넣었다. 같은 엔진 시험에서 「표 격자」로 해발선을 넣으면 「not the ground line」으로 거절된다.
- 코드 N6 줄 간격: 최소 간격을 `generateTable`과 같은 11.0으로 했다. 표의 글자 크기는 72 pt에서 자르지 않고 2–1296 pt로 묶는다(1296은 글자 명령의 상한, `typecmd.rs`).
  - 시험 기대값 변경: `strata_boundaries_past_ten_and_with_a_table_already_there`의 최소 간격 11.55 → 11.0. 근거: 검토 N6, `stratigraphy.rs`의 `generateTable` 인라인 간격 `max(크기×1.65, 11.0)`. 9 pt 표의 기대값(14.85)은 그대로다.
  - 6 pt 표(간격 11.0)와 100 pt 표(간격 165, 글자 100 pt) 경우를 더했다.
- 코드 N8 상태줄: 높이 칸이 열릴 때 지금 메시지가 「지표선을 누르세요」나 「경사 표시를 넣을 선을 누르세요」일 때만 지운다.
- 화면 N2 설명 풍선: 칸의 펼침이 열려 있는 동안(그 칸 자리에 펼침이 붙어 있을 때) 칸 설명 풍선을 띄우지 않는다(`toolbar::flyout_open_at`). 시험 `tool_column_has_the_mockup_groups`가 오른쪽 누르기로 펼친 뒤 1초 머물러도 풍선 글이 없는지 본다. 고치기 전 코드에서는 839행에서 실패했다.
- H9 안전책(검토자 제안 ③, 사람 결정 전 기본값): `legend::lines`가 도구가 넣은 표시 그룹(`is_mark`) 안을 보지 않는다. 그래서 해발선·단면 기호·해발 눈금은 범례 줄의 「따로」로 세지 않고, 「범례대로」(`legend.revert`)와 「굵기 한 번에」(`legend.weights`)가 그 굵기를 바꾸지 않는다.
  - 새 엔진 시험 `level_lines_keep_their_weights_in_a_legend_row`: 0.8 「단면선」 줄에 넣은 해발선(0.5·0.3)이 줄의 선으로 세지지 않고, `legend.revert {all}`·`legend.weights`(값·비율, `overrides` 포함) 뒤에도 0.5·0.3 그대로다.
  - 바뀌는 동작: 「굵기 한 번에」로 도면 전체를 줄여도 해발선·단면 기호는 줄지 않는다. 이것도 H9에 넣어 사람이 정한다.
  - **고침(마지막 확인 검토 B)**: 위 「범례대로가 바꾸지 않는다」는 `all`(전체)일 때만 맞다. 해발선 묶음이나 A–A′를 **골라서** [범례대로](`legend.revert {}`·`{pen}`)를 누르거나, 고른 채 범례 줄을 눌러 옮기면(`legend.pick {move}`) 지금도 줄 굵기가 된다. 수정 루프 2회가 끝나 고치지 않고 H9에 넣었다.

## 사람 결정 (승인 때 확인)
- **H9 해발선·단면 기호가 들어갈 범례 줄** (화면 검토 5·14, 확인 검토 「H9 판단」)
  - 지금: 해발선 묶음·단면 기호는 그 순간 고른 범례 줄의 레이어에 들어간다.
  - 위험(검토자 재현): 0.8 「단면선」 줄에 해발선을 넣고 [범례대로]를 누르면 해발선 16개가 모두 0.8이 되어 10 cm 선과 50 cm 선이 구분되지 않았다. 전체 범위는 재작업 2로 막았지만, 골라서 누르는 길은 아직 남아 있다(아래).
  - 답: ① 지금 줄 그대로, ② 해발선은 「해발선」 줄 레이어로(묶음 하나를 0.3·0.5 두 레이어로 나눠야 해서 자동 갱신 구조가 바뀜), ③ 지금 줄에 두되 범례 셈·[범례대로]·[굵기 한 번에]에서 도구가 넣은 표시를 뺀다.
  - 추천과 지금 구현: ③(데이터를 지키고 레이어 구조를 바꾸지 않음). ②가 필요하면 D6(해발선 자동 갱신) 때 함께 한다.
  - ③이 지금 걸리는 곳: 범례판의 「따로 N」 셈, 도면 메뉴 「범례대로 되돌리기」의 전체 범위, 「굵기 한 번에」, `legend.set`으로 줄 모양을 바꿀 때 따라오는 선, 줄 편집 카드의 「선 N」·「모두 고르기」. 대상은 이름으로 알아보는 표시 묶음 전부(해발선, 해발 눈금, 단면 기호, 표고, 층위 번호, 토층 설명, 토층 설명표, 축척 막대, 방위표)다. 도판 PDF·SVG는 모든 선을 그대로 넣는다(마지막 확인 검토 A).
  - ③이 아직 걸리지 않는 곳(마지막 확인 검토 B, 지금도 위험 남음): 표시 묶음을 골라 펜 칸이나 메뉴의 [범례대로]를 누를 때, 고른 채 범례 줄을 눌러 옮길 때. 펜 칸은 해발선 묶음을 고르면 「범례와 다름」과 [범례대로]를 띄운다. 경로 하나로 된 표시(「비탈면 빗금」)도 셈에 들어간다(S3).
  - 결정할 것: ①/②/③ 가운데 무엇으로 할지, ③이면 위 「아직 걸리지 않는 곳」에도 걸지(검토자 추천: 건다).
- **H10 손으로 그린 토층선을 지표선·경사 표시 기준선으로**: 추천대로 쓸 수 있게 구현했다(재작업 2). 막아야 하면 알려 주면 된다.
- 앞 코드 검토의 H1–H8은 이번에도 정해지지 않았다. 지금 구현을 그대로 적는다.
  - H1 해발 기준점: 넣은 높이를 지표선 가장 낮은 점의 높이로 저장(`elevation.create` 모델). 지표선을 고치면 누른 자리의 높이가 바뀔 수 있다.
  - H2 설명 칸 위치: 첫 경계선 아래 24 pt.
  - H3 격자 표가 있을 때: 표 배치는 두고 따로 「토층설명 7p」 칸(구현됨, 이 선택을 확인받으면 됨).
  - H4 줄 모양: 경계선은 「Ⅰ층 : 」, 인라인 표는 「I (①) : …」로 서로 다르다.
  - H5 시선 방향: 그린 방향으로 꺾임 방향을 정함. 두 점 사이 잇는 선 없음.
  - H6 해발선 간격: 엔진 기본(10 cm 가는 선 + 50 cm 굵은 선). 디자인 5장 ④는 「50㎝마다」.
  - H7 안내 언어: 한국어만 번역. 다른 언어 화면은 영어.
  - H8 11층부터: Ⅺ·Ⅻ 대신 `XI`, `XII` …(`stratigraphy.number`와 같은 규칙).

### 재작업 2 검증 (클라우드 컨테이너, Linux) — 2026-10-16
로그는 scratchpad `j024/r2-*.log`(git에 넣지 않음).

| 명령 | 종료 코드 | 결과 | 기록 |
|---|---|---|---|
| `cargo test -p vectorcraft-engine --lib` | 0 | 1,705 통과 · 0 실패(새 시험 2) | `r2-engine.log` |
| 엔진 통합 시험 6개(위와 같은 명령) | 0 | 6 + 15 + 12 + 14 + 13 + 225 통과 · 0 실패 | `r2-engine-int.log` |
| `cargo test -p vectorcraft-tools` | 0 | 302 통과(새 시험 1) | `r2-tools.log` |
| `cargo test -p vectorcraft-doc` | 0 | 177 통과 | `r2-doc.log` |
| `cargo test -p vectorcraft-ui-egui` | 0 | 1,066 + 6 통과 · 0 실패(시험 2개에 단계 더함) | `r2-ui.log` |
| `cargo test -p vectorcraft` | 0 | 66 통과 | `r2-app.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 경고 없음(앱·웹 crate 포함) | `r2-clippy.log` |
| `cargo fmt --all -- --check` | 0 | 통과 | `r2-fmt.log` |
| `cargo xtask layers` | 0 | 27 crate, 층 위반 없음 | `r2-layers.log` |
| `twmissing.py` | 0 | 글자열 255개 중 번역 빠짐 0 | 터미널 |
| 고치기 전 코드로 새 시험 돌리기(임시로 되돌린 뒤 복원) | 101 | 선 집기 2.74초로 실패, Ctrl+Z 4438행·풍선 839행 실패 | 터미널 |
| GitHub 「Trowel CI」 run 38960953412 (커밋 `89105ae`) | - | **성공** | GitHub Actions |

펜 옆 단추를 고친 뒤 다시 돌린 것(로그 `j024/r2b-*.log`): `cargo fmt --all -- --check` 0, `cargo test -p vectorcraft-ui-egui` 0(1,066 + 6 통과), `cargo test -p vectorcraft` 0(66 통과), `cargo clippy --workspace --all-targets -- -D warnings` 0. 고친 곳은 `canvas.rs` 한 줄과 화면 시험이라 엔진·도구·문서 시험은 `89105ae` 결과를 쓴다. GitHub CI는 다음 커밋에서 본다.

### 완료 기준 7 확인 3 · 기준 8 다시 (새로 띄운 앱, `89105ae` 릴리스 빌드 sha256 `b8b7d9c7…`)
- `scene_d4r2.py`(앞 장면에 두 단계를 더함)로 돌렸다. 결과 scratchpad `shots/d4r2/`, 보고 그림 `/mnt/project-files/design/trowel-run/d4/`.
  - 키 1/2/4/3, 빈 곳 안내, 높이 칸 닫힘, 토층선 Backspace는 앞과 같다.
  - 그리는 중 Ctrl+Z: 토층선 Ⅰ–Ⅲ 뒤 점 셋 → Ctrl+Z → 마지막 점만 지워지고 Ⅰ–Ⅲ 그대로 → Enter로 「토층선 Ⅳ」(`ctrlz-pair.png`).
  - 토층선 위 해발선: 해발선 도구로 「토층선 Ⅰ」을 누르면 높이 칸이 열린다(`trowelAsk {for: level, text: "32.5"}`), Esc로 닫힘.
  - 범례: 재작업 2 전 「평면외곽선 따로 1」 「단면선 따로 16」이 이제 없다(`legend-before-after.png`).
  - 표고: `ui.render` 4배에서 ▽와 「32.5m」(`spot-render-crop.png`).
- 기준 8 비교(`d4-compare-tools.png`): 처음 띄운 화면이 `a33d898` 화면과 바이트까지 같다(도구 열에 바뀐 것 없음). 칸·가름줄·글자 자리는 앞 측정 그대로.
  - **남은 차이**: (1) 고고학 밖 칸 아이콘(R·S·K·I·H·Z 등)이 VectorCraft 그림이라 시안 그림과 모양이 다르고 1–3 px 크다(D4 범위 밖). (2) 해발선·단면 기호가 들어가는 범례 줄은 사람 결정 H9로 남는다(셈에서 빠져 「따로」는 이제 안 뜸). 길게 누르기 펼침은 화면 확인 검토가 XTest로 확인했다. 그래서 「통과」가 아니라 「차이 1건 + 결정 1건 남음」이다.

## 마지막 확인 검토 (재작업 2 뒤, 대상 `a33d898..06ce1d4`) — 2026-10-16
- 코드(scratchpad `review/j024-code-r2.md`): 판정 **막는 것 있음**(A). 앞 지적 코드 N1–N8, 화면 N1–N2는 모두 해결(N5는 일부 남음, 아래 S4). 5만 선 도면에서 해발선 도구 포인터 한 번 94 ms(재작업 전 8,917 ms). 되돌리기는 Ctrl+Z·편집 메뉴·펜 막대·캔버스 오른쪽 누르기·클래식 창 모두 마지막 점만.
- 화면(scratchpad `review/j024-ui-r2.md`, 새로 띄운 `06ce1d4` 앱): 판정 **막는 것 있음**(같은 A). 화면 N1·N2, 코드 N3·N7·N8 해결. 「따로 N」 안 뜸, 「굵기 한 번에」(×0.8, 따로 바꾼 선 포함)와 전체 [범례대로] 뒤에도 해발선 0.3/0.5 그대로. 처음 화면은 `a33d898`과 픽셀까지 같음. 8장 5·7·8번 끊기지 않음.

| # | 지적 | 무게 | 처리 |
|---|---|---|---|
| A | 도판 PDF·SVG 내보내기에서 해발선·해발 보조선·시선 화살표·표고 기호·격자 표 선이 소리 없이 빠짐(SVG path 19 → 7). 재작업 2가 `legend::lines`를 바꿨는데 `plate.rs`도 그것을 씀 | 막는 것(재작업 2의 회귀) | 고침: 범례 셈은 `legend::lines`(표시 건너뜀) 그대로, 도판은 새 `legend::all_lines`(모든 선, 재작업 2 전과 같은 걷기). 시험 `plates_print_the_level_lines_and_section_marks`(주 해발선·해발선·해발 보조선·시선 화살표가 원본과 도판에 같은 수). 고치기 전 코드에서 241행 실패 |
| B | H9 안전책이 전체 범위에만 걸림: 표시 묶음을 골라 [범례대로]·[범례에 반영]을 누르거나 고른 채 범례 줄을 누르면 해발선이 줄 굵기가 됨. `a33d898`에서도 같음(회귀 아님) | 고칠 것 | 수정 루프 2회가 끝나 고치지 않음 → **needs-human, H9에 넣음**. 기록의 틀린 문장을 바로잡음 |
| S1·C | 그리는 중 편집 메뉴 글이 「실행 취소: Soil Layer Boundary」(영어, 점 지우기인데 앞 단계 이름). 빈 도면에서는 메뉴가 꺼져 메뉴로는 점을 못 지움(Ctrl+Z·펜 막대·오른쪽 누르기는 됨) | 사소 | 남김(다음 작업에서 번역과 메뉴 글) |
| S2 | 점 지우기가 UI에만 있음: control 채널·MCP·실행 기록 패널의 `edit.undo`는 문서 단계를 되돌림 | 사소 | 남김(엔진 `undo`가 도구 상태를 보게 하는 것은 따로) |
| S3 | 경로 하나로 된 표시(「비탈면 빗금」)는 범례 셈에 들어감 | 사소 | 남김(H9에 적음) |
| S4 | 단면선 첫 점 뒤 다시 하기를 하면 미리 보기 글자가 낡음(N5의 나머지) | 사소 | 남김(만들어지는 것은 엔진이 다시 찾아 맞음) |

### A 고친 뒤 검증 (클라우드 컨테이너, Linux) — 2026-10-16
로그는 scratchpad `j024/r2c-*.log`.

| 명령 | 종료 코드 | 결과 |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | 통과 |
| `cargo test -p vectorcraft-engine --lib` | 0 | 1,706 통과 · 0 실패(새 시험 1) |
| `cargo test -p vectorcraft-engine --test cmd_plate --test m5_adversarial_stress` + 엔진 통합 시험 6개 | 0 | 4 + 6 + 15 + 12 + 11 + 14 + 13 + 225 통과 · 0 실패 |
| `cargo test -p vectorcraft-doc` | 0 | 177 통과 |
| `cargo test -p vectorcraft-ui-egui` | 0 | 1,066 + 6 통과 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 경고 없음 |
| GitHub 「Trowel CI」 run 38964274273 (커밋 `06ce1d4`) | - | **성공** |
| 새로 띄운 릴리스 빌드(`fbc78bd`, sha256 `474ccadc…`)에 검토자 스크립트 `r1_engine.py`(단면선·해발선·A–A′ 기호를 그리고 도판 SVG 내보내기) | 0 | path 19개, 굵기 0.2·0.3·0.35·0.4·0.5·0.8·1.2 (`a33d898`과 같음, 고치기 전 `89105ae`는 7개). 로그 scratchpad `review/j024-r2-code/logs/r1-fbc78bd.log` |
| GitHub 「Trowel CI」 run 38969988936 (커밋 `fbc78bd`) | - | **성공** |

## 승인 (approval)
- AI 검토: 코드·화면 독립 검토 2회와 마지막 확인 끝. 마지막 확인의 막는 것 A는 `fbc78bd`로 고침, 남은 것은 needs-human B(H9에 넣음)와 사소 S1–S4.
- 사용자 승인: **아직 없음**(2026-10-16 승인 대기로 올림). 승인 보고 `/mnt/project-files/design/trowel-run/d4/J024-승인보고.md`.
- 승인 대상 커밋: `8dff460` `0bd0e41` `5943506` `a33d898` `89105ae` `06ce1d4` `fbc78bd`. 이 기록 커밋 뒤 코드가 바뀌면 다시 검증하고 다시 승인받는다.
