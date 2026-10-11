# J003 — 메인 창을 디자인 2.1 3절과 같게

상태: 승인 대기 (AI 검토 통과, 사용자 승인 전) · 기준 커밋 `9ab182e865427a30a617a165527bc40ba011753a` · 코드 커밋 3개(`fc810fb`, 재작업 1차 `d0213f0`, 재작업 2차 `968d241`) · 화면 + 한글 대체 글꼴(`crates/text`) + `file.place` 새 인자(좌표·단위·파일 형식·의존성 변경 없음)

## 목표

사용자 지시(2026-10-11 06:20Z): 승인한 디자인과 실행한 앱이 같아야 한다. 디자인 2.1(https://claude.ai/artifact/3j9imUvqh8aPeZu8HLb6TZ, 2026-10-11 Version 6) 「3. 평면 화면」의 창 짜임(그림 `win-plan`, 1600 × 988)을 실제 앱에 만든다. 2.1 확인 카드(09:25Z, 추천 「추천대로 진행」)의 답을 기다리는 동안 추천 쪽으로 진행한다. 사용자가 2.1을 고치면 이 명세도 고친다.

- 보이는 짜임을 먼저 만든다: 캡션, 메뉴 줄(「도면」 추가), 리본 한 줄, 컨트롤 바, 도구 기둥(고고학 묶음), 도면집, 문서 탭, 캔버스 위 띠·확대 단추·배율, 오른쪽 기둥(그리기 팔레트 · 범례·레이어 · 속성), 상태줄.
- 칸마다 **이미 있는 명령**에 잇는다(열기·저장·펜·격자·자석·선 굵기·레이어 보이기/잠그기 …). 아직 없는 기능은 흐린 「준비 중」으로 두고, 마우스를 올리면 그 기능을 맡은 작업 번호(J005 …)를 보인다.
- 기존 VectorCraft 화면은 지우지 않는다. 새 내장 작업 영역 「Trowel」이 새 화면이고, 창 › 작업 영역에서 다른 작업 영역을 고르면 지금 화면으로 돌아간다. 처음 한 번만 Trowel로 시작한다.
- 화면 글자는 한국어. 원문은 영어로 두고(`tw!` 매크로), 한국어 행을 `ko.tsv`에 더한다.

## 완료 기준

각 항목의 시험은 `crates/ui-egui/src/trowel/tests.rs`(새 파일)에 두고, 따로 적은 것은 그 파일에 둔다. 모든 시험은 egui 헤드리스 프레임으로 실제 그리고 누른다(모델만 부르는 시험으로 대신하지 않는다).

1. **작업 영역과 첫 시작.** `workspaces.rs`: `Workspace`에 `trowel: bool`(serde 기본 false), `builtins()` 끝(`Web` 다음)에 `"Trowel"` — control_bar·rulers·toolbar·dock·status_bar 켬, task_bar 끔, dock_collapsed 켬, toolbar_double·toolbar_advanced 끔, trowel 켬. `capture`(115-133)·`apply`(136-154)가 `trowel`을 옮긴다. `state.rs` `UiState`: `trowel: bool`, `trowel_started: bool`(둘 다 기본 false, 기존 필드·기본값 불변). `pub fn start_trowel(ui: &mut UiState)`: `trowel_started`가 false일 때만 「Trowel」을 적용하고 true로 둔다. 호출: `apps/vectorcraft/src/main.rs` `load_prefs`(553) 바로 뒤, `apps/vectorcraft-web/src/web.rs:122` 뒤. 시험 `workspaces::tests::trowel_starts_once_and_the_classic_layout_comes_back`: 기본 `UiState`에 `start_trowel` → `trowel`·`rulers` 켬, `task_bar` 끔, `workspace == "Trowel"`; 「Essentials」 적용 → `trowel` 끔; 다시 `start_trowel` → 그대로 끔; `serde_json::from_str::<UiState>("{}")` → 두 필드 false; `trowel` 키가 없는 저장된 작업 영역 JSON → false. 기존 `essentials_is_the_default_layout`, `all_requested_workspaces_exist`는 고치지 않고 통과.
2. **창 짜임 크기.** `lib.rs` `ui_frame`(1141-1171)이 `self.ui.trowel`이면 `trowel::frame(self, ui)`, 아니면 기존 블록을 그대로 부른다. 떠 있는 패널·대화상자·명령 찾기·창 가장자리(1172-1177 앞뒤)는 공통. 시험 `regions_match_the_mockup_at_1600_by_988`: 창 1600 × 988, `custom_titlebar = true`, 문서 1개일 때 영역(임시 메모리 `trowel-regions`)이 캡션 32, 메뉴 28, 리본 96, 컨트롤 바 40, 상태줄 32 높이, 도구 기둥 56, 도면집 216, 오른쪽 기둥 304 너비, 문서 탭 32 높이, 캔버스 1024 × 728 (각 ±1 px). `screen_mode` 2 → 캡션·메뉴·리본·컨트롤 바 없음, 3 → 상태줄·기둥·탭도 없음.
3. **캡션.** `custom_titlebar`일 때만 32 px 줄: 흙손 표시 16 px(`brand::paint_mark`), 「{문서} * - Trowel」, 오른쪽 창 단추(`titlebar::caption_buttons`), 나머지는 창 끌기(`titlebar::drag_area`). `chrome::window_title`(119-124)도 같은 형식으로: 문서 없음 「Trowel」, 깨끗 「foo.svg - Trowel」, 고침 「foo.svg * - Trowel」. **시험 기댓값 변경**: `chrome.rs`의 `window_title_follows_the_active_file`, `sync_window_title_sends_only_on_change`의 문자열(이유: 디자인 2.1 캡션 = Strata와 같은 「 - 」와 「 *」 표기). `window_title_with_no_document_is_the_app_name`(「Trowel」)은 그대로 통과. 시험 `caption_shows_the_document_and_unsaved_mark`.
4. **메뉴 줄.** 28 px, 메뉴 10개: 파일 편집 오브젝트 문자 선택 효과 **도면** 보기 창 도움말(원문 File Edit Object Type Select Effect Drawing View Window Help). 「도면」 아래 2 px `oat` 밑줄. 「도면」 말고는 기존 `menus::menu_tree` 내용을 그대로 쓴다. 「도면」 항목: 굵기 한 번에…(J005), 범례대로 되돌리기(J005), 잘린 면(J014), 해발선 만들기(J015), 축척과 좌표계…(J006), 밑그림 맞추기…(J008), 범례표 넣기(J012), 표제란 넣기(J012), Strata로 보내기…(J019) — 모두 흐린 「준비 중」. 네이티브 메뉴(macOS)일 때 창 안 메뉴 줄은 기존처럼 숨는다. 시험 `drawing_menu_sits_before_view_and_waits_for_its_jobs`.
5. **리본.** 높이 96, 묶음 7개 · 칸 24개, 칸 52 × (아이콘 상자 44 × 44, 둥글기 12, 아이콘 24) + 11 px 글자, 묶음 제목 11 px, 오른쪽 「명령 찾기」 150 × 32. 칸 상태 = 쉼(desk)·켜짐(hand 바탕, ground 아이콘, 굵게)·보임(oat 바탕 + ring 테두리)·저장(action 바탕, 흰 아이콘)·준비 중(wash 바탕, 아이콘 45 %). 연결:

   | 묶음 | 칸 → 명령 |
   |---|---|
   | 파일 | 열기 `file.open` · 저장 `file.save`(문서가 고쳐졌으면 저장 모양, 문서 없으면 꺼짐) · 가져오기 `file.place`(문서 없으면 꺼짐) |
   | 새 도면 | 평면 `file.new` · 단면·유물·도판 준비 중(J004) |
   | 밑그림 | 맞추기 준비 중(J008) · 밑그림 `file.place` |
   | 그리기 | 펜 `pen` · 연필 `pencil` · 칠통 `livePaintBucket`(`tool.select`, 지금 도구면 켜짐) · 단면선·표고 준비 중(J009) |
   | 도면 요소 | 방위·축척·범례표·표제란 준비 중(J012) |
   | 보기 | 격자 `view.grid` · 자석 `view.snapToPoint`(켜져 있으면 보임) |
   | 내보내기 | PDF `file.export.pdf` · DXF `ui.dxfOptionsDialog` · Strata로 준비 중(J019) |

   「명령 찾기」 → 명령 찾기 창(`ui.palette_open`). 시험 `ribbon_follows_the_mockup_and_runs_commands`: 칸 순서, 펜 칸을 눌러 도구가 `pen`이 되고 그 칸이 켜짐, 격자 칸이 격자를 켜고 보임 모양, 고친 문서에서 저장 칸이 저장 모양, 준비 중 칸을 눌러도 아무것도 바뀌지 않음, 명령 찾기 칸이 창을 연다.
6. **컨트롤 바.** 40 px. 그리기 도구(pen·pencil·lineSegment·rectangle 묶음·curvature·paintbrush·blobBrush)일 때: 「{도구} ›」 | 범례 칸(현재 레이어 색 패치 + 이름 + 다음 선 굵기 「{w} pt」, 누르면 레이어 목록 → `layer.setCurrent`) | 추정(파선)(J014)·닫으면(J009)·펜 필압(J016) 준비 중 | 안내 「굵기 · 빗금 · 선종은 오른쪽 「그리기 팔레트」에서」 | 오른쪽 자석(`view.snapToPoint`) + 허용치(`prefs.set snapToPointTolerance`, px). 다른 도구일 때: 기존 컨트롤 바 내용(`chrome.rs` 219-319를 함수로 빼서 동작 그대로) + 오른쪽 자석·허용치. 시험 `control_bar_shows_the_layer_and_next_weight`: 펜일 때 현재 레이어 이름과 「1 pt」, 목록에서 다른 레이어를 고르면 현재 레이어가 바뀜, 자석 단추가 스냅을 끄고 켬, 선택 도구일 때 기존 내용.
7. **도구 기둥.** 56 px, 단추 40 × 27(둥글기 8, 아이콘 19), 오른쪽 아래 단축키 글자(`shortcut_editor::tool_shortcut`). 묶음 [V A] | [P N \ M T] | [R S C] | [K I] | [H Z] (주 도구 selection, directSelection, pen, pencil, lineSegment, rectangle, type, rotate, scale, scissors, livePaintBucket, eyedropper, hand, zoom). 칸마다 대신 도구를 길게 누르기·오른쪽 클릭으로 펼친다(`toolbar`의 펼침 그대로). 칸에 보이는 도구 = 지금 도구가 그 칸에 있으면 그것, 아니면 그 칸에서 마지막에 쓴 것, 아니면 주 도구. 「고고학」 제목 아래 단면선·표고·유물 번호·주혈 깊이·경사 표시는 준비 중(J009). 맨 아래 칠·선 상자 30(`panels::proxy`). 시험 `tool_column_has_the_mockup_groups`: 주 도구 14개 순서, 펜 단추를 누르면 `pen`, `curvature`를 고르면 펜 칸이 curvature를 보임, 고고학 단추는 눌러도 도구가 바뀌지 않음.
8. **도면집.** 216 px: 머리 「도면집」 + ＋(`file.newDialog`), 제목 「열린 도면」, 줄 = 열린 문서(아이콘, 이름, 지금 문서는 oat 바탕), 누르면 그 문서로(`document.activate`, 홈 닫음). 바닥 11 px: 「축척 정하지 않음 · 좌표계 정하지 않음」(J006)과 대지 크기(일반 단위). 시험 `drawing_set_lists_open_documents_and_switches`.
9. **문서 탭.** 32 px: 「홈」 + 문서 탭(이름, 지금 탭은 2 px ink 밑줄, 올리면 × → `unsaved::close`), 오른쪽 구석 「{문서} ● 저장 안 됨」(고쳐졌을 때, clay 점). 시험 `tabs_switch_documents_and_show_unsaved`.
10. **캔버스와 위에 뜬 것.** 눈금자 켬, 작업 표시줄(task bar) 끔. 캔버스 위(desk 바탕, ring 테두리): 왼쪽 위 띠(도구 아이콘, 「{도구} · {현재 레이어}」, 도구 안내 첫 문장), 오른쪽 아래 확대 기둥 ＋ － 맞춤(`view.zoomIn`·`view.zoomOut`·`view.fitArtboard`), 그 옆 「화면 {n} %」. 시험 `canvas_overlays_zoom_and_name_the_tool`.
11. **오른쪽 기둥(304).**
    - 그리기 팔레트: 굵기 칩 0.1 0.2 0.3 0.5 0.8 1.0 → `stroke.set {weight}`(지금 굵기와 같은 칩이 켜짐), ＋ 준비 중(J018); 빗금 45°·135°·번갈아 준비 중(J014), 없음 켜짐; 선종 실선 → `stroke.set {dash: null}`(대시가 없으면 켜짐), 파선·점선·일점쇄선 준비 중(J014); 단추 잘린 면(J014)·범례에 더하기·범례대로(J005) 준비 중; ⋯ → 선 패널.
    - 범례 · 레이어: 머리 + 「레이어 {n}」 + ＋(`layer.new`) + ⋯(레이어 패널 메뉴); 제목 「이 도면의 레이어」; 줄(24 px, 위에서부터) = ▸(펼치면 객체 줄 20 px), 눈(`layer.setProps visible`), 자물쇠(`locked`), 색 패치, 이름(현재 레이어 굵게), 객체 수; 누르면 `layer.setCurrent`, 두 번 누르면 레이어 옵션; 바닥 「굵기 한 번에」 단추 준비 중(J005).
    - 속성: 머리 「속성 · {선택 요약}」 + ⌄ 접기; 종류·시대 준비 중(J009); 한 줄 요약(선택의 가로 × 세로, 일반 단위; 없으면 「선택 없음」); 「자세히…」 → 속성 패널.
    - 시험 `palette_weight_chip_sets_the_next_stroke`, `legend_rows_show_hide_lock_and_pick_the_layer`, `properties_summarise_the_selection`.
12. **상태줄.** 32 px, 12 px 흐린 글자: 왼쪽 메시지(`ui.status`, 없으면 도구 안내) | 도구 이름 | 자석 켬/끔 | ● 저장 안 됨 N건(고친 문서 수, 0이면 숨김) | X · Y(`hover_doc`, 일반 단위, 캔버스 밖이면 빈칸) | 「축척·좌표계 정하지 않음」(J006). 시험 `status_bar_shows_tool_snap_unsaved_and_position`.
13. **색.** `theme.rs` `Tokens`에 `ring`, `action`, `on_action`, `clay`, `clay_text`(Plate: #B0AEA5, #B5573A, #FFFFFF, #D97757, #9C4A2F; 다른 프리셋은 제 팔레트에서 고른 값). `trowel/`의 제품 코드에는 색 값이 없다(모두 `Tokens`). 시험 `theme::tests::plate_has_the_trowel_tokens`, `trowel_code_takes_colours_from_tokens`(소스 검사: `Color32::from_`, `Color32::WHITE`, `Color32::BLACK` 없음).
14. **아이콘.** `assets/icons/tw-*.svg` 37개(24 × 24, 선 1.8, 원저작), `icon_data.rs` `ICONS` 행, `ASSETS.md` 행. 시험 `every_trowel_icon_is_bundled`; `cargo xtask assets` exit 0.
15. **번역.** `lib.rs`의 `tl!` 옆에 `tw!(s)` = `i18n::tr_ctx(i18n::current(), "trowel", s)`. `ko.tsv`에 행 추가(같은 원문이 이미 다른 뜻으로 있으면 `trowel` 문맥 행, 아니면 일반 행). 시험 `i18n::tests::every_tw_literal_is_translated_in_korean`(`tw!("…")` 리터럴 전부와 「도면」 메뉴 글자). 기존 i18n 시험 전부 고치지 않고 통과. 다른 언어는 영어 원문으로 보인다(위험 참고).
16. **고전 화면 불변.** `UiState::default()`는 `trowel`이 꺼져 있으므로 기존 시험은 3번에 적은 것 말고는 고치지 않고 통과한다. `cargo test -p vectorcraft-ui-egui` 실패 0.
17. **웹.** `cargo xtask wasm` exit 0(`cargo xtask ci`에 들어 있음).
18. **실제 화면.** `VECTORCRAFT_LOCALE=ko VECTORCRAFT_NO_PREFS=1 xvfb-run …`으로 1600 × 988 창을 띄우고 제어 채널로 시안과 비슷한 문서(이름 「광령리1」, 시안의 레이어 이름 12개, 도형 몇 개, 펜 도구)를 만든 뒤 `ui.screenshot`. `/mnt/project-files/design/trowel-run/J003-plan.png`와 시안 `win-plan.png`를 나란히 붙인 `J003-side-by-side.png`, 남은 차이 목록을 빌드 절에 적는다. 이 문서는 커밋하지 않는다.
19. `cargo xtask ci` exit 0.

## 변경 범위

- 새 파일: `crates/ui-egui/src/trowel/{mod,caption,menu,ribbon,control,tools,drawings,center,palette,legend,props,status,tests}.rs`, `assets/icons/tw-*.svg` 37개.
- `crates/ui-egui/src/lib.rs`: `mod trowel;`, `tw!` 매크로(10 옆), `ui_frame` 분기(1141-1171).
- `crates/ui-egui/src/workspaces.rs`: 필드·내장 「Trowel」·`start_trowel`·시험 1개. 기존 작업 영역 9개의 값 불변.
- `crates/ui-egui/src/state.rs`: `UiState` 필드 2개(기본 false).
- `crates/ui-egui/src/theme.rs`: 토큰 5개와 시험. 기존 토큰 값 불변.
- `crates/ui-egui/src/chrome.rs`: `window_title` 형식과 그 시험 2개, 컨트롤 바 내용을 함수로 빼기(동작 불변), 도구 안내를 `pub(crate)`로.
- `crates/ui-egui/src/toolbar.rs`: 칸·펼침·떠 있는 도구 함수를 `pub(crate)`로(동작 불변). `crates/ui-egui/src/menus.rs`: 메뉴 그리기 함수를 `pub(crate)`로(동작 불변).
- `crates/ui-egui/src/icon_data.rs`, `ASSETS.md`: 아이콘 행.
- `crates/ui-egui/src/i18n/ko.tsv`(행 추가만), `crates/ui-egui/src/i18n/tests.rs`(시험 1개).
- `apps/vectorcraft/src/main.rs`, `apps/vectorcraft-web/src/web.rs`: `start_trowel` 한 줄씩.
- **바꾸지 않는 것**: `crates/engine`(명령·문서 모형·파일 형식), 다른 크레이트, 기존 명령 id·환경설정 키, 고전 화면 함수의 동작(`chrome::app_bar`·`status_bar`·`hint_bar`·`doc_tabs`, `toolbar::show`, `dock::show`), 다른 언어 카탈로그, `docs/brand`.

## 검증 방법

로그는 `/tmp/claude-0/-home-claude/79152f61-3d49-5a1c-acde-dc6cabf2d3d7/scratchpad/factory/J003/*.log`, 종료 코드를 모두 적는다.

```
cd /home/claude/vector
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p vectorcraft-ui-egui --lib -- trowel:: workspaces:: theme:: chrome::tests::window_title chrome::tests::sync_window_title i18n::
cargo test -p vectorcraft-ui-egui
cargo xtask assets
cargo xtask ci
```

실제 화면(18번)은 `cargo build -p vectorcraft` 뒤 `VECTORCRAFT_LOCALE=ko VECTORCRAFT_NO_PREFS=1 xvfb-run -a -s "-screen 0 1600x1000x24" target/debug/vectorcraft --control <포트>`, `ui.resize {width:1600, height:988}`, `engine.execute` 명령들, `ui.screenshot`. 스크린샷과 시안을 나란히 보고 차이를 적는다.

## 범위 밖

- J004 홈 화면·새 도면 견본·도면집 폴더(지금 도면집은 열린 문서 목록), J005 범례 = 레이어 데이터(레이어별 굵기·「따로 N」·굵기 한 번에), J006 축척·TM 좌표(눈금자·상태줄 X/Y·도면집 바닥의 EPSG), J008 밑그림 맞추기, J009 고고학 도구와 종류·시대, J012 도판 요소, J014 빗금·선종·잘린 면, J016 펜 필압, J018 사용자 굵기 추가, J019 Strata로 보내기.
- 시안 그림 속 도면 내용(유구, 방위표, 축척 막대)과 시안의 「광령리1」 도면집 이름: 데이터가 생기는 작업에서 맞춘다.
- macOS 네이티브 메뉴의 「도면」, 다른 언어의 Trowel 글자 번역.

## 위험·사람 검토 필요

- 화면만 바뀐다. 좌표·단위·수치·파일 형식 변경이 없어 ◆ 항목은 없다. 다만 사용자 화면 설정 파일(UI 환경설정 JSON)에 키 3개(`trowel`, `trowel_started`, 작업 영역의 `trowel`)가 더해진다. 옛 파일은 기본값으로 읽히고(serde default), `deny_unknown_fields`가 없어 옛 판도 새 파일을 읽는다.
- 이 판을 처음 켜면 기존 사용자도 한 번 Trowel 화면으로 바뀐다(의도). 되돌리기: 창 › 작업 영역 › Essentials.
- 창 제목 형식이 J001 때와 달라진다(시험 기댓값 2곳 변경, 이유는 3번).
- 다른 언어에서는 Trowel 글자가 영어로 보인다. 사용자가 한국어로만 쓰므로 받아들이고 기록한다.
- 「준비 중」 칸이 많다. 화면은 시안과 같아 보여도 기능은 뒤 작업에서 들어온다. 승인 보고에 준비 중 목록을 그대로 적는다(부분 작업을 「디자인 완성」으로 말하지 않는다).
- 공장 규칙의 「별도 git worktree」 대신 작업 브랜치 제자리에서 만든다: 디스크 여유 12 GB, worktree와 따로 둔 target 디렉터리는 25 GB 넘게 든다. J001·J002·J013과 같은 예외.
- (빌드 중 더함) 대체 글꼴 두 가지(빌드 절 벗어난 점 8)는 Trowel 창 밖에도 닿는다: 고전 화면의 굵은 한글·중국어가 굵어지고, Noto CJK가 없고 Noto Sans KR·나눔고딕이 있는 리눅스에서는 문서 글자의 한글 대체 글꼴도 바뀐다(문서에 고른 글꼴에 한글이 없을 때만). 윈도우·맥의 문서 글자는 그대로다. 문서 그림이 달라질 수 있는 변경이라 사람 검토에 올린다.

## 기준 커밋

`9ab182e865427a30a617a165527bc40ba011753a` (`git rev-parse HEAD`, 2026-10-11)

## 빌드 (build)

2026-10-11, 기준 `9ab182e`(그대로). 작업 브랜치 제자리(위험 절의 예외). 시험 먼저: 빨강 로그 `red1.log`·`red2.log`(Trowel 시험 15개가 함수가 없어 실패), `red3.log`(글꼴 시험), `red4.log`(굵은 키 시험), `red5.log`(캔버스 표지 시험) 뒤에 구현했다.

손댄 파일:

- 새 파일: `crates/ui-egui/src/trowel/{mod,caption,menu,ribbon,control,tools,drawings,center,palette,legend,props,status,tests}.rs`(2,337줄), `assets/icons/tw-*.svg` 37개.
- 고친 파일: `ASSETS.md`(아이콘 37행), `apps/vectorcraft/src/main.rs`·`apps/vectorcraft-web/src/web.rs`(`start_trowel` 한 줄씩), `crates/ui-egui/src/{lib,state,workspaces,theme,chrome,toolbar,menus,dock,icon_data,ui_fonts}.rs`, `crates/ui-egui/src/panels/{layers,stroke}.rs`, `crates/ui-egui/src/i18n/{ko.tsv,tests.rs}`, `crates/text/src/{fontdb.rs,tests_sysfonts.rs}`, `factory/backlog.md`, 이 파일.

완료 기준별 구현:

| 기준 | 구현 |
|---|---|
| 1 | `workspaces.rs` 내장 「Trowel」과 `start_trowel`, `state.rs` 필드 2개, 앱 두 곳에서 호출 |
| 2 | `lib.rs` `ui_frame`이 `trowel::frame`으로 갈라짐. 패널 id `trowel-caption` … `trowel-right` |
| 3 | `trowel/caption.rs`, `chrome::window_title` 「{문서} * - Trowel」 |
| 4 | `trowel/menu.rs` + `menus::menu_bar_with`(「도면」을 「보기」 앞에 끼움), 9항목 모두 준비 중 |
| 5 | `trowel/ribbon.rs` 묶음 7개·칸 23개, 칸 모양 5가지 |
| 6 | `trowel/control.rs`: 그리기 도구일 때 범례 칸(레이어 목록)·준비 중 3개·안내·자석·허용치, 다른 도구는 `chrome::control_bar_contents` |
| 7 | `trowel/tools.rs`: 묶음 5개·주 도구 14개, 펼침은 `toolbar::open_flyout`·`flyout_press`, 고고학 5개 준비 중 |
| 8 | `trowel/drawings.rs`: 열린 문서 목록, ＋ 새 문서, 바닥 3줄 |
| 9–10 | `trowel/center.rs`: 탭 줄(홈·문서·×·구석 「저장 안 됨」), 캔버스, 띠·확대 단추·배율 카드 |
| 11 | `trowel/palette.rs`(높이 166), `trowel/legend.rs`, `trowel/props.rs`(112, 접으면 33) |
| 12 | `trowel/status.rs` |
| 13 | `theme.rs` 토큰 5개, `trowel/`에 색 값 없음 |
| 14 | 아이콘 37개 + `icon_data.rs`·`ASSETS.md` 행 |
| 15 | `lib.rs` `tw!`, `ko.tsv`에 `trowel` 문맥 89행 |
| 16 | 고전 화면 함수는 `pub(crate)`로 열거나 몸통을 함수로 뺀 것 말고 그대로 |

스펙에서 벗어난 점과 이유:

1. 기준 5: 칸은 **23개**다. 명세의 24개는 셈 실수(시안도 23개).
2. 기준 2: 영역은 임시 메모리 `trowel-regions` 대신 egui `PanelState`와 `canvas_rect`로 잰다. 「캔버스 1024 × 728」은 눈금자를 포함한 영역이고, 그리기 칸은 1008 × 712(시험은 (288, 244)–(1296, 956)).
3. 기준 4: 메뉴 이름은 앱 전체 한국어 번역 그대로다(Object = 「객체」). 시안은 「오브젝트」. 한 앱에서 같은 낱말을 두 가지로 쓰지 않으려고.
4. 기준 10: 캔버스 위 카드는 desk 바탕·ring 테두리 대신 surface 바탕·border 테두리·그림자(시안 그림의 실제 모양, 어두운 테마는 그림자 없음). 배율은 앱의 배율 글자(`state::zoom_label`, 예 「77.44 %」).
5. 기준 11: 레이어 줄 두 번 누르기(레이어 옵션)는 만들지 않았다. ⋯가 레이어 패널을 연다. 속성의 「자세히…」 글자 대신 ⋯ 단추.
6. 기준 12: 고친 문서가 0이면 칸을 숨기지 않고 「모두 저장됨」을 보인다(오른쪽 칸들이 흔들리지 않게).
7. 기준 18: 문서 이름은 「1호 주거지 평면」(시안의 탭 이름). 「광령리1」은 조사 이름이라 도면집 데이터(J004·J006)가 생기면 쓴다. 화면은 `xvfb-run` 대신 스크립트 `scratchpad/j003/drive.py`(Xvfb :97, `LANG=ko_KR.UTF-8`, 제어 채널 `app.language ko`)로 찍었다.
8. **명세에 없던 변경**(스크린샷을 시안과 견주다 찾은 차이를 고침):
   - 한글 굵은 글씨: 설치된 대체 글꼴이 모든 글꼴 묶음에 보통 굵기 하나로만 들어가, 굵게 쓴 제목·켜진 칸·현재 레이어가 한글에서는 보통 글씨로 보였다. 대체 글꼴 가족의 Bold 면을 semibold 묶음에 먼저 넣는다(`ui_fonts.rs`). 시험 `ui_fonts::tests::{a_fallback_faces_bold_is_its_familys_bold_style, the_semibold_family_draws_with_the_fallback_familys_bold_face}`. 고전 화면의 굵은 한글·중국어도 같이 굵어진다.
   - 리눅스 한글 글꼴: Noto CJK가 없는 리눅스에서는 한글이 파일 경로 순서로 먼저 나오는 Unifont(비트맵풍)로 그려졌다. 선호 대체 목록(`crates/text/src/fontdb.rs` `SYSTEM_FALLBACKS`)의 Noto Sans CJK JP 다음에 Noto Sans CJK KR·Noto Sans KR·NanumGothic을 더했다. 윈도우(맑은 고딕이 먼저)·맥은 그대로다. 이 목록은 문서 글자의 대체 글꼴에도 쓰이므로, 그 글꼴이 있는 리눅스에서는 문서의 한글 대체도 Unifont 대신 그 글꼴이 된다. 시험 `tests_sysfonts::hangul_falls_back_to_a_korean_ui_family_when_one_is_installed`(한국어 글꼴이 없는 기계에서는 건너뜀). **명세의 「바꾸지 않는 것: 다른 크레이트」를 벗어나므로 위험 절에 더한다.**
   - 도구 안내: 캔버스 띠와 상태줄에서 키 이름을 굵게(고전 안내 줄과 같게), 부분 사이를 「|」 대신 「·」로. 시험 `trowel::tests::hints_show_their_keys_in_bold`.
   - 한국어: 어색한 도움말 7줄을 고쳤다(예 「첫 번째 점을 클릭하세요 닫으려면」 → 「첫 번째 점 클릭 닫기」, 「완료하려면」 → 「완료」. 원문은 그대로, 고전 화면 안내 줄에도 같이 보임). 캔버스에 영어로 뜨던 표지 8개(anchor, center, path, artboard, bleed, guide, align, Puppet Warp 경고)에 행을 더했다. 시험 `i18n::tests::canvas_labels_are_translated_in_korean`. Trowel의 「{n} objects」는 「개체」 대신 앱 전체 용어 「객체」.
   - `chrome::first_tab_shown`, `panels::layers::{layer_colour, painted_name}`, `panels::stroke::set`, `dock::set_column_left`를 `pub(crate)`로(동작 그대로).

### 시안과 남은 차이 (`/mnt/project-files/design/trowel-run/J003-side-by-side.png`, 왼쪽 시안·오른쪽 앱)

| # | 곳 | 시안 | 앱 | 맡은 작업 |
|---|---|---|---|---|
| 1 | 흐린 「준비 중」 칸 | 모두 쓸 수 있는 모양 | 아래 목록은 흐림, 올리면 작업 번호 | 목록 참고 |
| 2 | 도면집 | 평면·단면·유물·도판 묶음, 도면마다 축척, 머리에 조사 이름 | 「열린 도면」 목록 | J004 |
| 3 | 범례 · 레이어 | 줄마다 선 견본·굵기(pt)·★·「따로 N」, 「조사 데이터 / 그리기」 두 묶음, 「평면용 ▾」 | 레이어 색 패치·이름·객체 수, 「레이어 12개」 | J005 |
| 4 | 축척·좌표 | TM 좌표 눈금자, 탭·배율 칸·상태줄의 「1:20」, 「5186 중부원점 · 높이 해발」, 도면집 바닥의 EPSG·원점 | mm 눈금자, 「축척·좌표계 정하지 않음」 | J006 |
| 5 | 그리는 중 정보 | 띠 「점 5 · 3.86 m」, 속성 「이 번 1.19 m · 지금까지 3.86 m」, 상태줄 상황 문장 | 띠·상태줄은 도구 안내, 속성은 선택 크기 | J009 |
| 6 | 컨트롤 바 범례 칸 | 「굴광선 (유구 면) 0.8 pt」, 선 견본 | 「굴광선 0.8 pt」, 레이어 색 | J005 |
| 7 | 캔버스 그림 | 밑그림 스캔·방위표·축척 막대·번호 | 도면 데이터라 없음(스크립트가 그린 도형만) | J008·J012 |
| 8 | 허용치 | 12 px | 2 px(앱 기본값, 설정 범위 1–8 px) | 바꾸지 않음 |
| 9 | 메뉴 | 「오브젝트」 | 「객체」 | 위 벗어난 점 3 |
| 10 | 창 단추 | 작고 흐린 글자 단추 | 윈도우 표준 크기(공통 코드 `titlebar`) | 바꾸지 않음 |
| 11 | 글꼴 | Noto Sans KR(라틴 글자까지) | 라틴·숫자·「·」는 Source Sans 3, 한글은 설치된 글꼴(윈도우 맑은 고딕, 이 화면은 나눔고딕). 「·」 둘레가 시안보다 좁음 | 바꾸지 않음 |
| 12 | 배율 | 「34 %」 | 「77.44 %」(앱 표기) | 바꾸지 않음 |
| 13 | 속성 머리 | ⌄ 하나 | ⋯(속성 패널) ⌄ | 위 벗어난 점 5 |
| 14 | 탭 구석 | 「광령리1 ● 저장 안 됨」(조사 이름) | 「1호 주거지 평면 ● 저장 안 됨」(문서 이름) | J004 |
| 15 | 세로 눈금자 | 숫자를 돌려 씀 | 숫자를 한 자씩 세로로(기존 눈금자) | 바꾸지 않음 |

「준비 중」(흐림, 올리면 작업 번호): 리본 단면·유물·도판(J004), 맞추기(J008), 단면선·표고(J009), 방위·축척·범례표·표제란(J012), Strata로(J019) / 메뉴 「도면」 9항목 / 컨트롤 바 추정(파선)(J014)·닫으면(J009)·필압 따라(J016) / 도구 기둥 고고학 5개(J009) / 팔레트 굵기 ＋(J018), 빗금 45°·135°·번갈아·선종 파선·점선·일점쇄선·잘린 면(J014), 범례에 더하기·범례대로(J005) / 범례 바닥 모든 굵기 바꾸기·따로→범례대로(J005) / 속성 종류·시대·번호(J009) / 상태줄 축척·좌표계(J006).


## 검증 (verify)

2026-10-11. 변경 커밋 `fc810fbed1b0a2d04d27ba23ae318c0135c92bc6`(`J003: 메인 창을 디자인 2.1 평면 화면처럼 만듦`, 기준 `9ab182e` 위 1개). 로그는 `/tmp/claude-0/-home-claude/79152f61-3d49-5a1c-acde-dc6cabf2d3d7/scratchpad/factory/J003/`. 모든 명령은 `/home/claude/vector`에서.

시험 먼저(빨강) → 구현 뒤(초록):

| 명령 | 종료 코드 | 결과 | 로그 |
|---|---|---|---|
| `cargo test -p vectorcraft-ui-egui --lib -- workspaces::tests::trowel theme::tests::plate_has chrome::tests::window_title chrome::tests::sync_window_title i18n::tests::every_tw` | 101 | 1 통과 · 5 실패(기대한 빨강: 작업 영역·토큰·창 제목 형식·번역 행이 아직 없음) | `red1.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel:: workspaces:: theme:: chrome::tests::window_title chrome::tests::sync_window_title i18n::` | 101 | 50 통과 · 15 실패(기대한 빨강: Trowel 화면 함수가 빈 껍데기) | `red2.log` |
| 같은 명령(구현 뒤) | 0 | 65 통과 | `green1.log` |
| `cargo test -p vectorcraft-text --lib -- tests_sysfonts::hangul` · `cargo test -p vectorcraft-ui-egui --lib -- ui_fonts::` | 101 · 101 | 한글 대체가 Unifont로 나와 실패 · `bold_face`가 없어 컴파일 실패(기대한 빨강) | `red3.log` |
| `cargo test -p vectorcraft-text --lib -- tests_sysfonts::` · `cargo test -p vectorcraft-ui-egui --lib -- ui_fonts:: trowel::` | 0 · 0 | 20 통과 · 21 통과 | `green2.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel::tests::hints` | 101 | 1 실패(기대한 빨강: 키가 굵지 않고 「|」가 보임) | `red4.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel::` | 0 | 16 통과 | `green3.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- i18n::tests::canvas_labels` | 101 | 1 실패(기대한 빨강: 「path」 등이 영어) | `red5.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- i18n:: trowel:: ui_fonts::` | 0 | 59 통과 | `green4.log` |
| `cargo build -p vectorcraft` (화면용 두 번) | 0 · 0 | | `build-dev.log` · `build-dev2.log` |
| `python3 scratchpad/j003/drive.py target/debug/vectorcraft <out> 7991`(Xvfb 1600 × 988, 한국어, 제어 채널로 문서·레이어 12개·도형·펜 도구) | 0 | `J003-plan.png`, 시안과 나란히 `J003-side-by-side.png`, 차이 15가지는 빌드 절 | `/mnt/project-files/design/trowel-run/app.log` |

전체 관문:

| 명령 | 종료 코드 | 결과 | 로그 |
|---|---|---|---|
| `cargo xtask ci` 1회차 | 1 | clippy `type_complexity` 1건(새 코드 `ui_fonts.rs`의 대체 글꼴 수신 채널 타입) → `type Fallback`으로 고침 | `ci-1.log` |
| `cargo xtask ci` 2회차 | 1 | fmt·clippy 통과, test 단계에서 링커가 `Bus error`로 죽음: **디스크가 가득 참(환경 부족, 코드 실패 아님)**. 묵은 빌드 파일(`target/debug/{deps,build,.fingerprint}` 23 GB)을 지우고 처음부터 다시 | `ci-2.log` |
| `cargo xtask ci` 3회차(`fc810fb`, 미커밋 변경 없음) | 없음(멈춤) | fmt·clippy 통과, test 단계 컴파일 중 12:16:02Z에 멈춤: 같은 커밋의 GitHub 실행(아래 행)이 통과해 디스크를 아끼려고. **로컬 3회차는 통과로 세지 않는다** | `ci-3.log` |
| GitHub Actions 「Trowel CI」 [실행 38137425667](https://github.com/kesekikwon-netizen/vector/actions/runs/38137425667)(push `fc810fb`) | success | 「cargo xtask ci (Linux)」 success(11:56:58Z–12:11:32Z), 「Windows x64 release build」 success(–12:15:05Z) | GitHub 실행 기록 |

판정: 첫 커밋 `fc810fb`는 전체 관문을 GitHub에서 통과했다. 이어서 독립 검토로 넘김.

## 검토 (review) 1차

2026-10-11, 대상 커밋 `fc810fb`(기준 `9ab182e`). 검토자 둘은 구현자와 다른 에이전트이고 읽기만 했다(저장소를 고치지 않음). 보고서 전문은 스크래치패드 `review1-ui.md`, `review1-code.md`.

- UI 검토(ui-reviewer, 12:14Z): 시안 `win-plan.png`와 앱 `J003-plan.png`를 픽셀로 견줌. 영역 경계·리본 묶음·범례 줄 위치는 1 px 안에서 같음, 테마 pass. **판정 rework**(1번 하나로).
- 정확성·보안 검토(code-reviewer + correctness-reviewer + security-reviewer 한 번에, 12:21Z): 코드만 읽고 빌드·시험은 돌리지 않음. 패닉 경로·명령 id·고전 화면 불변·wasm·보안(새 외부 입력은 prefs의 bool 3개뿐)은 문제없음. **판정 rework**(1·2번).

UI 검토 지적(줄 번호는 `fc810fb`):

| # | 심각도 | 곳 | 내용 | 조치 |
|---|---|---|---|---|
| U1 | major | `trowel/legend.rs:173`, `control.rs:136-158`, `center.rs:135` | 잠기거나 숨긴 레이어를 「그릴 레이어」로 보여 주지만 엔진(`insertion_parent`)은 다른 레이어에 넣는다. 알림이 없어 범례가 조용히 틀어진다 | 고침 R1 |
| U2 | major(조건부) | `ribbon.rs:148`, `tools.rs:39-86` | 1600 × 988보다 작은 창에서 리본 끝·명령 찾기·도구 기둥 아래가 잘림 | 고침 R2 |
| U3 | minor | `caption.rs:13-26` | 캡션과 메뉴 사이 선 없음 | 고침 R6 |
| U4 | minor | `mod.rs:152`, `drawings.rs:45`, `control.rs:71` | 판 제목·도면집 지금 줄·켜진 「자석」이 굵음(시안은 보통) | 고침 R6 |
| U5 | minor | `control.rs:55` | 허용치 칸만 모양이 다름 | 고침 R7 |
| U6 | minor | `props.rs:52-53` | 고른 것이 없어도 흙색 「● 필요」 | 고침 R6(J009까지 숨김) |
| U7 | minor | `mod.rs:171-196` 외 | 「준비 중」을 눌러도 반응 없음, 상자형은 살아 있는 칸과 비슷, 툴팁에 작업 번호만 | 고침 R8 |
| U8 | minor | `menu.rs:27` | 메뉴의 「범례대로」가 끝나지 않은 말 | 고침 R6 |
| U9 | minor | `ko.tsv:4124, 4162` | 「개체」와 「객체」가 섞임 | 고침 R12 |
| U10·U11·U13–U16 | nit | `ribbon.rs:142`, `menu.rs:18`, `center.rs:193-194`, `control.rs:120-133`, `legend.rs:147-149`, `drawings.rs:79` | 리본 끝 세로선, 「도면」 밑줄 자리, 확대 ＋/－ 흐림, 범례 칸(굵기 색·▼·너비 196), 범례 ▸·자물쇠 색, 도면집 바닥 크기 글꼴 | 고침 R6 |
| U12 | nit | `brand.rs:39-42` | 캡션의 흙손 표시(16 px)에서 흙색 기준선이 거의 검게 보임 | **남김**: 16 px 전용 그림이 필요(J001 아이콘 일). 아래 「남긴 것」 |
| U17 | nit | `ko.tsv` | 안내 글 다듬기(「선택에 추가」, 「시작점」, 객체 선택/복제, 드래그/끌기, 실선 툴팁) | 고침 R12 |
| U18 | nit | `tools.rs` | 숨은 도구가 있다는 표시 없음 | 고침 R11 |
| U19 | nit | 스크린샷 | 포인터가 패스 위라 스마트 가이드 표지 「패스」가 찍힘 | 다시 찍을 때 포인터를 빈 곳에 |
| U-a | 기록 이견 | 차이 표 #8 | 허용치 시안 12 px, 앱 범위 1–8 px라 넣을 수 없음 | **사용자 결정**으로 넘김(백로그 J016 메모) |
| U-b | 기록 이견 | 차이 표 #12 | 배율 칸만 정수로 하면 시안과 같음 | 고침 R6 |
| U-c | 기록 이견 | 벗어난 점 8 | 안내가 굵은 먹색 키, 시안은 조용한 muted 한 문장 | 고침 R9 |
| U-d | 기록 이견 | 기준 5 | 밑그림 타일이 가져오기와 같은 창을 엶 | 고침 R10(아래 사용자 확인 항목) |
| U-e | 범위 밖 | `theme.rs:181` | 「Measured from Illustrator 2026」 주석(J003 이전 커밋 `9449c92`), 클린룸 규칙과 부딪힐 수 있음 | **사람 확인**(이 작업에서 안 고침) |

정확성·보안 검토 지적(줄 번호는 `fc810fb`):

| # | 심각도 | 곳 | 내용 | 조치 |
|---|---|---|---|---|
| C1 | major | `tools.rs:31-98`, `ribbon.rs:128-162` | 고정 배치라 흔한 화면(1080p 125 %, 1440 × 900, 웹, 최소 창 800 × 500)에서 칠·선 상자·도구·리본 끝이 잘리고 눌리지 않음 | 고침 R2 |
| C2 | major | `legend.rs:98-109, 56-75` | 큰 레이어를 펼치면 프레임마다 모든 줄을 배치하고 선택 `Vec`을 O(n·m)으로 찾아 앱이 멈춤 | 고침 R3 |
| C3 | minor | `status.rs:38-46, 51-59` | 첫 메시지 뒤로 상태줄에 도구 안내가 다시 나오지 않음(`ui.status`는 비워지지 않음) | 고침 R4 |
| C4 | minor | `fontdb.rs:413-416` | 새 한국어 대체 글꼴 셋이 한글만이 아니라 한자·가나에도 쓰임. 새 시험은 그 줄을 지키지 못함 | 고침 R5 |
| C5 | minor | `control.rs:79-102` | 「준비 중」 상자가 잘리지 않아 좁은 창에서 자석·허용치 위에 겹치고 클릭을 가져감 | 고침 R7 |
| C6 | minor | `control.rs:10-27` | 그리기 도구 목록이 명세보다 넓어 기준점 도구에서 고전의 변환 단추가 사라짐 | 고침 R7. 펜·곡률 도구의 변환 단추가 빠지는 것은 명세의 선택이라 사람에게 알림 |
| C7 | minor | `dock.rs:19, 220, 243, 279`, `menus.rs:1136-1145` | 튀어나오는 패널이 리본·컨트롤 바를 덮음, 자동 접기 예외 띠가 범례 ▸·눈 위, 패널 펼침을 끄면 범례·속성 ⋯가 아무 일도 안 함 | 고침 R13 |
| C8 | minor | `workspaces.rs:53-61` | 첫 실행의 `start_trowel`이 기존 배치를 되돌릴 길 없이 덮음 | 고침 R14 |
| C9 | minor | `tests.rs:445-466, 553-556` | 깨져도 통과하는 시험 2개 | 고침 R15 |
| C10 | nit | `ui_fonts.rs:135-160`, `ko.tsv`, `drawings.rs:82`, `tools.rs:47-55`, `status.rs:51-59` | 가변 글꼴의 Bold 인스턴스, 면 단위 중복 막기, 「개체」, 도면집 바닥 툴팁이 바닥 전체, 도구 칸 기억이 temp, 상태 문구 복사 | 고침 R5·R12·R16 |

## 재작업 (rework) 1차

2026-10-11, 기준 `fc810fb`(J003 첫 커밋, 이미 push됨). 같은 작업 브랜치 제자리. 커밋은 `J003: …` 두 번째 하나로 더한다: 첫 커밋이 push돼 있어 고쳐 쓰면(amend) 강제 push가 필요하므로, 「작업당 커밋 1개」를 「빌드·재작업 단계마다 1개」로 읽는다.

시험 먼저: 고친 동작마다 실패하는 시험을 먼저 쓰고 빨강을 본 뒤 고쳤다. `red6.log`(잠김 레이어), `red7.log`(작은 창 1536: 「명령 찾기」가 사라짐), `red8.log`(화면 시험 9개 실패 + 글꼴 크레이트는 `fallback_families`가 없어 컴파일 실패), `red9.log`(좁은 창에서 「준비 중」 글자가 자석 위에) → `green5.log`·`green6.log`·`green7.log`. 지키지 못한 곳 두 가지를 그대로 적는다.

- 리본 맞춤 코드를 시험보다 먼저 썼다. 알아차린 뒤 그 코드를 스크래치패드(`j003/ribbon-draft.rs`)로 옮기고 `ribbon.rs`의 내 미커밋 변경만 되돌린 다음, 시험을 써서 빨강(`red7.log`)을 보고 다시 넣었다.
- 허용치 칸(R7)의 시험 `snap_tolerance_is_picked_from_a_list`는 코드 뒤에 썼다. 그래서 고르는 줄을 일부러 끈 판으로 돌려 이 시험이 실패하는 것(4 px를 골라도 2 px 그대로)을 확인하고 되돌렸다(`green7-mutation.log`, 종료 101). 그 실행은 시험 두 개의 문제도 드러냈다. 새 시험 `control_bar_keeps_snap_in_reach_in_a_small_window`의 허용 목록이 「2 px」 한 덩어리였는데 새 허용치 칸은 「2」와 「px」를 따로 그린다(목록을 고침, 이번 회차에 새로 쓴 시험이라 시험 변경 아님). 기존 시험 `the_semibold_family_draws_with_the_fallback_familys_bold_face`는 R5의 새 규칙과 부딪혔다(아래 「바꾼 시험」 넷째).

| # | 지적 | 바꾼 것 | 시험 |
|---|---|---|---|
| R1 | U1 | 새 `trowel::draw_target`: 새 선이 실제로 들어갈 레이어(`insertion_parent`)와, 지금 레이어가 잠기거나 숨었을 때의 까닭(「{layer} 잠김: {target}에 그립니다」 등 4가지, 그릴 곳이 없으면 「그릴 수 있는 레이어가 없습니다」). 캔버스 띠는 실제 레이어 이름 + 흙색 까닭, 컨트롤 바 범례 칸은 흙색 테두리 + 실제 이름 + 툴팁, 범례의 지금 줄은 흙색 이름 + 툴팁. 범례 칸 목록에서 잠기거나 숨긴 레이어는 흐리게, 오른쪽에 자물쇠·눈 표시 | `a_locked_or_hidden_current_layer_shows_where_lines_go`(잠그고 숨긴 뒤 띠·범례 칸·범례 줄 글자와 펜으로 그린 선의 부모) |
| R2 | U2, C1 | 리본: 창이 좁아지면 ① 52 px 칸 + 「명령 찾기」 150 → ② 명령 찾기를 32 px 돋보기 단추로 → ③ 44 px 칸, 그래도 넘치면 리본을 가로로 스크롤(휠로도). 묶음 끝마다 세로선. 도구 기둥: 단추 높이 27 → 23, 그래도 모자라면 도구 칸을 세로로 스크롤하고 칠·선 상자는 바닥에 고정 | `small_windows_keep_every_tile_and_tool_in_reach`(1536 × 816, 1366 × 728, 800 × 500에서 23칸·14슬롯·고고학 5개·칠·선 상자가 패널 안에 있거나 스크롤로 닿음) |
| R3 | C2 | 범례 목록을 `ScrollArea::show_viewport`로: 보이는 줄만 배치·`interact`, 객체 이름도 보이는 줄만 만든다. 선택은 프레임마다 한 번 `HashSet`. 줄 위치는 레이어마다 누적 높이로 계산 | `a_big_open_layer_lays_out_only_the_rows_in_view`(객체 1,500개를 펼쳐도 배치되는 줄 5–59개, 휠로 맨 아래까지 가서 그 줄을 눌러 선택) |
| R4 | C3 | 상태줄 왼쪽 메시지는 바뀐 뒤 6초만 보이고 다시 도구 안내로. 진행 중인 작업 표시(스피너 문구)는 계속. 고전 상태줄과 문구를 `chrome::status_message` 하나로 합침 | `status_bar_shows_messages_for_a_while` |
| R5 | C4, C10 | `fontdb.rs`: 한국어 가족 셋(Noto Sans CJK KR, Noto Sans KR, NanumGothic)을 `SYSTEM_FALLBACKS`에서 빼서 `HANGUL_FALLBACKS`로, **한글 글자에만** Noto Sans CJK JP 바로 뒤에서 시도(`fallback_families`). 한자·가나·나머지는 이 작업 전과 같은 목록. `ui_fonts.rs`: 중복 막기를 (글꼴, 가족) 단위로, 굵은 면은 semibold 가족에서는 보통 면 앞·다른 가족에서는 보통 면 뒤에도 넣음(egui는 글꼴이 처음 들어올 때만 가족을 받으므로, 나중에 그 굵은 면이 대체 글꼴로 다시 와도 이미 모든 가족에 있음). 가변 글꼴의 이름 있는 인스턴스는 굵은 면으로 고르지 않음 | `tests_sysfonts::korean_fallback_families_are_for_hangul_only`(설치된 글꼴 없이 도는 순수 함수 시험), `ui_fonts::tests::a_bold_face_that_is_a_fallback_too_joins_every_family`. 가변 글꼴 건너뛰기는 이 기계에 가변 글꼴이 없어 시험 없음 |
| R6 | U3, U4, U6, U8, U10·U11·U13–U16, U-b | 캡션 아래 선, 판 제목·도면집 지금 줄·켜진 자석 보통 굵기, 「● 필요」는 J009까지 숨김, 메뉴 항목 「범례대로 되돌리기」(팔레트 단추는 시안대로 「범례대로」), 리본 내보내기 뒤 세로선, 「도면」 밑줄을 글자 바로 밑(글자 폭 + 양쪽 10 px), 확대 ＋/－를 기기 픽셀에 맞춤, 범례 칸 너비 196·굵기 faint·▼, 범례 ▸(muted 삼각형)·눈(켬 hand, 끔 ring)·자물쇠(잠김 ink, 풀림 ring)·숨긴 줄 faint·객체 줄 ink2에 가까운 `icon`, 도면집 바닥 크기는 mono 먹색, 배율 칸은 정수 「77 %」(10 % 아래는 소수 한 자리) | `canvas_overlays_zoom_and_name_the_tool`(배율 형식), 나머지는 새 화면으로 확인 |
| R7 | U5, C5, C6 | 허용치 칸을 시안 `.field`처럼(둥글기 8, 숫자 mono 먹색, 「px」 faint), 누르면 1–8 px 목록. 컨트롤 바의 두 배치를 `room`으로 자른 자식 `Ui`에서 그림. 그리기 배치 도구를 명세 목록으로 줄임(기준점 추가·삭제·변환, 매끄럽게, 원호는 고전 컨트롤 바) | `snap_tolerance_is_picked_from_a_list`, `control_bar_keeps_snap_in_reach_in_a_small_window`, `control_bar_shows_the_layer_and_next_weight` |
| R8 | U7 | 「준비 중」을 누르면 상태줄에 「{기능}: 준비 중 (J0xx)」 4초. 상자형 준비 중 칸은 wash 바탕(리본처럼), 툴팁은 「{기능} · 준비 중 · J0xx」 | `status_bar_shows_messages_for_a_while`(리본 단면 칸, 범례 「모든 굵기 바꾸기…」) |
| R9 | U-c | 띠와 상태줄의 도구 안내를 시안처럼 한 가지 글꼴·muted 색으로(키를 굵게 하지 않음), 부분 사이는 「·」 | `hints_read_plainly_as_in_the_mockup` |
| R10 | U-d | 밑그림 타일: 템플릿 레이어가 없으면 「가져오기」 창을 **템플릿을 미리 고른 채** 연다(`file.place {template: true}`, 새 인자). 템플릿 레이어가 있으면 보이기/숨기기(`layer.setProps`)이고, 보일 때 격자·자석처럼 「보임」 모양 | `underlay_tile_places_a_template_then_shows_and_hides_it` |
| R11 | U18 | 다른 도구가 숨은 도구 칸의 툴팁 끝에 「길게 누르면 다른 도구가 나옵니다」 | 화면 확인(툴팁) |
| R12 | U9, U17, C10 | `ko.tsv`: 「개체 목록 접기/펼치기」 → 「객체 …」, 안내 「Shift+클릭 추가」 → 「Shift+클릭 선택에 추가」, 「첫 번째 점 클릭」 → 「시작점 클릭」, 「객체를 선택」 → 「객체 선택」, 안내의 「끌기」 → 「드래그」(「핸들 드래그」도), 실선 툴팁 「끊지 않은 선」 → 「끊김 없이 이어진 선」. 새 원문 10줄의 한국어, 안 쓰게 된 2줄(「Coming soon · {job}」, 「Required」) 삭제. 일반 행(고전 화면 안내 줄에도 보임)은 번역만 바뀌고 원문은 그대로 | `i18n::tests::every_tw_literal_is_translated_in_korean` 외 i18n 시험 전부 |
| R13 | C7 | `dock.rs`: 튀어나오는 패널 자리를 `Flyout {left, top, keep}`로. 고전 도크는 지금처럼(위 110, 아이콘 기둥 38 px 띠), Trowel은 오른쪽 기둥 왼쪽·기둥 위 + 6, 자동 접기 예외는 ⋯ 단추 셋뿐. Trowel에서는 레이어·속성·라이브러리 패널도 늘 튀어나옴(`window.panel`과 그 체크 표시도) | `panels_pop_out_beside_the_right_column` |
| R14 | C8 | `start_trowel`: 지금 배치가 그 작업 영역과 다르면 덮기 전에 「Trowel 이전 배치」라는 사용자 작업 영역으로 남김(이미 그 이름이 있으면 건너뜀). 기본 배치 그대로면 남기지 않음 | `workspaces::tests::trowel_keeps_a_changed_layout_to_go_back_to` |
| R15 | C9 | 확대 단추·띠·배율 칸을 펜으로 눌러도 문서 객체 수가 그대로인지, 두 번째 잠금 클릭 뒤 잠금이 풀렸는지 확인을 더함 | 그 두 시험 |
| R16 | C10 | 도면집 바닥의 「준비 중 · J006」 툴팁을 축척·좌표계 두 줄에만, 도구 칸이 마지막에 쓴 도구를 저장되는 `app.ui.slot_tool`(키 `trowel/<칸>`)에 기억 | 화면·기존 시험 |

**바꾼 시험(시험 변경).** 기존 시험의 기댓값을 바꾼 것은 넷이고, 모두 위 지적을 따른 동작 변경 때문이다.

- `trowel::tests::hints_show_their_keys_in_bold`(첫 커밋) → 지우고 `hints_read_plainly_as_in_the_mockup`로 바꿈. 키를 굵게 한 것은 첫 빌드가 명세에서 벗어난 점 8이었고, UI 검토(U-c)가 시안과 다르다고 짚었다.
- `trowel::tests::canvas_overlays_zoom_and_name_the_tool`: 배율 글자 기댓값 「77.44 %」(앱 `zoom_label`) → 정수 「77 %」(U-b). 객체 수 확인을 더함(C9).
- `trowel::tests::control_bar_shows_the_layer_and_next_weight`: 기준점 변환·추가, 매끄럽게 도구는 그리기 배치가 아니라 고전 컨트롤 바를 기대(C6).
- `ui_fonts::tests::the_semibold_family_draws_with_the_fallback_familys_bold_face`: 다른 가족에 굵은 면이 「없다」 → 「보통 면 뒤에만 있다」(R5). 시험 설명의 뜻(다른 가족은 보통 면으로 그림)은 그대로다.

나머지 기존 시험은 기댓값을 바꾸지 않았다(`legend_rows_show_hide_lock_and_pick_the_layer`는 확인만 더함; 리본·도구 목록은 같은 값의 상수 `TILES`·`SLOTS`·`ARCH`로 옮김).

**명세에서 벗어난 점(이번 회차).**

- 허용치 칸은 고전 `range_field` 대신 목록에서 고른다. 직접 쳐 넣기와 휠로 한 단계씩은 없어졌다(값은 전처럼 1–8 정수).
- `file.place`에 인자 `template`(참·거짓, 「템플릿」 칸을 미리 체크)을 더했다. 기존 인자·동작은 그대로다.
- 고친 파일: `crates/text/src/{fontdb,tests_sysfonts}.rs`, `crates/ui-egui/src/{chrome,dock,menus,place,ui_fonts,workspaces}.rs`, `crates/ui-egui/src/i18n/ko.tsv`, `crates/ui-egui/src/trowel/{caption,center,control,drawings,legend,menu,mod,palette,props,ribbon,status,tests,tools}.rs`, `factory/backlog.md`, 이 파일.

**남긴 것.**

- U12 캡션 흙손 표시: 16 px에서 흙색 줄이 검게 보인다. 16 px용 그림을 따로 그려야 하는 아이콘 일(J001 쪽)이라 이 작업에서 고치지 않는다.
- U-a 허용치: 시안 12 px는 앱 범위(1–8 px) 밖이다. 범위를 바꾸면 환경설정 값의 뜻이 바뀌므로 사용자에게 묻는다(백로그 J016 메모).
- U-e `theme.rs:181`의 「Measured from Illustrator 2026」 주석: J003 이전 코드다. 클린룸 규칙에 비추어 사람이 확인한다.
- C6: 명세대로 펜·곡률 도구에서는 고전 컨트롤 바의 「변환: 모퉁이/매끄럽게」 단추가 보이지 않는다(기준점 변환 도구에서는 보임).

### 재검증 (재작업 1차)

2026-10-11. 대상은 `fc810fb` 위의 미커밋 변경(이 절과 함께 커밋). 로그는 `/tmp/claude-0/-home-claude/79152f61-3d49-5a1c-acde-dc6cabf2d3d7/scratchpad/factory/J003/`, 명령은 `/home/claude/vector`에서.

| 명령 | 종료 코드 | 결과 | 로그 |
|---|---|---|---|
| `cargo test -p vectorcraft-ui-egui --lib -- trowel::tests::a_locked_or_hidden` | 101 | 1 실패(기대한 빨강: 잠긴 지금 레이어에 그린다고 보임) | `red6.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel::tests::small_windows` | 101 | 1 실패(기대한 빨강: 1536 폭에서 「명령 찾기」가 창 밖) | `red7.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel:: i18n::tests::every_tw` | 0 | 18 통과 | `green5.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel::` | 0 | 18 통과 | `green6.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel:: workspaces::tests::trowel ui_fonts::tests::a_bold_face` · `cargo test -p vectorcraft-text --lib -- tests_sysfonts::korean_fallback` | 101 · 101 | 17 통과 · 9 실패(기대한 빨강: 새 시험 6개와 기댓값을 바꾼 시험 3개) · `fallback_families`가 없어 컴파일 실패(기대한 빨강) | `red8.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel::tests::control_bar_keeps` | 101 | 1 실패(기대한 빨강: 좁은 창에서 「굵기 그대로」·「필압 따라」가 자석 위에) | `red9.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel:: workspaces:: ui_fonts:: i18n:: dock::`(허용치 고르기를 일부러 끈 판) | 101 | 78 통과 · 3 실패: 의도한 실패(4 px를 골라도 2 px) + 위 「시험 먼저」에 적은 시험 문제 둘 | `green7-mutation.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel:: workspaces:: ui_fonts:: i18n:: dock:: menus:: place::` | 0 | 119 통과 | `green7.log` |
| `cargo test -p vectorcraft-text --lib -- tests_sysfonts::` | 0 | 21 통과(한국어 글꼴이 있는 이 기계에서 건너뛴 시험 없음) | `green7-text.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 경고 없음 | `rework-clippy.log` |
| `CARGO_PROFILE_DEV_DEBUG=0 cargo xtask ci`(디스크를 아끼려고 디버그 정보만 끔, 13:37:17Z–14:00:06Z) | 0 | 7단계 모두 통과(fmt, clippy, test, assets, brands, layers, wasm). 시험 묶음 117개에서 5,116 통과 · 0 실패 · 4 무시(이 작업 전부터 `#[ignore]`) | `rework-ci.log` |

CI 시작 뒤 코드 파일은 바뀌지 않았다(`find crates apps xtask -newermt 13:37:00` 결과 없음). 화면 확인과 GitHub CI는 이 커밋 뒤에 이어 적는다.

### 화면 확인 (재작업 1차)

2026-10-11 14:05Z, `d0213f0`으로 빌드한 `target/debug/vectorcraft`(`CARGO_PROFILE_DEV_DEBUG=0 cargo build -p vectorcraft`, 종료 0, `build-r1.log`)를 `python3 -I scratchpad/j003/drive2.py target/debug/vectorcraft <out> 7993`(종료 0)으로 찍었다. 첫 회차와 같은 문서·같은 레이어 12개·펜 도구다. 그림은 `/mnt/project-files/design/trowel-run/`에 있다.

| 그림 | 확인한 것 |
|---|---|
| `J003r1-plan.png`(1600 × 988, 포인터는 빈 곳) | 스마트 가이드 「패스」 표지 없음(U19). 상태줄·띠 안내가 한 가지 글꼴 muted(R9). 배율 「화면 77 %」(U-b). 속성의 「● 필요」 없음(U6). 허용치 칸 모양(U5). 캡션 아래 선(U3, 시안 y=32·앱 y=31) |
| `J003r1-locked.png`(지금 레이어 = 잠긴 「기준점」) | 띠 「펜 · 조사구역 경계  기준점 잠김: 조사구역 경계에 그립니다」, 컨트롤 바 범례 칸 흙색 테두리 + 「조사구역 경계」, 범례 「기준점」 이름 흙색(R1) |
| `J003r1-soon.png`(리본 「단면」을 누른 뒤) | 상태줄 왼쪽 「단면: 준비 중 (J004)」(R8) |
| `J003r1-1366.png`(1366 × 728) | 리본 23칸 모두 44 px로, 명령 찾기는 돋보기 단추. 도구 기둥은 단추를 낮추고 고고학 묶음은 스크롤, 칠·선 상자는 바닥. 범례는 스크롤. 겹친 글자 없음(R2) |
| `J003r1-side-by-side.png` | 왼쪽 시안 `win-plan.png`, 오른쪽 `J003r1-plan.png` |

시안과 남은 차이(빌드 절의 표에서 바뀐 것만 적는다. 나머지 2–7, 9–11, 13–15행은 그대로):

| # | 곳 | 시안 | 앱 | 맡은 작업 |
|---|---|---|---|---|
| 1 | 흐린 「준비 중」 칸 | 모두 쓸 수 있는 모양 | 흐림. 올리면 「{기능} · 준비 중 · J0xx」, 누르면 상태줄에 4초 | 목록 참고 |
| 8 | 허용치 | 12 px | 2 px(앱 범위 1–8 px) | **사용자 결정**(백로그 J016 메모) |
| 12 | 배율 | 「화면 34 % · 도면 1:20」 | 「화면 77 %」: 표기는 시안과 같은 정수, 값은 문서 확대율. 「도면 1:20」은 축척이 생기면 | J006 |
| 16 | 리본 「밑그림」·「격자」 | 켜짐 | 꺼짐: 이 문서에 템플릿 레이어가 없고 격자를 끈 상태라서(모양 차이 아님) | — |
| 17 | 범례 ▸ | 9 px muted 글자 ▸ | 7 px 삼각형, 시안보다 조금 진함 | nit, 2차 검토에 맡김 |
| 18 | 컨트롤 바 간격 | | 흐린 「추정 (파선)」 칸이 글꼴 폭 때문에 조금 좁아 뒤 칸들이 최대 7 px 왼쪽 | 바꾸지 않음 |

**GitHub CI (재작업 1차 커밋).** `d0213f0`을 작업 브랜치에 올린 뒤 「Trowel CI」 run 38145551715가 성공했다(리눅스 `cargo xtask ci` 관문, 윈도우 x64 빌드). 첫 커밋 `fc810fb`의 run 38137425667도 성공이었다.

## 검토 (review) 2차

2026-10-11, 대상 커밋 `d0213f0`(기준 `fc810fb`). 검토자 둘은 1차와 같이 구현자와 다른 에이전트이고, 읽기만 했다(저장소를 고치지 않음). 보고서 전문은 스크래치패드 `review2-ui.md`, `review2-code.md`.

- **UI 검토(ui-reviewer, 14:26Z).** 1차 지적을 기록·시험·화면으로 하나씩 확인했다(U1–U19 고침, U12·U-a·U-e는 까닭 있는 남김). 앱을 따로 띄워 31장을 더 찍었다(1536·1440·1366·1280·1024·800 폭, 툴팁·목록·⋯ 패널·잠김·숨김). 시안과 1 px 안에서 같아진 곳: 컨트롤 바, 범례 줄의 ▸·눈·자물쇠, 판 제목, 메뉴 밑줄, 리본 끝 선, 확대 기둥, 상태줄 안내. **판정 pass**. 새로 찾은 것 minor 3, nit 7.
- **정확성·보안 검토(code-reviewer + correctness-reviewer + security-reviewer 한 번에, 14:28Z).** C1–C10이 기록대로 고쳐졌음을 확인했다. 패닉 경로, egui id, 범례 가상화의 경계, `Flyout`, `start_trowel`, `file.place`의 `template` 인자(제어 채널·MCP 입력), 허용치 목록, 고전 화면 불변을 점검했다. CI가 만든 시험 바이너리로 대상 시험 130개 + 21개를 돌려 모두 통과했다(빌드는 하지 않음). 보안 문제 없음. **판정 pass**. 새로 찾은 것 minor 1(UI N3과 같은 것), nit 3.

둘 다 pass였지만, minor 3건(작은 창의 겹침 둘, 같은 메시지가 다시 안 나옴)은 사용자가 바로 볼 수 있는 결함이다. 그래서 승인 보고서로 넘기지 않고 **재작업 2차(마지막 회)**에서 고쳤다. 공장 규칙상 수정 루프는 최대 2회라, 이 뒤에 남는 것은 `needs-human`으로 넘긴다.

UI 검토 지적(N, 줄 번호는 `d0213f0`):

| # | 심각도 | 곳 | 내용 | 조치 |
|---|---|---|---|---|
| N1 | minor | `trowel/mod.rs:96-100`, `legend.rs:56-64, 230-231` | 창 높이가 약 600 px보다 낮으면 오른쪽 기둥이 겹침. 800 × 500에서는 「굴광선」 줄 위에 「속성 선택 없음」이 그려지고 속성 칸이 안 보임. 1차 때부터 있었음 | 고침 R19 |
| N2 | minor | `center.rs:31-50` | 좁은 창(이 문서 이름으로 약 900 px 아래)이나 긴 이름에서 지금 탭 이름이 오른쪽 구석 「{문서} ● 저장 안 됨」과 겹침. 지금 탭은 자리가 없어도 늘 그렸기 때문. 1차 때부터 있었음 | 고침 R20 |
| N3 | minor | `status.rs:62-70` | 상태줄 6초를 메시지 글자로 잼: 바로 앞과 같은 메시지(두 번째 저장, 같은 오류)는 다시 나오지 않음 | 고침 R17 |
| N4 | nit | `control.rs:152` | 범례 칸의 레이어 목록이 약 610 px로 넓어 자물쇠·눈 표시가 이름에서 450 px 떨어짐 | 고침 R21 |
| N5 | nit | `control.rs:126, 138-143` | 「그릴 수 있는 레이어가 없습니다」일 때 패치 자리가 비고 「0.8 pt」가 붙음 | 고침 R21 |
| N6 | nit | `palette.rs:54, 66, 87` | 팔레트의 준비 중 칩(빗금, 선종, 굵기 ＋)은 wash 바탕이 아님 | 고침 R22 |
| N7 | nit | `ko.tsv:3195-3196` | 선택 도구 안내 「여러 객체를 선택」의 「를」(U17 남은 것) | 고침 R24 |
| N8 | nit | `ribbon.rs:158-163` | 1265–1480 px(1440 × 900 포함)에서 명령 찾기 칸(150)이 들어갈 자리가 남는데 돋보기로 접힘 | 고침 R23 |
| N9 | nit(기록) | `factory/backlog.md` | 남긴 것 U12·U-e가 백로그에 없음 | 고침 R26 |
| N10 | nit(번역) | `ko.tsv` `{what}: 준비 중 ({job})` | 「모든 굵기 바꾸기…: 준비 중 (J005)」처럼 말줄임표 바로 뒤에 쌍점 | 고침 R24 |

정확성·보안 검토 지적(K, 줄 번호는 `d0213f0`):

| # | 심각도 | 곳 | 내용 | 조치 |
|---|---|---|---|---|
| K1 | minor | `status.rs:63-69` | N3과 같음. 다른 파일의 직접 쓰기(`app.ui.status = …`)도 `app.status()`로 바꾸라는 제안 | 고침 R17 |
| K2 | nit | `workspaces.rs:60` | `start_trowel`이 첫 프레임 전에 불려 `tw!`가 아직 영어: 한국어 사용자에게도 「Layout before Trowel」로 남음 | 고침 R18 |
| K3 | nit | `fontdb.rs:433` | `is_hangul`이 괄호 한글(㈀), 원 한글(㉠), 방점을 빼먹음. ㉠은 보고서·도면 번호에 흔함 | 고침 R25 |
| K4 | nit | `tests.rs:590` | 바꾼 시험이 상태줄 안내의 앞 두 부분만 봄 | 고침 R26 |

범위 밖(한 줄, UI 검토): 선택 도구 이름이 「선택 영역 (도구)」로 나오는 것은 앱 전체 번역(`ko.tsv:2484, 2487`)이고 J003이 만든 것이 아니다.

## 재작업 (rework) 2차

2026-10-11, 기준 `d0213f0`(재작업 1차 커밋, 이미 push됨). 같은 작업 브랜치 제자리, 커밋은 `J003: …` 세 번째 하나로 더한다(1차 회차와 같은 까닭). 공장 규칙의 마지막 수정 루프다.

시험 먼저: 고친 동작마다 실패하는 시험을 먼저 쓰고 빨강을 본 뒤 고쳤다.

- `red-r2-ui.log`: 화면 시험 7개가 기대한 까닭으로 실패. 800 × 500 오른쪽 기둥 글자 겹침 10쌍, 그릴 레이어 없음 칸에 「1 pt」, 레이어 목록 줄 너비 586, 팔레트 칩 바탕 흰색, 1536 폭 명령 찾기 32, 같은 메시지가 다시 안 나옴, 1366 긴 이름에서 구석 이름이 안 비켜남.
- `red-r2-text.log`: ㉠이 한글로 잡히지 않음.
- `red-r2-ws.log`: `start_trowel`이 언어를 받지 않아 컴파일 실패(새 시그니처를 쓰는 시험).
- 탭 시험은 실제 글자 폭을 재어(`Site 1` 27.6 px 등) 경우를 다시 골랐다. 고친 코드를 빼고 최종 시험을 돌려 실패를 다시 확인했다(`red-r2-tabs-final.log`, 종료 101).
- 상태줄 안내 전체 문장 확인(K4)은 이미 맞는 동작을 더 엄격히 보는 것이라 처음부터 통과했다.
- 고친 뒤 화면 확인(800 × 500에서 휠로 내림)에서 R19의 결함을 하나 더 찾았다(R27). 이것도 시험을 먼저 써서 빨강을 보고 고쳤다(`red-r2-legend.log`, 종료 101: 1280 × 662에서 맨 아래 레이어 이름이 바닥 선 476보다 아래 480까지 그려짐).

| # | 지적 | 바꾼 것 | 시험 |
|---|---|---|---|
| R17 | N3, K1 | `UiState.status_seq`(저장하지 않음)를 두고 `app.status()`가 부를 때마다 올린다. 상태줄 6초는 (번호, 글자)가 바뀐 때부터 잰다. 직접 쓰던 곳을 `status()`로 바꿈: `lib.rs` 4곳(붙여넣기 오류, 명령 오류, 링크 열기, 도구 고르기 오류)과 다른 파일 16곳(`sysclip`, `font_menu`, `dock`, `control`의 제어 채널 `ui.set`, `panels/{image_trace ×2, opentype, glyphs, separations ×3, mod ×2}`, `toolbar`, `find_font ×2`). 신호로 쓰는 「quit」(`unsaved.rs`), 비우기(`picks.rs`, `file.new`), 덧붙이기(`print.rs`)는 그대로 | `status_bar_shows_messages_for_a_while`(같은 메시지 다시, 같은 명령 오류 두 번) |
| R18 | K2 | `start_trowel(ui, lang)`: 남기는 작업 영역 이름을 넘겨받은 언어로 짓는다(`tr_ctx(lang, "trowel", "Layout before Trowel")`). 호스트 둘(`apps/vectorcraft/src/main.rs`, `apps/vectorcraft-web/src/web.rs`)은 환경설정을 읽은 뒤 `app.ui_language()`를 넘긴다 | `workspaces::tests::the_kept_layout_is_named_in_the_window_language`(한국어 → 「Trowel 이전 배치」) |
| R19 | N1 | 오른쪽 기둥: 팔레트 166 + 범례 최소 156(머리·캡션 60 + 레이어 두 줄 48 + 바닥 48) + 속성(112, 접으면 33)이 창에 들어가지 않으면 기둥 전체를 세로로 스크롤한다(`right-scroll`). 들어가면 전과 같다(약 662 px 높이부터). 범례의 위·아래 높이를 상수 `ABOVE`·`BELOW`·`FOOT_H`·`MIN_HEIGHT`로 | `a_short_window_scrolls_the_right_column_without_overlap`(1600·1366·800에서 기둥 안 글자가 겹치지 않음. 800 × 500에서 휠로 속성 ⋯까지 내려가 눌러 속성 패널을 엶) |
| R20 | N2 | 탭 줄: 오른쪽 구석이 탭에 자리를 먼저 내준다. 모든 탭이 들어갈 때만 문서 이름(탭에도 있음), 지금 탭이 들어갈 때 「● 저장 안 됨」, 아니면 흙색 점만(올리면 「저장 안 됨」). 그래도 모자라면 지금 탭 이름을 「…」로 줄인다(탭 최소 56). 탭 이름은 ×(홈 탭은 탭 끝) 앞에서 끝남 | `the_tabs_and_the_name_at_their_right_never_overlap`(1600 / 1366 긴 이름 / 800 중간 이름 / 800 긴 이름: 글자끼리, 탭과 구석 글자가 겹치지 않음. 지금 탭 이름은 탭 안, 마지막 경우만 줄임) |
| R21 | N4, N5 | 컨트롤 바 범례 칸: 레이어 목록 줄 너비를 260으로 고정(이름은 오른쪽 표시 앞에서 자름). 그릴 레이어가 없으면 색 패치와 굵기를 빼고 글자를 칸 왼쪽부터 | `control_bar_shows_the_layer_and_next_weight`(줄 너비 220–280), `a_locked_or_hidden_current_layer_shows_where_lines_go`(칸 글자가 「그릴 수 있는 레이어가 없습니다」 하나, 칸 왼쪽 12 안) |
| R22 | N6 | 팔레트의 준비 중 칩(빗금 45°·135°·번갈아, 굵기 ＋)과 선종 칸(파선·점선·일점쇄선)을 다른 준비 중 칸처럼 wash 바탕으로 | `palette_weight_chip_sets_the_next_stroke`(여섯 칸의 바탕색) |
| R23 | N8 | 리본이 좁아지는 순서: ① 52 칸 + 명령 찾기 → ② 44 칸 + 명령 찾기(1383 px부터) → ③ 44 칸 + 돋보기(1265 px부터) → 가로 스크롤. 1차의 「52 칸 + 돋보기」 단계(1481–1598 px)는 뺐다: 검토 제안처럼 ③ 앞에 단계만 넣으면 창을 줄일 때 명령 찾기 칸이 접혔다가 다시 펴진다 | `small_windows_keep_every_tile_and_tool_in_reach`(1440 × 900 추가, 1383 이상에서 명령 찾기 150·아래에서 32) |
| R24 | N7, N10 | `ko.tsv` 「여러 객체를 선택」 → 「여러 객체 선택」(일반 행이라 고전 안내 줄에도). 「준비 중」 알림은 이름 끝 「…」을 뗀다(「모든 굵기 바꾸기: 준비 중 (J005)」). 툴팁은 그대로 | `status_bar_shows_messages_for_a_while` |
| R25 | K3 | `is_hangul`에 방점(U+302E–302F), 괄호 한글(U+3200–321E), 원 한글(U+3260–327F)을 더함 | `tests_sysfonts::korean_fallback_families_are_for_hangul_only`(㉠·㉿·㈀·방점, 아닌 쪽에 괄호 한자 ㈠) |
| R26 | K4, N9 | 시험 `hints_read_plainly_as_in_the_mockup`이 상태줄 안내를 전체 문장으로 맞춰 봄. 백로그에 U12(J001 줄의 후속)와 U-e(J003 줄의 사람 확인)를 옮김 | 그 시험 |
| R27 | 화면 확인(R19 결함) | 범례 목록이 자기 칸보다 길게 그려짐: egui 스크롤 영역은 높이가 64 px보다 작으면 64로 늘리는데, 범례가 가장 낮을 때 목록 칸은 48 px(두 줄)라 셋째 줄이 「굵기 한 번에」 줄의 선과 버튼 위로 16 px 넘어감(창 높이 약 680 px 아래). 목록 스크롤에 `min_scrolled_height(0.0)`을 주어 칸 높이 그대로 둠 | `the_legend_list_stays_above_its_footer`(1280 × 662·800 × 500, 레이어 6개: 어떤 줄도 바닥 선을 넘지 않고, 끝까지 내리면 맨 아래 레이어가 잘리지 않음) |

시험 도우미: 화면 시험의 `Painted`에 보이는 부분(`shown` = 글자 사각형 ∩ 자르기 사각형)을 더하고, 보이는 글자끼리 겹치는 쌍을 찾는 `overlapping_texts`를 더했다. 스크롤로 가려진 글자는 겹침으로 세지 않는다.

**바꾼 시험(시험 변경).**

- `trowel::tests::status_bar_shows_messages_for_a_while`: 기댓값 「Change All Weights…: coming soon (J005)」 → 「Change All Weights: coming soon (J005)」. 까닭: N10(말줄임표 바로 뒤 쌍점). 확인 두 가지를 더함(같은 메시지 다시, 같은 오류 두 번).
- `trowel::tests::hints_read_plainly_as_in_the_mockup`: 상태줄 확인을 앞 두 부분 `contains`에서 전체 문장 `assert_eq`로(K4, 더 엄격).
- `trowel::tests::small_windows_keep_every_tile_and_tool_in_reach`: 1440 × 900을 더하고 명령 찾기 폭을 확인함. 1536 × 816에서 기대하는 모양이 1차의 돋보기에서 명령 찾기 칸(150)으로 바뀜(R23 동작 변경).
- `workspaces::tests`의 `start_trowel` 호출 다섯 곳과 화면 시험 도우미 `Harness::sized`: 새 인자(언어)만 넘김. 기댓값은 그대로.
- 새 시험 `trowel::tests::the_legend_list_stays_above_its_footer`(R27).
- 나머지는 확인만 더했다(`control_bar_shows_the_layer_and_next_weight`, `a_locked_or_hidden_current_layer_shows_where_lines_go`, `palette_weight_chip_sets_the_next_stroke`, `korean_fallback_families_are_for_hangul_only`). 지운 확인문은 없다.

**명세·검토 제안에서 벗어난 점.**

- R23은 검토 제안(③ 앞에 단계 하나 더하기)과 다르게 1차의 ② 단계를 뺐다. 까닭은 위 표. 1536 폭(1920 × 1080, 125 %)에서는 이제 칸이 44 px이고 명령 찾기 칸이 보인다.
- R20에서 구석의 문서 이름은 탭이 모두 들어갈 때만 보인다. 이름은 지금 탭에도 있으므로 정보는 줄지 않는다.
- R17의 번호는 고전 화면에 영향이 없다(고전 상태줄은 다음 메시지까지 계속 보임).
- R27은 검토 지적이 아니라 이번 재작업의 화면 확인에서 찾은 것이다. 같은 재작업(2차) 안에서 고쳤고, 3차 확인 검토가 함께 본다.

**고친 파일.** `apps/vectorcraft/src/main.rs`, `apps/vectorcraft-web/src/web.rs`, `crates/text/src/{fontdb,tests_sysfonts}.rs`, `crates/ui-egui/src/{control,dock,find_font,font_menu,lib,state,sysclip,toolbar,workspaces}.rs`, `crates/ui-egui/src/panels/{glyphs,image_trace,mod,opentype,separations}.rs`, `crates/ui-egui/src/i18n/ko.tsv`, `crates/ui-egui/src/trowel/{center,control,legend,mod,palette,ribbon,status,tests}.rs`, `factory/backlog.md`, 이 파일. 좌표·단위·파일 형식·의존성·CI 설정은 바꾸지 않았다.

### 재검증 (재작업 2차)

2026-10-11. 대상은 `d0213f0` 위의 미커밋 변경(이 절과 함께 커밋). 로그는 `/tmp/claude-0/-home-claude/79152f61-3d49-5a1c-acde-dc6cabf2d3d7/scratchpad/factory/J003/`, 명령은 `/home/claude/vector`에서, 모두 `CARGO_PROFILE_DEV_DEBUG=0`(디스크를 아끼려고 디버그 정보만 끔).

| 명령 | 종료 코드 | 결과 | 로그 |
|---|---|---|---|
| `cargo test -p vectorcraft-ui-egui --lib -- trowel::` | 101 | 19 통과 · 7 실패(기대한 빨강: 800 × 500 기둥 겹침, 그릴 레이어 없음 칸의 굵기, 레이어 목록 폭, 팔레트 칩 바탕, 1536 폭 명령 찾기, 같은 메시지 다시, 1366 긴 이름의 구석) | `red-r2-ui.log` |
| `cargo test -p vectorcraft-text --lib -- tests_sysfonts::korean` | 101 | 1 실패(기대한 빨강: ㉠이 한글로 잡히지 않음) | `red-r2-text.log` |
| `cargo test -p vectorcraft-ui-egui --lib --no-run` | 101 | 컴파일 오류 5곳(기대한 빨강: `start_trowel`이 언어를 받지 않음) | `red-r2-ws.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel::tests::the_tabs`(`center.rs`만 `d0213f0`으로 돌린 판, 끝난 뒤 되돌리고 `cmp`로 같음 확인) | 101 | 1 실패(최종 탭 시험이 고치기 전 코드에서 실패함을 다시 확인) | `red-r2-tabs-final.log` |
| `cargo test -p vectorcraft-text --lib -- tests_sysfonts::` | 0 | 21 통과 | `green-r2-text.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel:: workspaces::` | 0 | 34 통과 | `green-r2-ui.log` |
| `cargo xtask ci`(15:03:41Z–15:14:24Z) | 0 | 7단계 통과. 5,119 통과 · 0 실패 · 4 무시 | `rework2-ci-first.log` |
| `cargo check --target wasm32-unknown-unknown -p vectorcraft-web`(웹 앱. xtask의 wasm 단계는 이 앱을 컴파일하지 않아 따로) | 0 | 경고 2개(`ask_update_changed`·`is_c_locale`의 `dead_code`, 이 작업이 건드리지 않은 파일, 전부터 있음) | `rework2-web-check.log` |
| `cargo build -p vectorcraft` → 화면 1회차(15:18Z) | 0 | 800 × 500에서 휠로 내리자 범례 목록 셋째 줄이 「굵기 한 번에」 선 위로 넘어감을 찾음(R27) | `build-r2.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel::tests::the_legend_list_stays_above_its_footer` | 101 | 1 실패(기대한 빨강: 1280 × 662에서 「Layer 1」이 선 476 아래 480까지) | `red-r2-legend.log` |
| `cargo test -p vectorcraft-ui-egui --lib -- trowel:: workspaces::` | 0 | 35 통과 | `green-r2b-ui.log` |
| `cargo xtask ci`(15:29:50Z–15:33:45Z) | 0 | 7단계 모두 통과(fmt, clippy, test, assets, brands, layers, wasm). 시험 묶음 117개에서 5,120 통과 · 0 실패 · 4 무시(이 작업 전부터 `#[ignore]`) | `rework2-ci.log` |
| `cargo check --target wasm32-unknown-unknown -p vectorcraft-web` | 0 | 위와 같은 경고 2개 | `rework2-web-check-b.log` |
| `cargo build -p vectorcraft` → 화면 2회차(15:35Z) | 0 | 아래 「화면 확인 (재작업 2차)」 | `build-r2b.log` |

두 번째 CI 시작 뒤 코드 파일은 바뀌지 않았다(`find crates apps xtask -newermt 15:29:00` 결과 없음). GitHub CI는 이 커밋을 올린 뒤 이어 적는다.

### 화면 확인 (재작업 2차)

2026-10-11 15:35Z, R27까지 고친 코드로 빌드한 `target/debug/vectorcraft`를 `python3 -I scratchpad/j003/drive3.py target/debug/vectorcraft <out> 7995`(종료 0)로 찍었다. 1차와 같은 문서·레이어 12개·펜 도구다. 1회차(15:18Z, R27 전)와 비교하면 800 × 500 두 장의 범례 목록만 다르다(픽셀 비교). 그림은 `/mnt/project-files/design/trowel-run/`에 있다.

| 그림 | 확인한 것 |
|---|---|
| `J003r2-plan.png`(1600 × 988) | 1차와 같은 짜임. 팔레트의 준비 중 칩(빗금 45°·135°·번갈아, 굵기 ＋)과 선종 칸(파선·점선·일점쇄선)이 다른 준비 중 칸처럼 wash 바탕(R22) |
| `J003r2-soon.png`(범례 바닥 「모든 굵기 바꾸기…」를 누른 뒤) | 상태줄 「모든 굵기 바꾸기: 준비 중 (J005)」, 쌍점 앞 「…」 없음(R24) |
| `J003r2-layer-list.png`(컨트롤 바 범례 칸을 누름) | 목록 폭 약 260, 「기준점」의 자물쇠가 이름 바로 오른쪽(R21) |
| `J003r2-no-layer.png`(레이어를 모두 잠금) | 칸 왼쪽부터 「그릴 수 있는 레이어가 없습니다」, 흙색 테두리, 색 패치·굵기 없음(R21) |
| `J003r2-1536.png`(1536 × 816) | 리본 44 px 칸 + 명령 찾기 칸(R23). 탭과 「1호 주거지 평면 ● 저장 안 됨」이 겹치지 않음 |
| `J003r2-1440.png`(1440 × 900) | 44 px 칸 + 명령 찾기 칸(R23). 범례 12줄이 모두 보임 |
| `J003r2-800.png`(800 × 500) | 탭 오른쪽은 흙색 점만(R20). 오른쪽 기둥 위쪽에 팔레트, 아래 범례 두 줄, 겹친 글자 없음(R19) |
| `J003r2-800-props.png`(800 × 500, 오른쪽 기둥 위에서 휠) | 속성 칸까지 내려감(R19). 범례 목록 두 줄이 「굵기 한 번에」 선 위에서 끝남(R27) |
| `J003r2-side-by-side.png` | 왼쪽 시안 `win-plan.png`, 오른쪽 `J003r2-plan.png` |

시안과 남은 차이는 「화면 확인 (재작업 1차)」의 표와 같다(달라진 행 없음). 더 본 것 하나: 작은 창에서 범례 목록 아래에 줄이 더 있으면 보이는 끝 줄이 흐려진다(egui 기본 스크롤 흐림, 20 px·50 %, 「더 있음」 표시). 숨긴 레이어의 흐린 이름과 비슷해 보일 수 있으나 앱 전체 기본값이라 이 작업에서는 바꾸지 않았고, 3차 확인 검토에 맡긴다.

**GitHub CI (재작업 2차 커밋).** `968d241`을 작업 브랜치에 올린 뒤 「Trowel CI」 run 38152149511이 성공했다(15:38:54Z–15:54:04Z, 리눅스 `cargo xtask ci` 관문과 윈도우 x64 릴리스 빌드 모두 success): https://github.com/kesekikwon-netizen/vector/actions/runs/38152149511

## 검토 (review) 3차 (확인)

2026-10-11, 대상 커밋 `968d241`(기준 `d0213f0`). 수정 루프는 2회로 끝났으므로 이번 판정은 pass 또는 needs-human이다. 검토자 둘은 1·2차와 같이 구현자와 다른 에이전트이고, 읽기만 했다(저장소를 고치지 않음, 빌드하지 않음). 보고서 전문은 스크래치패드 `review3-ui.md`, `review3-code.md`.

- **UI 검토(ui-reviewer, 15:45–16:05Z).** N1–N10, K1–K4, R27을 그림·코드·시험으로 하나씩 확인했다. 같은 실행 파일을 따로 띄워 27장을 더 찍었다(오른쪽 기둥 경계 높이 700·663·662·661·640·600, 리본 단계 경계 폭 1599–1264의 8개, 긴 이름 도면 셋의 탭 줄 폭 7개, 같은 오류 두 번, 선택 도구 안내). 기준 그림은 `J003r2-plan.png`와 픽셀 단위로 같다. 제안과 다르게 한 R23(단계를 빼서 한 방향으로만 좁아짐)과 R20(구석 이름은 탭이 모두 들어갈 때만)은 둘 다 타당하다고 봤다. **판정 pass**. 새로 찾은 것 nit 2.
- **정확성·보안 검토(code-reviewer + correctness-reviewer + security-reviewer 한 번에).** K1–K4와 N3이 기록대로 해결됐고, 남긴 상태 직접 쓰기 4곳(`unsaved.rs` 「quit」 신호, `picks.rs`·`file.new` 비우기, `print.rs` 덧붙이기)의 까닭이 타당하다. 시험 아닌 코드에 새 패닉 경로·음수 폭·egui id 충돌·보안 문제가 없다. 바꾼 시험 4줄은 근거가 있고 약해진 시험이 없다(지운 확인문 2개는 더 강한 확인으로 바뀜). CI가 만든 시험 바이너리로 ui-egui lib 971개, `trowel:: workspaces::` 35개, text `tests_sysfonts::` 21개를 돌려 모두 통과했다. **판정 pass**. 새로 찾은 것 nit 1(UI의 F1과 같은 것).

| # | 심각도 | 곳 | 내용 | 조치 |
|---|---|---|---|---|
| F1 (= 정확성 검토 P1) | nit, 재작업 2차에서 생긴 회귀 | `trowel/palette.rs:119`(관련 `:104-107`) | R22가 반복문 뒤에 선종 상자 테두리(`input_border`)를 다시 그리면서, 고른 「실선」 칸의 먹색 테두리를 왼쪽·위·아래 세 변에서 덮는다. 칸 가장자리 색이 1차 (20, 20, 19)에서 (194, 192, 182)로 바뀜. 시안과 다른 선택 칩(굵기 0.8, 빗금 없음)은 네 변 모두 먹색. 바탕색은 그대로라 무엇을 골랐는지는 보인다. R22 시험은 바탕색만 봐서 놓쳤다. 고치기는 한 줄(선택 칸 테두리를 상자 테두리 뒤에 그림)과 테두리 색 확인 시험 | 고치지 않음(수정 루프 끝). **사용자 결정**: 승인 전에 고칠지, 선종 칸을 다시 만드는 J014에서 고칠지(두 검토 권고 J014). 백로그 J014에 메모 |
| F2 | nit(화면 확인 절의 관찰을 판단) | `trowel/legend.rs:110`(목록 스크롤), `trowel/mod.rs:103`(기둥 스크롤) | egui 기본 스크롤 흐림(아래 20 px, 50 %) 때문에 범례 목록이 넘치면 보이는 끝 줄이 흐려진다. 1600 × 988에서도 레이어를 펼쳐 목록이 넘치면 생긴다. 800 × 500과 창 높이 약 662–700에서는 흐려지는 줄이 지금 레이어(「굴광선」)라, 이름 색과 연한 눈이 숨긴 레이어와 비슷해 보일 수 있다. 정보는 사라지지 않는다 | 고치지 않음. **사용자 결정**, 권고는 범례를 다시 만드는 J005에서 범례 목록 안의 흐림만 끄기(`scroll.fade.strength = 0`). 백로그 J005에 메모 |
| (R23 참고) | 모양 선택, 결함 아님 | `trowel/ribbon.rs` | 창 폭 1481–1598에서 리본 칸이 시안(52)보다 8 px 좁고 명령 찾기 칸 왼쪽이 약 100–215 px 빈다(1536 폭에서 약 160). 칸 폭을 44–52 사이에서 이어서 맞추면 빈 곳이 없어진다 | **사용자 결정**(권고: 이대로 승인, 후속에서 정함) |

의견(지적 아님, 정확성 검토): 스크롤로 화면 밖에 밀려난 ⋯ 단추도 `Flyout.keep`에 들어간다. 실제로 겪을 일이 거의 없고 고치려면 한 줄이다. 참고(UI 검토, 1차부터): 폭 1100·1024에서 앞 탭이 가려졌다는 표시는 없다(도면집에 모두 있음).

**판정 정리.** 두 검토 모두 pass(3차). 남은 nit 2개와 모양 선택 하나는 공장 규칙(수정 루프 최대 2회)에 따라 이 작업에서 고치지 않고 사람 결정으로 넘긴다(승인 보고서 6절).

## 승인 (approval)

- AI 검토: 통과(3차 확인, 2026-10-11 16:05Z). 근거: 위 「재검증 (재작업 2차)」의 `cargo xtask ci` 종료 0(5,120 통과·0 실패), 웹 앱 wasm check 종료 0, GitHub CI run 38152149511 success, 검토 3차 두 판정 pass.
- 사용자 승인: **대기**. AI 검토 통과는 사용자 승인이 아니다. 승인 뒤 코드가 바뀌면 다시 검증하고 다시 승인받는다.
- 승인 대상: 작업 브랜치 `claude/project-thread-5e8gjw`의 코드 커밋 `fc810fb`·`d0213f0`·`968d241`(기준 `9ab182e`). 그 뒤의 `factory:` 기록 커밋은 이 파일과 백로그만 바꾼다.
- 승인 보고서: `/mnt/project-files/design/trowel-run/J003-approval.md`(프로젝트 공유 폴더). 그림은 같은 폴더의 `J003r2-*.png`, `J003r1-*.png`.
- 사용자 결정을 기다리는 것: 승인 여부, F1(승인 전 / J014), F2(J005), R23 폭. 이전부터 남은 것: 자석 허용치 12 px(J016), `theme.rs:181` 주석의 클린룸 확인, 한글 대체 글꼴 목록, 펜·곡률 도구의 변환 단추, 밑그림 타일 동작.
- 기록 보존: 승인 근거(명령·종료 코드·결과·판정)는 이 파일과 `factory/backlog.md`에 있고 Git으로 남는다. 시험 로그·스크린샷은 스크래치패드(세션이 끝나면 사라짐)와 공유 폴더에 두고 커밋하지 않는다. GitHub 실행 로그는 깃허브에 남는다.
- 되돌리기: `git revert 968d241 d0213f0 fc810fb`(작업 브랜치). 화면만은 창 › 작업 영역 › Essentials. main에는 아무것도 합치지 않았다.
