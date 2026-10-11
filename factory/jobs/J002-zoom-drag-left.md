# J002 — Animated Zoom 방향 환경설정: 왼쪽으로 끌면 확대 (기본 켬)

상태: spec · 기준 커밋: 아래 참조 · 커밋 1개 (`J002: …`)

## 목표

Performance › Animated Zoom이 켜진 상태에서 줌 도구를 **왼쪽**으로 끌면 확대, 오른쪽으로 끌면 축소되게 하는 환경설정 `zoomDragLeftIn`(영문 레이블 "Drag left to zoom in (Animated Zoom)", 한국어 「왼쪽으로 끌면 확대 (애니메이션 줌)」)을 추가하고 **기본값을 켬**으로 둔다. 크기는 그대로(100 pt = 2배). 끄면 Illustrator 방향(오른쪽 = 확대)으로 돌아간다. 클릭/Alt+클릭 단계 줌, 누르고 있기(Alt = 축소), Animated Zoom 꺼짐 때의 영역 줌은 바꾸지 않는다. 힌트 막대의 문구가 방향을 따라간다.

**스프링 로드 Z**(Z를 0.3 s 이상 누르고 있는 동안만 줌 도구, 놓으면 이전 도구)는 이 작업에 **넣지 않고** 후속 작업 **J002b**로 뺀다. 이유는 「범위 밖」에 있다.

## 완료 기준

1. `crates/engine/src/lib.rs` `Prefs`에 `pub zoom_drag_left_in: bool`(`:536` `animated_zoom` 다음, 문서 주석 포함)이 있고 `Prefs::default()`(`:750` 다음)가 `true`다. `#[serde(default, rename_all = "camelCase")]` 덕에 키는 `zoomDragLeftIn`, 옛 환경설정 파일은 기본값을 받는다. 확인: `crates/engine/src/tests_prefs.rs` 새 시험 `zoom_drag_left_in_defaults_on_and_resets_with_performance`(모델 `:129-157` `gpu_preference_defaults_to_automatic_and_validates`):
   - `Session::new().prefs.zoom_drag_left_in == true`; `prefs.get {key: zoomDragLeftIn}` → `true`
   - `prefs.set {key, value: false}` → `false`; `"false"` 문자열도 받아 `false`; `1`, `"yes"`, `null`은 오류이고 값 불변
   - `serde_json::from_value::<Prefs>(json!({"animatedZoom": true}))` → `zoom_drag_left_in == true` (옛 파일)
   - `prefs.list`의 해당 행: `category == "Performance"`, `section == "GPU Performance"`, `label == "Drag left to zoom in (Animated Zoom)"`, 종류 bool
   - `false`로 둔 뒤 `prefs.reset {category: "Performance"}` → `true`
2. `crates/engine/src/cmd/prefscmds.rs:288` 다음에 `p!("zoomDragLeftIn", "Performance", "GPU Performance", "Drag left to zoom in (Animated Zoom)", bool)`. 기존 시험 `every_spec_matches_a_prefs_field_and_back`, `list_describes_every_pref`, `prefs_serde_round_trip_and_tolerates_missing_fields`가 통과한다(필드와 행 중 하나라도 빠지면 자동 실패).
3. `crates/ui-egui/src/canvas.rs:630`의 부호가 환경설정을 따른다. 시험 `canvas::tests::animated_zoom_scrubs_and_holds_else_the_zoom_tool_zooms_to_an_area`(`:3023-3071`)를 갱신해 통과한다. (이유: 사용자가 기본 방향을 왼쪽=확대로 결정.)
   - 기본값: `press(vec2(-100.0, 0.0), 0)` 뒤 `z1/z0 == 2`(메시지 "100 points left doubles the zoom"), 눌린 지점 고정 단언 유지; `press(vec2(100.0, 0.0), 0)` 뒤 `z0`로 복귀
   - 클릭 단계(`next_zoom`), 40프레임 누르기(`> z2 * 1.2`), 고정점 단언은 **그대로**
   - 추가: `prefs.set {key: zoomDragLeftIn, value: false}` 뒤 `press(+100)`이 2배, `press(-100)`이 복귀(Illustrator 방향)
   - 기존 루프(`animatedZoom`/`gpuPerformance` false → ZoomBox 영역 줌)는 그대로 통과
