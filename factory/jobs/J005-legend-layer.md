# J005 — 범례 = 레이어

상태: verify (빌드 끝, 2026-10-11) · 기준 커밋 `5ac2a6ca5000f9071f1e8555b9e9d5b59104c9bd` · **◆ 파일 형식**(레이어에 `legend` 필드) · **◆ 수치**(선종 대시 길이, 굵기 반올림, 대시 판정 허용치)

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
