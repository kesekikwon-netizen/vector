# J003 — 메인 창을 디자인 2.1 3절과 같게

상태: verify · 기준 커밋 `9ab182e865427a30a617a165527bc40ba011753a` · 커밋 1개 (`J003: …`) · 화면만(좌표·단위·파일 형식·의존성 변경 없음)

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