4. `crates/ui-egui/src/chrome.rs`: 상수 `ANIMATED_ZOOM_LEFT_HINT: &str = "zoom.animatedLeft"`와 `hint_for` 표(661-670과 같되 여섯째 조각이 `" left or right to zoom in or out  |  "`; 앞 공백 1개·뒤 `"  |  "` 정확히), `hint_strings()`(`:757`)가 `.chain([ANIMATED_ZOOM_HINT, ANIMATED_ZOOM_LEFT_HINT])`, `hint_bar`(`:816`)가 `canvas::zoom_drag_left_in(&prefs)`면 LEFT 힌트를 고른다. 시험 `chrome::tests::the_zoom_hint_follows_animated_zoom`(`:971-998`) 갱신: 기본 `contains(" left or right to zoom in or out") && contains("Hold")`; `prefs.set zoomDragLeftIn=false` 뒤 `contains(" right or left to zoom in or out")`; `animatedZoom=false` 뒤 `" to zoom into an area"`이고 `Hold` 없음. (이유: 기본 방향 결정.)
5. `crates/ui-egui/src/canvas.rs:745-747` 옆에 `pub(crate) fn zoom_drag_left_in(p: &Prefs) -> bool { animated_zoom(p) && p.zoom_drag_left_in }`가 있고, `:630`은 이 값을 `app.view_mut()`(`:637`) 차용 **전에** 지역 변수로 읽는다(`:626`의 `dt` 옆). `SCRUB_ZOOM`·`SCRUB_SLOP`·`HOLD_DELAY`·`HOLD_ZOOM` 값과 `:631-632`의 Alt 부호는 불변. 확인: `grep -n 'LN_2 / 100.0\|SCRUB_SLOP: f32 = 3.0\|HOLD_DELAY: f32 = 0.3\|HOLD_ZOOM: f64 = std::f64::consts::LN_2' crates/ui-egui/src/canvas.rs` → 4줄.
6. `crates/ui-egui/src/prefs_dialog.rs:260-263` 옆에 `PrefKind::Bool if sp.key == "zoomDragLeftIn" => ui.add_enabled_ui(d.bool("gpuPerformance") && d.bool("animatedZoom"), |ui| bool_row(ui, d, sp.key, sp.label))`. 새 시험 `prefs_dialog::tests::zoom_direction_row_shows_under_animated_zoom`(모델 `:509-539`): `open(Some("Performance"))` 두 프레임 뒤 글자 모양에 `"Drag left to zoom in (Animated Zoom)"`이 있고, `animatedZoom`을 끈 대화상자에서도 글자는 남아 있다(흐려짐은 색으로 확인하지 않아도 됨). 기존 `prefs_persist_through_ui_state`가 통과한다.
7. i18n: complete 카탈로그 10개(ja, ko, de, es, fr, it, ru, uk, zh-hant, zh-hans)에 빈 컨텍스트 행 2개를 추가한다 — 레이블 `Drag left to zoom in (Animated Zoom)`와 조각 ` left or right to zoom in or out  |  `(앞 공백·뒤 `  |  ` 바이트 동일). ko는 `왼쪽으로 끌면 확대 (애니메이션 줌)`와 ` 왼쪽으로 끌면 확대, 오른쪽으로 끌면 축소  |  `. 아울러 `ko.tsv:3220`의 기존 조각 번역을 ` 오른쪽으로 끌면 확대, 왼쪽으로 끌면 축소  |  `로 고친다(지금은 「좌우 화살표」라고 적힌 오역; 시험 기댓값이 아니라 번역 교정). cs·pt-br는 추가하지 않는다. 통과해야 하는 시험: `complete_languages_translate_every_menu_string`, `bundled_catalogs_are_consistent`, `spaced_catalogs_keep_the_spaces_around_fragments`, `partial_catalogs_only_translate_strings_the_ui_shows`, `every_tl_literal_is_translated`.
8. 바뀌지 않는 동작의 시험이 수정 없이 통과한다: `canvas::tests::cmd_space_zooms_and_space_pans_whatever_the_tool`, `canvas::tests::space_pressed_mid_drag_moves_what_the_tool_draws`, `shortcuts` 시험 전부(Tab/도구 글쇠, 비라틴 자판), `tests_synthetic`(제어 채널 `ui.key`), `prefs_dialog::tests::graphics_processor_preference_shows_and_persists`.
9. `docs/target-app-parity.md:387`의 「right in and left out」 문장을 「left zooms in and right out by default (Drag left to zoom in); off, Illustrator's right = in」으로 고치고 상태 줄·개정 이력 한 줄. 확인: `grep -c 'Drag left to zoom in' docs/target-app-parity.md` ≥ 1.
10. `cargo xtask ci` → exit 0.

## 변경 범위

