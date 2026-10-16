상태: verify (재작업 2)

# J023 (디자인 3.0 D3): 오른쪽 칸 — 범례 → 펜 → 속성

## 명세 (spec)
- 기준 커밋: `be5f225` (J022 재작업 1·2)
- 작업 브랜치: `claude/project-thread-ams9bp`
- 근거: 디자인 3.0 (https://claude.ai/artifact/GmVPKvYKq4ADAe5ZbmZLdc) 2장 중복 표(선 굵기·선 종류·현장 표준 12종·범례대로·색)와 「서로 맞지 않는 것」(굵기 목록), 3장 원칙 1·2·4, 4장 시안 오른쪽 칸, 5장 ⑦·⑧·⑨, 7장 「범례 줄 하나의 모양」 「펜이 범례와 다를 때」, 10장 ⑦·⑧·⑨ 줄, 11장 D3.

### 완료 기준
1. 오른쪽 칸(폭 304, 220~560 조절)은 위에서부터 범례(남는 높이 모두) → 펜 → 속성 순서다. 펜의 세 버튼이 생기거나 없어질 때 포인터가 펜 위에 있으면 펜 윗변은 그대로 있고(누른 칩이 포인터 밑에 그대로), 포인터가 펜을 떠나면 다시 맞춘다. 확인: `the_right_column_is_legend_pen_properties`, `a_double_click_on_a_chip_types_a_weight_as_the_buttons_come`.
   - 명세 고침(구현 중): 처음 명세의 「범례와 펜 사이를 끌어서 높이 바꾸기」는 디자인 3.0 시안·글에 없어 뺐다(범례가 남는 높이를 모두 씀).
2. 굵기 목록은 앱 전체에서 하나: 0.1 · 0.15 · 0.2 · 0.3 · 0.4 · 0.5 · 0.6 · 0.8 · 1.0 pt (엔진 `legend::LADDER`). 펜의 굵기 칩, 줄 편집 카드의 굵기 칩, 「굵기 한 번에」의 한 단계 올림·내림이 모두 이 목록을 쓴다. 확인: `one_weight_list_everywhere`, 엔진 `legend.weights` 단계 시험.
3. 펜 (5장 ⑧, 4장 시안 `.pen`)
   - 머리 34: 「펜」과 「평면외곽선 · 다음에 그릴 선」(고른 선이 있으면 「· 고른 선 N개」), 오른쪽 ⋯(획 패널).
   - 굵기: 칩 9개 24×30, 간격 2, 칩마다 굵기 비율 선 견본과 「.1」~「1.0」. 고른 칩은 2 먹색 테두리와 oat 바탕. 칩을 두 번 누르면 직접 입력(pt). 누르면 다음 선 또는 고른 선의 굵기.
   - 선: 실선 · 파선 · 점선 · 일점쇄선 한 묶음 버튼(높이 26).
   - 색: 검정 · 적색 · 청색 · 흑갈 · 암갈 · 황갈 · 회색 · 백색 견본 20×20과 ⋯(견본 패널). 클릭은 선 색, Alt+클릭은 면 색.
   - 빗금: 없음 · 45° · 135° · 번갈아 한 묶음 버튼. 닫힌 면을 고르고 누르면 그 면에 빗금, 번갈아를 다시 누르면 방향을 뒤집음, 이웃과 같은 방향이면 「!」.
   - 상태: 「✓ 범례와 같음」 또는 「● 범례와 다름 · 굵기 0.6 → 0.8」. 다를 때만 [이번만 이렇게] [범례에 반영] [범례대로](높이 24).
     - 이번만 이렇게: 아무것도 바꾸지 않고 세 버튼을 접음(그 모양이 다시 바뀌면 다시 나옴). 범례 줄에는 「따로 N」이 붙음. 누르지 않고 그리기 도구로 그리면 이것으로 봄(7장). 나중에 선택 도구로 그 선을 고르면 버튼이 다시 나와 범례에 반영할 수 있음.
     - 범례에 반영: 그 줄의 모양을 펜(또는 고른 선)의 굵기·선 종류로 바꾸고, 그 줄의 「따로」가 아닌 선이 모두 따라옴(`legend.set`, 되돌리기 한 번).
     - 범례대로: 펜과 고른 선을 줄의 모양으로(`legend.revert`).
   - 없앰: 「실무」 줄, 「0.4파」 칩, 굵기 ＋ 칩, 선/면 전환 칩, 색 없음 칩, 「잘린 면」 「범례에 더하기」 「범례대로」(아래) 버튼.
   - 확인: `the_pen_follows_the_mockup`, `the_pen_says_when_it_differs_from_the_legend`, `the_three_buttons_do_what_they_say`, `drawing_on_counts_as_just_this_once`, `palette_weight_chip_sets_the_next_stroke`, `palette_line_types_set_the_dash`.
4. 범례 (5장 ⑦, 7장)
   - 머리 36: 「범례」, 범례 묶음 「평면 ▾」(평면·단면·유물 중 고른 묶음이 ★ 다음 맨 위, 나머지는 원래 순서), ＋(새 줄: 펜 모양, 고른 선이 있으면 첫 선의 모양으로 「범례 줄 N」을 만들고 고른 선을 그 줄로 옮긴 뒤 줄 편집을 엶, `legend.add`), ⋯(줄 편집, 현장 표준 12종 다시 넣기, 쓰지 않는 줄 정리, 굵기 한 번에…, 양식 저장·불러오기…, 레이어 패널; 도면이 없으면 레이어 패널만).
   - 명세 고침(구현 중): ＋는 처음 명세의 「빈 레이어 + 줄 편집」 대신 없어진 펜의 「범례에 더하기」가 하던 일(펜 모양으로 새 줄)을 맡는다. 그 버튼이 없어져도 그 기능이 한 자리에 남게 하려는 것.
   - 목록: 「★ 자주 씀」, 「평면」, 「단면」, 「유물」, 「밑그림」(밑그림 레이어), 「그 밖」(묶음 없는 레이어) 칸으로 나눔. 빈 칸은 뺌.
   - 줄 하나(높이 27): 눈 16 · 자물쇠 16 · 견본 34(굵기 비율의 선과 선 종류, 면 줄도 선, 표시 줄은 그 표시, 밑그림은 옅은 상자) · 이름(★ 줄은 「★ 이름」 흙색) · 「따로 N」(흙색 바탕 흰 글자, 이름 뒤) · 굵기 · 개체 수 26. 지금 줄은 oat 바탕.
   - 누르면 지금처럼 그 줄로 그림(고른 것을 옮김), 두 번 누르면 줄 편집. 개체 수를 누르면 그 줄의 개체를 모두 고름. Alt+눈은 이 줄만 보이기.
   - 오른쪽 클릭: 줄 편집 · 이 줄의 개체 모두 고르기 · ★ 자주 씀 · 위로 · 아래로 · 지우기.
   - 없앰: 줄 앞 화살표와 개체 목록(↺ 포함), 아래 「굵기 한 번에」 「따로 N → 범례」 줄, ＋ 오른쪽 클릭의 규칙 고르기 메뉴, 머리의 「평면 범례 넣기」 목록, Strata 칸 나눔.
   - 확인: `the_legend_follows_the_mockup`, `legend_rows_menu_and_count_work`, `legend_rows_show_the_look_star_and_overrides`, `legend_rows_show_hide_lock_and_pick_the_layer`, `the_legend_more_menu_runs_its_items`, `the_legend_more_items_wait_for_a_drawing`, `the_legend_plus_makes_a_row_of_the_pens_look`, `a_long_legend_lays_out_only_the_rows_in_view`, `the_legend_list_stays_above_the_pen`.
5. 속성 (5장 ⑨)
   - 고른 것이 없으면 머리 34만(「속성 · 고른 것 없음 ▸」), 고르면 저절로 펼쳐짐(약 150). ▸/▾로 접고 펼 수 있음.
   - 머리: 「선 3개 · 평면외곽선」처럼 고른 것과 레이어.
   - 크기: 가로 × 세로를 실제 길이 m(도면 축척으로 환산, 1:30 도면의 30 mm = 0.90 m). 축척이 없으면 종이 위 mm.
   - 닫힌 면 하나면 면적 ㎡와 둘레 m(`trowel.polygonStats`, 축척이 있을 때).
   - 위치 X·Y: 좌표계가 있으면 TM m(상태줄과 같은 축 순서·자리 나눔).
   - 정렬 6개(왼쪽·가운데·오른쪽·위·가운데·아래, `object.align`): 두 개 이상 골랐을 때.
   - 없앰: 종류·번호·시대 「고르세요」 칸(J009 기능이 생길 때까지), 머리의 ⋯(속성 패널 띄우기; 패널은 창 메뉴에 그대로).
   - 확인: `the_properties_fold_and_show_real_sizes`.
6. 새로 띄운 앱 화면과, 선을 고르고 굵기를 바꾼 화면을 4장 시안·7장 「다를 때」 그림과 나란히 놓고 오른쪽 칸을 비교한 결과와 남은 차이 목록을 남긴다. 차이가 남아 있으면 「통과」라고 하지 않는다.
7. `cargo fmt --all -- --check`, `cargo clippy` (ui-egui, engine), `cargo test -p vectorcraft-ui-egui`, `cargo test -p vectorcraft-engine`의 범례 시험, GitHub 「Trowel CI」(`cargo xtask ci`, wasm 포함) 통과.

### 변경 범위
- `crates/ui-egui/src/trowel/{mod,palette,legend,props,rowedit,weights,tests}.rs`
- 명세 고침(구현 중): `crates/ui-egui/src/theme.rs`에 `ok` 색 하나(「✓ 범례와 같음」의 초록, 시안 HTML `#3F6B31`). trowel 화면 색은 토큰에서만 온다는 규칙 때문.
- 명세 고침(재작업 1): `engine/cmd/legend.rs`의 `legend.revert`에 `pen` 인자(고른 선과 함께 펜도 되돌림), 현장 표준 12종의 ★를 7장 표대로 3줄로.
- `crates/engine/src/cmd/legend.rs`: `LADDER` 6개 → 9개와 명령 설명, 엔진 범례 시험
- `i18n/*.tsv`: 새 `tw!` 글자열 번역, 쓰이지 않게 된 줄 정리

### 범위 밖
- 새 도면이 처음부터 현장 표준 12종을 갖고 시작하는 것, 엔진 범례 묶음 규칙(평면·단면·유물 15·12·7줄)을 12종으로 합치는 것: 9장 새 도면 대화상자의 「범례」 칸과 함께 할 일이라 이번에 하지 않는다. `legend.addGroup` 명령은 그대로 두고(명령 찾기·MCP로 닿음) 범례 화면의 묶음 넣기 목록만 뺀다.
- 줄을 끌어 순서 바꾸기: 위로·아래로 메뉴로 대신하고 남은 차이로 보고한다.
- 도구 열의 칠/선 상자(D4), 요소함(D5).

### 위험 (사람 검토)
- ◆ 「굵기 한 번에」의 한 단계 올림·내림이 0.1 · 0.2 · 0.3 · 0.5 · 0.8 · 1.0 에서 9단계로 바뀐다(예: 0.3 다음이 0.5에서 0.4로). 수치 규칙 변경이라 사람 검토 대상.
- 펜 색 견본 8개의 실제 색값은 지금 앱의 값(검정 20,20,20 · 적색 210,35,35 …)을 그대로 쓴다. 시안 그림의 색(#E8334F 등)과 다르지만 도면에 칠해지는 색이 바뀌지 않게 둔다.
- 「잘린 면」 버튼(0.8 pt 테두리 + 번갈아 빗금)이 없어진다. 빗금 「번갈아」는 빗금만 넣고 테두리 굵기는 범례 줄(단면선 0.8 등)이 정한다. `hatch.cutFace` 명령은 그대로 남는다.
- 범례의 개체 목록과 개체별 ↺가 없어진다. 따로 그린 선을 되돌리는 길은 펜의 「범례대로」(고른 선)와 도면 메뉴 「범례대로 되돌리기」(고른 것이 없으면 전부).
- 기존 시험 기대값 변경: 팔레트 줄·버튼, 범례 화살표·개체 목록·아래 줄, 속성 「곧」 칸을 확인하던 시험은 디자인 3.0에 맞게 바꾼다(근거: 승인된 디자인 3.0 5장 ⑦~⑨).

## 구현 요약 (build)
- `trowel/mod.rs`: 오른쪽 칸을 범례(남는 높이) → 펜 → 속성으로 나눔. 칸 머리 그리기 `column_head`. 펜의 세 버튼이 포인터 밑에서 생기거나 없어지면 펜 윗변을 붙잡아 둠(`right_parts`의 hold).
- `trowel/legend.rs`: 새로 씀. 머리(범례 · 묶음 ▾ · ＋ · ⋯), 칸 나눔(★ 자주 씀 · 고른 묶음 · 나머지 묶음 · 밑그림 · 그 밖), 줄 27(눈 · 자물쇠 · 견본 34×10 · 이름 · 따로 N · 굵기 · 개체 수), 보이는 줄만 배치, 오른쪽 클릭 메뉴, ⋯ 메뉴. 개체 목록·아래 줄·규칙 메뉴·Strata 칸은 뺌.
- `trowel/palette.rs`: 펜으로 새로 씀. 머리(펜 · 줄 이름 · 다음에 그릴 선/고른 선 N개 · ⋯), 굵기 칩 9개, 선 종류, 색 8 + ⋯, 빗금 4, 상태 줄과 세 버튼. 그리기 도구로 그으면 「이번만 이렇게」로 봄.
- `trowel/props.rs`: 새로 씀. 접힘·펼침, 크기 m(축척), 면적·둘레(`trowel.polygonStats`), 위치(TM), 정렬 6.
- `trowel/rowedit.rs`: 줄 편집 카드의 굵기 칩이 같은 9개 목록(짧은 이름 「.6」).
- `theme.rs`: `ok` 색(「범례와 같음」 표시의 초록).
- `engine/cmd/legend.rs`: `LADDER` 9개와 명령 설명.
- `i18n/ko.tsv`: 새 문자열 번역, 이번에 쓰이지 않게 된 trowel 줄 36개 정리(전부터 안 쓰이던 7줄은 그대로).

## 시험 바꿈 (기대값을 바꾼 근거: 승인된 디자인 3.0 5장 ⑦~⑨, 7장)
- 지움: `object_rows_show_overrides_and_revert`(개체 목록 없앰), `the_group_menu_adds_a_legend_group`·`a_group_added_from_the_menu_draws_with_its_first_favourite`(머리의 묶음 넣기 목록 없앰, 엔진 `legend.addGroup` 시험은 그대로), `the_chosen_line_type_chip_keeps_its_border`(시안 `.seg`에 먹색 테두리 없음), `double_clicking_a_row_moves_nothing_when_its_first_click_shifts_the_rows`(줄이 열리지 않아 첫 클릭에 줄이 움직이지 않음; 두 번 클릭이 아무것도 옮기지 않는 것은 `double_clicking_a_row_with_lines_selected_moves_nothing`이 그대로 확인).
- 바꿔 씀: 짧은 창 스크롤, 범례 목록이 펜 위에 머묾, 긴 범례는 보이는 줄만, 패널 띄우기(⋯ → 획), 속성(실제 크기), 줄 모양, 칸 순서, ★ 칸, ⋯ 메뉴, 도면 없을 때, 선 종류, ＋, 두 번 탭, 표시 견본 자리(왼쪽 56, 34×10), 굵기 한 번에를 ⋯에서 엶, 늘 보이는 팔레트(0.4 파선·색 대상·색 없음 확인 뺌), 카드 크기(아래 줄 단추 확인 뺌), 흐린 단추 설명(굵기 한 번에 반쪽 뺌).
- ◆ `weights_card_value_and_step_modes`의 한 단계 기대값: 내림 (0.3, 0.5, 0.8) → (0.3, 0.6, 0.8), 올림 (0.5, 0.8, 1.0) → (0.4, 0.8, 1.0). 9단계 목록 때문(위험 ◆).
- 이름으로 짝: `a_big_open_layer_lays_out_only_the_rows_in_view` → `a_long_legend_lays_out_only_the_rows_in_view`, `palette_legend_buttons_add_and_revert` → `the_legend_plus_makes_a_row_of_the_pens_look`와 `the_three_buttons_do_what_they_say`(펜의 「범례에 더하기」 「범례대로」 버튼 없앰), `properties_summarise_the_selection` → `the_properties_fold_and_show_real_sizes`, `the_weights_button_waits_for_a_drawing` → `the_legend_more_items_wait_for_a_drawing`(아래 줄 없앰, ⋯로 옮김), `the_legend_list_stays_above_its_footer` → `the_legend_list_stays_above_the_pen`, `the_footer_opens_the_weights_card_and_reverts_all_overrides` → 지움(아래 줄 없앰; 굵기 한 번에는 `the_legend_more_menu_runs_its_items`, 모두 되돌리기는 `drawing_menu_opens_the_weights_card_and_reverts`가 확인), `strata_rows_have_their_own_section` → 지움(Strata 칸 나눔 없앰, 완료 기준 4).
- 새로: `the_right_column_is_legend_pen_properties`, `one_weight_list_everywhere`, `the_pen_follows_the_mockup`, `the_pen_says_when_it_differs_from_the_legend`, `the_three_buttons_do_what_they_say`, `drawing_on_counts_as_just_this_once`, `a_double_click_on_a_chip_types_a_weight_as_the_buttons_come`, `the_legend_follows_the_mockup`, `legend_rows_menu_and_count_work`, `the_legend_more_menu_runs_its_items`, `the_legend_more_items_wait_for_a_drawing`, `the_legend_plus_makes_a_row_of_the_pens_look`, `the_properties_fold_and_show_real_sizes`.

## 검증 (verify, 클라우드 컨테이너, Linux)
| 명령 | 종료 코드 | 결과 | 기록 |
|---|---|---|---|
| `cargo fmt --all -- --check` | 0 | 통과 | scratchpad `factory/d3-fmt.log` |
| `cargo test -p vectorcraft-ui-egui` (재작업 전) | 0 | 1,050 통과 · 0 실패 | `factory/d3-ui-all.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel i18n` (화면 고침 뒤) | 0 | 136 통과 · 0 실패 | 터미널 |
| `cargo clippy -p vectorcraft-ui-egui -p vectorcraft-engine --all-targets -- -D warnings` | 0 | 경고 없음 | `factory/d3-clippy.log`, `d3-clippy2.log` |
| `cargo test -p vectorcraft-engine --lib -- legend` | 0 | 29 통과 | `factory/d3-engine-legend.log` |
| GitHub 「Trowel CI」 | - | 아직 안 돌림(푸시 전) | - |

## 화면 비교 (새로 띄운 앱, 1440×880, Xvfb + llvmpipe)
- 그림: scratchpad `shots/d3-compare-right.png`(시안 · 새로 띄운 앱 · 현장 표준 12종과 선 1개 · 선을 고르고 0.8), `shots/d3-compare-differs.png`(7장 「다를 때」 그림 · 앱).
- 같은 것: 칸 순서와 높이, 범례 머리(범례 · 평면 ▾ · ＋ · ⋯), ★ 칸과 묶음 칸, 줄 27의 눈·자물쇠·견본·★ 흙색 이름·굵기·개수, 지금 줄 oat, 「따로 1」 흙색 표, 펜 머리 「평면외곽선 · 다음에 그릴 선 / 고른 선 1개」, 굵기 칩 9개와 고른 칩 테두리, 선·색·빗금 줄, 「✓ 범례와 같음」, 「● 범례와 다름 · 굵기 0.6 → 0.8」과 세 버튼, 속성 접힘 「고른 것 없음」.
- 남은 차이:
  1. 새 도면은 범례 줄 1개(굴광선)로 시작한다. 시안은 현장 표준 12종으로 시작(범위 밖: 9장 새 도면 대화상자).
  2. 「단면빗금」 줄 견본이 빗금 상자가 아니라 선이다. 줄 자료에 「빗금 줄」 표시가 없어서(자료 형식 변경이라 다음 작업, 사람 검토).
  3. 색 견본의 「검정」(20,20,20)이 기본 펜 색과 달라 시안처럼 검정에 고른 고리가 생기지 않는다(색값을 바꾸면 도면 색이 바뀌어 사용자 결정).
  4. 속성 머리 오른쪽 표시가 시안의 ▸ 삼각형이 아니라 꺾쇠 아이콘이다.
  5. 줄을 끌어 순서 바꾸기가 없다(오른쪽 클릭 위로·아래로로 대신).
  6. 「따로」 표는 범례와 다른 선이 있을 때 붙는다. 펜만 다르고 아직 그리지 않았을 때는 펜의 상태 줄이 알린다(7장 표의 「펜이 범례와 다르면 이름 뒤에 따로 표」를 이렇게 읽음).
  7. 밑그림 줄의 잠긴 자물쇠는 먹색 채움 아이콘이 아니라 선 아이콘이다(이번 장면에 밑그림 없음, 시험으로만 확인).

## 검토 (review) 1차 — 2026-10-16, 기준 6684b51
- 코드 검토(code-reviewer): **수정 필요**. 막는 것 2, 고칠 것 2, 사소한 것 6.
  1. (막는 것) 선택 도구로 고른 선에서 「이번만 이렇게」를 눌러도 버튼이 접히지 않음(`buttons_show`가 선택 도구면 눌림을 보지 않음). 앱을 띄워 재현.
  2. (막는 것) 펜·속성이 프레임마다 고른 것 수 × 문서 크기만큼 계산(`layer_of`·`node`가 매번 트리를 훑음, 프레임마다 두 번). 묶은 선 8,000개를 고르면 한 프레임 651 ms(실측, dev 빌드).
  3. (고칠 것) 고른 선이 있을 때 「범례대로」가 펜을 되돌리지 않음(7장 「펜과 고른 선을」).
  4. (고칠 것) 시험이 느슨함: 굵기 단계 시험이 옛 6단계와 구별 못 함(0.4 → 0.5/0.3), 속성 위치가 형식만 봄, 1·3번 시험 없음.
  - 사소한 것: 읽지 않는 `Apart.lines`, 모듈 밖에서 안 쓰는 `pub(super)`, 「이번만 이렇게」·속성 접힘 값에 도면 uid 없음, 스크롤 배치에서 펜 고정 값이 안 지워짐, 속성 위치에 「m」 없음, 영어 복수형(「1 lines」), `even_segments`와 `segments` 비슷함·`SAMPLE_PX`가 굵기 목록을 다시 적음, 기록(시험 이름 짝, `theme.rs` 명세 고침 줄, backlog 상태, 10장 ⑧ 「색 견본은 theme 토큰으로」).
  - CLAUDE.md 금지 사항(패닉·인덱싱·색 토큰·명령) 문제 없음. clippy·fmt·시험 1,058·엔진 범례 29·trowel_e2e 225 통과 확인.
- 화면 검토(ui-reviewer): **남은 차이 있음**. 칸 순서·폭 304·머리 36·줄 27·칩 24×30·견본 20×20·버튼 24는 시안과 같음.
  1. (막는 것) 색 견본을 누르면 선이 흰색이 됨: 0–255 정수 `[210,35,35]`를 보내는데 엔진은 `[r,g,b]`를 0–1로 읽음. D3 전 팔레트부터 있던 버그.
  2. (막는 것) 코드 검토 1번과 같음.
  3. (막는 것) 세 버튼이 포인터 밑에서 없어질 때 펜 윗변이 40 내려가 포인터가 빗금 「45°」 위로 감(완료 기준 1).
  4. (고칠 것) ★ 자주 씀이 12종에서 6줄(시안·7장 표는 평면외곽선·내선(바닥)·단면선 3줄).
  5. (고칠 것) 펜 높이 198(시안 190): 색 줄 32(시안 26).
  6. (고칠 것) 머리 오른쪽 버튼이 시안보다 7–9 왼쪽.
  7. (고칠 것) 속성 ▸/▾ 대신 꺾쇠. 8. 잠긴 자물쇠가 선 아이콘(시안 먹색 채움). 9. 밑그림 줄 이름이 흐림(시안 먹색). 10. 속성 위치에 m 없음.
  - 사용자 결정·기록: 견본 색값이 시안과 다름, 이 환경에 굵은 한글 글꼴 없음(J022와 같음), 글꼴이 섞여 숫자가 작아 보임, 속성 접힘 34(글)와 시안 그림 36, 1–3 px 차이(흐린 글자 토큰, 견본 선 길이, 자물쇠 크기, 따로 표 높이, 버튼 사이), 포인터가 펜 위에 있을 때 속성이 잠시 밀려남, ⋯ 첫 항목 이름 「줄 편집…」(5장 ⑦은 「범례 편집」).
  - 구현자 남은 차이 7개: 1·2·5·6은 맞고 받아들일 수 있음, 3은 근거가 부정확(견본값은 앞으로 누를 색만 바꿈, 게다가 지금은 흰색이 칠해짐), 4·7은 고칠 것.

## 재작업 1 (rework 1) — 2026-10-16
- 색 견본(화면 막는 것 1): 엔진이 읽는 `"#rrggbb"`로 보냄. 지금까지 어떤 견본도 제 색을 칠한 적이 없어 지킬 기존 색이 없으므로 시안 색을 씀: 검정 `#000000`(기본 펜 색과 같아 새 도면에서 시안처럼 고리가 보임; 시안 그림은 `#141413`), 적색 `#E8334F`, 청색 `#2B5FB8`, 흑갈 `#4A3426`, 암갈 `#7A5235`, 황갈 `#B08A52`, 회색 `#8C8C8C`, 백색 `#FFFFFF`. 견본 색은 도면에 칠하는 색이라 theme 토큰이 아닌 상수로 둠(어두운 테마가 종이 위 색을 바꾸면 안 됨): 10장 ⑧ 「theme 토큰으로」를 따르지 않은 이유.
- 「이번만 이렇게」(막는 것 2): 누른 것이 고른 선이 있을 때였으면 그 고른 것(id 해시)을 함께 기억해, 도구와 상관없이 그 선들에서는 접힘. 그려서 생긴 것이나 고른 것 없이 누른 것은 전처럼 선택 도구로 나중에 고르면 다시 나옴. 기억에 도면 uid를 넣음.
- 펜 윗변(막는 것 3): 붙잡는 동안 펜을 가장 컸던 높이로 보고 포인터가 그 안에 있으면 그대로 둠. 버튼이 사라진 자리는 빈칸이고 속성은 아래에 그대로. 스크롤 배치에서는 붙잡은 값을 지움.
- 속도(막는 것 2): 펜의 비교 결과를 (도면 uid, 판, 지금 레이어, 펜 굵기·선 종류)로, 속성을 (uid, 판, 축척)으로 기억해 바뀔 때만 다시 셈. 고른 것과 그 레이어는 한 번 훑어 찾음(`selected_in_rows`, 다 찾으면 멈춤), 레이어 모양은 레이어마다 한 번.
- 범례대로(고칠 것 3): 엔진 `legend.revert`에 `pen` 인자. 고른 선이 있으면 선과 펜을 함께 지금 줄 모양으로. 엔진 시험 더함.
- ★(화면 4): 현장 표준 12종의 ★를 평면외곽선·내선(바닥)·단면선 3줄로(7장 표). 이미 만든 도면의 줄은 그대로(새로 넣을 때만).
- 펜 높이(화면 5): 색 줄 26, 펜 192(버튼이 있으면 232).
- 머리 버튼(화면 6): 오른쪽 칸 머리의 버튼을 오른쪽 끝에서 16(시안 ⋯ 자리), 24 간격. 다른 패널 머리는 그대로.
- 속성 ▸/▾(화면 7): 채운 삼각형. 잠긴 자물쇠(화면 8): 시안 `#i-lock`대로 채운 몸통과 고리를 그림(모든 잠긴 줄). 밑그림 이름(화면 9): 숨긴 줄만 흐리게. 위치 단위(화면 10, 코드 사소한 것): 「X … · Y … m」.
- 사소한 것: `Apart.lines` 지움, `sections`·`refused`·`sample`을 모듈 안으로(`badge`는 줄 편집 카드가 씀), 속성 접힘 기억에 uid, 영어 단수형 4줄(「1 line」 등, 한국어 번역 4줄 더함), `SAMPLE_PX`는 굵기 목록 길이에 묶인 픽셀 값만 둠.
- 따르지 않은 것: `even_segments`와 `segments`를 합치는 것. 앞은 칸을 똑같이 나누고 고른 칸은 oat 바탕만(시안 펜 `.seg`), 뒤는 글자 폭대로 나누고 고른 칸에 먹색 테두리(리본·상태줄)라 모양이 달라 둘로 둠.
- 새로 찾은 것(범위 밖, 기록): 엔진 `doc.setScale`은 문서를 바꾸면서 판(revision)을 올리지 않고 되돌리기 단계도 남기지 않음. 그래서 속성의 기억에 축척을 함께 넣음. 좌표계 명령이라 고치려면 사람 검토가 필요해 이번에 손대지 않음.
- 기대값 바꿈과 근거: 펜 높이 198 → 192(화면 5), 범례 머리 버튼 자리 오른쪽에서 24 → 16(화면 6), 밑그림 이름 색 흐림 → 먹색(화면 9, 시안), 「1 lines」 → 「1 line」, 굵기 단계 시험 0.4 → (0.5, 0.3)을 0.5 → (0.6, 0.4)로(9단계를 가리게, 코드 4).
- 새 시험: `just_this_once_folds_for_selected_lines_too`, `the_swatches_paint_their_colours`, `the_pen_stays_put_as_its_buttons_go`, `selected_objects_are_found_with_their_rows`; 더한 확인: 범례대로가 펜도(`the_three_buttons_do_what_they_say`), 위치 TM 값과 북·동 순서(`the_properties_fold_and_show_real_sizes`), 잠긴 자물쇠 채움, 엔진 `revert_brings_overrides_back`의 `pen`.

## 검증 (재작업 1 뒤, 클라우드 컨테이너, Linux)
| 명령 | 종료 코드 | 결과 | 기록 |
|---|---|---|---|
| `cargo fmt --all -- --check` | 0 | 통과 | scratchpad `factory/J023/rework1-verify.log` |
| `cargo clippy -p vectorcraft-ui-egui -p vectorcraft-engine --all-targets -- -D warnings` | 0 | 경고 없음 | 같은 파일 |
| `cargo test -p vectorcraft-ui-egui` | 0 | 1,056 + 6 통과 · 0 실패 | 같은 파일 |
| `cargo test -p vectorcraft-engine --lib -- legend` | 0 | 29 통과 | 같은 파일 |
| `cargo test -p vectorcraft-engine --test trowel_e2e` | 0 | 225 통과 | 같은 파일 |
| `twmissing.py` / `twunused.py` | 0 | 글자열 240개 중 번역 빠짐 0 / 안 쓰는 줄 7개(D3 전부터) | 터미널 |
| 속성 ▸ 크기·색만 바꾼 뒤 `cargo test -p vectorcraft-ui-egui --lib -- trowel i18n`, `cargo fmt --all -- --check` | 0, 0 | 140 통과 · 0 실패 | 터미널 |
| GitHub 「Trowel CI」 | - | 재작업 1 커밋 push 뒤 확인 | - |

## 화면 비교 (재작업 1 뒤, 새로 띄운 앱 1440×880, Xvfb + llvmpipe)
- 그림: scratchpad `shots/d3r1-compare-right.png`(시안 · 새로 띄운 앱 · 현장 표준 12종으로 바꾸고 밑그림 「실측 사진」을 더한 장면 · 그 장면에서 선을 고르고 0.8), `shots/d3r1-compare-differs.png`(7장 그림 · 앱), `shots/d3r1-pen-2x.png`(펜과 속성 머리 2배). 장면 스크립트 `shots/scene_d3r1.py`(★를 따로 켜지 않음: 12종 자체의 ★ 3줄).
- 이번에 맞춘 것: 검정 견본에 고른 고리(새 도면), 펜 높이 192(색 줄 26), 머리 ＋·⋯·▸ 자리, 속성 ▸ 작은 회색 삼각형, ★ 자주 씀 3줄, 「위치 X … · Y … m」.
- 남은 차이:
  1. 새 도면은 범례 줄 1개(★ 굴광선 (유구 면))로 시작한다. 시안은 현장 표준 12종(범위 밖: 9장 새 도면 대화상자).
  2. 「단면빗금」 견본이 빗금 상자가 아니라 선이다. 12종 자료에서 단면빗금과 평면외곽선이 모두 「면」 표시라 가릴 수 없다(줄 자료에 빗금 표시를 더하는 것은 파일 형식 변경이라 사람 결정).
  3. ★ 줄 순서가 레이어 순서를 따른다(평면외곽선 · 내선(바닥) · 단면선). 시안 그림은 평면외곽선 · 단면선 · 내선(바닥).
  4. 줄을 끌어 순서 바꾸기가 없다(오른쪽 클릭 위로·아래로로 대신, 명세 범위 밖).
  5. 「따로」 표는 범례와 다른 선이 있을 때 붙는다. 펜만 다르고 아직 그리지 않았을 때는 펜의 상태 줄이 알린다.
  6. 이 컨테이너에는 굵은 한글 글꼴이 없어 「범례」「펜」「속성」 머리와 고른 칸 「실선」「없음」이 굵게 보이지 않는다(J022와 같음, 사용자 PC에서 따로 확인).
  7. 1–3 px 차이: 흐린 글자 토큰(#73726C, 시안 #9A988F, J001 색), 「따로」 표 높이 16(시안 약 13), 견본 테두리 색, 견본 선 길이 32/30, 열린 자물쇠 11×13 #B0AEA5(시안 9×11 #9A988F), ▸ 5×6(시안 7×7), 펜 192(시안 그림 190, 글은 선 묶음 26), 속성 접힘 34(시안 그림 36, 글 34). (확인 검토: 세 버튼 사이는 이제 4로 시안과 같음.)
  8. 견본 색은 시안 색이지만 검정은 시안 그림의 #141413이 아닌 #000000(기본 펜 색, 고리가 보이게).
  9. 글꼴이 섞여 「굵기 0.6 → 0.8」의 숫자가 한글보다 작아 보인다(이 환경 글꼴).
  10. 버튼이 생길 때 포인터가 펜 위에 있으면 속성 머리가 잠시 칸 밖으로 밀린다(펜 윗변 고정의 설계, 포인터가 떠나면 돌아옴).
  11. 범례 ⋯ 첫 항목 이름 「줄 편집…」(5장 ⑦은 「범례 편집」; 명세가 고른 이름).

## 확인 검토 (재작업 1 뒤, cfaec4f) — 2026-10-16
- 코드 검토: **수정 필요**. 1차 지적 1–10 모두 해결 확인(실행·측정), 막는 것 없음.
  1. (고칠 것, 재작업 1이 만든 회귀) 지금 레이어가 하위 레이어면 펜이 범례 줄을 못 찾음(`doc.layers`에서만 찾음). 하위 레이어 물체를 골랐다 풀기만 해도 생김. 앱에서 재현.
  - 사소한 것: 속성 기억 키가 화면 언어·미리보기 경계 설정을 놓침, 펜 기억 키를 만들 때 지금 레이어를 매 프레임 찾음(받아들임), 큰 묶음에 원인 모를 비용이 칠·선 똑같이 남음(묶음 4,000개 Trowel 42 ms · 클래식 18 ms; D3 때문이던 비용은 사라짐: 선 묶음 8,000개 651 → 151 ms. 다음 성능 작업으로 기록).
- 화면 검토: **남은 차이 있음**. 지난번 막는 것 1–3(견본 색, 선택 도구 「이번만 이렇게」, 버튼이 없어질 때 펜 윗변)과 고칠 것 4–10을 앱에서 눌러 모두 해결로 확인.
  1. (고칠 것) 고른 선의 굵기를 바꾸면 다음에 그릴 선(펜)도 같이 바뀜(엔진 `stroke.set`이 굵기는 늘 다음 선에도 넣음). 「이번만 이렇게」와 겹치면 고른 것을 푼 뒤 펜이 버튼 없이 범례와 다름.
  2. (고칠 것) 고른 선에 누른 「이번만 이렇게」가 그 선을 다시 골라도 계속 접힘.
  - 기록: 검정 고리가 흐림(반투명 2 px), 남은 차이 7번의 「세 버튼 사이 약 6」은 지난 값(지금 4), 목록에 빠진 작은 차이들.

## 재작업 2 (rework 2, 마지막) — 2026-10-16
- 펜이 고른 선만 바꿈(화면 1): 엔진 `stroke.set`에 `newArt: false` 인자(고친 물체가 있을 때 다음 선 굵기를 그대로 둠; 기본은 전과 같아 클래식 화면은 그대로). 펜은 고른 선이 있을 때 이 인자를 붙임. 색 견본은 범례 모양(굵기·선 종류)에 들지 않아 전처럼 다음 선 색도 바꿈.
- 「이번만 이렇게」를 고른 선에 눌렀으면 고른 것이 바뀔 때 잊음(화면 2): 같은 선을 다시 고르면 버튼이 다시 나옴.
- 하위 레이어(코드 1): 지금 레이어의 맨 위 레이어(`layer_of`)로 줄을 찾음. 펜 머리의 이름도 그 줄 이름.
- 속성 기억 키에 화면 언어와 미리보기 경계 설정(코드 사소한 것).
- 검정 고리: 견본 밖 2에서 1.5 두께로 바깥쪽에 그림(시안 box-shadow 2 px 바탕 + 1.5 px 먹).
- 새·바꾼 시험: `the_pen_on_a_sublayer_is_against_its_row`(새), `just_this_once_folds_for_selected_lines_too`(펜 칩으로 고른 선을 바꿔도 다음 선 굵기는 그대로, 같은 선을 다시 고르면 버튼), 엔진 `stroke_set_can_leave_the_next_lines_weight`(새). 기대값 바꿈: 견본 고리 확인을 바깥 테두리 폭(견본 + 4)에 맞춤.
- 수정 루프 2회를 다 씀. 이 뒤에 남는 지적은 needs-human으로 올린다.

## 검증 (재작업 2 뒤, 클라우드 컨테이너, Linux)
| 명령 | 종료 코드 | 결과 | 기록 |
|---|---|---|---|
| `cargo fmt --all -- --check` | 0 | 통과 | scratchpad `factory/J023/rework2-verify.log` |
| `cargo clippy -p vectorcraft-ui-egui -p vectorcraft-engine --all-targets -- -D warnings` | 0 | 경고 없음 | 같은 파일 |
| `cargo test -p vectorcraft-ui-egui` | 0 | 1,057 + 6 통과 · 0 실패 | 같은 파일 |
| `cargo test -p vectorcraft-engine --lib` | 0 | 1,696 통과 · 0 실패 | 같은 파일 |
| `cargo test -p vectorcraft-engine --test trowel_e2e` | 0 | 225 통과 | 같은 파일 |
| GitHub 「Trowel CI」 `cfaec4f`(재작업 1) | - | 통과(push 실행 success) | GitHub Actions |
| GitHub 「Trowel CI」 재작업 2 커밋 | - | push 뒤 확인 | - |
