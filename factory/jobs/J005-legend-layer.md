# J005 — 범례 = 레이어

상태: review (재작업 1차 끝, 재검토 전, 2026-10-11) · 기준 커밋 `5ac2a6ca5000f9071f1e8555b9e9d5b59104c9bd` · **◆ 파일 형식**(레이어에 `legend` 필드, 너그러운 읽기와 값 정리) · **◆ 수치**(선종 대시 길이, 굵기 반올림, 대시 판정 허용치)

## 목표

디자인 2.1(https://claude.ai/artifact/3j9imUvqh8aPeZu8HLb6TZ) 7절 「범례 · 레이어」 판과 그림 ①②③, 그리고 사용자 요구(2026-10-11: 레이어 창이 앱의 중심, 범례 줄 = 이름 + 레이어 + 굵기 + 선종, 줄을 고르면 레이어와 펜이 바뀌고 고른 것이 있으면 그 선이 옮겨 감, 자주 쓰는 줄은 흙색 글자와 ★로 위에, 모든 선의 굵기를 한 번에)를 실제 기능으로 만든다. J003에서 「준비 중 (J005)」으로 둔 칸을 모두 살린다.

- 범례 줄 하나 = 맨 위 레이어 하나. 줄의 모양(굵기 pt + 선종)이 그 레이어 선의 기본 모양이다.
- 줄의 모양을 바꾸면 그 모양을 따르던 선이 함께 바뀌고, 손으로 다르게 바꾼 선은 「따로」로 남아 보인다. 「범례대로」가 되돌린다.
- 굵기 한 번에(비율 · 값 바꾸기 · 한 단계), 평면용 · 단면·입면용 · 유물용 범례 묶음 넣기, 그리다가 「범례에 더하기」.
- 모든 동작은 엔진 명령(`legend.*`)이라 메뉴·MCP·제어 채널에서 똑같이 쓴다. 범례는 도면 파일(`.vectorcraft`)에 함께 저장된다.

## 용어 (이 작업에서 정한 뜻)

- **줄(row)**: 맨 위 레이어. 하위 레이어는 줄이 아니고 그 줄에 속한다.
- **모양(look)**: 줄의 굵기(pt)와 선종. 굵기가 없는 줄(기호·글자·밑그림 줄)은 모양이 없어 선을 바꾸지 않고 「따로」도 없다.
- **선(line)**: 줄 아래(그룹과 하위 레이어 안까지)의 패스·컴파운드 패스 가운데 맨 위 선(stroke)이 있고 그 칠이 「없음」이 아닌 것. 안내선, 클리핑 패스, 컴파운드 안의 패스, 글자·이미지·심볼·블렌드·엔벨로프·반복은 선이 아니다(그 안으로 들어가지도 않는다).
- **따로(override)**: 모양이 있는 줄의 선 가운데 맨 위 선의 굵기가 줄 굵기와 1e-9 pt 넘게 다르거나(기존 기준: 같은 굵기 고르기, `crates/engine/src/cmd/select.rs:101`) 선종이 다른 선. 대시 오프셋·모서리 맞춤·끝 모양·이음·색은 보지 않는다.
- **선종**: 실선 · 파선 · 점선 · 일점쇄선 · 사용자(그 밖의 대시). 길이는 종이 mm로 정해 pt로 바꿔 넣는다(1 mm = 72/25.4 pt). 파선 3.0/1.5, 점선 0.6/1.2, 일점쇄선 3.0/1.2/0.6/1.2 mm — 사용자 앱 Strata의 선 모양 값(hgis `src/core/FeaturePresets.cpp:21-23`, 「구」·「수혈」·「도랑」). 선의 대시가 이 셋 가운데 하나와 원소 수가 같고 원소마다 1e-9 pt 안이면 그 선종, 대시가 없거나 그려지지 않으면(`Dash::is_dashed` 거짓) 실선, 그 밖은 사용자 선종(그 대시 그대로).
- **펜**: 다음에 그릴 선의 모양(새 아트 기본값, `Session::new_art_stroke`). 펜·연필·선·사각형 등 모든 그리기 도구에 쓰인다.

## 완료 기준

엔진 시험은 `crates/engine/src/tests_legend.rs`(새 파일), 화면 시험은 `crates/ui-egui/src/trowel/tests.rs`에 두고, 따로 적은 것은 그 파일에 둔다. 화면 시험은 J003처럼 egui 헤드리스 프레임에서 실제로 그리고 누른다.

### 문서 모형과 파일 형식 ◆

1. **범례 모형.** 새 파일 `crates/doc/src/legend.rs`(`pub mod legend`, `pub use legend::{Legend, LegendGroup, LegendSample, LineType}`):
   - `Legend { weight: Option<f64>, line: LineType, favourite: bool, sample: LegendSample, strata: Option<String>, group: Option<LegendGroup> }`(모두 serde 기본값, 기본값과 같은 필드는 쓰지 않음). `weight`는 pt.
   - `LineType { Solid, Dashed, Dotted, DashDot, Custom(Vec<f64>) }` — 파일에는 `"solid" | "dashed" | "dotted" | "dashdot" | {"custom": [pt…]}`. `dash_pt() -> Option<Vec<f64>>`(위 mm 값 × 72/25.4, 실선 `None`), `of(&StrokeLayer) -> LineType`(용어의 판정).
   - `LegendSample { Line, Area, Mark(String) }` — 줄 앞 보기 그림(선·면·기호 글자). 파일에는 `"line" | "area" | {"mark": "△"}`.
   - `LegendGroup { Plan, Section, Finds }` — 파일에는 `"plan" | "section" | "finds"`.
   - 함수: `Legend::look() -> Option<(f64, &LineType)>`(굵기가 있을 때만), `lines(layer: &Node) -> Vec<NodeId>`(용어의 「선」, 위에서 아래 순서), `is_override(legend, line: &Node) -> bool`, `Document::legend(layer) -> Option<&Legend>`.
   - 시험(`legend::tests`): `dash_lengths_are_stratas_paper_mm`(세 선종의 pt 값 = mm × 72/25.4, 실선은 대시 없음), `line_type_is_read_back_from_a_stroke`(세 값 → 그 선종, 대시 없음·0만 있는 대시 → 실선, 다른 대시 → 사용자, 1e-9 안팎 경계), `lines_are_stroked_paths_and_compounds_under_groups_and_sublayers`(글자·이미지·안내선·클리핑 패스·칠 없는 선·컴파운드 안 패스 제외, 하위 레이어와 그룹 안은 포함), `an_override_differs_in_weight_or_line_type`(1e-9 경계, 오프셋·끝 모양은 무시, 굵기 없는 줄은 따로 없음).
2. **레이어에 범례.** `NodeKind::Layer`에 `legend: Option<Box<Legend>>`(`#[serde(default, skip_serializing_if = "Option::is_none")]`). 만드는 곳 2곳(`crates/doc/src/node.rs:623` `Node::layer`, `crates/eps/src/import/ai/read.rs:621`)은 `legend: None`. 파일 형식 판(VERSION 4)은 그대로. 시험 `crates/format/tests/legend.rs`(새 파일): `a_legend_survives_save_and_open`(모든 필드와 사용자 선종·기호 보기, 저장 → 열기 → 같음), `a_layer_without_a_legend_saves_as_before`(범례 없는 문서의 저장 JSON에 `legend` 키가 없음), `older_files_open_without_a_legend`(`legend` 없는 판 4 JSON → `None`), `unknown_layer_fields_are_ignored`(모르는 레이어 키가 있는 JSON도 열림 — 옛 판이 새 파일을 여는 근거). 기존 형식 시험(`prop_format`, `native_v3`, `native_v4`, `save_options`, `links`)은 고치지 않고 통과.
3. **문서 조사.** `document.inspect`(레이어 JSON, `crates/engine/src/inspect.rs:70-77`)에 `legend`(범례가 있을 때 그 JSON, 없으면 `null`). 시험 `inspect_reports_the_legend`.

### 엔진 명령 (`crates/engine/src/cmd/legend.rs`, 메뉴 자리 없음 `[]`)

모든 명령은 문서가 없으면 「no document open」, 잘못된 인자는 `EngineError`(어느 인자가 왜 틀렸는지)로 끝나고 패닉하지 않는다. 문서를 바꾸는 명령은 실행 한 번이 실행 취소 한 번이다. `id`는 맨 위 레이어여야 한다(하위 레이어·객체 id는 오류).

4. **`legend.set` — 「Set Legend Entry」.** `{id?: 레이어(기본 현재 레이어), weight?: pt|null, line?: solid|dashed|dotted|dashdot|[d,g,…], favourite?: bool, sample?: line|area|{mark}, strata?: 키|null, group?: plan|section|finds|null, clear?: true}`. 범례가 없는 레이어면 기본 범례를 만든 뒤 적는다. `clear`는 범례를 지운다(선은 그대로). 굵기는 유한한 수여야 하고 0–1000 pt로 자른다(기존 `stroke.set`과 같은 범위). **모양이 바뀌면 바꾸기 전 모양을 따르던 선(따로가 아닌 선)이 새 굵기·선종을 받는다.** 따로인 선, 잠긴 선(자신·조상 잠김)은 그대로다. 바꾸기 전 모양이 없었으면(처음 모양을 줌) 선은 그대로이고, 다른 선은 「따로」로 보인다. 선종을 바꿀 때 대시 오프셋·모서리 맞춤은 지금 값을 둔다(`stroke.set`의 새 무늬 규칙). 줄(레이어)이 잠겨 있으면 오류. 결과 `{id, legend, lines: 바뀐 선 수}`. 시험: `setting_a_row_restyles_its_matching_lines_and_keeps_overrides`(0.8 실선 줄, 맞는 선 3 + 따로 0.5 하나 → 0.6: 셋만 0.6, 실행 취소 한 번에 모두 되돌아옴), `a_new_look_leaves_existing_lines_alone`, `line_type_follows_the_row`(실선 → 파선: 맞는 선이 파선 대시를 받고 끝 모양·색은 그대로), `locked_rows_refuse_and_locked_lines_are_skipped`, `legend_set_checks_its_params`(NaN·문자 굵기, 모르는 선종, 하위 레이어 id, 빈 인자 → 오류이고 문서 그대로).
5. **`legend.pick` — 「Use Legend Entry」.** `{id, move?: bool}`. 그 줄을 현재 레이어로 하고(`layer.setCurrent`처럼 실행 취소 단계가 아님), 줄에 모양이 있으면 펜이 그 굵기와 선종(대시)을 받는다(색·끝 모양은 그대로). `move: true`이고 고른 객체가 있으면, 그 줄 밖의 고른 객체를 그 줄 맨 위로 옮기고(고른 순서대로, 선택은 유지), 옮긴 선은 줄 모양을 받는다 — 이 옮김이 실행 취소 한 번. 줄이 잠겼거나 숨겨져 있으면 옮기지 않고 오류(문서·현재 레이어 그대로). 결과 `{id, moved}`. 시험: `picking_a_row_sets_the_current_layer_and_the_pen`(실행 취소 기록 없음, 모양 없는 줄은 펜 그대로), `picking_with_move_brings_the_selection_and_its_look`(다른 레이어의 선 2개 → 줄 맨 위, 0.8, 이미 그 줄에 있던 고른 선은 제자리, 실행 취소 한 번에 제자리·제 굵기), `picking_a_locked_or_hidden_row_does_not_move`.
6. **`legend.revert` — 「Revert to the Legend」.** `{ids?: [id…], all?: bool}`. `ids`의 선, `all`이면 도면의 모든 따로 선, 둘 다 없으면 고른 객체(그룹이면 그 안의 선)가 제 줄의 모양으로 돌아간다(굵기와 대시만). 모양 없는 줄의 선·잠긴 선은 건너뛴다. 고른 것도 `ids`·`all`도 없으면 펜이 현재 줄의 모양을 받는다(문서 그대로). 결과 `{lines, skipped}`. 시험 `revert_brings_overrides_back`(ids · 선택 · all · 아무것도 없을 때 펜, 잠긴 선 건너뜀, 실행 취소 한 번).
7. **`legend.weights` — 「Change Weights at Once」.** `{how: ratio|value|step, ratio?: >0, from?: pt, to?: pt, step?: 1|-1, overrides?: bool}`.
   - 모양 있는 줄의 굵기를 규칙대로 바꾸고, 그 모양을 따르던 선은 줄을 따라 바뀐다. 따로 선은 `overrides: true`일 때만 제 굵기에 같은 규칙을 쓴다. 모양 없는 줄·범례 없는 레이어의 선도 제 굵기에 같은 규칙을 쓴다.
   - `ratio`: × 비율. `value`: 굵기가 `from`과 1e-9 안인 것만 `to`로. `step`: 칩 사다리 0.1 · 0.2 · 0.3 · 0.5 · 0.8 · 1.0(J003 팔레트 칩, 디자인 2.1 7절)에서 지금 값보다 1e-9 넘게 큰(작은) 가장 가까운 값, 없으면(끝) 그대로.
   - 결과는 0.001 pt로 반올림하고(기존 pt 칸 표시 자리수, `crates/doc/src/lib.rs:206-208`) 0–1000 pt로 자른다.
   - 잠긴 줄·선은 건너뛰고 센다. 숨긴 레이어·객체의 선은 바꾼다.
   - 미리 보기는 명령이 아니라 엔진 함수 `cmd::legend::plan_weights(&Document, &인자) -> WeightPlan`(명령과 같은 인자·같은 계산, 문서를 바꾸지 않음)로 화면이 읽는다.
   - 결과 `{rows: [{id, name, from, to}], lines, skipped}`.
   - 시험: `weights_by_ratio_round_to_a_thousandth`(1.0 → 0.8, 0.8 → 0.64, 0.5 → 0.4, 0.2 → 0.16, 0.25 × 0.8 = 0.2, 0.7087 × 0.8 → 0.567, 맞는 선은 줄을 따르고 따로는 그대로, `overrides`면 따로도, 범례 없는 레이어의 선도, 잠긴 선은 그대로이고 `skipped`에 셈, 숨긴 레이어는 바뀜, 실행 취소 한 번), `weights_by_value_change_only_that_weight`, `weights_step_along_the_chip_ladder`(0.5 ↑ 0.8, 0.6 ↓ 0.5, 0.6 ↑ 0.8, 1.0 ↑ 그대로, 0.1 ↓ 그대로, 0 ↑ 0.1), `weights_plan_matches_the_command`(같은 인자의 미리 보기와 실행 결과가 같음), `legend_weights_checks_its_params`(how 없음, 비율 0·음수·NaN, from 없이 value, step 2 → 오류이고 문서 그대로).
8. **`legend.addGroup` — 「Add Legend Group」.** `{group: plan|section|finds}`. 디자인 2.1 7절 표의 줄을 맨 위 레이어로 표 순서대로(표의 첫 줄이 맨 위) 도면 맨 위에 더한다. 같은 Strata 키나 같은 이름(앞뒤 공백 무시)의 레이어가 이미 있으면 그 줄은 건너뛴다(기존 레이어는 고치지 않는다). 레이어 색은 `layer.new`와 같은 차례. 하나도 더하지 않으면 실행 취소 기록도 없다. 결과 `{added: [id…], skipped: n}`. 표(이름 · 굵기 pt · 선종 · 보기 · Strata 키 · ★ · 레이어 옵션; 「추정선」은 줄이 아니라 J009의 켜고 끄기라 뺀다):

   | 묶음 | 줄 |
   |---|---|
   | 평면 15 | 조사구역 경계 1.0 일점쇄선 면 `survey_area` · 시굴격자 0.5 면 `trial_trench` · ★굴광선 (유구 면) 0.8 면 `feature_poly` · ★바닥선 0.5 면 · 유구 선 (구·벽구) 0.5 선 `feature_line` · ★단면 위치선 0.2 일점쇄선 선 `section_line` · 기준점 굵기 없음 기호「△」 `control_points` · 유물 출토 위치 0.2 기호「③」 `artifact_point` · 내부 시설 0.5 면 · 돌 0.5 면 · 경사 표시 0.2 면 · 표고점 굵기 없음 기호「·152」 · 격자 · 도곽 0.1 선 · 글자 · 번호 굵기 없음 기호「가」 · 밑그림 굵기 없음 면, 템플릿 |
   | 단면·입면 12 | ★단면선 (잘린 땅) 0.8 선 · ★잘린 돌·유물·기와 0.8 면 · 보이는 돌·유물 0.5 면 · ★해발선 50 cm 0.5 선 · 해발선 10 cm 0.2 선 · 해발 숫자 굵기 없음 기호「152.5m」 · ★토층선 0.5 선 · 불분명한 토층선 0.5 점선 선 · 교란 0.5 면 · 생토 굵기 없음 면 · 토층 번호 굵기 없음 기호「①」 · desca 원본 굵기 없음 선, 템플릿·숨김 |
   | 유물 7 | ★외형선 0.8 선 · ★단면 0.8 면 · 내면선 0.5 선 · 중심선 0.2 일점쇄선 선 · 문양 · 조정흔 0.2 선 · 복원선 0.5 파선 선 · 계측선 굵기 없음 선, 인쇄 안 함 |

   선종을 적지 않은 줄은 실선. 템플릿은 `layer.setProps`의 템플릿과 같은 옵션. 시험: `adding_the_plan_group_makes_its_rows_in_order`(15개, 이름·굵기·선종·★·Strata·보기·템플릿, 첫 줄이 맨 위, 다시 넣으면 하나도 더하지 않고 실행 취소 기록 없음, 실행 취소 한 번에 모두 사라짐), `section_and_finds_groups_set_their_layer_options`(desca 원본 숨김·템플릿, 계측선 인쇄 안 함), `rows_already_there_are_not_added_twice`(같은 Strata 키·같은 이름).
9. **`legend.add` — 「Add to Legend」.** `{name?}`. 현재 레이어 바로 위(없으면 맨 위)에 새 줄을 만들고 현재 레이어로 한다. 모양은 고른 첫 선의 맨 위 선(굵기 + 선종 판정), 고른 선이 없으면 펜. 보기는 그 선이 닫혀 있으면 면, 아니면 선. 고른 객체는 새 줄로 옮기고 제 모양을 지킨다(다른 모양의 선은 「따로」). 이름이 없으면 `Document::next_layer_name()`. 레이어 만들기와 옮기기가 실행 취소 한 번. 결과 `{id, moved}`. 시험 `adding_a_row_takes_the_selected_lines_look_and_moves_them`(고른 선 0.35 파선 → 새 줄 0.35 파선 · 옮겨짐 · 현재 레이어, 실행 취소 한 번), `adding_a_row_with_nothing_selected_takes_the_pen`.
10. **명령 목록과 번역.** 6개 명령이 `command_specs()`에 들어가고 기존 시험 `tests::every_command_has_unique_id_and_doc`(`crates/engine/src/tests.rs:619`), `command_sweep`(빈 인자·문서 없음·엉터리 인자·`every_command_undoes_exactly`)을 고치지 않고 통과. 명령 이름 6개를 메뉴 완성 언어 10개(ja, ko, de, es, fr, it, ru, uk, zh-hans, zh-hant; `crates/ui-egui/src/i18n/mod.rs`의 `complete_menus`) 카탈로그에 더해 `i18n::tests::complete_languages_translate_every_menu_string`가 통과.

### 화면 (Trowel 오른쪽 기둥과 메뉴)

11. **범례 줄 모양.** 줄(24 px) = ▸ · 눈 · 자물쇠 · **보기**(20 × 12: 모양 있는 줄은 그 굵기를 화면 px로 바꾼 선 또는 사각형, 선종 대시; 기호 줄은 그 글자; 굵기 없는 줄은 1 px 흐린 선/사각형; 범례 없는 레이어는 J003처럼 레이어 색 패치) · **★**(자주 씀, `clay`) · 이름(★ 줄은 `clay_text` 굵게; 현재 레이어는 J003처럼 귀리색 바탕이고 글자는 보통 굵기 — 시안 `.lrow.sel`과 J003 검토 결정) · **「따로 N」 표**(caution 글자·테두리·바탕, N > 0일 때) · **굵기**(mono 10 「0.8」, 모양 있는 줄만) · 객체 수. 화면 px 굵기 = 디자인 2.1 표의 값(1.0 → 2.5, 0.8 → 2.0, 0.5 → 1.25, 0.3 → 0.85, 0.2 → 0.6, 0.1 → 0.45, 그 사이는 선형). 색은 모두 `Tokens`(새 토큰 `caution`, `caution_edge`, `caution_wash` — Plate #7A5A00 · #D9C27A · #F7EFD6, 다른 프리셋은 제 팔레트에서). 시험 `legend_rows_show_the_look_star_and_overrides`, `theme::tests::every_preset_has_the_caution_tokens`.
12. **줄 순서와 묶음 제목.** Strata 키가 있는 줄이 하나라도 있으면 「조사 데이터 · Strata와 같은 레이어」(키 있는 줄) 다음 「그리기 · Trowel」(나머지 줄과 범례 없는 레이어)으로 나누고, 없으면 J003의 「이 도면의 레이어」 하나. 묶음 안에서는 ★ 줄이 먼저, 그다음 레이어 순서(위에서 아래). 시험 `favourites_come_first_in_each_section`, `strata_rows_have_their_own_section`.
13. **줄 누르기.** 누르면 `legend.pick {id}`. 선택 도구(기존 `is_selection_tool`의 selection · directSelection · groupSelection, `crates/tools/src/catalog.rs:159`, 그리고 lasso · magicWand)를 들고 고른 객체가 있으면 `move: true`(위험 절의 결정 1). 상태줄에 「선 N개를 「{줄}」로 옮겼습니다」. 눈·자물쇠·▸는 J003 그대로. 컨트롤 바 범례 칸의 목록도 `legend.pick`(옮기지 않음)으로 바꾸고 색 패치 대신 보기를 그린다. 시험 `clicking_a_row_picks_it_for_the_pen`(현재 레이어·펜 굵기·대시), `clicking_a_row_with_the_selection_tool_moves_the_selection`, `clicking_a_row_while_drawing_does_not_move_the_new_line`(펜 도구 + 고른 선 → 옮기지 않음).
14. **펼친 객체 줄.** 따로 선의 객체 줄 오른쪽에 「{굵기} 따로 ↺」, ↺를 누르면 `legend.revert {ids: [그 선]}`. 시험 `object_rows_show_overrides_and_revert`.
15. **머리 묶음 단추.** 머리 부제(J003 「레이어 {n}」) 자리에 묶음 단추: 줄이 가장 많은 묶음 「평면용 ▾」/「단면·입면용 ▾」/「유물용 ▾」(같으면 평면 · 단면 · 유물 순), 묶음 줄이 없으면 「범례 넣기 ▾」. 누르면 「평면용 범례 넣기 · 단면·입면용 범례 넣기 · 유물용 범례 넣기」 → `legend.addGroup`, 상태줄 「범례 줄 N개를 넣었습니다」(0이면 「더할 줄이 없습니다」). 시험 `the_group_menu_adds_a_legend_group`.
16. **바닥 「굵기 한 번에」.** J003의 준비 중 단추 두 개를 살린다: 「모든 굵기 ×0.8 …」 → 굵기 한 번에 카드(18번), 「따로 N → 범례대로」 → `legend.revert {all: true}`(N = 도면의 따로 수, 0이면 흐리고 누를 수 없음). 시험 `the_footer_opens_the_weights_card_and_reverts_all_overrides`.
17. **줄 고치기 카드(그림 ②).** 줄을 두 번 누르거나 「범례에 더하기」 뒤에 오른쪽 기둥 왼쪽에 304 px 카드로 뜬다(그 줄 높이에 맞추고 창 안으로). ✕ · Esc · 카드 밖 누르기로 닫힌다. 내용:
    - 머리 「범례 줄」 + 줄 이름.
    - 이름(글자 칸, Enter·칸 떠남 → `layer.setProps` 이름), Strata(읽기만: 「유구_면 · feature_poly」, 없으면 「없음」; 고르기는 J019 준비 중) + 「이 줄이 곧 레이어입니다」.
    - 굵기: 칩 0.1 … 1.0(지금 굵기와 같으면 켜짐) + 숫자 칸 「{w} pt」 + 「= {mm} mm」(pt × 25.4/72, 소수 둘째 자리) → `legend.set {weight}`. 「굵기 없음」 칩 → `weight: null`.
    - 선종: 실선 · 파선 · 점선 · 일점쇄선(사용자 선종이면 아무것도 켜지지 않고 「사용자」 표시) → `legend.set {line}`.
    - 추정(J009) · 빗금(J014) · 글자(J012): 흐린 준비 중.
    - ★ 자주 씀 켜고 끄기 → `legend.set {favourite}`.
    - 이 줄의 선: 「{N}개」 · 「따로 {M}」 · 「모두 고르기」(그 줄의 선을 모두 고름) · 「범례대로」(`legend.revert {ids: 그 줄의 따로 선}`).
    - 저장: 「이 조사에만(J004) · 내 기본값(J018) · 양식 파일(J018)」 흐린 준비 중 + 「지금은 이 도면 파일에 저장됩니다」. 「레이어 옵션…」 단추 → `ui.layerOptions`(J003 명세 11번에 적은 「두 번 누르면 레이어 옵션」은 J003 코드에 없다. 두 번 누르기는 이제 이 카드를 연다).
    - 시험 `double_clicking_a_row_opens_its_editor`(내용·닫기 세 가지), `the_row_editor_changes_the_row_and_its_lines`(칩 0.5 → 맞는 선 0.5 · 따로 그대로, 숫자 칸, 선종, ★, 이름, 모두 고르기, 범례대로).
18. **굵기 한 번에 카드(그림 ③).** 바닥 단추나 「도면 › 굵기 한 번에…」로 같은 자리에 뜬다. 어디: 「이 도면」 켬, 「도면집 전체」 준비 중(J004). 어떻게: 비율(칸 ×0.8, 0.1–10) · 값 바꾸기(「0.5 → 0.4」 두 칸) · 한 단계(「한 단계 가늘게」 기본 · 「한 단계 굵게」). 따로: 「따로 바꾼 선도 함께」 켜고 끄기. 미리 보기 표: 범례 줄 · 지금 · 바꾼 뒤 — 같은 (지금, 바꾼 뒤)인 줄은 「 · 」로 묶음, 그 아래 「선 {N}개가 바뀝니다. Ctrl+Z 한 번이면 모두 돌아옵니다.」, 잠긴 것이 있으면 「잠긴 선 {M}개는 그대로 둡니다.」 단추 「바꾸기」(action 색) → `legend.weights`, 「닫기」. 시험 `weights_card_previews_and_applies_a_ratio`(표 값 = 엔진 계획, 바꾸기 뒤 굵기, Ctrl+Z 한 번), `weights_card_value_and_step_modes`.
19. **그리기 팔레트.** 선종 칩 네 개를 살린다: 고른 선(없으면 펜)에 `stroke.set {dash: 그 선종의 대시}`(실선은 `null`), 고른 첫 선(없으면 펜)의 선종 칩이 켜짐. 「범례에 더하기」 → `legend.add {name: 「범례 줄 {n}」}` 뒤 그 줄의 고치기 카드. 「범례대로」 → `legend.revert {}`. **F1(J003 검토 3차)** 고른 선종 칸의 먹색 테두리가 네 변 모두 보이게(선종 줄을 새로 그리므로 여기서 고친다). 시험 `palette_line_types_set_the_dash`, `palette_legend_buttons_add_and_revert`, `the_chosen_line_type_chip_keeps_its_border`.
20. **도면 메뉴.** 「굵기 한 번에…」 → 18번 카드, 「범례대로 되돌리기」 → 고른 것이 있으면 `legend.revert {}`, 없으면 `{all: true}`. 나머지 7항목은 J003 그대로 준비 중. 시험 `drawing_menu_opens_the_weights_card_and_reverts`.
21. **F2(J003 검토 3차).** 범례 목록의 스크롤 흐림(egui 기본 20 px · 50 %)을 그 목록에서만 끈다 — 넘칠 때 보이는 끝 줄이 숨긴 레이어처럼 흐려 보이지 않게. 시험 `the_legend_list_does_not_fade_its_last_row`.
22. **번역.** 새 `tw!` 글자는 모두 `ko.tsv`에(기존 시험 `every_tw_literal_is_translated_in_korean`). 묶음 줄 이름은 문서 내용이라 한국어로만 들어간다(위험 절).
23. **J003 시험 변경**(커밋에 `시험 변경:` 줄로 적음): `palette_weight_chip_sets_the_next_stroke`(선종 세 칩이 준비 중 목록에서 빠짐 — J005가 살림), `drawing_menu_sits_before_view_and_waits_for_its_jobs`(J005 두 항목이 켜짐), `status_bar_shows_messages_for_a_while`(누르던 「모든 굵기 바꾸기」가 살아나 다른 준비 중 단추 「잘린 면」(J014)으로 바꿈 — 확인하는 것은 같음). 그 밖의 J003 시험은 고치지 않고 통과.
24. **실제 화면.** J003과 같은 방법(`VECTORCRAFT_LOCALE=ko VECTORCRAFT_NO_PREFS=1 xvfb-run …`, 제어 채널)으로 1600 × 988 창에 「평면용 범례 넣기」 + 선 몇 개(따로 하나 포함)를 그린 문서를 띄워 찍는다: 평면 화면, 줄 고치기 카드, 굵기 한 번에 카드, 펼친 줄의 「따로 ↺」. 시안(`win-plan.png`, 7절 그림 ②③)과 나란히 붙인 그림과 남은 차이를 검증 절에 적는다. 그림은 `/mnt/project-files/design/trowel-run/J005-*.png`(커밋하지 않음).
25. 고전 화면 불변(`trowel`이 꺼진 화면의 동작·시험 그대로), `cargo check --target wasm32-unknown-unknown -p vectorcraft-web` exit 0, `cargo xtask ci` exit 0.

## 변경 범위

- 새 파일: `crates/doc/src/legend.rs`, `crates/engine/src/cmd/legend.rs`, `crates/engine/src/tests_legend.rs`, `crates/format/tests/legend.rs`, 화면 카드 `crates/ui-egui/src/trowel/{rowedit,weights}.rs`.
- `crates/doc/src/{lib,node}.rs`: 모듈·다시 내보내기, `NodeKind::Layer.legend`, `Document::legend`. `crates/eps/src/import/ai/read.rs:621`: `legend: None`.
- `crates/engine/src/{cmd/mod.rs,lib.rs,inspect.rs}`: 명령 등록, 시험 모듈, 레이어 JSON.
- `crates/ui-egui/src/trowel/{legend,palette,menu,control,mod,tests}.rs`, `crates/ui-egui/src/theme.rs`(토큰 3개 + 시험), `crates/ui-egui/src/i18n/*.tsv`(행 추가만: ko는 화면 글자와 명령 이름, 다른 9개 언어는 명령 이름 6개).
- `factory/backlog.md`, 이 파일.
- **바꾸지 않는 것**: 기존 명령의 id·인자·동작(`stroke.set`, `layer.*` 등), 파일 형식 판(4), 기존 노드 필드, 고전 화면, 다른 크레이트(`eps`의 한 줄 말고), 환경설정 키, 기존 토큰 값, `docs/`의 진척 문서(J003과 같게 Trowel 작업 묶음이 끝날 때 한꺼번에).

## 검증 방법

로그는 `/tmp/claude-0/-home-claude/79152f61-3d49-5a1c-acde-dc6cabf2d3d7/scratchpad/factory/J005/*.log`, 종료 코드를 모두 적는다. 명령은 `/home/claude/vector`에서 `CARGO_PROFILE_DEV_DEBUG=0`(디스크를 아끼려고 디버그 정보만 끔).

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p vectorcraft-doc --lib -- legend::
cargo test -p vectorcraft-format --test legend
cargo test -p vectorcraft-engine --lib -- tests_legend:: tests::every_command_has_unique_id_and_doc
cargo test -p vectorcraft-engine --test command_sweep
cargo test -p vectorcraft-ui-egui --lib -- trowel:: theme:: i18n::
cargo xtask ci
cargo check --target wasm32-unknown-unknown -p vectorcraft-web
```

시험 먼저: 위 시험을 먼저 쓰고, 함수·명령이 없어 실패하는 빨강 로그를 남긴 뒤 구현한다. 실제 화면(24번)은 `cargo build -p vectorcraft` 뒤 J003의 `drive3.py`를 J005용으로 늘려 찍는다.

## 범위 밖

- 빗금(J014), 추정선 켜고 끄기(J009), 이름표 글자 크기(J012), 저장할 곳 「이 조사에만」(J004 — 도면집 폴더가 J004에서 생김), 「내 기본값」·양식 파일(J018), 굵기 한 번에의 「도면집 전체」(J004), 범례표(J012), 기호 그리기 도구(J009), Strata 레이어 고르기·보내기(J019).
- 축척과 종이 단위(J006): 지금은 범례 굵기를 문서 pt에 그대로 쓴다.
- 범례 목록에서 끌어서 순서 바꾸기(레이어 패널 › ⋯에 있음), 하위 레이어 줄.
- 다른 언어의 Trowel 화면 글자(명령 이름 6개만 메뉴 완성 언어 10개에. cs·pt-br는 기존처럼 영어로 보임).

## 위험·사람 검토 필요

- **◆ 파일 형식.** 레이어에 `legend` 필드가 생긴다. 형식 판은 4 그대로(더하기만 한 선택 필드). 옛 파일은 범례 없이 열리고, 범례 없는 문서는 전과 같은 JSON으로 저장된다. 옛 판 앱(J005 이전)은 새 파일을 열 수 있지만(모르는 필드 무시) 범례를 모르므로, 옛 판에서 다시 저장하면 범례가 사라진다. 문서 수준의 모르는 키는 지키지만(`extra`) 노드 수준은 지키지 않는 기존 규칙 때문이다.
- **◆ 수치 — 선종 길이.** 파선 3.0/1.5, 점선 0.6/1.2, 일점쇄선 3.0/1.2/0.6/1.2 mm(Strata의 구·수혈·도랑 선 값). 한국 발굴 보고서 관례에서 정해진 길이를 찾지 못해 사용자 앱의 값을 따랐다. 사람 확인.
- **◆ 수치 — 반올림과 범위.** 굵기 한 번에 결과 0.001 pt 반올림(기존 pt 칸 자리수), 0–1000 pt(기존 `stroke.set`). 한 단계는 칩 사다리, 끝에서는 그대로.
- **◆ 수치 — 대시 판정 허용치 1e-9 pt.** 대시에는 기존 기준이 없어 굵기 기준(1e-9, `select.rs:101`)을 썼다. 이유: Trowel이 넣는 대시는 같은 상수로 계산해 저장·열기 뒤에도 같은 비트이고, 다른 앱에서 둥근 값으로 들어온 대시는 「사용자」 선종으로 보여 줄 뿐 바꾸지 않는다. 사용자 확인 필요.
- **단위.** 범례 굵기는 문서 pt에 그대로 쓴다. 축척(J006)이 들어오면 종이 pt와 문서 단위가 갈라질 수 있어 J006에서 다시 본다. 줄 고치기 카드의 mm 표시는 표시만(저장은 pt).
- **동작 결정(보고에 적어 사람 확인).**
  1. 줄을 누를 때 고른 선을 옮기는 것은 선택 도구를 들었을 때만이다. 펜·연필로 막 그린 선은 고른 채로 남아 있어, 다음 선을 그리려고 다른 줄을 누를 때 방금 그린 선이 따라 옮겨 가는 사고를 막으려고 사용자 말씀(「고른 것이 있으면 그 선이 옮겨 감」)을 좁혀 읽었다.
  2. 처음 모양을 주는 줄의 기존 선은 바꾸지 않고 「따로」로 보인다. 「범례대로」 한 번으로 맞춘다(말없이 그림을 바꾸지 않으려고).
  3. ★ 줄은 묶음 안에서 위로 올린다. 디자인 글(「★는 처음부터 자주 쓰는 줄로 위에 고정합니다」)과 사용자 말씀을 따랐고, 시안 그림의 줄 순서(레이어 순서 그대로)와는 다르다.
  4. 굵기 한 번에는 숨긴 레이어·객체의 선도 바꾸고, 잠긴 것은 건너뛰고 수를 알린다.
  5. 묶음 넣기는 같은 Strata 키나 이름의 레이어가 있으면 그 줄을 건너뛰고 기존 레이어는 고치지 않는다.
- 묶음 줄 이름(「굴광선 (유구 면)」 …)과 Strata 이름은 문서 내용이라 화면 언어와 상관없이 한국어로 들어간다.
- 명령 이름 6개의 외국어 9개 번역은 내가 넣는다(원어민 확인 없음, J003 한국어와 같은 처지).
- J003 승인 카드의 답과 상관없이 F1·F2를 여기서 고친다(이 작업이 그 코드를 새로 그림). 사용자가 J003을 「고친 뒤 승인」으로 고르면 고친 커밋이 J005라고 알린다.
- 공장 규칙의 「별도 git worktree」 대신 작업 브랜치 제자리에서 만든다(디스크 여유 22 GB, worktree와 따로 둔 target은 25 GB 넘게 듦). J001–J003과 같은 예외.

## 기준 커밋

`5ac2a6ca5000f9071f1e8555b9e9d5b59104c9bd` (`git rev-parse HEAD`, 2026-10-11)

## 빌드 (build)

2026-10-11, 기준 `5ac2a6c`(그대로). 작업 브랜치 제자리(위험 절의 예외). 시험을 먼저 쓰고 빨강을 본 뒤 구현했다. 로그는 `/tmp/claude-0/-home-claude/79152f61-3d49-5a1c-acde-dc6cabf2d3d7/scratchpad/factory/J005/`.

손댄 파일:

- 새 파일: `crates/doc/src/legend.rs`(344줄), `crates/engine/src/cmd/legend.rs`(681), `crates/engine/src/tests_legend.rs`(698), `crates/format/tests/legend.rs`(78), `crates/ui-egui/src/trowel/{card,rowedit,weights}.rs`(206 · 275 · 265).
- 고친 파일: `crates/doc/src/{lib,node}.rs`, `crates/eps/src/import/ai/read.rs`(한 줄), `crates/engine/src/{lib,inspect}.rs`, `crates/engine/src/cmd/{mod,layer}.rs`(`locked_within`·`apply_options`를 `pub(crate)`로), `crates/ui-egui/src/theme.rs`, `crates/ui-egui/src/widgets.rs`(`select_all_on_focus`를 `pub(crate)`로), `crates/ui-egui/src/trowel/{mod,legend,palette,menu,control,tests}.rs`, `crates/ui-egui/src/i18n/{ko,ja,de,es,fr,it,ru,uk,zh-hans,zh-hant}.tsv`(행 추가, ko는 J003 행 1개 번역 바꿈·1개 뺌), `factory/backlog.md`, 이 파일.

완료 기준별 구현:

| 기준 | 구현 |
|---|---|
| 1 | `doc::legend`: `Legend`·`LineType`(`dash_pt`·`from_dash`·`of`·`same`)·`LegendSample`·`LegendGroup`, `lines`·`stroke_of`, `Legend::{look, differs}`, `Node::legend`, `Document::legend` |
| 2 | `NodeKind::Layer.legend`(`Option<Box<Legend>>`, 없으면 쓰지 않음), 만드는 곳 2곳 `legend: None` |
| 3 | `inspect.rs` 레이어 JSON에 `legend`(있을 때만 — 아래 벗어난 점 1) |
| 4–9 | `cmd/legend.rs`: `legend.set`·`pick`·`revert`·`weights`(+ `plan_weights`)·`addGroup`·`add`, 칩 사다리 `LADDER` |
| 10 | 명령 6개 `command_specs()`에, 이름 6개를 메뉴 완성 언어 10개 카탈로그에 |
| 11 | `trowel/legend.rs` 줄 그리기(보기·★·「따로 N」 표·굵기·객체 수), 토큰 `caution`·`caution_edge`·`caution_wash` |
| 12 | 줄 묶음 「조사 데이터 · Strata와 같은 레이어」/「그리기 · Trowel」, ★ 먼저 |
| 13 | 줄 누르기 → `legend.pick`(선택 도구일 때 `move`), 컨트롤 바 범례 칸과 목록이 보기를 그리고 `legend.pick` |
| 14 | 펼친 객체 줄 「{굵기} 따로 ↺」 |
| 15 | 머리 묶음 단추와 메뉴 → `legend.addGroup` |
| 16 | 바닥 「모든 굵기 ×0.8 …」 → 굵기 한 번에 카드, 「따로 N → 범례대로」 → `legend.revert {all}` |
| 17 | `trowel/card.rs`(카드 틀: ✕·Esc·밖 누르기, 기둥 높이 안, 넘치면 스크롤) + `trowel/rowedit.rs` |
| 18 | `trowel/weights.rs`(미리 보기는 `plan_weights`를 문서 판마다 한 번 계산) |
| 19 | `trowel/palette.rs`: 선종 칩 넷 → `stroke.set {dash}`, 「범례에 더하기」 → `legend.add` + 고치기 카드, 「범례대로」 → `legend.revert {}`. 공통 나눔 단추 `segments`(켜진 칸을 맨 나중에 그려 테두리 네 변이 보임 — F1) |
| 20 | `trowel/menu.rs`: 첫 두 항목이 실행됨 |
| 21 | 범례 목록 스크롤 흐림 끔 |
| 22 | `ko.tsv` `trowel` 문맥 행(카드·팔레트·메뉴 글자 58행 추가) |

스펙에서 벗어난 점과 이유:

1. 기준 3: `legend`는 범례가 없으면 `null`이 아니라 키를 쓰지 않는다(다른 선택 필드와 같게). 시험 `inspect_reports_the_legend`가 그렇게 확인.
2. 기준 13: 컨트롤 바는 목록뿐 아니라 닫힌 칸에도 줄 보기를 그린다(시안의 칸 모양).
3. 기준 17: 「굵기 없음」은 칩 줄이 아니라 숫자 칸 줄 오른쪽의 켜고 끄기. 잠긴 줄의 카드는 흐리고 「이 줄의 레이어가 잠겨 있습니다…」를 보인다. 카드는 Esc를 글자 칸이 키보드를 갖지 않을 때만 받고, 밖 누르기는 캔버스·도구에도 그대로 간다.
4. 기준 17: 줄 두 번 누르기를 egui의 두 번 누르기 판정 대신 **같은 줄을 0.3초 안에 두 번** 누르는지로 센다. egui는 다른 곳(카드의 ✕)을 누르고 0.6초 안에 줄을 두 번 누르면 두 번째를 「세 번 누르기」로 세어 카드가 다시 열리지 않았다(원인 확인: 같은 단계에 1초 쉼을 넣은 시험은 통과 — `probe-dblclick.log`).
5. 기준 19: 고른 선들의 대시가 서로 다르면 선종 칩은 하나도 켜지지 않는다(굵기 칩이 다른 굵기에서 그러는 것과 같게). 「범례 줄 {n}」의 n은 1부터 아직 없는 이름의 첫 번호. 아무것도 고르지 않고 「범례대로」를 누르면 상태줄에 「다음 선은 「{줄}」 줄 모양으로 그립니다」. 팔레트 두 단추·도면 메뉴 두 항목에 도움말을 더했다.
6. 기준 19: J003 선종 칸의 「실선」 도움말 「No dashes」는 칸을 공통 단추로 다시 그리며 없어져 `ko.tsv` 행도 뺐다.
7. 기준 20: 메뉴 이름 「Change Weights at Once…」의 한국어를 「굵기 한 번에 바꾸기…」에서 명세·시안의 「굵기 한 번에…」로 바꿨다(J003 행).
8. 리팩터: 컨트롤 바의 「자석」·「추정(파선)」 켜고 끄기를 공통 `toggle`·`toggle_soon`으로(모양·동작 같음, J003 시험 그대로 통과). 선종 목록은 `trowel::line_types()` 하나를 카드와 팔레트가 같이 쓴다.

시험 변경(이미 있던 시험의 기대·찾는 대상을 바꾼 것, 커밋에 `시험 변경:`으로 적음):

| 시험 | 바꾼 것 | 이유 |
|---|---|---|
| `palette_weight_chip_sets_the_next_stroke`(J003) | 준비 중 목록에서 선종 「파선」·「일점쇄선」을 뺌 | J005가 살림(명세 23) |
| `drawing_menu_sits_before_view_and_waits_for_its_jobs`(J003) | 「모두 꺼짐」 → 첫 두 항목 말고 꺼짐 | J005가 살림(명세 23) |
| `status_bar_shows_messages_for_a_while`(J003) | 누르는 준비 중 단추 「모든 굵기 바꾸기」 → 「잘린 면」(J014) | J005가 살림, 확인하는 것은 같음(명세 23) |
| `a_short_window_scrolls_the_right_column_without_overlap`(J003) | 범례 위치를 찾는 글자 「이 도면의 레이어」(묶음 제목) → 「범례 · 레이어」(판 머리) | 묶음 제목이 이제 목록 안에서 줄과 함께 스크롤된다(묶음이 둘일 수 있어서). 확인하는 것(범례가 속성 위)은 같음. **명세 23에 없던 변경** |
| `the_legend_list_stays_above_its_footer`(J003) | 휠을 굴릴 자리를 묶음 제목 대신 판 머리에서 잼 | 같은 이유. 두 번째 창 크기에서는 목록이 이미 끝까지 내려가 있어 묶음 제목이 보이지 않음. **명세 23에 없던 변경** |
| `double_clicking_a_row_opens_its_editor`(J005 새 시험, 통과 전) | Strata 칸 기대 「유구_면」 → 「유구_면 · feature_poly」, 카드 위치를 「줄 높이」 → 「줄 높이 또는 상태줄 위로 밀림」 | 명세 17의 글자대로 고침, 짧은 창에서는 카드가 상태줄을 덮지 않게 위로 밀리는 것이 맞음 |

시험 손잡이(harness) 변경: 프레임마다 `textures_delta.clear()`(글꼴 판이 바뀌면 epaint가 「처리 안 한 텍스처」로 패닉), 글자 굵기·글꼴·색을 읽는 도우미, `status_line`·`segments_in`·`gradients_over`·`font_of`.

빨강 → 초록(시험 먼저):

| 단계 | 빨강 로그(실패 이유) | 초록 로그 |
|---|---|---|
| 범례 모형 | `red1-doc.log`(`Legend` 없음, 컴파일 실패) · `red4-doc.log`(`from_dash` 없음) | `green1-doc.log` 5 통과 · `green4-doc.log` 6 통과 |
| 파일 형식 | `red2-format.log`·`red2b-format.log`(범례가 저장되지 않음 / 범례 없는 레이어에도 키가 생김 — `dbg-format.log`로 찾음) | `green2-format.log` 4 통과 |
| 엔진 명령 | `red3-engine.log`(`cmd::legend` 없음) | `green3-engine.log` 20 통과·1 실패(옮기기와 줄 모양 주기가 실행 취소 한 번에 함께 돌아오지 않음) → `green3b-engine.log` 22 통과, `sweep.log`(command_sweep) 15 통과 |
| 변이 확인 | `mutant-pick.log`(옮긴 선에 모양 주기를 일부러 빼면 시험이 잡음) | — |
| 명령 이름 번역 | `red5-i18n.log`(ja 등 6개 이름 없음) | `green6-theme-i18n.log` |
| 토큰 | `red6-theme.log`(`caution` 없음) | `green6-theme-i18n.log` 7 통과 |
| 범례 판 | `red7-ui.log` 9 실패 | `green7-ui.log` 69 통과·5 실패(J003 시험 셋은 위 시험 변경, 텍스처 패닉은 손잡이) → `green7b-ui.log` 73 통과·1 실패(카드 아직 없음) |
| 변이 확인 | `mutant-carry.log`(선택 도구 조건을 빼면 「그리던 선이 옮겨 감」을 시험이 잡음) | — |
| 카드 | `red8-cards.log` 5 실패 | `green8-cards.log` 3 통과·2 실패(카드 아래가 잘림: egui Area가 지난 크기만 내줌 → `set_max_height`) → `green8b-cards.log` 4 통과·1 실패(위 벗어난 점 4) → `green8c-trowel.log` Trowel 41 통과 |
| 카드 번역 | `red9-i18n.log`(카드 글자 50개 번역 없음) | `green9-i18n-trowel.log` 78 통과 |
| 팔레트·메뉴·컨트롤 바 | `red10-palette-menu-control.log` 5 실패(선종·범례 단추가 준비 중, 메뉴 항목 꺼짐, 목록이 색 패치) | `green10-palette-menu-control.log` Trowel·번역 83 통과 |

## 검증 (verify)

2026-10-11, 대상 커밋 `c14a4ab`(빌드한 작업 트리를 그대로 커밋, 작업 브랜치에 올림). 명령은 `/home/claude/vector`에서 `CARGO_PROFILE_DEV_DEBUG=0`. 로그는 빌드 절과 같은 폴더.

| 명령 | 시각(UTC) | 종료 코드 | 결과 | 로그 |
|---|---|---|---|---|
| `cargo xtask ci` 1회차 | 18:50:56–18:52:18 | 1 | fmt 통과, clippy 실패: `type_complexity` 2건(`tests_legend.rs` 535·561줄, 이번 변경의 시험 코드) → `type RowLook` 별칭으로 고침 | `ci1-xtask.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | 18:53 | 0 | 경고 없음 | `clippy2.log` |
| `cargo xtask ci` 2회차 | 18:53:50–19:04:37 | 0 | 7단계 모두 통과(fmt, clippy, test, assets, brands, layers, wasm). 시험 바이너리 118개, 5,171 통과·0 실패·4 무시(전부터 `#[ignore]`) | `ci2-xtask.log` |
| `cargo check --target wasm32-unknown-unknown -p vectorcraft-web` | 19:04:52–19:05:26 | 0 | 경고 2개는 이번에 손대지 않은 파일(`dialogs/missing_links.rs:69`, `i18n/mod.rs:272`: wasm에서 안 쓰는 함수)이고 기준 커밋부터 있던 것 | `web-check.log` |
| `cargo build -p vectorcraft` | 19:05:54–19:08:30 | 0 | 화면 확인용 실행 파일 | `build-app.log` |

검증 방법 절의 대상 시험은 2회차 전체 시험 안에서 돌았다(`ci2-xtask.log`에서 셈): doc `legend::` 6, format `tests/legend.rs` 4, engine `tests_legend::` 21 + `every_command_has_unique_id_and_doc` 1, `command_sweep` 15, ui-egui `trowel::` 46 · `theme::` 6 · `i18n::` 37, 모두 통과.

완료 기준별 확인:

| 기준 | 확인 | 결과 |
|---|---|---|
| 1–3 문서·형식 | doc `legend::` 6, format `tests/legend.rs` 4(저장·열기, 범례 없는 레이어는 키 없음, 모르는 필드 무시, 옛 파일), engine `inspect_reports_the_legend` | 통과 |
| 4–10 엔진 명령 | engine `tests_legend::` 21, `command_sweep` 15(새 명령 포함 무작위 인자), 명령 목록 시험 | 통과 |
| 11–21 화면 | ui-egui `trowel::` 46(J003 시험 + J005 새 시험), `theme::tests::every_preset_has_the_caution_tokens` | 통과 |
| 22 번역 | `i18n::tests::every_tw_literal_is_translated_in_korean` 외 `i18n::` 37 | 통과 |
| 23 J003 시험 변경 | 빌드 절 표, 커밋 메시지 `시험 변경:` 6줄 | 기록함 |
| 24 실제 화면 | 아래 | 확인함(차이 목록) |
| 25 고전 화면·wasm·관문 | 전체 시험 실패 0, 웹 앱 wasm check 0, `cargo xtask ci` 0 | 통과 |

실제 화면(기준 24): `scratchpad/j005/drive5.py`(J003 `drive3.py`를 늘림, Xvfb, `VECTORCRAFT_LOCALE=ko VECTORCRAFT_NO_PREFS=1`, 제어 채널 7993)로 1600 × 988 창에 새 문서 「1호 주거지 평면」 → `legend.addGroup {group: plan}` → 기본 레이어 지움 → 줄마다 `legend.pick` 뒤 도형을 그림(굴광선 줄의 「2호 수혈」만 0.5 pt로 바꿔 따로 1) → 조사구역 경계·기준점 잠금 → 굴광선 줄을 고르고 펜. 찍은 그림(커밋하지 않음): `/mnt/project-files/design/trowel-run/J005-plan.png`(평면 화면), `J005-override.png`(굴광선 줄 펼침, 「0.5 따로 ↺」), `J005-row-card.png`(줄을 두 번 누름, 0.12초 간격의 제어 채널 누르기 두 번), `J005-weights-card.png`(바닥 「모든 굵기 ×0.8 …」), 시안과 나란히 `J005-side-by-side.png`(`win-plan.png` 옆), `J005-cards-side-by-side.png`(그림 ②③ 옆).

화면에서 확인한 동작: 묶음 넣기 15줄, 두 묶음 제목과 ★ 줄 먼저, 잠긴 줄의 자물쇠, 「따로 1」 표와 펼친 줄의 「0.5 따로 ↺」, 바닥 「따로 1 → 범례대로」, 두 번 누르기로 카드가 줄 높이에 뜸(닫기 세 가지는 화면에서 따로 확인하지 않음, 시험 `double_clicking_a_row_opens_its_editor`가 확인), 굵기 한 번에 카드의 미리 보기(같은 값 줄 묶음, 「선 14개가 바뀝니다」, 잠긴 선 1개 알림).

시안과 남은 차이(검증에서 내가 찾은 것, 독립 검토 전):

| 곳 | 시안 | 앱 | 판단 |
|---|---|---|---|
| 기호 줄 보기(기준점 △, 유물 출토 위치 ③) | △는 12 × 11 px 선 그림, ③은 11 px 글자 | 둘 다 글꼴 글자 11 pt라 △가 5 px 남짓으로 작음 | **고칠 것**(검토 뒤 재작업에서) |
| 굴광선 줄 이름 | 「굴광선」 | 「굴광선 (유구 면)」(§7 표 이름)이 「따로 1」 표 때문에 「굴광선 (유구…」로 줄어듦 | 이름은 표대로. 줄인 이름의 전체를 보는 방법은 검토 의견을 봄 |
| 유구 선 (구·벽구) 보기 | 납작한 사각형 | 선 | §7 표의 「보기」(선)를 따름 |
| 내부 시설·경사 표시 보기 | 점·세로줄 무늬 | 빈 사각형 | 무늬·빗금은 J014 |
| 줄 순서 | 레이어 순서 | ★ 줄이 묶음 맨 위 | 명세 위험 절 결정 3 |
| 바닥선의 묶음 | 조사 데이터 | 그리기 · Trowel | 명세 12(Strata 키로 나눔, 바닥선은 키 없음) |
| 펼친 객체 줄 순서 | 1호 · 2호 · 3호 | 3호 · 2호 · 1호(위에 있는 것 먼저) | J003 레이어 판 규칙 그대로 |
| 객체 줄 면적 「32.91 ㎡」·「그리는 중」 | 있음 | 없음 | J009 |
| 「0.5 따로 ↺」 | 글자 12 px | 10 px 남짓, ↺가 더 굵음 | nit |
| 줄 고치기 카드 | ✕ 없음(본문 그림), Strata 고르기, 추정·빗금·글자·저장이 쓸 수 있는 모양 | ✕, Strata 읽기만, 흐린 준비 중, 「굵기 없음」·「지금은 이 도면 파일에 저장됩니다」·「레이어 옵션…」 더함 | 명세 17, J004·J009·J012·J014·J018·J019, 빌드 벗어난 점 3 |
| 줄 고치기 카드 「= 0.28 mm」 | 14 px | 11 px 남짓 | nit |
| 굵기 한 번에 표 | 줄 높이 약 35 px, 「Ctrl+Z」 키 상자 | 약 26 px, 그냥 글자 | nit |
| 굵기 한 번에 「도면집 전체」 | 쓸 수 있는 모양 | 흐림 | J004 |
| 캔버스 | 스캔·TM 좌표·방위표 | 시험용 도형, mm 눈금자 | J006·J008·J012(J003 표와 같음) |

GitHub 「Trowel CI」: `c14a4ab`의 run 38166616175가 19:06:35Z에 시작(결과는 승인 절에 적는다).

## 검토 1차 (review)

2026-10-11, 대상 커밋 `c14a4ab`(코드. 뒤 커밋 `905641e`는 이 기록만 바꿈). 두 검토자 모두 읽기 전용 하위 에이전트이고 제품 코드를 고치지 않았다. 검토 전문과 근거 그림은 커밋하지 않고 `scratchpad/factory/J005/review-code-result.md`, `review-ui-result.md`, `scratchpad/review-ui-J005/`에 둔다.

| 검토 | 한 일 | 판정 |
|---|---|---|
| 코드·정확성·보안 (code · correctness · security를 한 번에) | `git diff 5ac2a6c c14a4ab`를 읽고 egui 0.36.2 소스로 추적, doc·format 시험을 직접 돌림, 손으로 만든 `.vectorcraft` 파일을 `vectorcraft-cli run`으로 열어 봄 | **rework**: major 2(C1, C2), minor 7, nit 5 |
| 화면 (ui-reviewer) | `c14a4ab` 실행 파일 사본을 Xvfb와 제어 채널로 띄워 장면 9개를 찍고 시안(`win-plan.png`, 그림 ②③)과 견줌 | **rework**: major 4(U1–U4), minor 7, nit 12 |

◆ 항목에 대한 코드 검토 의견은 모두 동의다. 1 mm = 72/25.4 pt. Strata 대시 길이는 hgis `src/core/FeaturePresets.cpp:21-23`과 같고, Strata가 단위를 MM으로 적으므로 굵기와 상관없는 길이로 읽는 것이 맞다. 1e-9 pt 허용치, 굵기 한 번에(비율·값·한 단계, 0.001 pt 반올림, 0–1000 자르기), 화면 px 표, 「= mm」 표시, 파일 형식(선택 필드, 판 4 그대로)도 동의했다. 사용자에게 알릴 점은 하나다: 옛 빌드로 새 파일을 열었다가 다시 저장하면 줄 모양이 소리 없이 사라진다(레이어와 선은 남음). 행동 결정 1–5도 타당하다고 보았고, 결정 1이 두 번 누르기와 겹치는 경우(C2)만 사람이 정할 일로 남겼다.

지적과 처리(처리는 아래 재작업 1차):

| 번호 | 정도 | 지적 | 처리 |
|---|---|---|---|
| C1 | major | 굵기 한 번에: Enter 없이 비율·값을 넣고 「바꾸기」를 누르면 화면과 다른 옛 값으로 바뀜. 범위 밖 값이면 거절 알림이 덮임 | 고침: 카드가 끝난 뒤의 규칙으로 실행하고, 같은 프레임에 거절이 있으면 실행하지 않음. 시험 `weights_card_changes_by_the_ratio_just_typed` |
| C2 = U2 | major · 사람 결정 | 선택 도구로 선을 고른 채 줄을 두 번 누르면 첫 누르기가 고른 선을 그 줄로 옮김 | 검토자 안 (b)로 고침: 두 번째 누르기가 첫 누르기의 옮김을 실행 취소로 되돌리고 카드를 엶(같은 줄·같은 도면·그 사이 바뀐 것 없음일 때만). 상태줄 「고른 선은 옮기지 않고 그대로 둡니다」. 시험 `double_clicking_a_row_with_lines_selected_moves_nothing`. **사용자 확인 필요** |
| C3 | minor | 그리지 않는 사용자 대시(`[]`, `[0,0]`, 음수)가 언제나 「따로」로 보이고 「범례대로」가 끝나지 않음 | 고침: 그리는 대시끼리 견주고(`LineType::drawn`), 그리지 않는 패턴은 실선. 시험 `a_pattern_that_draws_nothing_is_solid` |
| C4 | minor | 파일의 범례 값이 명령의 검사·상한을 거치지 않음(굵기 -5, 1e308, 아주 긴 대시·글자) | 고침: 읽을 때 `Legend::tidied`(굵기는 0–1000으로 자르고 수가 아니면 없음 / 대시는 그리는 것만 16개까지, 아니면 실선 / 기호 32자, Strata 키 64자). 상한 상수를 doc으로 옮겨 명령과 같이 씀. 시험 `odd_values_are_tidied_as_the_commands_take_them`, `odd_legend_values_open_tidied`, 속성 시험 `any_legend_opens_tidy` |
| C5 | minor · ◆ · 사람 결정 | 모르는 값(`"group":"drawings"`, `"line":"longdash"`) 하나로 파일 전체가 「VectorCraft 파일이 아님」으로 거부됨 | 검토자 권고 (b)로 고침: 범례를 너그럽게 읽어, 읽을 수 없는 범례는 그 레이어에서만 빠지고 그림은 열림. 형식 판은 4 그대로. 시험 `layers_read_their_legend_leniently`, `a_legend_this_version_cannot_read_is_left_out`. **사용자 확인 필요** |
| C6 | minor | pt 칸에 쓰다가(Enter 없이) 칩을 누르면 칸 값이 이기고 실행 취소가 두 단계 | 고침: 같은 프레임에 칩이 눌렸으면 칩이 이기고 한 단계. 시험 `a_weight_chip_pressed_while_typing_wins` |
| C7 = U5 | minor | 칸에 쓰는 중 Esc 한 번에 카드까지 닫힘 | 고침: 지난 프레임 끝에 칸이 키보드를 가졌는지 기억해, 그 Esc는 쓴 것만 되돌림. 시험 `esc_in_a_card_field_keeps_the_card` |
| C8 | minor | 굵기 한 번에 카드가 지난 설정으로 다시 열림 | 고침: 열 때마다 처음(×0.8)부터. 시험 `weights_card_opens_afresh` |
| C9 | minor | 퍼즈·속성 시험에 범례가 없음 | 고침: `format/tests/legend.rs`에 속성 시험 2개(아무 범례 값이나 정리되어 열림, 범례가 든 파일을 자르고 바이트를 바꿔도 패닉 없음), `command_sweep`의 잡동사니 인자에 범례 명령 13개 |
| C10 | nit | 실행 취소 이름 둘(「Legend Entry」, 「Move to Legend Entry」)이 번역되지 않음 | 고침: 명령 이름 「Set Legend Entry」·「Use Legend Entry」로 맞춤(이미 번역됨). 시험 `legend_undo_steps_are_named_after_their_commands` |
| C11 | nit | `legend.add`가 고른 순서가 아니라 그리기 순서의 첫 선을 씀 | 고침: 고른 순서. 시험 `adding_a_row_takes_the_first_selected_lines_look` |
| C12 | nit | ×1.0이 0.001 격자 밖의 굵기(0.35355)를 반올림해 「바뀜 1」 | 그대로: 명세 7의 반올림 규칙 |
| C13 | nit | 하위 레이어의 범례도 줄처럼 그려짐 | 고침: `Document::legend`는 맨 위 레이어만. 시험 `only_top_level_layers_are_legend_rows` |
| C14 | nit · 시험 변경 | 빌드 때 바꾼 상태줄 시험이 「…」 떼기를 더는 보지 않는데 시험 변경표에 없음 | 고침: 단위 시험 `a_soon_note_drops_a_dialogs_ellipsis`, 아래 시험 변경표에 적음 |
| U1 | major | 카드 밖 캔버스를 눌러 닫으면 그 누르기가 펜에도 가서 한 점짜리 패스가 남음 | 고침: 카드를 치운 누르기는 거기서 끝남(캔버스가 그 프레임의 누르기를 도구에 넘기지 않음). 터치패드 톡 누르기도 같음(아래 결함 둘째). 시험 `a_press_on_the_canvas_only_puts_a_card_away`, `a_tap_on_the_canvas_only_puts_a_card_away` |
| U3 | major · J003 퇴행 | 평면용 범례를 넣은 뒤 리본 「밑그림」이 스캔을 놓지 못하고 빈 「밑그림」 줄만 숨김 | 고침: 무엇이 든 템플릿 레이어만 밑그림으로 셈. 빈 「밑그림」 줄만 있으면 놓기 창이 뜸. 시험 `the_underlay_tile_places_a_scan_beside_the_empty_underlay_row` |
| U4 | major | 기호 보기 △·③가 시안보다 작음 | 고침: △는 12 × 11 px 선 그림, 다른 기호는 12 pt 글자를 24 px 안에 맞춤(작아도 7 pt). 시험 `mark_samples_are_as_large_as_the_mockup` |
| U6 | minor | 잘린 이름 전체를 볼 길이 없음 | 고침: 이름이 잘린 줄은 풍선 첫 줄에 전체 이름, 굵기 한 번에 표의 잘린 칸과 카드 머리 부제(「…」로 잘림)에 풍선. 시험 `cut_names_and_apart_badges_say_themselves_on_hover` |
| U7 | minor | 도면이 없을 때 「모든 굵기 ×0.8 …」가 켜져 있음 | 고침: 도면이 있을 때만 켜짐. 시험 `the_weights_button_waits_for_a_drawing` |
| U8 | minor | 잠긴 줄을 누르면 펜만 그 모양을 받고 선은 다른 줄로 가서 바로 「따로」 | 고침: 잠기거나 숨은 줄은 고르지 않고 상태줄로 까닭을 알림(범례 판·컨트롤 바 목록, 엔진 `legend.pick`도 거절). 시험 `a_locked_or_hidden_row_is_not_picked`, `picking_a_locked_or_hidden_row_is_refused` |
| U9 | minor | 굵기 없는 줄에서도 선종을 바꿀 수 있는데 아무 변화가 없음 | 고침: 굵기가 없으면 선종 칸이 흐리고 눌러도 바뀌지 않음. 시험 `a_row_without_a_weight_has_its_line_types_dimmed` |
| U10 | minor | 두 번 누르기 간격 0.3초 | 고침: 0.5초(Windows 기본값). 시험 `a_slow_double_click_opens_the_editor` |
| U11 | minor | 「따로 N」 표에 뜻 풀이가 없음 | 고침: 풍선 「굵기나 선 종류를 따로 바꾼 이 줄의 선: {n}개」(U6 시험) |
| U12 | nit | 바닥 단추가 26 px(시안 24), ×·→·…가 작고 가늚 | 높이 24 px로 고침(시험 `cards_match_the_mockup_sizes`). ×·→·…의 크기·굵기는 Source Sans 3 자체 글리프 모양이라 그대로(처음에 「글꼴 대체 탓」이라 적은 것을 검토 2차에서 바로잡음) |
| U13 | nit | 카드 칩 줄 22 px(시안 24), 「이 줄이 곧 레이어입니다」 구간이 촘촘함 | 고침: 24 px, 그 줄 18 px(같은 시험) |
| U14 | nit | 「pt」가 칸 오른쪽 끝, 「= mm」 11 px | 고침: pt는 숫자 바로 뒤, 「= mm」 12 pt(같은 시험) |
| U15 | nit | 카드의 줄 개수가 고정폭 글꼴이 아님 | 고침(같은 시험) |
| U16 | nit | 「×0.8」의 ×가 작고 흐림 | 고침: 칸 글꼴·먹색(같은 시험) |
| U17 | nit | 굵기 한 번에 표 줄 26 px(시안 34), 「Ctrl+Z」 키 상자 없음 | 줄 34 px·머리 30 px로 고침(같은 시험). 키 상자는 그대로 |
| U18 | nit | 「0.5 따로 ↺」 글자가 시안보다 작음(10 px) | 그대로: 줄의 다른 표 글자(10 pt)와 같게 둠 |
| U19 | nit | 0.1·0.2 줄의 보기가 굵기 없는 줄의 흐린 보기와 비슷함 | 그대로: 디자인 2.1 7절 표(0.1 pt → 0.45 px)를 따름. 바꾸려면 사용자 결정 |
| U20 | nit | 「범례에 더하기」로 만든 줄의 카드 머리가 「범례 줄 범례 줄 1」 | 그대로: 명세 19의 이름. 「새 줄 {n}」으로 바꿀지 사용자에게 물음 |
| U21 | nit | 상태줄 「{n}개를 「{row}」 줄로 옮겼습니다」가 명세 13의 「선 N개를」과 다름 | 그대로(벗어난 점으로 적음): 옮기는 것이 선만이 아님(글자·이미지·그룹) |
| U22 | nit | 컨트롤 바 범례 목록이 판과 다른 순서 | 고침: 판과 같은 묶음·★ 순서. 시험 `control_bar_lists_rows_in_the_legend_order` |
| U23 | nit | 1366 × 728 창에서 카드가 탭 줄을 가림 | 고침: 카드는 캔버스 위 끝 아래로. 시험 `a_card_stays_under_the_tab_row` |

## 재작업 1차 (rework)

2026-10-11, 기준 `905641e`(작업 브랜치 제자리, 빌드 절의 예외 그대로). 지적마다 실패하는 시험을 먼저 쓰고 빨강을 본 뒤 고쳤다. 로그는 빌드 절과 같은 폴더이고 이름이 `rw1-`로 시작한다.

손댄 파일: `crates/doc/src/{legend,node}.rs`, `crates/engine/src/cmd/legend.rs`, `crates/engine/src/tests_legend.rs`, `crates/engine/tests/command_sweep.rs`(잡동사니 인자 13개), `crates/format/tests/legend.rs`, `crates/ui-egui/src/canvas.rs`(2줄: Trowel 카드를 치운 누르기를 도구에 넘기지 않음. 카드가 없는 고전 화면에서는 일어나지 않음), `crates/ui-egui/src/trowel/{card,control,legend,mod,palette,ribbon,rowedit,weights,tests}.rs`, `crates/ui-egui/src/i18n/ko.tsv`(4행 추가), 이 파일.

빨강 → 초록:

| 단계 | 빨강 | 초록 |
|---|---|---|
| 범례 모형 (C3·C4·C5·C13) | `rw1-red1-doc.log` 4 실패 | `rw1-green1-doc.log` 10 통과 |
| 파일 형식 (C4·C5·C9) | 시험을 먼저 썼지만 빨강 로그를 남기지 않고 구현까지 감. 아래 변이 확인으로 보완 | `rw1-green2-format.log` 7 통과·1 실패(`any_legend_opens_tidy`: 자른 기호 끝에 공백이 남음) → 자른 뒤 다시 다듬게 고쳐 `rw1-green2b-format.log` 8 통과 |
| 변이 확인 | `rw1-mutant-read.log`: 너그러운 읽기와 정리를 일부러 빼면 format 시험 3개가 실패 | — |
| 엔진 (C10·C11·U8) | `rw1-red3-engine.log` 2 실패, `rw1-red3b-engine.log` 1 실패 | `rw1-green3-engine.log` 25 통과, `rw1-sweep.log`(command_sweep) 15 통과 |
| 화면 (C1·C2·C6·C7·C8·C14, U1–U23) | `rw1-red4-ui.log` 새 시험 17개 실패, `rw1-red5-u12.log` 1 실패(바닥 단추 26 px) | `rw1-green4-ui-hung.log` 멈춤(아래 결함) → 고친 뒤 `rw1-green5-ui.log` 99 통과·1 실패(J003 밑그림 시험, 아래 시험 변경) → `rw1-green6-ui.log` Trowel·번역·토큰 106 통과 |
| 터치패드 톡 누르기 (재작업 중 찾음) | `rw1-red6-tap.log` 캔버스를 톡 누르면 카드가 닫히지 않음, `rw1-red7-tap.log`(첫 고침 뒤) 카드를 여는 톡이 그 카드를 바로 닫음 | `rw1-green7-ui.log` Trowel·번역·토큰 108 통과 |

재작업 중 찾아 고친 결함: 첫 초록 실행이 멈췄다. 시험 스레드가 모두 잠금(futex)에서 기다리며 CPU를 쓰지 않았다. C7을 고치며 `card::show`가 `ctx.data_mut` 안에서 `ctx.egui_wants_keyboard_input()`(같은 문맥의 읽기 잠금)을 부른 것이 원인이다. egui의 디버그 빌드 잠금은 10초 기다린 뒤 패닉하고, 엔진 안전망이 그 패닉을 프레임마다 잡아 시험이 아주 느리게 돌거나 실패했다. 릴리스 빌드였다면 카드를 여는 순간 앱이 멈췄을 결함이다. 읽기를 `data_mut` 앞으로 옮겨 고쳤고, 바뀐 코드에 같은 모양(잠금 안에서 문맥 읽기)이 더 없는지 diff에서 찾아 확인했다.

재작업 중 찾아 고친 결함 둘째(검토에 없던 것, 빌드 때부터): 터치패드의 톡 누르기처럼 누름과 뗌이 한 프레임에 오면, egui는 그 프레임의 화면 코드가 돌기 전에 누른 자리(`press_origin`)를 지운다. 그래서 카드 밖을 톡 누르면 카드가 닫히지 않고 그 누르기가 펜으로 갔다(U1이 톡에서는 고쳐지지 않았음). 처음에는 반대로 짐작해 「카드를 여는 톡이 그 카드를 닫는다」는 시험을 먼저 썼는데 통과했고, 그 까닭을 egui 소스에서 찾다가 위 원인을 확인했다. 누른 자리를 그 프레임의 이벤트에서 읽게 고치자(`rw1-red6-tap.log` → 통과), 이번에는 카드를 여는 톡이 그 카드를 바로 닫았다(`rw1-red7-tap.log`). egui가 같은 id의 카드가 전에 있던 자리를 계속 기억하기 때문이다. 카드를 연 프레임을 `Open.since`에 적고, 그 프레임의 누르기로는 닫지 않게 고쳤다(`card::Place.fresh`). 시험 `a_tap_on_the_canvas_only_puts_a_card_away`, `two_taps_open_the_editor_again`(카드가 열린 채 팔레트 「범례에 더하기」를 톡 누르는 경우 포함).

명세에서 벗어난 점과 이유:

1. 명세 5: `legend.pick`은 잠기거나 숨은 줄을 옮기지 않을 때도 거절한다(U8). 그 줄을 고르면 펜만 그 모양을 받고 선은 다른 레이어로 들어가 바로 「따로」가 되기 때문이다. 템플릿인 「밑그림」(잠김)·「desca 원본」(숨김) 줄도 고를 수 없고, 고르면 상태줄이 까닭을 말한다.
2. 명세 13: 상태줄은 「선 N개를 …」이 아니라 「{n}개를 「{줄}」 줄로 옮겼습니다」. 옮기는 것이 선만이 아니기 때문이다(U21).
3. 명세 17(두 번 누르기, C2 b): 첫 누르기가 옮긴 것은 두 번째 누르기가 실행 취소로 되돌린다. 되돌린 단계는 다시 하기(Ctrl+Shift+Z)에 남아, 그것을 누르면 옮김이 다시 된다. 첫 누르기의 옮김이 새 실행 취소 단계라서, 그 앞에 쌓여 있던 다시 하기 기록은 사라진다(검토 2차 C17에서 더함).
4. U1: 검토자 안(캔버스를 덮는 투명 Area)은 쓰지 않았다. egui는 눌린 Area를 그 층의 맨 위로 올리므로, 한 번 눌린 뒤에는 그 Area가 카드를 덮는다. 대신 그 프레임에 카드를 치운 누르기를 캔버스가 도구에 넘기지 않는다. 오른쪽 기둥·리본 같은 다른 곳의 누르기는 지금처럼 카드를 닫으며 제 일도 한다(검토자도 그래도 된다고 함).
5. U3: 타일로 놓은 스캔은 `file.place`가 만드는 별도 「Template …」 레이어로 들어가고, 빈 「밑그림」 줄은 그대로 남는다. 스캔을 그 줄 안에 넣는 것은 밑그림을 다루는 J008에서 한다.
6. C14: 「…」 떼기 코드는 그대로 두고 단위 시험으로 되살렸다.

시험 변경(이미 있던 시험의 기대·준비를 바꾼 것, 커밋에 `시험 변경:`으로 적음):

| 시험 | 바꾼 것 | 이유 |
|---|---|---|
| `legend_rows_show_the_look_star_and_overrides`(J005) | 기호 줄의 글자 목록 기대에서 「△」를 뺌 | △를 글자 대신 선 그림으로 그림(U4). 크기와 모양은 새 시험 `mark_samples_are_as_large_as_the_mockup`이 봄 |
| `underlay_tile_places_a_template_then_shows_and_hides_it`(J003) | 「Scan」 템플릿 레이어에 사각형 하나를 그린 뒤 템플릿으로 함. 기대는 그대로 | U3 고침으로 밑그림은 무엇이 든 템플릿 레이어만이다. 시험이 놓인 스캔 대신 쓰던 빈 레이어는 이제 밑그림이 아니므로, 시험 주석(「with one placed」)대로 스캔이 놓인 상태를 만듦 |
| `status_bar_shows_messages_for_a_while`(J003, 빌드 때 바꿈) | 이번에는 바꾸지 않음 | 빌드 때 바꾼 뒤 「…」 떼기를 더는 보지 않게 된 범위 축소를 빌드 절 표에 적지 않았다(C14). 단위 시험 `a_soon_note_drops_a_dialogs_ellipsis`로 되살림 |

시험 손잡이(harness): `type_into`를 `type_over`(쓰기만) + Enter로 나눔(기대 변화 없음), `tap`(누름과 뗌을 한 프레임에), `rest_at`(풍선이 뜨게 머무름), `enabled`(흐린 칸 확인), `row_weight`, `triangles_in`.

재작업 1차 검증(이 절의 커밋에 든 작업 트리 그대로). 명령은 `/home/claude/vector`에서 `CARGO_PROFILE_DEV_DEBUG=0`, 로그는 위와 같은 폴더.

| 명령 | 시각(UTC) | 종료 코드 | 결과 | 로그 |
|---|---|---|---|---|
| `cargo xtask ci`(톡 결함을 고치기 전) | ~20:53:36 | 없음(멈춤) | fmt·clippy 통과 뒤 시험 빌드 중에 내가 멈춤: 톡 결함을 먼저 고치려고. 통과로 세지 않음 | `rw1-ci-xtask-stopped.log` |
| `cargo xtask ci` | 21:03:38–21:10:35 | 0 | 7단계 모두 통과(fmt, clippy, test, assets, brands, layers, wasm). 시험 바이너리 118개, 5,201 통과·0 실패·4 무시(전부터 `#[ignore]`) | `rw1-ci-xtask.log` |
| `cargo check --target wasm32-unknown-unknown -p vectorcraft-web` | 21:10:43–21:11:15 | 0 | 경고 2개는 검증 절과 같은 기존 것(`dialogs/missing_links.rs:69`, `i18n/mod.rs:272`) | `rw1-web-check.log` |
| `cargo build -p vectorcraft` | 21:11:21–21:13:52 | 0 | 화면 확인용. 검토자가 쓰도록 사본 `scratchpad/factory/J005/vectorcraft-rw1` | `rw1-build-app.log` |

두 관문 사이에 `cargo fmt --all -- --check`가 이번에 쓴 시험 코드의 줄 바꿈(`crates/doc/src/legend.rs`, `crates/format/tests/legend.rs`)을 지적해 `cargo fmt --all`로 고쳤다(그 두 파일만 바뀜, 로그는 남기지 않음).

대상 시험은 위 전체 시험 안에서 돌았다(`rw1-ci-xtask.log`에서 셈): doc `legend::` 10, format `tests/legend.rs` 8, engine `tests_legend::` 24 + `every_command_has_unique_id_and_doc` 1, `command_sweep` 15, ui-egui `trowel::` 65 · `theme::` 6 · `i18n::` 37, 모두 통과. 첫 검증(5,171)보다 30개 많다(doc 4, format 4, engine 3, 화면 19). 완료 기준 1–25의 확인은 검증 절 표 그대로이고, 재작업으로 더한 시험은 검토 1차 표의 「처리」 칸에 있다.

실제 화면(기준 24): `scratchpad/j005/drive6.py`(drive5.py와 같은 도면에 검토 장면을 더함, Xvfb, 제어 채널 7993, 위 사본 실행 파일)로 찍었다. 그림은 커밋하지 않고 `/mnt/project-files/design/trowel-run/`에 둔다.

- `J005r1-plan.png` 평면 화면, `J005r1-side-by-side.png` 시안(`win-plan.png`)과 나란히.
- `J005r1-tip.png`: 잘린 이름 「굴광선 (유구…」에 머물면 풍선 첫 줄이 「굴광선 (유구 면)」, 둘째 줄이 누르기 안내.
- `J005r1-row-card.png`·`J005r1-weights-card.png` 두 카드, `J005r1-cards-side-by-side.png` 그림 ②③과 나란히.
- `J005r1-u1-press.png`(U1): 펜을 든 채 줄 고치기 카드 밖 캔버스를 누름. 제어 채널 `ui.click`은 누름과 뗌을 한 프레임에 보내므로 톡 누르기와 같다. 카드만 닫히고 레이어별 객체 수가 그대로이며(드라이버 출력 `U1 counts equal: True`), 그림이 `J005r1-plan.png`와 픽셀까지 같다(점 하나 남지 않음).
- `J005r1-u2-double.png`(U2·C2): 선택 도구로 「2호 수혈」을 고른 채 「시굴격자」 줄을 두 번 누름(0.12초 간격). 카드가 그 줄 높이에 뜨고 「이 줄의 선 0개」, 상태줄 「고른 선은 옮기지 않고 그대로 둡니다」. 드라이버 출력: 2호 수혈은 굴광선 줄에 그대로, 현재 레이어는 시굴격자, 마지막 실행 취소 단계는 그 앞의 「Layer Options」 그대로(옮김이 되돌려짐).
- `J005r1-u8-locked.png`(U8): 잠긴 「조사구역 경계」 줄을 누름. 상태줄 「「조사구역 경계」 줄이 잠겨 있습니다: 잠금을 풀어야 이 줄로 그립니다」, 현재 레이어는 시굴격자 그대로, 실행 취소 단계 없음.
- `J005r1-u3-underlay.png`(U3): 빈 「밑그림」 줄만 있을 때 리본 「밑그림」을 누름. 놓기 창을 불렀다(이 기계에는 파일 창이 없어 rfd가 실패하고 상태줄이 「cancelled」, `shots6/app.log`의 `Failed to pick files`). 타일은 켜짐으로 보이지 않는다.

J003 영역: `J003r2-plan.png`와 영역별로 픽셀을 견줬다. 캡션·메뉴·리본·도구 기둥·도면집·탭·속성·상태줄은 같고, 다른 곳은 컨트롤 바 범례 칸(줄 보기)·팔레트 선종과 범례 단추·범례 판(모두 J005)이다. 첫 J005 그림(`J005-plan.png`)과는 리본 「밑그림」 타일(U3: 빈 밑그림 줄을 밑그림으로 세지 않아 꺼짐)과 범례 판(△·③ 보기, 바닥 단추 높이)만 다르다.

시안과 남은 차이(검증 절 표에서 바뀐 것만, 나머지는 그 표 그대로):

| 곳 | 시안 | 앱 | 판단 |
|---|---|---|---|
| 기준점 △ 보기 | 선 그림, 잉크 12 × 10 px | 선 그림, 잉크 12 × 10 px | 같음(U4 고침) |
| 유물 출토 위치 ③ 보기 | 잉크 10 × 11 px | 잉크 11 × 9 px, 대체 글꼴이라 획 모양이 조금 다름 | 받아들임 |
| 표고점 「·152」·글자 「가」 보기 | 8 px 고정폭 · 10 px 글자(잉크 18 × 6, 9 × 10) | 모든 기호를 12 pt 한 규칙으로(잉크 20 × 8, 11 × 11) | 1–2 px 큼. 검토자 권고(기호 11–12 px)를 따른 결과 |
| 리본 「밑그림」 | 켜짐(스캔이 놓인 도면) | 꺼짐 | 그림에 스캔이 없음(U3 고침대로). 밑그림 다루기는 J008 |
| 굵기 한 번에 비율 칸 | 「×0.8」 | 「× 0.8」(× 뒤 약 4 px) | nit |
| 굵기 한 번에 표 | 「Ctrl+Z」 키 상자 | 그냥 글자 | U17에서 남김 |
| 줄 고치기 카드 「= 0.28 mm」 | 12 px(`.note`. 검증 절의 「14 px」는 그림을 잘못 잰 것) | 12 pt | 같음(U14 고침) |
| 표 줄·칩 줄·바닥 단추 높이 | 34 · 24 · 24 px | 34 · 24 · 24 px | 같음(U12·U13·U17 고침) |

범위 밖 발견(고치지 않음): 파일 창을 취소하면 상태줄에 영어 「cancelled」가 보인다. 고전 코드(`crates/ui-egui/src/place.rs:147`, `io.rs:80` 등)의 기존 동작이라 J005와 상관없다. 번역할 때 함께 본다.

GitHub 「Trowel CI」(검증 절의 이어서): `c14a4ab`의 run 38166616175는 곧이어 올린 `905641e`가 같은 브랜치에서 시작되며 취소됐다(워크플로 `concurrency: cancel-in-progress`). `905641e`(= `c14a4ab` 코드 + 검증 기록)의 run 38167165216이 19:14:24Z–19:29:33Z에 성공했다. 이 커밋의 실행은 다음 기록에 적는다.

## 검토 2차 (review)

2026-10-11, 대상 커밋 `ae10c88`(재작업 1차). GitHub 「Trowel CI」의 `ae10c88` 실행 run 38175958714는 21:20:29Z–21:38:51Z에 성공했다. 두 검토자 모두 읽기 전용 하위 에이전트이고 제품 코드를 고치지 않았다. 검토 전문은 커밋하지 않고 `scratchpad/factory/J005/review2-code-result.md`, `review2-ui-result.md`에, 근거 그림과 구동 스크립트는 `scratchpad/rv2-J005/`, `scratchpad/review-ui-J005-r2/`에 둔다.

| 검토 | 한 일 | 판정 |
|---|---|---|
| 코드·정확성·보안 (code · correctness · security를 한 번에) | `git diff 905641e ae10c88`을 읽음. Xvfb와 제어 채널로 앱을 움직여 두 번 누르기를 재현. 손으로 만든 `.vectorcraft` 14개와 대시 18개짜리 선이 든 파일을 `vectorcraft-cli run`으로 엶. doc·format 시험을 직접 돌림 | **rework**: major 1(C16), minor 1(C15), nit 3 |
| 화면 (ui-reviewer) | `vectorcraft-rw1` 사본을 Xvfb에서 제어 채널과 실제 X 입력(XTEST)으로 움직여 찍고 시안과 견줌 | **rework**: major 1(U24), minor 4(C1 나머지, U25, U26, U19 다시), nit 4 |

1차 지적의 상태: 코드 검토는 C1·C3–C14·U8을 해결로, C2를 일부(C16·C17이 남음)로, C12 남김을 타당으로 보았다. 화면 검토는 U1·U2·U4–U11·U13–U15·U22·U23·C6·C8을 해결로, U3(놓는 자리가 U24)·U16·U17·C1(표 쪽)을 일부로, U12·U18·U20·U21 남김을 타당으로, U19 남김을 일부 타당으로 보았다. ◆ 의견: 굵기 0–1000, 기호 32자, 키 64자 상한과 너그러운 읽기에 동의. 대시 16개는 `legend.add`도 같은 상한을 지킬 때 동의(C15). 사람이 정할 것으로 (1) 읽지 못한 범례가 있을 때 열면서 알릴지, (2) 뒤 작업이 범례 값(선종·묶음 등)을 늘릴 때 형식 판을 올릴 규칙을 들었다.

지적과 처리(처리는 아래 재작업 2차):

| 번호 | 정도 | 지적 | 처리 |
|---|---|---|---|
| C16 | major | 펼친 줄의 선을 고른 채 그 아래 줄을 두 번 누르면, 첫 누르기의 옮김으로 펼친 줄이 짧아져 두 번째 누르기가 그 아래 줄에 닿음. 선이 누르지 않은 줄로 가 그 모양을 받고, 카드는 열리지 않음 | 고침: 두 번 누르기를 줄이 아니라 목록이 셈. 줄을 고른 누르기의 줄·시각·자리를 기억해, 0.5초 안에 그 자리에서 egui `max_click_dist`(6) 안을 다시 누르면 지금 그 자리에 무엇이 있든 처음 줄의 카드를 열고 옮김을 되돌림. 목록 밖을 누르면 잊음. 뗀 자리는 그 프레임의 이벤트에서 읽음(펜·터치는 뗀 뒤 포인터가 사라짐). 시험 `double_clicking_a_row_moves_nothing_when_its_first_click_shifts_the_rows` |
| C15 | minor | 대시가 16개를 넘는 선으로 「범례에 더하기」를 하면, 저장·열기 뒤 그 줄이 실선이 되어 선이 모두 「따로」 | 고침: `legend.add`가 만드는 줄도 파일처럼 정리(`Legend::tidied`). 그런 선이면 처음부터 실선 줄이고, 선은 제 대시를 지녀 「따로」. 저장·열기 뒤에도 같음. 시험 `a_row_from_a_line_with_many_dashes_opens_again_as_it_was` |
| C17 | nit | 선을 고른 채 두 번 누르면 앞서 쌓인 다시 하기 기록이 사라짐 | 기록: 재작업 1차의 벗어난 점 3에 더함 |
| — | nit | `legend.add`가 고른 객체마다 Vec을 훑음(선이 없는 아주 큰 선택에서 O(n²)) | 고침: `HashSet` |
| — | nit | `document.inspect`가 하위 레이어의 범례도 보고함(`Document::legend`는 맨 위 레이어만) | 고침: 맨 위 레이어만 보고(`document.node`의 요약도 같게). 시험 `inspect_leaves_out_a_sublayers_legend` |
| U24 | major | 리본 「밑그림」이 스캔을 지금 줄 바로 아래에 영어 「Template scan.png」 레이어로 놓아, 그 아래 12줄의 선을 덮음 | 고침: `file.place`에 `underlay` 인자(레이어 이름에 쓸 낱말)를 더함. 있으면 맨 아래에 잠긴 템플릿 레이어 「{낱말} · {파일 이름}」을 만듦(여러 파일을 싣는 놓기 커서도 같음). 타일은 낱말 「밑그림」으로 부르고, 놓기 창의 「템플릿」 풍선은 「모든 선 아래, 맨 아래의 잠긴 밑그림 레이어에 놓습니다」. 시험 `an_underlay_goes_on_a_template_layer_at_the_bottom`, `queued_underlays_go_to_the_bottom_too`, `the_underlay_tiles_scan_goes_under_every_line` |
| C1 나머지 | minor | 굵기 한 번에: Enter를 누르기 전에는 미리 보기 표가 친 값을 따르지 않음 | 고침: 칸에 쓰는 중인 비율·값으로 계획(읽을 수 없거나 범위 밖이면 지난 값). 시험 `the_weights_table_follows_the_ratio_being_typed` |
| U25 | minor | 새 도면에 묶음을 넣어도 지금 줄이 빈 「Layer 1」로 남아, 처음 그은 선이 범례 밖으로 감 | 고침: 판의 묶음 메뉴로 넣으면 그 묶음의 첫 ★ 줄(평면은 굴광선)을 고름. 빈 기본 레이어를 지울지는 사용자 결정. 시험 `a_group_added_from_the_menu_draws_with_its_first_favourite` |
| U26 | minor | 굵기 한 번에: 「바꾸기」 뒤에도 카드가 열린 채 다음 계획을 보여, 다시 누르면 한 번 더 바뀜 | 고침: 바꾸면 카드를 닫고 상태줄이 알림. 시험 `the_weights_card_closes_after_change` |
| U19 | minor · 사람 결정 | 0.2·0.1 줄 보기가 굵기 없는 줄보다 흐림. 시안 평면 화면은 줄 보기를 1 px 먹으로 그림 | 그대로: 디자인 2.1 7절 표(0.2 pt → 0.6 px, 0.1 pt → 0.45 px)를 따름. 사용자에게 물음(검토자 안: 시안처럼 1 px 먹, 또는 1 px 미만 선을 픽셀 가운데에 맞춤) |
| U27 | nit | 「따로」 풍선의 「선 종류」가 팔레트·카드의 「선종」과 다름 | 고침: 「굵기나 선종을 따로 바꾼 이 줄의 선: {n}개」 |
| U28 | nit | 흐린 칸이 까닭을 말하지 않음(도면이 없을 때 「모든 굵기 ×0.8 …」, 굵기 없는 줄 카드의 선종) | 고침: 풍선 「도면을 열면 굵기를 바꿀 수 있습니다」, 「굵기가 없는 줄은 선종이 없습니다: 먼저 굵기를 주세요」. 시험 `dimmed_controls_say_why_on_hover` |
| U29 | nit | 줄만 바뀌고 따르는 선이 없을 때 「선 0개가 바뀝니다」 | 고침: 「범례 줄 {r}개가 바뀝니다. 따르는 선은 없습니다. {undo} 한 번이면 돌아옵니다.」 시험 `the_weights_note_counts_rows_when_no_line_follows` |
| U12 | nit · 기록 | ×·→·…는 대체 글꼴이 아니라 Source Sans 3 자체 글리프 | 고침: 검토 1차 표 U12의 문구 |

범위 밖 발견(J003 코드, 고치지 않음): 컨트롤 바 범례 칸이 긴 이름을 220 px에서 자르지 않아 굵기 글자가 이름 끝에 겹친다(`crates/ui-egui/src/trowel/control.rs:146`, `157–161`). 백로그 J003 줄의 남은 것에 적었다. 파일 창을 취소하면 보이는 영어 「cancelled」는 재작업 1차에 적은 그대로다.

## 재작업 2차 (rework)

2026-10-11, 기준 `ae10c88`(작업 브랜치 제자리). 수정 루프 두 번 가운데 마지막이다. 지적마다 실패하는 시험을 먼저 쓰고 빨강을 본 뒤 고쳤다. 로그는 같은 폴더이고 이름이 `rw2-`로 시작한다.

손댄 파일: `crates/engine/src/cmd/{edit,legend}.rs`, `crates/engine/src/cmd/place/mod.rs`, `crates/engine/src/inspect.rs`, `crates/engine/src/{tests_legend,tests_place}.rs`, `crates/ui-egui/src/place.rs`, `crates/ui-egui/src/dialogs/place.rs`, `crates/ui-egui/src/trowel/{card,legend,ribbon,rowedit,weights,tests}.rs`, `crates/ui-egui/src/i18n/ko.tsv`(4행 더함, 1행 고침), 이 파일, `factory/backlog.md`.

빨강 → 초록:

| 단계 | 빨강 | 초록 |
|---|---|---|
| 엔진: C15·inspect | `rw2-red1-engine.log` 2 실패(대시 많은 선으로 만든 줄이 다시 열면 다름, 하위 레이어의 범례를 보고함) | 아래 엔진 초록 |
| 화면: C16 | `rw2-red2-ui.log` 1 실패(두 번 누르기 가운데 줄이 밀리자 선이 다른 줄로 감) | 아래 화면 초록 |
| 엔진: U24 | `rw2-red3-engine.log` 2 실패(`underlay` 인자를 모름) | `rw2-green1-engine.log`: 이름에 legend·place가 든 엔진 시험 62 통과 |
| 화면: U24·U25·U26·C1 나머지·U28·U29 | `rw2-red4-ui.log` 6 실패 | `rw2-green2-ui.log`: ui-egui 라이브러리 시험 1,017 통과 |

도중 실패(같은 이름의 로그가 덮어써져 남지 않음): (1) `legend.rs`의 match 갈래 타입이 달라(E0308) 컴파일 오류. (2) 시험 2개 실패. 놓기 창 풍선을 처음에 `tl!`로 써서 `every_tl_literal_is_translated`가 일본어 번역이 없다고 실패해 Trowel 전용 `tw!`로 바꿈. 새 시험 `dimmed_controls_say_why_on_hover`가 고친 도면을 `file.close`로 닫으려다 저장 물음에 막혀, 도면이 없는 경우를 새 하네스로 만들게 그 시험을 고침(새 시험이라 시험 변경이 아님).

C16의 다른 길(검토자 안 (c): 옮김을 두 번 누르기 시간이 지난 뒤에 함)은 쓰지 않았다. 줄을 눌러 선을 옮기는 주된 흐름이 0.5초 늦게 보이기 때문이다. 그래서 C17은 기록으로 남긴다.

명세에서 벗어난 점과 이유(재작업 1차 목록에 이어서):

7. U24: 엔진 명령 `file.place`·`file.place.queue`에 `underlay` 인자를 더했다(MCP·제어 채널에도 보임). 주지 않으면 동작은 그대로이고, 고전 화면은 쓰지 않는다. 고전 화면의 템플릿 레이어 이름 「Template {파일}」도 그대로다.
8. U25: 판의 묶음 메뉴로 넣을 때만 첫 ★ 줄을 고른다. 엔진 `legend.addGroup`은 지금 줄을 바꾸지 않는다(명세 8의 「있던 레이어는 건드리지 않음」 그대로, 스크립트는 고를 줄을 스스로 정함).
9. U26: 굵기 한 번에 카드는 「바꾸기」 뒤 닫힌다(명세 18은 닫는 때를 정하지 않았음). 다시 열면 처음(×0.8)부터다(C8).
10. U29: 명세 18의 안내 「선 {N}개가 바뀝니다 …」는 따르는 선이 있을 때만 쓰고, 줄만 바뀔 때는 줄 수를 말한다.

시험 변경(이미 있던 시험의 준비를 바꾼 것, 기대는 그대로, 커밋에 `시험 변경:`으로 적음):

| 시험 | 바꾼 것 | 이유 |
|---|---|---|
| `weights_card_value_and_step_modes`(J005) | 「값」 다음 단계마다 바닥 「모든 굵기 ×0.8 …」로 카드를 다시 열고, 「한 단계 굵게」 앞에서 「한 단계」를 다시 고름 | U26으로 「바꾸기」 뒤 카드가 닫히고, 다시 열면 처음(비율)부터라서(C8). 단계마다 확인하는 굵기 값은 그대로 |
| `weights_card_changes_by_the_ratio_just_typed`(J005) | 「20」을 치기 전에 카드를 다시 엶 | 같음. 거절 확인은 그대로 |

재작업 2차 검증(이 절의 커밋에 든 작업 트리 그대로). 명령은 `/home/claude/vector`에서, 로그는 위와 같은 폴더.

| 명령 | 시각(UTC) | 종료 코드 | 결과 | 로그 |
|---|---|---|---|---|
| `cargo xtask ci`(디버그 정보를 끄지 않음) | 22:38경–22:53:35 | 없음(멈춤) | fmt·clippy 통과 뒤 시험 바이너리를 링크하다 디스크가 차서 `ld`가 Bus error로 죽음(`vectorcraft-svg` 시험 `css_rules`). 통과로 세지 않음 | `rw2-ci-xtask-diskfull.log` |
| `CARGO_PROFILE_DEV_DEBUG=0 cargo xtask ci` | 22:55:33–23:03:57 | 0 | 7단계 모두 통과(fmt, clippy, test, assets, brands, layers, wasm). 시험 바이너리 118개, 5,212 통과·0 실패·4 무시(전부터 `#[ignore]`) | `rw2-ci-xtask.log` |
| `CARGO_PROFILE_DEV_DEBUG=0 cargo check --target wasm32-unknown-unknown -p vectorcraft-web` | 23:04:10–23:04:31 | 0 | 경고 2개는 전과 같은 기존 것(`dialogs/missing_links.rs:69`, `i18n/mod.rs:272`) | `rw2-web-check.log` |
| `CARGO_PROFILE_DEV_DEBUG=0 cargo build -p vectorcraft` | 23:04:31–23:06:03 | 0 | 화면 확인용. 검토자가 쓰도록 사본 `scratchpad/factory/J005/vectorcraft-rw2` | `rw2-build-app.log` |

디스크 사고: 이번 재작업의 대상 시험(위 빨강·초록 로그)을 재작업 1차와 달리 `CARGO_PROFILE_DEV_DEBUG=0` 없이 돌렸다. 결과는 같지만 시험 바이너리마다 디버그 정보가 들어가(700 MB–1 GB) 첫 관문이 세션 디스크 한도를 넘겼다. 22:00 뒤에 생긴 빌드물(`target/debug/{deps,examples}`의 2,823개, 19.7 GB, 그리고 그 시각 뒤의 `build/` 폴더)만 지우고 재작업 1차처럼 `CARGO_PROFILE_DEV_DEBUG=0`으로 다시 돌렸다. 저장소와 작업 트리는 건드리지 않았다.

대상 시험은 위 전체 시험 안에서 돌았다(`rw2-ci-xtask.log`에서 셈): engine `tests_legend::` 26, ui-egui `trowel::` 72 · `i18n::` 37 · `theme::` 6, doc `legend::` 10, 모두 통과. 이번에 더한 시험 11개(엔진 4: `a_row_from_a_line_with_many_dashes_opens_again_as_it_was`, `inspect_leaves_out_a_sublayers_legend`, `an_underlay_goes_on_a_template_layer_at_the_bottom`, `queued_underlays_go_to_the_bottom_too`. 화면 7: 위 검토 2차 표의 시험)와 바꾼 기존 시험 2개가 모두 통과했다. 재작업 1차(5,201)보다 11개 많다.

실제 화면(시안과 나란히)과 확인 검토는 이 커밋을 대상으로 하고, 다음 기록 커밋에 적는다.