- `crates/engine/src/lib.rs`: `:536` 뒤 필드, `:750` 뒤 기본값. `#[serde(default, rename_all = "camelCase")]`(`:419-421`)와 기존 필드·기본값은 불변.
- `crates/engine/src/cmd/prefscmds.rs`: `:288` 뒤 `p!` 행 1개. `spec/validate/set_values/reset/list`(`:351-658`)는 PREF_SPECS를 읽으므로 수정 없음. 기존 행·카테고리 `"Performance"`·섹션 `"GPU Performance"` 불변(카탈로그 문자열).
- `crates/engine/src/tests_prefs.rs`: 새 시험 1개.
- `crates/ui-egui/src/canvas.rs`: `:76-77` ZoomScrub 문서, `:626-630`(지역 변수 + `* if left_in { -1.0 } else { 1.0 }`), `:734-735` SCRUB_ZOOM 문서(방향은 환경설정을 따른다), `:745-747` 옆 helper, 시험 `:3023-3071`. `:561-592` 누름 처리, `:600-694` 드래그 계속·놓기, `:759-763` `zoom_about`, `:436-477` 커서는 불변.
- `crates/ui-egui/src/chrome.rs`: `:585-588` 상수, `:661-670` 옆 표, `:757`, `:816`, 시험 `:971-998`.
- `crates/ui-egui/src/prefs_dialog.rs`: `:260-263` 옆 arm, 시험 1개. `snapshot/restore`(`:52-78`) 불변.
- `crates/ui-egui/src/i18n/{ja,ko,de,es,fr,it,ru,uk,zh-hant,zh-hans}.tsv`: 행 2개씩(ko는 3220행 교정 포함). 기존 행 삭제 없음.
- `docs/target-app-parity.md:387`.
- **바꾸지 않는 것**: `animatedZoom` 키·레이블·`gpuPerformance` 의존(`canvas.rs:745-747`, `prefs_dialog.rs:261-263`); 클릭/Alt+클릭 단계(`canvas.rs:681-685`)와 ZoomBox(`:667-679`); Space 팬·Cmd+Space 줌(`:439-441`, `:562-565`); `Session::switch_tool/select_tool`(`engine/src/tooling.rs`); `shortcuts.rs`(이 작업은 글쇠 처리를 건드리지 않는다); 환경설정 영속 경로 `UiState.engine_prefs`; 브리프의 식별자(포맷 이름·확장자·환경설정 폴더·app_id·크레이트 이름·MCP 이름).

## 검증 방법

종료 코드와 로그 경로(`/tmp/claude-0/-home-claude/79152f61-3d49-5a1c-acde-dc6cabf2d3d7/scratchpad/factory/J002/*.log`)를 기록한다. 모두 exit 0.

```
cd /home/claude/vector
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p vectorcraft-engine -- tests_prefs
cargo test -p vectorcraft-ui-egui --lib -- canvas::tests::animated_zoom canvas::tests::cmd_space canvas::tests::space_pressed chrome::tests::the_zoom_hint prefs_dialog::tests i18n shortcuts tests_synthetic
cargo test -p vectorcraft-ui-egui          # 크레이트 전체
grep -n 'LN_2 / 100.0\|SCRUB_SLOP: f32 = 3.0\|HOLD_DELAY: f32 = 0.3\|HOLD_ZOOM: f64 = std::f64::consts::LN_2' crates/ui-egui/src/canvas.rs
cargo xtask ci
```

커밋: `claude/project-thread-5e8gjw`에 **커밋 1개**, 첫 줄 `J002: 애니메이션 줌 방향 환경설정 — 왼쪽으로 끌면 확대(기본 켬)`, 본문 한국어에 `시험: …` 줄과 「옛 환경설정 파일을 가진 사용자도 업그레이드 뒤 기본값 켬을 받아 방향이 바뀐다(의도된 결정)」 한 줄, 마지막 두 줄은 정확히:

```
Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01SE7CNTckuSnG5kNiUFh6EX
```

push·merge·브랜치 전환·`git reset/clean/stash` 금지.

## 범위 밖

### J002b — 스프링 로드 Z (후속 작업, queued)

리더 맵은 작은 설계를 제시했지만(「feasible」), 다음 이유로 이 커밋에 넣지 않는다. 사용자 결정은 「작고 안전할 때만」이고, 작업당 커밋 1개 규칙에서 **1줄 부호 변경 + 카탈로그 행**인 J002와 **글쇠 눌림/놓임 상태 기계**를 한 검토 단위로 묶으면 안전하지 않다:

- 놓임 이벤트를 `shortcuts::handle`의 모든 이른 return(스크럽 `:212`, 대화상자/팔레트 `:215-228`, 타이핑 가드 `:272`, 문자 도구 `:282-301`, busy `:327-350`) 앞에서 읽어야 하고, `WindowFocused(false)`(egui가 keys_down을 비움)·자동 반복 Text 이벤트·비라틴 자판(`physical_key`)·단축키 재지정(`shortcut_editor::tool_for_key`)·포인터가 눌린 채 Z를 놓는 경우(복원을 pointer-up까지 미룸)·제어 채널 `ui.key`(누름+놓음이 한 프레임 → 탭 → 영구 전환 유지)까지 6가지 경우의 시험이 필요하다(`shortcuts.rs`의 기존 `frame` 헬퍼는 프레임마다 새 `Context`라 시간이 0으로 돌아가므로 공유 ctx + `RawInput.time`으로 새 헬퍼가 필요).
- 리더 스스로 「80줄을 넘거나 위험 하나라도 시험으로 못 잡으면 별도 커밋」을 권했다.

