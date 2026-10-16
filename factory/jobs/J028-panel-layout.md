상태: build

# J028 (디자인 3.0 D4+): 칸 배치 — 레이어 띠, 옮기기·크기 바꾸기, 배치 처음대로

## 명세 (spec)
- 기준 커밋: `ae3bf9e` (J024 승인 대기 기록, 코드는 `fbc78bd`)
- 작업 브랜치: `claude/project-thread-ams9bp`
- 근거: 사용자 요청(2026-10-16, 코디네이터가 전달)
  - 21:21Z 「레이어도 보여야하지않을까? 그런데 크지않게 오른쪽에 세로로」
  - 21:22Z 「모든창은 재배치할수있게하고 크기를 줄이고 늘일수있게해줘 그리고 잘못되었을때 설정초기화하면 다시 그대로 돌아갈수있게」
  - 「그렇지만 초기에 완벽한 디자인 배치가 되어 사용자가 만지지않아도 편할 배치를 해주ㅝ」
  - 결정 카드 21:24Z: 칸은 「양옆 칸 안에서」만 옮긴다(떠 있는 창 없음).
- 시안: 디자인 3.1 (https://claude.ai/artifact/GmVPKvYKq4ADAe5ZbmZLdc, 버전 3) 4장 시안 ⑪, 5장 ⑪, 9장 좁은 화면, 13장 전체. 원본 `/mnt/project-files/design/trowel-design-v3.src.html`.
- 시안 승인: 2026-10-16 22:18:54Z, 사용자가 결정 카드에서 「추천대로」를 고름(정할 것 4–6 = 추천값: 띠에는 그린 것이 있는 레이어만, 1400 미만이면 띠 접기, 눈·자물쇠는 범례와 띠 둘 다).

### 완료 기준
1. 처음 배치(설정 없는 새 실행)가 13장 막대와 같다.
   - 1440 × 880: 도구 열 50 · 서랍 236 · 캔버스 718 · 오른쪽 열 304 · 레이어 띠 132.
   - 창 너비 1400 미만에서 처음 실행하면 띠가 접혀 28이다(정할 것 5). 사용자가 펴면 그 뒤로 펴 둔 것을 기억한다.
   - 캔버스가 480보다 좁아지면 띠, 그다음 서랍 순서로 접힌다. 서랍은 9장 규칙(1200 미만 접힘)을 그대로 따른다.
   - 확인: 화면 시험(1440 × 880, 1366 × 768, 1280 × 720의 칸 사각형), 새로 띄운 앱 캡처를 시안과 나란히.
2. 레이어 띠(⑪)가 시안처럼 보인다.
   - 머리 「레이어」, ＋(지금 레이어 안에 하위 레이어), «(접기). 줄 높이 24: 눈, ▸/▾, 레이어 색 막대, 이름, (잠겼거나 포인터가 있을 때) 자물쇠, 지금 레이어면 흙색 펜 표시와 oat 바탕.
   - 쌓인 순서(위가 앞). 하위 레이어와 객체는 10씩 들여 쓴다. 요소함에서 놓은 요소는 「도면 요소」 레이어 아래에 보인다(6장).
   - 그린 것이 없는 레이어는 맨 아래 「빈 레이어 N개」 한 줄로 접힌다(정할 것 4). 누르면 펼친다.
   - 접힌 띠(28): » 단추, 세로 글자 「레이어」, 지금 레이어 색 막대.
   - 확인: 화면 시험(줄 순서·들여쓰기·빈 레이어 줄), 새로 띄운 앱 캡처.
3. 레이어 띠 동작은 기존 명령으로 한다(새 엔진 명령 없음이 목표, 필요하면 빌드 절에 이유를 적는다).
   - 줄 누르기 = `layer.setCurrent`. 범례에서도 그 줄이 골라지고 펜이 그 줄 모양이 된다. 고른 선은 옮기지 않는다.
   - 객체 줄 누르기 = 그 객체 고르기(Shift: 더하기).
   - 눈·자물쇠 = `layer.setProps` `visible`/`locked`. 범례 줄과 같은 상태라 서로 같이 바뀐다(정할 것 6). Alt+눈 = 이 레이어만 보기.
   - 끌기 = 쌓는 순서 바꾸기, 다른 레이어 위에 놓으면 그 안으로. 한 번의 되돌리기.
   - 두 번 누르기 = 이름 바꾸기(범례 줄 이름도).
   - 확인: 화면 시험(누르기·눈·자물쇠·끌기·되돌리기), 범례와 띠가 같은 상태를 보이는지.
4. 칸 옮기기: 도면집·요소함, 범례, 펜, 속성, 레이어를 머리로 끌어 왼쪽 열·오른쪽 열·띠 열의 위·아래·칸 사이에 놓는다.
   - 끄는 동안 칸이 반투명하게 따라오고 놓을 자리에 흙색 줄이 보인다. 캔버스 위에서 놓으면 원래 자리로 돌아간다. 떠 있는 창은 만들지 않는다.
   - 열이 비면 그 열은 사라지고 캔버스가 넓어진다. 비어 사라진 열로는 그쪽 가장자리로 끌어 다시 놓는다.
   - 도구 열·리본·상태줄·캔버스는 옮기지 않는다.
   - 확인: 화면 시험(합성 포인터로 펜을 왼쪽 열로 옮기고 사각형 확인), 새로 띄운 앱 캡처.
5. 크기 바꾸기: 열 너비는 캔버스 쪽 가장자리(서랍 140–450, 오른쪽 열 220–560, 띠 112–240), 칸 높이는 칸 사이 경계로.
   - 범례는 줄 2개, 펜은 굵기 한 줄, 속성은 머리 34까지 줄고, 그보다 좁으면 칸 안에서 스크롤한다.
   - 경계를 두 번 누르면 그 경계만 처음 값으로.
   - 확인: 화면 시험(최소·최대 너비, 높이 경계, 두 번 누르기).
6. 배치는 사용자 설정(UiState 저장)에 들어가고 도면 파일에는 들어가지 않는다. 껐다 켜도 그대로다.
   - 기존 작업 공간(`crates/ui-egui/src/workspaces.rs` `Workspace`, `capture`, `apply`)에 Trowel 칸 배치를 더한다. 예전 설정 파일(배치 필드 없음)은 처음 배치로 연다.
   - 확인: 설정 저장·읽기 시험(새 필드 있음/없음), 문서 저장 파일에 배치가 없는 것.
7. 처음대로: 「창 › 배치 처음대로」(맨 위)와 「편집 › 환경설정 › [환경설정 재설정]」이 모두 1의 처음 배치로 되돌린다(숨긴 칸도 다시 보임).
   - 창 메뉴에 칸 이름 다섯 개(✓ 표시, 누르면 숨기기·마지막 자리에 다시 열기)를 둔다. 그 밖의 창 메뉴는 그대로.
   - 확인: 시험(옮기고 크기 바꾼 뒤 두 길로 처음대로 → 1의 사각형과 같음), 새로 띄운 앱 캡처가 처음 캡처와 같음.
8. 기존 동작을 바꾸지 않는다: 범례·펜·속성 칸 안의 그림과 동작, 도구 열, 리본, 상태줄, VectorCraft(Trowel 아닌) 창 배치.
   - 확인: 기존 화면 시험 모두 통과(기대값을 바꾸면 근거를 빌드 절에 적는다).
9. 시안과 나란히 비교: 새로 띄운 앱을 1440 × 880과 1366 × 768에서 캡처해 4장·13장과 나란히 놓고 남은 차이를 모두 적는다. 차이가 남으면 「통과」라고 하지 않는다.
10. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, 바뀐 crate 시험, `cargo xtask layers`, GitHub 「Trowel CI」 통과.

### 변경 범위
- `crates/ui-egui/src/trowel/mod.rs`: 칸 크기 상수(47–56, `SET_W` 216 → 236), `frame`(110–135)과 `right_column`·`right_parts`(145–235)를 열·칸 배치 모델로.
- 새 `crates/ui-egui/src/trowel/layers.rs`(레이어 띠), 칸 머리 끌기·놓기 자리 표시.
- `crates/ui-egui/src/trowel/drawings.rs:15–18`(서랍 너비·범위), `legend.rs`·`palette.rs`·`props.rs`는 놓일 사각형만 받게(안의 그림은 그대로).
- `crates/ui-egui/src/workspaces.rs`(배치 저장·처음대로), `crates/ui-egui/src/state.rs`(UiState 필드), `crates/ui-egui/src/prefs_dialog.rs:197`(재설정이 배치도), `crates/ui-egui/src/menus.rs`(창 메뉴 「배치 처음대로」, 칸 이름 다섯 개; UI 명령은 `UI_COMMANDS`에).
- `crates/ui-egui/src/i18n/ko.tsv`(새 글자), `crates/ui-egui/src/trowel/tests.rs`(화면 시험).
- 바꾸면 안 되는 것: 엔진 명령의 뜻과 인자, 문서 파일 형식, 범례·펜·속성 칸 안의 동작, 클래식 VectorCraft 창.

### 검증 방법
- `CARGO_TARGET_DIR=/home/claude/target-trowel CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0` 아래에서
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p vectorcraft-ui-egui`, `cargo test -p vectorcraft`(설정 저장), 엔진을 건드리면 `cargo test -p vectorcraft-engine`
  - `cargo xtask layers`
- 릴리스 빌드를 새 HOME으로 띄워 control 채널로 1440 × 880·1366 × 768 캡처, 칸 옮기기·처음대로 순서를 실제로 해 보고 캡처.
- push 뒤 GitHub 「Trowel CI」.

### 범위 밖
- 떠 있는 창, 칸을 탭으로 겹치기, 작업 공간 여러 개 저장(기존 VectorCraft 기능은 그대로 둠).
- 요소함 내용(D5), H9 결정(해발선이 들어갈 범례 줄), 새 엔진 레이어 기능.

### 위험·사람 검토 필요
- ◆ 사용자 설정 파일 형식에 필드가 생긴다(예전 파일과 호환: 없으면 처음 배치).
- 레이어 띠의 눈·자물쇠는 범례 줄과 같은 스위치를 두 곳에 둔다(정할 것 6, 「한 기능 한 자리」의 예외).
- 범례 셈과 「빈 레이어」 판단이 다르면 띠와 범례가 서로 다른 레이어 수를 보일 수 있다. 같은 기준(레이어 안 객체 수)을 쓴다.

## 구현 (build)
- 새 파일
  - `crates/ui-egui/src/trowel_layout.rs`: 칸 배치 모델(`Layout`: 열 셋에 놓인 칸, 열 너비, 손으로 접고 편 것, 손으로 정한 칸 높이, 숨긴 칸과 그 자리), 창 너비에 맞춘 열 너비·접기(`shown`), 예전·망가진 설정을 처음 배치로 읽는 `lenient`, UI 명령 `trowel.layout.reset|panel|move|size|get`.
  - `crates/ui-egui/src/trowel/layout.rs`: 열 그리기(칸 쌓기, 칸 사이 경계, 캔버스 쪽 너비 가장자리, 접힌 열 28 레일), 칸 머리 끌기·놓을 자리 표시·반투명 칸.
  - `crates/ui-egui/src/trowel/layers.rs`: 레이어 띠(⑪). 기존 명령만 쓴다: `legend.pick`, `layer.setCurrent`, `layer.selectAll`, `layer.setProps`, `layer.move`, `layer.newSublayer`, 범례의 「이 레이어만 보기」. **새 엔진 명령 없음.**
- 바꾼 파일: `trowel/mod.rs`(오른쪽 열 코드를 layout.rs로), `drawings.rs`(Panel 대신 받은 사각형에 그림, 서랍 216 → 236), `legend.rs`(범례 카드가 열이 왼쪽에 있으면 오른쪽으로 뜸), `palette.rs`·`props.rs`(최소 높이 상수만), `state.rs`(`UiState.trowel_layout`), `workspaces.rs`(작업 공간에 배치, 「필수 재설정」도 처음 배치), `prefs_dialog.rs`(환경설정 재설정이 배치도), `menus.rs`(창 메뉴 맨 위 「배치 처음대로」와 칸 다섯 개 ✓, 클래식 창에서는 숨김, 빈 가름줄 정리), `i18n/ko.tsv`.
- 22:22Z 요청 반영: 창이 좁아지면 열은 사라지지 않고 먼저 최소 너비까지 좁아진다(띠 → 서랍 → 오른쪽 열 순서). 그래도 캔버스가 480보다 좁으면 띠, 그다음 서랍이 접힌다. 손으로 편 열은 저절로 접지 않는다. 배치는 설정 파일에 저장되어 다음 실행에도 그대로 열린다.
- 열의 최소 너비는 그 안의 가장 넓은 칸이 정한다(`Panel::least_width`): 펜 274(색 줄 여덟 칸과 ⋯), 범례·속성 220, 도면집 140, 레이어 띠 112. 그래서 오른쪽 열은 펜이 있는 한 274보다 좁아지지 않고, 펜을 서랍 열이나 띠 열로 옮기면 그 열이 274로 넓어진다. 칸이 나가면 사용자가 정한 너비로 돌아간다. 새로 띄운 앱에서 펜을 서랍 열(236)로 옮겼을 때 흰 색 칸과 「일점쇄선」이 잘리던 것을 이렇게 고쳤다(J028 이전에도 오른쪽 열을 손으로 220까지 줄이면 같은 일이 있었다).
- 만들면서 찾아 고친 결함: 펜 칸의 「세 단추가 생기거나 사라질 때 펜 위치 고정」 상태를 펜이 없는 다른 열이 매 프레임 지웠다. 그래서 고고학 단추를 누를 때 펜 칸이 움직였다(기존 시험 3개 실패로 드러남). 이제 펜이 있는 열만 그 상태를 다루고, 칸 경계를 끄는 동안에는 고정하지 않는다.

### 기대값을 바꾼 기존 시험과 근거
- `regions_match_the_mockup_at_1600_by_988`: 서랍 216 → 236, 오른쪽 열이 띠(132)만큼 왼쪽으로, 띠 열 1468–1600, 캔버스 302–1164. 근거: 디자인 3.1 13장 막대(사용자 승인 22:18:54Z).
- `the_tabs_never_overlap`: 기대값은 그대로(800 × 500에서 긴 이름이 잘림). 서랍과 띠를 손으로 편 800 × 500 사례를 더했다.
- `the_pen_bar_keeps_clear_of_the_zoom_buttons`: 800 × 500에서 이제 공간이 있어 단축키 글자가 보이고 확대 단추 옆 가운데에 놓인다. 서랍과 띠가 접혀 캔버스가 404로 넓어졌기 때문이다(9장 규칙, 이전에는 약 208). 「확대 단추 위, 글자 뺌」은 서랍과 띠를 손으로 편 800 × 500(캔버스 208) 사례로 계속 시험한다.

### 새 시험
- 모델: `the_first_layout_is_the_mockups`, `a_narrow_canvas_narrows_the_columns_then_folds_the_strip_then_the_drawer`, `a_column_is_as_wide_as_its_widest_panel_needs`, `panels_move_hide_and_come_back_where_they_were`, `a_saved_layout_reads_leniently`, `the_layout_is_kept_with_the_preferences`.
- 화면: `the_first_layout_is_sheet_13_in_each_window`(1440 × 880, 1366 × 768, 1280 × 720, 1100 × 700), `the_layer_strip_lists_what_is_drawn_and_folds_the_empty_layers`, `the_strip_picks_shows_and_locks_as_the_legend_does`, `the_strip_renames_and_reorders_its_rows`, `a_panel_moves_by_its_head_within_the_side_columns`, `columns_and_panels_size_by_their_edges`, `either_reset_brings_back_the_first_layout`, `the_window_menu_lists_the_panels_only_in_trowel`.