J002b 설계(리더 맵 그대로; 스펙 작성 시 그대로 쓴다): `shortcuts.rs`에 `PasteChord`(`:114-163`)를 본뜬 `SpringTool { previous: &'static str, key: Key, physical: Option<Key>, since: f64, used: bool, release_pending: bool }`와 `SPRING_HOLD: f64 = 0.3`; `VectorcraftApp`에 `pub(crate) spring_tool: Option<SpringTool>`(`lib.rs:379` 옆, `:476`에 `None`); 디스패치 사이트 `:452-454`에서 고른 도구가 `"zoom"`이고 이전 도구가 zoom이 아닐 때만 기록(자동 반복 무력화, 타이핑/busy/대화상자 가드 상속; 글쇠는 누름 이벤트에서 얻고 `Key::Z`를 박지 않음); `:210` 직후에 `WindowFocused(false)`(기록 삭제)와 놓임(logical 또는 physical 일치)을 스캔; 기록이 있는 동안 포인터 누름이 있으면 `used`; 놓일 때 `now - since >= 0.3 || used`면 포인터 버튼이 모두 떨어진 뒤 `app.session.switch_tool(previous, view)`(`tooling.rs:84` 「임시 도구에서 복귀」, 오류는 `lib.rs:663-665`처럼 status로), 아니면 탭이라 zoom 유지. 커서·힌트·툴바는 `tool_id()`를 따르므로 추가 작업 없음. 시험 `shortcuts::tests::z_held_borrows_the_zoom_tool_and_a_tap_keeps_it`(공유 ctx, `RawInput { time: Some(t) }`): 탭(<0.3 s) 유지 / ≥0.3 s 복원 / 누른 채 포인터 누름 → 놓을 때 복원 / 문자 도구 편집 중 Text("z") 무시 / `WindowFocused(false)` 중간 → 복원 없이 정리 / physical_key 놓임 일치. `cargo xtask wasm`·`layers` 확인.

### 그 밖의 범위 밖
- 스프링 로드를 모든 도구 글쇠로 일반화.
- 휠 줌·핀치 줌 방향(별개 환경설정 `Zoom with Mouse Wheel`).
- ROADMAP 수치(이 변경은 패리티를 움직이지 않는다; `docs/gaps.md`·`docs/ui-parity.md`는 한 줄 메모만 선택).

## 위험·사람 검토 필요

- **기본값 뒤집기**: 옛 환경설정 파일(키 없음)은 serde 기본값으로 `true`를 받아 업그레이드 즉시 Animated Zoom 방향이 바뀐다 — 사용자 결정(의도됨); 커밋 본문에 적는다.
- `app.view_mut()`(`:637`) 차용 안에서 `app.session.prefs`를 읽으면 컴파일이 안 된다 — 지역 변수로 먼저 읽는다.
- i18n: 10개 카탈로그 모두에 두 행이 있어야 하고(한 곳이라도 빠지면 `complete_languages_translate_every_menu_string` 실패), 조각의 앞 공백·뒤 `  |  `가 바이트 단위로 같아야 하며(`spaced_catalogs_keep_the_spaces_around_fragments`), 중복 행은 `bundled_catalogs_are_consistent`가 잡는다.
- `hint_strings()`에 LEFT 힌트를 체인하지 않으면 새 조각이 i18n 완전성 검사에 들어가지 않아 번역 누락이 조용히 지나간다 — 완료 기준 4가 요구.
- `prefs_dialog` 시험에서 흐려짐(disabled)은 색으로 판정하기 어려워 글자 존재만 확인한다; 실제 흐려짐은 스크린샷으로 한 번 본다(선택).
- 사람 검토: 좌표/단위 알고리즘에 해당하지 않음(부호 하나); 패키징·파일 포맷 무관. 검토 필요 없음.
- 브리프의 HEAD `99ee574` ≠ 실제 HEAD `2d297f8`; 실제 HEAD 위에서 작업한다. J001이 먼저 커밋되면 그 위에서 작업하고 기준 커밋을 기록에 갱신한다(겹치는 파일: `prefs_dialog.rs`, `prefscmds.rs`, `lib.rs`, `ko.tsv` 등 — 서로 다른 줄).

## 기준 커밋

`2d297f89db8652317922cc5830e20addbb979645` (`git -C /home/claude/vector rev-parse HEAD`, 브랜치 `claude/project-thread-5e8gjw`, 2026-10-11)
