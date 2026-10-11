# J001 — Trowel 제품명 + 새 앱 아이콘 + "Plate" UI 테마(기본값)

상태: spec · 기준 커밋: 아래 참조 · 커밋 1개 (`J001: …`)

## 목표

VectorCraft 포크를 고고학 도면 앱 **Trowel(트로웰)** 로 보이게 한다. 세 가지를 한 커밋에 넣는다.

- **A. 이름**: 사용자에게 보이는 제품명을 Trowel로 바꾼다. 창 제목은 `<파일> — Trowel` / 문서가 없으면 `Trowel`, 미저장 `*` 접두사는 그대로. 디스크·프로토콜 식별자(포맷 이름, 확장자, 환경설정 폴더, app_id, 크레이트 이름, 로그 대상, URL, MCP 이름)는 **하나도 바꾸지 않는다**. 업스트림(VectorCraft/ArtCraft) 크레딧은 About 상자에 남기고 보강한다.
- **B. 아이콘**: 흙손 날(아이보리 획)이 점토색 기준선과 점선 수평선 두 줄 위에 놓인 슬레이트 둥근 타일. 권(프로젝트 소유자)이 코드로 그린 원작, MIT OR Apache-2.0. `assets/app-icon/` 파일 **이름은 모두 유지**하고 내용만 바꾼다(패키징 수정 불필요).
- **C. 테마**: 다섯 번째 밝기 프리셋 `plate`(영문 "Quiet Plate", 한국어 「조용한 도판」), 모서리 반경 8. **새 환경설정의 기본값**이며, 저장된 환경설정이 있는 기존 사용자는 자기 밝기를 유지한다.

## 완료 기준

각 항목은 시험 또는 명령으로 확인한다. 기댓값을 바꾸는 시험에는 이유를 적었다(모두 사용자가 확정한 제품명·기본 테마 결정에 따른 표시 문자열 변경이다).

### A. 제품명

1. `crates/ui-egui/src/brand.rs`에 `pub const APP_NAME: &str = "Trowel";`가 있고, `crates/ui-egui/src/lib.rs`가 `pub use brand::APP_NAME;`로 내보내며, `native_menu.rs:41`은 `pub use crate::brand::APP_NAME;`로 바뀐다. 확인: `grep -n 'APP_NAME' crates/ui-egui/src/brand.rs crates/ui-egui/src/lib.rs crates/ui-egui/src/native_menu.rs`.
2. `chrome.rs` 시험 `window_title_with_no_document_is_the_app_name`, `window_title_follows_the_active_file`, `sync_window_title_sends_only_on_change`의 기댓값이 `"Trowel"`, `"foo.svg \u{2014} Trowel"`, `"*foo.svg \u{2014} Trowel"`(`ViewportCommand::Title` 포함)로 바뀌고 통과한다. (이유: 제품명 결정.)
3. `tests_nativemenu.rs` `vectorcraft_gets_a_mac_app_menu`: 메뉴 제목이 `["Trowel", "File", "Edit", "Object", "Type", "Select", "Effect", "View", "Window", "Help"]`, `shown("help.about") == ("About Trowel", None)`, `shown("app.quit") == ("Quit Trowel", Some("Cmd+Q"))`, `shown(native_menu::HIDE) == ("Hide Trowel", Some("Ctrl+Cmd+H"))`로 바꾸고 통과한다. (이유: 제품명 결정. 함수 이름은 그대로 둬도 된다.)
4. `menus.rs:3595` 시험(창 안 메뉴 막대에 앱 메뉴 제목이 없다)이 `APP_NAME`과 비교하도록 바뀌고 통과한다. `menus.rs:2753`의 필터도 `*title != crate::brand::APP_NAME`이어야 한다(아니면 창 안 막대에 "Trowel" 메뉴가 생겨 이 시험이 잡는다).
5. `i18n/tests.rs:229`가 `crate::native_menu::APP_NAME`으로 앱 메뉴를 찾고, `MENU_KEEP_AS_IS[0]`(`:458`)이 `"Trowel"`이다. i18n 시험 5개가 모두 통과한다: `bundled_catalogs_are_consistent`, `partial_catalogs_only_translate_strings_the_ui_shows`, `complete_languages_translate_every_menu_string`, `every_tl_literal_is_translated`, `menu_catalogs_translate_every_menu_label`.
6. 12개 카탈로그(`crates/ui-egui/src/i18n/*.tsv`)에서 source 열에 `VectorCraft`가 남는 행은 다음뿐이다: 크레딧 2행(`VectorCraft on getartcraft.com`, `VectorCraft on GitHub`), 보존 이름 행(`VectorCraft Defaults`, `VectorCraft Default`, `[VectorCraft Default]`, `Differences from VectorCraft Default:`, `VectorCraft Generic CMYK (SWOP-like)`, `VectorCraft Swatches (.vcswatches)`), 그리고 범위 밖 크레이트(format/eps/svg/web)의 `@msg` 행(`not a VectorCraft file: {0}`, `can't save version…`, `file version…`, `is not an image VectorCraft reads`, `it uses `{at}`…`, `it has {_1}…`, `an image VectorCraft can't read…`, `Loading VectorCraft…`, `VectorCraft failed to start: {e}`). 확인: `grep -n $'\t[^\t]*VectorCraft[^\t]*\t' crates/ui-egui/src/i18n/*.tsv` 결과를 위 목록과 대조한다. 이름을 바꾼 행은 source 열과 번역 안의 제품명을 **함께** 바꾸고, 자리표시자(`{what}`, `{folder}`, `{summary}`, `{path}`, `{e}`)와 말줄임표(`…`)는 그대로 둔다. 한국어 행은 커밋 bca3bea의 정책(제품명은 영문 유지)대로 `Trowel`을 라틴 문자로 쓴다: `About Trowel → Trowel 정보`, `Quit Trowel → Trowel 종료`, `Hide Trowel → Trowel 숨기기`, `Trowel Help… → Trowel 도움말…`, `Welcome to Trowel → Trowel에 오신 것을 환영합니다`, `Applies the next time Trowel starts. → 다음에 Trowel을 시작할 때 적용됩니다.` 등.
7. `prefs_dialog.rs:533`·`:564`의 기댓값이 `"Applies the next time Trowel starts."`로 바뀌고 통과한다. (이유: 제품명.)
8. `tests_place.rs:506`이 `contains("Trowel documents")`로 바뀌고 통과한다. (이유: 제품명.)
9. `dialogs/save_options.rs:304`의 레이블이 `["Trowel", "Trowel Template", "PDF", "SVG", "SVG Compressed", "PDF-compatible .ai"]`, `:344`가 `("Trowel Template", &["vctemplate"][..])`로 바뀌고 통과한다. 같은 시험의 파일 이름(`Untitled-1.vectorcraft`, `.vctemplate`)과 포맷 id는 그대로다. (이유: 저장 대화상자 포맷 레이블만 제품명으로; id/확장자 불변.)
10. `crates/engine/src/tests_placed_document.rs:238`이 `(Some("Trowel"), Some(true), Some("ok"))`로 바뀌고 통과한다. (이유: Links 패널 Format 표시값.)
11. About 상자(`dialogs/about.rs`): `:12`는 `tl!("About Trowel")`, `:55`는 `crate::brand::APP_NAME`, 크레딧 줄(`:66-68`)은 정확히 다음 문장이다. 이 문장은 `tl!` 리터럴이므로 complete 카탈로그 10개(ja, ko, de, es, fr, it, ru, uk, zh-hant, zh-hans)에 행이 있어야 `every_tl_literal_is_translated`가 통과한다(옛 `Part of ArtCraft…` 행은 모든 카탈로그에서 지운다; zh-hans의 Noto Sans SC 변형 행도).
    - EN: `Based on VectorCraft by the ArtCraft Team. MIT OR Apache-2.0. Fonts: Source Sans 3, Inter, JetBrains Mono (OFL). Icons: Lucide (ISC) + VectorCraft + Trowel.`
    - KO: `ArtCraft Team의 VectorCraft를 바탕으로 만들었습니다. MIT OR Apache-2.0. 글꼴: Source Sans 3, Inter, JetBrains Mono (OFL). 아이콘: Lucide (ISC) + VectorCraft + Trowel.`
    - 확인: `grep -c 'Based on VectorCraft by the ArtCraft Team' crates/ui-egui/src/dialogs/about.rs crates/ui-egui/src/i18n/{ja,ko,de,es,fr,it,ru,uk,zh-hant,zh-hans}.tsv` → 모두 1.
12. 식별자 보존. 다음 grep이 각각 정확히 1줄을 돌려준다:
    - `grep -c 'prefs_path_for("VectorCraft", "vectorcraft")' apps/vectorcraft/src/main.rs` → 1 (`prefs_dir.rs:28`도 그대로)
    - `grep -c 'with_app_id("ai.storyteller.vectorcraft")' apps/vectorcraft/src/main.rs` → 1
    - `grep -c 'id: "vectorcraft"' crates/engine/src/cmd/fileio/mod.rs` → 1, `grep -c 'id: "template"' …` → 1
    - `grep -c '"format": "vectorcraft-shortcuts"' crates/ui-egui/src/shortcut_editor.rs` → 1
    - `grep -c 'PRESETS: &\[&str\] = &\["VectorCraft Defaults", "Classic Defaults"\]' crates/ui-egui/src/shortcut_editor.rs` → 1
    - `grep -c 'APP_ID: &str = "vectorcraft"' crates/engine/src/cmd/help.rs` → 1 (혹은 그 줄의 현재 형태)
    - `grep -c '"VectorCraft Templates"' crates/engine/src/cmd/fileio/save.rs` → 1
    - `grep -c '"name": "vectorcraft"' crates/mcp/src/server.rs` → 1
    - `git diff HEAD --stat -- packaging apps/vectorcraft/build.rs apps/vectorcraft-web crates/format crates/eps crates/svg crates/color crates/pdf` → 빈 출력.

### B. 아이콘

13. `assets/app-icon/vectorcraft.svg`와 `vectorcraft-small.svg`가 브리프의 SVG와 바이트 단위로 같다: `cmp /tmp/claude-0/-home-claude/79152f61-3d49-5a1c-acde-dc6cabf2d3d7/scratchpad/brief/trowel-icon.svg assets/app-icon/vectorcraft.svg` 와 `cmp … assets/app-icon/vectorcraft-small.svg` → exit 0. `grep -c 'viewBox="0 0 512 512"' assets/app-icon/vectorcraft.svg` → 1 (`packaging/icons.sh:21-22`가 이 리터럴에 의존).
14. `packaging/icons.sh`가 저장소 루트에서 exit 0으로 끝난다(허용되는 stderr는 `warning: iconutil not found…` 한 줄). 그 뒤 `cmp assets/app-icon/vectorcraft-small.svg assets/app-icon/hicolor/scalable/apps/ai.storyteller.vectorcraft.svg` → exit 0.
15. 재생성 결과: `git diff --stat HEAD -- assets/app-icon` 에 `vectorcraft-1024.png`, `vectorcraft-macos-512.png`, `vectorcraft.ico`, `hicolor/{16x16,24x24,32x32,48x48,64x64,128x128,256x256,512x512}/apps/ai.storyteller.vectorcraft.png`, `hicolor/scalable/apps/ai.storyteller.vectorcraft.svg`가 모두 나타나고, `git diff --quiet HEAD -- assets/app-icon/vectorcraft.icns` → exit 0 (**.icns는 그대로**; 이 상자에 iconutil이 없다).
16. 픽셀 검사(exit 0):
    ```
    python3 -I - <<'EOF'
    from PIL import Image
    p = Image.open("assets/app-icon/hicolor/128x128/apps/ai.storyteller.vectorcraft.png").convert("RGBA")
    assert p.size == (128, 128), p.size
    assert p.getpixel((0, 0))[3] == 0 and p.getpixel((3, 3))[3] == 0, "corner must be transparent"
    assert p.getpixel((64, 64)) == (20, 20, 19, 255), p.getpixel((64, 64))
    assert p.getpixel((64, 96)) == (217, 119, 87, 255), p.getpixel((64, 96))   # clay datum
    m = Image.open("assets/app-icon/vectorcraft-macos-512.png").convert("RGBA")
    assert m.size == (512, 512) and m.getpixel((0, 0))[3] == 0 and m.getpixel((30, 30))[3] == 0 and m.getpixel((256, 256))[3] == 255
    print("ok")
    EOF
    ```
17. `cargo test -p vectorcraft-ui-egui --lib brand::` (`the_icon_decodes_to_a_rounded_128_px_tile`, `the_app_bar_paints_the_icon_from_one_cached_texture`)와 `chrome::tests::system_mode_hides_the_mark`가 통과한다.
18. `ASSETS.md` 25-39행(아이콘 17파일)을 **같은 경로로** 제자리 수정: Author `권 (project owner)`, Source `Original code-drawn SVG (\`assets/app-icon/vectorcraft.svg\`), rendered by \`packaging/icons.sh\``, Licence 그대로, Notes `App icon (trowel blade, datum and level lines)`; `vectorcraft.icns` 행의 Notes만 `App icon; still holds the previous engraved-dragon render until packaging/icons.sh runs on macOS (iconutil)`. 확인: `grep -c 'engraved dragon' ASSETS.md` → 1, `grep -c 'trowel blade' ASSETS.md` → 16, `cargo xtask assets` → exit 0. 23-24행(LICENSE.txt/README.md)은 Source 열을 `Trowel`로 바꿔도 되고 그대로 둬도 된다.
19. `assets/app-icon/README.md`를 다시 쓴다: 제목 `# Trowel app icon`, 설명(흙손 날 아이보리 `#FAF9F5`, 점토 기준선 `#D97757`, 점선 수평선 2줄(아이보리 55 %), 슬레이트 타일 `#141413`), Tile 절(`viewBox 0 0 512 512`, `rx=112`) 유지, Provenance = 권(프로젝트 소유자)이 Trowel을 위해 코드로 그린 원작 SVG, 제3자·Adobe 소재 없음, MIT OR Apache-2.0; Files 표에서 트레이싱 크기 설명을 지우고 두 SVG가 같은 파일(약 0.9 KB)임을 적으며, `vectorcraft.icns` 행에 「macOS에서 `packaging/icons.sh`를 돌리기 전까지는 이전 용 그림」이라고 쓴다. How it reaches each OS·Regenerate 절 유지. `LICENSE.txt` 1-5행의 출처 문장도 같은 내용으로 고친다(라이선스 선택과 icons.sh 문장 유지). 확인: `grep -c 'Trowel' assets/app-icon/README.md` ≥ 1, `grep -c 'vectorize_tile\|ArtCraft drawing\|drawn in ArtCraft' assets/app-icon/README.md assets/app-icon/LICENSE.txt` → 0, `grep -c 'iconutil' assets/app-icon/README.md` ≥ 1.
20. 문구 갱신: `crates/ui-egui/src/brand.rs:1` 문서 주석(용 → 흙손), `README.md:345-346`(흙손 아이콘, 권의 코드 원작, 슬레이트/아이보리/점토; `README.md:300`의 raw.githubusercontent URL은 그대로 두고 업스트림 용 그림이 계속 보인다고 한 줄), `docs/target-app-parity.md:394`의 `dragon app icon` → `app icon`. 확인: `grep -c 'dragon' crates/ui-egui/src/brand.rs README.md docs/target-app-parity.md` → 각각 0, 1(README의 URL 안내 줄), 0.
21. `cargo xtask brands` → exit 0; `cargo test -p xtask` 통과(`the_tree_is_clean`, `packaging_describes_a_vector_app`). `packaging/` 아래 파일은 바꾸지 않는다(14번의 icons.sh 실행은 출력물만 쓴다).

### C. Plate 테마

22. `theme.rs` `brightness_roundtrip`을 확장해 통과한다: `Brightness::ALL.len() == 5`, `Brightness::default() == Brightness::Plate`, `Brightness::parse("plate") == Some(Plate)`, `Brightness::parse("Quiet Plate") == Some(Plate)`, `Plate.label() == "Quiet Plate"`, `Plate.id() == "plate"`, `!Tokens::for_brightness(Plate).dark`, `Plate.icon_on != Plate.tool_active`, `Plate.selection_fill == hex(0xE3DACC)`, `Plate.radius == 8`, `Plate.window_radius == 8`. (기존 단언은 그대로.)
23. 새 시험 `theme::tests::old_presets_keep_their_rendering`: Dark/MediumDark/MediumLight/Light 각각 `icon_on == text_strong`, `selection_fill == accent.gamma_multiply(0.55)`, `window_radius == 6`, `radius == 2`, `accent == hex(0x378ef0)`. 또한 `git diff HEAD -- crates/ui-egui/src/theme.rs`에서 101-220행 구간의 변경은 새 필드 3개(`icon_on`, `selection_fill`, `window_radius`)의 추가 줄뿐이어야 한다(검토자가 diff로 확인).
24. 새 시험 `theme::tests::apply_takes_window_radius_selection_fill_and_icon_on_from_tokens`: `apply(&ctx, Plate)` 뒤 `ctx.style().visuals.window_corner_radius == CornerRadius::same(8)`, `menu_corner_radius` 동일, `visuals.selection.bg_fill == t.selection_fill`, `visuals.widgets.active.fg_stroke.color == t.icon_on`; `apply(&ctx, Light)` 뒤 `window_corner_radius == CornerRadius::same(6)`.
25. `Tokens::get`(`theme.rs:225`)의 대체값이 `Tokens::for_brightness(Brightness::default())`다(grep).
26. 기본값 세 곳이 함께 움직인다: `theme.rs:11` `#[default]`가 `Plate`, `state.rs:497` `brightness: Brightness::Plate`(또는 `Brightness::default()`), `crates/engine/src/lib.rs:739` `ui_brightness: s("plate")`. 확인: `grep -n 's("plate")' crates/engine/src/lib.rs`, `grep -n 'Brightness::Plate\|Brightness::default()' crates/ui-egui/src/state.rs`.
27. `crates/engine/src/tests_prefs.rs`: `set_normalizes_strings_labels_and_colours`에 `uiBrightness: "Quiet Plate"` → `"plate"` 정규화 단언을 더하고, 새 시험 `ui_brightness_defaults_to_plate_and_accepts_it`에서 `Prefs::default().ui_brightness == "plate"`, `prefs.set uiBrightness=plate` 성공, `prefs.list`의 `uiBrightness` 행 `options`에 `"plate"` 포함, `"purple"`은 여전히 거부(`set_rejects_out_of_range_and_wrong_types` 그대로 통과).
28. `prefs_dialog.rs` 새 시험 `fresh_app_resolves_to_plate_and_window_brightness_persists`: 새 앱에서 `apply_runtime` 한 프레임 뒤 `app.ui.brightness == Plate`이고 `app.session.prefs.ui_brightness == "plate"`; `app.run("window.brightness", json!({"brightness": "light"}))` 뒤 `prefs.ui_brightness == "light"`이고 `Tokens::get(&ctx).panel == Tokens::for_brightness(Light).panel`; 다시 `"plate"`로 돌아온다. 기존 `old_pref_files_keep_their_brightness`(null engine_prefs → Light 유지), `dialog_ok_applies_prefs_and_ui_fields`, `canvas_color_choices_paint_the_pasteboard`(matchUi == Plate의 pasteboard `#F0EEE6`), `dialog_ok_with_invalid_value_keeps_dialog_and_prefs`가 수정 없이 통과한다.
29. tool_active 우물 위 글리프 7곳이 `t.icon_on`을 쓴다: `toolbar.rs:213`·`:626`, `widgets.rs:30`·`:1758`, `dialogs/blend_options.rs:115`, `free_transform.rs:100`, `panels/character.rs:543`; `theme.rs:315`는 `w.active.fg_stroke = Stroke::new(1.0, t.icon_on)`(나머지 상태는 `t.text` 유지). 확인: `grep -c 'icon_on' crates/ui-egui/src/toolbar.rs` → 2, `widgets.rs` → 2, `dialogs/blend_options.rs` → 1, `free_transform.rs` → 1, `panels/character.rs` → 1, `theme.rs` ≥ 7.
30. `crates/engine/src/cmd/prefscmds.rs:262`의 choice 목록 끝에 `("plate", "Quiet Plate")`; `menus.rs:277` params doc `{brightness: dark|mediumDark|mediumLight|light|plate}`; `menus.rs:1161` 오류 문자열에 `|plate`; `control.rs:15` 주석은 선택. 확인: `grep -n 'plate' crates/engine/src/cmd/prefscmds.rs crates/ui-egui/src/menus.rs`.
31. i18n: complete 카탈로그 10개(ja, ko, de, es, fr, it, ru, uk, zh-hant, zh-hans)에 빈 컨텍스트 행 `\tQuiet Plate\t<번역>`을 알파벳 위치(ko는 `Quick Actions` 다음)에 추가한다. ko는 반드시 `조용한 도판`; 나머지는 빌더가 평이하게 번역(제안: ja 静かなプレート, de Ruhige Platte, es Placa tranquila, fr Planche calme, it Tavola quieta, ru Тихая пластина, uk Тиха пластина, zh-hans 静谧图版, zh-hant 靜謐圖版). cs·pt-br는 추가하지 않는다. `tests_nativemenu::language_and_appearance_are_in_the_app_menu`(Appearance 항목 5개, 하나만 체크)와 `clicks_run_their_command_and_the_menus_follow`가 통과한다.
32. 기존 시험이 수정 없이 통과한다: `tests_docsetup::the_bleed_draws_as_a_red_outline_around_each_artboard`(Plate bleed = `#A33B3B`), `workspaces::apply_classic_then_reset_to_essentials_keeps_brightness`, `panels/artboards.rs:286`.
33. 문서: `docs/ui-parity.md:158` 「five brightness levels, Quiet Plate default」, `docs/target-app-parity.md:374` 「Quiet Plate (default) and the four Illustrator levels」로 고치고 각 문서의 상태 줄·개정 이력에 한 줄. 확인: `grep -c 'Quiet Plate' docs/ui-parity.md docs/target-app-parity.md` → 각 ≥ 1.
34. `cargo xtask ci` → exit 0.

## 변경 범위

### A. 제품명 — 수정 파일과 사이트

비(非) i18n 사이트는 `APP_NAME` + `format!()`; 메뉴 레이블(`&'static str` 튜플)과 `tl!("…")` 리터럴은 **리터럴 치환**만(`tl_literals()`가 소스 텍스트에서 `tl!("`를 찾으므로 `format!`/`concat!` 불가).

- `crates/ui-egui/src/brand.rs`: `pub const APP_NAME` 선언(여기가 본가). `:1` 문서 주석.
- `crates/ui-egui/src/lib.rs:17` `mod brand;` 뒤에 `pub use brand::APP_NAME;`.
- `crates/ui-egui/src/native_menu.rs`: `:41` → `pub use crate::brand::APP_NAME;`; `:52` MAC_LABELS `"Hide Trowel"`; `:315` `"Hide Trowel"`; `:345` `"Quit Trowel"`; `:348`은 그대로.
- `crates/ui-egui/src/chrome.rs`: `:52` 앱 바 가운데 레이블 → `APP_NAME`; `:117-124` `window_title` → `format!("{}{} \u{2014} {APP_NAME}", …)` / `APP_NAME.into()`; 시험 `:1015-1027`, `:1059-1074`.
- `crates/ui-egui/src/menus.rs`: `:319` `"About Trowel"`, `:321` `"Quit Trowel"`, `:1947` 제목 `crate::brand::APP_NAME`, `:1948` `"About Trowel"`, `:2569` `"Trowel Help…"`(말줄임표 유지), `:2571` `"About Trowel"`, `:2753` 필터 `APP_NAME`, `:3595` 시험. **그대로**: `:2565-2566`(크레딧 링크), `:180`·`:573`(보존 프리셋 이름 인용), 주석.
- `crates/ui-egui/src/dialogs/about.rs`: `:12`, `:55`, `:66-68`(완료 기준 11의 문장).
- `tl!` 리터럴: `canvas.rs:1844` `"Welcome to Trowel"`, `io.rs:324`, `dialogs/recovery.rs:14`, `dialogs/place.rs:99` `"Trowel documents: …"`, `dialogs/missing_fonts.rs:321`(`{folder}` 유지), `prefs_dialog.rs:269`·`:320`.
- `crates/ui-egui/src/shortcut_editor.rs`: `:369` `"not a Trowel shortcut set (missing \`overrides\`)"`, `:461`·`:469` `"Trowel Shortcuts.json"`. **그대로**: `:20` PRESETS, `:23` LEGACY_PRESETS, `:365` `"vectorcraft-shortcuts"`.
- `crates/ui-egui/src/community.rs:62` 그대로(크레딧 링크).
- `apps/vectorcraft/src/main.rs`: `:106`·`:108` 상태 메시지 `format!("… restart {APP_NAME}")`(`vectorcraft_ui_egui::APP_NAME`), `:413` `.with_title(APP_NAME)`, `:549` `run_native(APP_NAME, …)`(eframe `persistence` 기능이 꺼져 있어 저장 키가 아님, `apps/vectorcraft/Cargo.toml:23`), `:591` `"… so Trowel started again on {summary}{hint}"`. **그대로**: `:148`·`:456`·`:488` 바이너리 이름 출력, `:164-174` prefs 경로, `:375`·`:377` 아이콘 경로, `:422` app_id, `:469`·`:638` 로그 줄.
- 엔진 표시 문자열(모두 리터럴): `crates/engine/src/cmd/fileio/mod.rs:447` `label: "Trowel"`, `:538` `label: "Trowel Template"`(id·확장자·mime 불변); `cmd/links.rs:896` `"format": "Trowel"`; `cmd/fileio/load.rs:175`·`:373`; `cmd/place/mod.rs:148`; `cmd/place/document.rs:58`; `cmd/findfiles/walk.rs:402-403`; `cmd/fontfiles.rs:342` `"Trowel's Fonts folder isn't set here: the desktop app and vectorcraft-cli set it"`(`vectorcraft-cli`는 바이너리 이름, 유지); `cmd/recovery.rs:819`; `cmd/fileio/save.rs:279-283`·`:354`; `cmd/fileio/affinity.rs:200`. **그대로**: `save.rs:101-107` Templates 폴더, `help.rs:9/27-28`, params doc·주석, 시험 데이터(`"Hello VectorCraft"`, EPS 마커 `%VectorCraft_BeginData:`).
- 시험 갱신: `chrome.rs`, `menus.rs:3595`, `tests_nativemenu.rs:69-72/97-100/203`, `i18n/tests.rs:229/458`, `prefs_dialog.rs:533/564`, `tests_place.rs:506`, `dialogs/save_options.rs:304/344`, `crates/engine/src/tests_placed_document.rs:238`.
- 카탈로그 12개: 이름을 바꾼 평문 키 11개(About/Hide/Quit/Help…/Applies…/Opening it…/copies the fonts/didn't quit/documents:/Welcome/크레딧 줄)와 `@msg` 키 5개(`The graphics processor…so Trowel started again on {summary}`, `Trowel doesn't search other apps'…: {path}`, `Trowel doesn't search the Shared or Public folder in Users: {path}`, `Trowel's Fonts folder isn't set here: the desktop app and vectorcraft-cli set it`, `not a Trowel shortcut set (missing \`overrides\`)`; de/es/fr/it/ja/ko/ru/uk)의 source·번역을 바꾼다. `zh-hans.tsv:2987`·`zh-hant.tsv:3079`의 `\tVectorCraft\tVectorCraft` 행은 `\tTrowel\tTrowel`로(앱 메뉴 제목; 두 카탈로그는 KEEPS_MENU_NAMES 밖이라 행이 필요하다). KEEPS_MENU_NAMES 10개 카탈로그(cs, de, es, fr, it, ja, ko, pt-br, ru, uk)에는 `Trowel` 단독 행을 **넣지 않는다**.

### B. 아이콘 — 수정 파일

- `assets/app-icon/vectorcraft.svg`, `vectorcraft-small.svg` ← 브리프 SVG(동일 내용). 파일 이름은 `brand.rs:8`, `main.rs:375/377`, `build.rs:9/15`, `icons.sh:11/26/28/33/36/44/54`, `xtask/src/bundle.rs:23`, `packaging/*/package.*`, `dmg/background.svg:8`, ASSETS.md 17행에 박혀 있으므로 **절대 바꾸지 않는다**.
- `packaging/icons.sh` 실행으로 재생성: `vectorcraft-1024.png`, `vectorcraft-macos-512.png`, `hicolor/<size>/apps/ai.storyteller.vectorcraft.png`(16-512), `hicolor/scalable/apps/ai.storyteller.vectorcraft.svg`, `vectorcraft.ico`(`cargo run -q -p xtask -- ico`; `xtask/src/ico.rs`는 256 px 초과를 거부). 실행 순서: 다른 cargo 빌드가 돌지 않을 때(44행이 `target/`의 빌드 락을 기다린다).
- `vectorcraft.icns`: 손대지 않는다(iconutil 없음). 결과: 실행 중인 앱의 Dock 아이콘은 흙손(`with_icon`), Finder·휴식 상태의 .app은 용 그림 — README/ASSETS.md에 명시, 사람 검토 항목.
- `assets/app-icon/README.md`, `assets/app-icon/LICENSE.txt`, `ASSETS.md:25-39`, `README.md:345-346`, `docs/target-app-parity.md:394`, `crates/ui-egui/src/brand.rs:1`.
- 새 파일 이름 금지(`cargo xtask assets`가 ASSETS.md 행 없는 자산을 거부).

### C. 테마 — 수정 파일

- `crates/ui-egui/src/theme.rs`
  - `:1` 모듈 문서 「five UI brightness levels (Illustrator's four plus Quiet Plate)」.
  - `:8-15` `enum Brightness`: `Plate` 변형을 **끝에 추가**, `#[default]`를 Plate로 이동. 기존 변형 이름·순서 불변(UiState JSON이 변형 이름을 저장).
  - `:18` `ALL: [Brightness; 5]`(Plate 마지막). `:19-37` `label()` `"Quiet Plate"`, `id()` `"plate"`; `parse()`는 ALL을 돌므로 수정 없음.
  - `:40-96` `Tokens`에 필드 3개 추가(기존 필드는 이름·의미 불변):
    - `pub icon_on: Color32` — tool_active 우물 **위에** 그리는 글리프/글자.
    - `pub selection_fill: Color32` — egui `Visuals.selection.bg_fill`(텍스트 선택·선택 가능한 행의 바탕).
    - `pub window_radius: u8` — egui 창·메뉴 모서리.
  - `:101-143` `base`(Dark): `icon_on: hex(0xf5f5f5)`(= base text_strong), `selection_fill: hex(0x378ef0).gamma_multiply(0.55)`(오늘의 `theme.rs:298` 식과 동일; 네 프리셋 모두 accent를 base에서 물려받으므로 한 값으로 충분), `window_radius: 6`. 각 arm에 `icon_on`만 추가: MediumDark `hex(0xffffff)`, MediumLight `hex(0x000000)`, Light `hex(0x000000)`(각 arm의 text_strong과 같은 값). **그 외 101-220행의 값은 한 글자도 바꾸지 않는다**(공개 Illustrator 스크린샷에서 측정한 값).
  - `:222` 앞에 `Brightness::Plate` arm — 아래 전체 토큰 표.
  - `:225` `Tokens::get` 대체값 → `Brightness::default()`.
  - `:296-297` `CornerRadius::same(t.window_radius)`; `:298` `v.selection.bg_fill = t.selection_fill`; `:315` 루프 뒤 `w.active.fg_stroke = Stroke::new(1.0, t.icon_on)`(다른 상태는 `t.text` 유지).
  - 시험 `:345` 확장 + 새 시험 2개(완료 기준 23·24).
- 우물 위 글리프 7곳 → `t.icon_on`: `toolbar.rs:213`(`if is_active { t.icon_on } else { t.icon }`), `toolbar.rs:626`, `widgets.rs:30`, `widgets.rs:1758`, `dialogs/blend_options.rs:115`, `free_transform.rs:100`, `panels/character.rs:543`. 선택: `titlebar.rs:79`(눌린 동안), `widgets.rs:46/84` flat_fill — 이번에는 두지 않는다.
- `crates/ui-egui/src/state.rs:497` 기본 밝기 Plate.
- `crates/engine/src/lib.rs:739` `s("plate")`. (이 값이 매 프레임 `prefs_dialog::apply_runtime`에 재해석되므로 빠뜨리면 새 설치가 2프레임째에 MediumDark로 돌아간다.)
- `crates/engine/src/cmd/prefscmds.rs:262` choice에 `("plate", "Quiet Plate")`(엔진은 프리셋 목록의 별도 사본을 갖는다; 빠뜨리면 `prefs.set uiBrightness=plate`가 거부되고 대화상자에 안 보이며, `window.brightness`가 쓴 id를 `apply_runtime`이 못 읽어 테마가 되돌아간다).
- `crates/ui-egui/src/menus.rs:277`·`:1161`, `control.rs:15`(주석).
- 카탈로그 10개에 `Quiet Plate` 행. `crates/engine/src/tests_prefs.rs`, `prefs_dialog.rs` 시험.
- `docs/ui-parity.md:158`, `docs/target-app-parity.md:374`.
- **그대로**: `prefs_dialog.rs:58-62` `restore()`(null engine_prefs 파일의 저장 밝기 유지), `:103-134` `apply_runtime`(모르는 id면 현재 테마 유지), `:80-100` canvasColor 매핑과 회색 값, 키 `uiBrightness`/`canvasColor`, 명령 id `window.brightness`·`ui.set {brightness}`, `anchor_selected`·`smart_guide` 필드(미사용이지만 공개 구조체; Plate 값은 준다), `theme\tAppearance` 컨텍스트 행(ja:3284, de:3214).

#### Plate 토큰 표 (`Brightness::Plate` arm, `..base` 위에 전부 명시)

| 필드 | 값 | Strata 토큰 / 근거 |
|---|---|---|
| `dark` | `false` | 밝은 테마 (`Visuals::light()` 기반) |
| `app_bar` | `#FAF9F5` | ground |
| `panel` | `#FAF9F5` | ground |
| `panel_darker` | `#F5F4ED` | wash (egui faint_bg) |
| `tab_strip` | `#F0EEE6` | desk |
| `input` | `#FFFFFF` | surface |
| `input_border` | `#C2C0B6` | edge |
| `button_border` | `#C2C0B6` | edge |
| `border` | `#DEDCD1` | line |
| `divider` | `#DEDCD1` | line |
| `text` | `#141413` | ink |
| `text_strong` | `#141413` | ink (36개 파일이 패널 위 강조 글자로 읽으므로 아이보리로 두지 않는다) |
| `text_dim` | `#5E5D59` | muted |
| `text_disabled` | `#73726C` | faint |
| `icon` | `#30302E` | hand |
| `hover` | `#E8E6DC` | press |
| `tool_active` | `#30302E` | hand (활성 도구 우물) |
| `icon_on` (신규) | `#FAF9F5` | ground — 우물 위 글리프는 아이보리 (ink 글리프는 hand 위에서 안 보인다) |
| `accent` | `#30302E` | hand (브리프: ink 또는 hand; `primary_button` hover 채움, 링크, 편집 중 테두리, 탭 밑줄) |
| `accent_strong` | `#141413` | ink (`primary_button`·check_row·radio는 흰 글자/체크를 그 위에 그린다 → 어두워야 함) |
| `selection_fill` (신규) | `#E3DACC` | oat — `accent.gamma_multiply(0.55)`는 ink 글자 밑의 잉크 반투명이 되어 읽을 수 없으므로 명시 값 |
| `row_selected` | `#E3DACC` | oat (항상 `t.text` ink가 위에 놓임) |
| `pasteboard` | `#F0EEE6` | desk (Canvas Color = Match User Interface Brightness가 보여주는 값) |
| `ruler` | `#F5F4ED` | wash |
| `ruler_tick` | `#87867F` | 브리프 지정 |
| `guide` | `#6A9BCC` | sky (guideColor 환경설정의 대체값) |
| `smart_guide` | `#C46686` | fig (미사용 토큰, 값만 제공) |
| `selection` | `#B5573A` | action (Layer 1 기본 선택색; ink 선 위에서 보임) |
| `anchor_selected` | `#B5573A` | action |
| `flyout_active` | `#9C4A2F` | clayText (플라이아웃 패널의 ink 글자와 구분돼야 함) |
| `measure_bg` | `Color32::from_rgba_unmultiplied(0x14, 0x14, 0x13, 235)` | ink 약 92 % (측정 레이블의 흰 글자 유지) |
| `button` | `#F0EEE6` | desk (egui inactive 채움; ground 위에서 보이되 조용하게, hover=press가 더 어둡다) |
| `radius` | `8` | 브리프 |
| `window_radius` (신규) | `8` | 브리프 (창·메뉴 모서리) |
| `font_size` | `13.0` | base 그대로 |
| `caption_close` | `#A33B3B` | block |
| `caption_close_text` | `Color32::WHITE` | base 그대로 |
| `highlight` | `#A33B3B` | block |
| `bleed` | `#A33B3B` | block |
| `error` | `#A33B3B` | block |
| `warning` | `#7A5A00` | caution |
| `axis_x` | `#D97757` | clay |
| `axis_y` | `#3F6B31` | done |
| `axis_z` | `#6A9BCC` | sky |

네 기존 프리셋의 렌더링에서 유일하게 달라지는 점(선언된 편차): 우물 위 글리프 7곳과 `theme.rs:315`가 `icon_on`(= 각 프리셋의 `text_strong`)을 쓰므로, 오늘 `t.text`를 쓰던 `widgets.rs:30`, `blend_options.rs:115`, `free_transform.rs:100`과 `t.accent`를 쓰던 `widgets.rs:1758`에서 활성 글리프가 `text` → `text_strong`(Dark: `#d8d8d8`→`#f5f5f5`, Light: `#1f1f1f`→`#000000`)으로 아주 조금 밝아진다. 토큰 값 자체는 불변이며, 가독성이 오르는 방향이다. 검토자는 이것 외의 차이가 없는지 diff로 본다.

### 변경 금지 목록 (A·B·C 공통)

- 포맷 id `"vectorcraft"`/`"template"`, 저장 파일 헤더 `"format": "vectorcraft"`/`"drawcraft"`, `vectorcraft_format::EXTENSION`/`LEGACY_EXTENSION`, 확장자 `.vectorcraft` `.drawcraft` `.vctemplate`, EPS 마커 `%VectorCraft_BeginData:`/`%VectorCraft_EndData`.
- 환경설정 폴더 `prefs_path_for("VectorCraft", "vectorcraft")`와 DrawCraft 대체 경로, 옆 폴더 Fonts/Swatches/Graphic Styles/Data Recovery/logs, `Documents/VectorCraft Templates`, 임시 폴더 `VectorCraft Save for Web`·`vectorcraft-*`.
- `with_app_id("ai.storyteller.vectorcraft")`, 아이콘 파일 경로·이름 전부.
- 크레이트·패키지 이름, `VectorcraftApp` 타입, `vectorcraft-cli` 바이너리 이름, 환경 변수 `VECTORCRAFT_*`, `--version` 출력, `eprintln!("vectorcraft: …")`, 로그 접두사 `log::…("vectorcraft: …")`.
- URL(`help.rs:9` `APP_ID`, getartcraft.com/apps/vectorcraft, github.com/storytold/vectorcraft, Discord), MCP `serverInfo.name`, `vectorcraft://` 리소스 URI, 도구·프롬프트 id.
- 보존 프리셋·라이브러리 이름: `"VectorCraft Defaults"`(단축키 세트, ui.json `shortcut_set`), `"VectorCraft Default"`(PDF 프리셋), `"VectorCraft Generic CMYK (SWOP-like)"`, `"format": "vectorcraft-shortcuts"`.
- egui id·텍스처 이름 `vectorcraft-brand-mark`, `vectorcraft-context-tag`, `vectorcraft-fonts-folder-pref`, `vectorcraft-find`.
- 업스트림 귀속: `NOTICE`, `LICENSE-MIT`, `credits.rs`, Help 링크 `VectorCraft on getartcraft.com`/`VectorCraft on GitHub`.
- 기존 밝기 프리셋 4개의 id·레이블·serde 변형 이름·토큰 값; `Brightness::parse`의 id/레이블 대소문자 무시 수용; `restore()`/`apply_runtime`의 기존 사용자 보호 경로; `uiBrightness`/`canvasColor` 키와 canvasColor id·회색 값.
- `packaging/*`, `apps/vectorcraft/build.rs`, `apps/vectorcraft-web/*`, `crates/{format,eps,svg,color,pdf,mcp}` — 이 작업에서 손대지 않는다.

## 검증 방법

빌더는 명령마다 종료 코드를 기록하고, 긴 출력은 스크래치패드(`/tmp/claude-0/-home-claude/79152f61-3d49-5a1c-acde-dc6cabf2d3d7/scratchpad/factory/J001/*.log`)에 저장해 경로를 기록한다. 순서대로 실행하고 모두 exit 0이어야 한다.

```
cd /home/claude/vector
# 아이콘 (다른 cargo가 돌지 않을 때)
packaging/icons.sh
cmp /tmp/claude-0/-home-claude/79152f61-3d49-5a1c-acde-dc6cabf2d3d7/scratchpad/brief/trowel-icon.svg assets/app-icon/vectorcraft.svg
cmp assets/app-icon/vectorcraft.svg assets/app-icon/vectorcraft-small.svg
cmp assets/app-icon/vectorcraft-small.svg assets/app-icon/hicolor/scalable/apps/ai.storyteller.vectorcraft.svg
git diff --quiet HEAD -- assets/app-icon/vectorcraft.icns
git diff --stat HEAD -- assets/app-icon            # 16개 파일, .icns 없음
python3 -I - <<'EOF'   # 완료 기준 16의 스크립트
...
EOF
# 서식·린트
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
# 손댄 크레이트 시험
cargo test -p vectorcraft-ui-egui
cargo test -p vectorcraft-engine -- tests_prefs tests_placed_document
cargo test -p xtask
# 자산·브랜드 게이트
cargo xtask assets
cargo xtask brands
# 식별자·문구 grep (완료 기준 6, 12, 18-20, 25-26, 29-31, 33)
grep -n $'\t[^\t]*VectorCraft[^\t]*\t' crates/ui-egui/src/i18n/*.tsv
grep -c 'engraved dragon' ASSETS.md
# 전체 게이트 (fmt, clippy, test --workspace, assets, brands, layers, wasm)
cargo xtask ci
```

`cargo xtask cleanroom`은 이 포크의 xtask에 없다(CLAUDE.md 문구만 남음) — 게이트로 쓰지 않는다. 화면 확인이 가능하면(`cargo run --release -p vectorcraft -- --control 7979` + `ui.screenshot`) Plate 기본 화면과 About 상자 PNG를 스크래치패드에 남긴다(필수 아님).

커밋: 현재 브랜치 `claude/project-thread-5e8gjw`에 **커밋 1개**, 메시지 첫 줄 `J001: Trowel 제품명·흙손 아이콘·Quiet Plate 기본 테마`, 본문 한국어, `시험: …` 줄(실행한 명령과 결과), 마지막 두 줄은 정확히:

```
Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_01SE7CNTckuSnG5kNiUFh6EX
```

push·merge·브랜치 전환·`git reset/clean/stash` 금지.

## 범위 밖

- 패키징·CI: `apps/vectorcraft/build.rs:16-19`(Windows ProductName), `packaging/macos/Info.plist.in`(CFBundleDisplayName/CFBundleName, CFBundleIconFile), `packaging/windows/vectorcraft.wxs`·`package.ps1`(MSI 배너의 `#e8573f` 줄무늬), `packaging/macos/dmg/background.{svg,tiff}`(용 그림+빨강 바탕), `packaging/linux/*`, `.desktop`, nfpm, `apps/vectorcraft-web/*`(`Loading VectorCraft…`) → 별도 작업, 사람 검토.
- `vectorcraft.icns` 재생성(macOS의 `packaging/icons.sh`) 또는 Linux용 icns 패커(`xtask icns`) 추가.
- 범위 밖 크레이트의 사용자 문자열: `crates/format/src/lib.rs:78-84`, `crates/eps`, `crates/svg/src/import/files.rs:130`, `crates/color/src/palette_io.rs:41`(`VectorCraft Swatches (.vcswatches)`), MCP `serverInfo.title`/INSTRUCTIONS/도구 설명(`crates/mcp`).
- 단축키 세트 `"VectorCraft Defaults"` → `"Trowel Defaults"` 개명(LEGACY_PRESETS 이전 필요), PDF 프리셋 이름.
- `titlebar.rs:79`·flat_fill(`widgets.rs:46/84`) 눌림 중 글리프 색, `font_menu.rs:359`·`all_tools.rs:63`의 `selection` 채움 위 글자 대비(아래 위험 참조).
- 46곳의 하드코딩 `CornerRadius::same(n)`(16개 위젯 파일)을 토큰으로 바꾸는 일.
- README `Status` 절·ROADMAP 수치 변경(제품명 변경은 패리티 수치를 움직이지 않는다; 진행 로그 한 줄은 선택).

## 위험·사람 검토 필요

- **사람 검토(패키징/CI)**: `.icns`가 구버전이라 macOS .app/Finder는 용 아이콘, 실행 중 Dock은 흙손. DMG 배경·MSI 배너 색상도 옛 빨강. 다음 macOS 세션에서 `packaging/icons.sh` 실행 + 패키징 이름 변경을 한 작업으로.
- **사람 검토(시각)**: Plate에서 `accent = hand`라 하이퍼링크(`theme.rs:300`, `canvas.rs:1896`), 선택된 탭 글자(`tabs.rs:191`), 스피너(`chrome.rs:575`), 선택 테두리(`swatches.rs:459-465`, `layers.rs:160/496/796`)가 일반 ink와 거의 같은 색이다. 브리프가 ink/hand로 못 박았으므로 그대로 두고, 스크린샷을 보고 action(`#B5573A`)으로 옮길지 사용자가 정한다. `selection = action` 위에 ink 글자가 놓이는 `font_menu.rs:359`·`all_tools.rs:63`도 같은 확인 대상.
- `radius 8`은 egui 위젯 모서리(`t.cr()`)에도 적용된다(높이 20 px 버튼이면 꽤 둥글다). 스크린샷으로 확인.
- 기본값 3곳(`theme.rs:11`, `state.rs:497`, `lib.rs:739`)과 엔진 choice 목록(`prefscmds.rs:262`) 중 하나라도 빠지면 새 설치가 첫 프레임 뒤 MediumDark로 튀거나 `plate`가 거부된다 — 완료 기준 26-28·30이 잡는다.
- serde: `UiState`가 `#[serde(default)]`이므로 `brightness` 키가 없는 ui.json(실제로는 없음)은 Plate가 된다; 옛 변형 이름이 든 파일은 영향 없음.
- i18n: 바꾼 키 하나라도 어느 카탈로그에 옛 행으로 남으면 `partial_catalogs_only_translate_strings_the_ui_shows`/complete 검사가 실패한다. 자리표시자 집합·말줄임표 동일성(`bundled_catalogs_are_consistent`).
- 저장 대화상자 포맷 레이블 변경은 `document.formats` JSON의 `label`과 손실 저장 경고 문구를 바꾼다(id·확장자는 그대로라 파일은 영향 없음; 레이블을 매칭하는 외부 스크립트는 깨진다).
- `tests_docsetup` bleed 시험은 `Brightness::default()`의 bleed(`#A33B3B`)로 윤곽을 찾는다; 같은 프레임에 같은 색 획(error/highlight)이 생기면 모호해진다(현재는 없음).
- 브리프는 HEAD를 `99ee574`로 적었으나 브랜치의 실제 HEAD는 `2d297f8`(i18n 커밋 하나 더). 실제 HEAD 위에서 작업한다.

## 기준 커밋

`2d297f89db8652317922cc5830e20addbb979645` (`git -C /home/claude/vector rev-parse HEAD`, 브랜치 `claude/project-thread-5e8gjw`, 2026-10-11)

## 구현 요약

기준 커밋 `431b364`(브랜치 `claude/project-thread-5e8gjw`; 명세의 `2d297f8` 위에 factory 명세 커밋 하나가 더 있었다) 위에서 작업했다. 커밋 1개.

### 바뀐 것

- **A. 제품명** — `crates/ui-egui/src/brand.rs`에 `pub const APP_NAME: &str = "Trowel"`(본가), `lib.rs`가 `pub use brand::APP_NAME`, `native_menu.rs:41`은 `pub use crate::brand::APP_NAME`. 창 제목(`chrome.rs`), 앱 바 가운데 레이블, 메뉴 트리 제목·필터(`menus.rs`), About 상자(`dialogs/about.rs`: 제목 `About Trowel`, 이름 `APP_NAME`, 크레딧 줄 = 완료 기준 11의 문장), Mac 앱 메뉴(`Hide/Quit Trowel`), `tl!` 리터럴 7곳(canvas, io, recovery, place, missing_fonts, prefs_dialog ×2), `shortcut_editor.rs`(오류 문구·내보내기 파일 이름), `apps/vectorcraft/src/main.rs`(상태 메시지 `{APP_NAME}`, `with_title`, `run_native`), 엔진 표시 문자열(포맷 label `Trowel`/`Trowel Template`, Links Format, load/place/recovery/walk/fontfiles/save/affinity 문구). 식별자(포맷 id, 확장자, `prefs_path_for`, app_id, `vectorcraft-shortcuts`, `VectorCraft Defaults/Default`, Templates 폴더, MCP 이름, URL)는 그대로다.
- **i18n** — 카탈로그 12개: 평문 키 11개(cs·pt-br는 자기가 가진 4·10개)와 `@msg` 키 5개의 source·번역에서 제품명을 Trowel로(ko는 조사 를→을, 가→이, 는→은), `brightness must be …` `@msg` 행 8개에 `|plate`, 옛 `Part of ArtCraft…` 행 11개 삭제(zh-hans의 Noto Sans SC 변형 포함), complete 카탈로그 10개에 새 크레딧 행(ko는 명세 문장, 나머지는 평이한 번역)과 `Quiet Plate` 행(ko 「조용한 도판」, `Quickly start a new file` 다음·`Quit Trowel` 앞). zh-hans/zh-hant의 `\tVectorCraft\tVectorCraft` → `\tTrowel\tTrowel`. KEEPS_MENU_NAMES 카탈로그에는 `Trowel` 단독 행 없음. `MENU_KEEP_AS_IS[0]`·i18n 시험은 `APP_NAME`을 본다.
- **B. 아이콘** — `assets/app-icon/vectorcraft.svg`·`vectorcraft-small.svg` ← 브리프 SVG(바이트 동일). `packaging/icons.sh`로 PNG 10개·scalable SVG·`.ico` 재생성(`.icns`는 iconutil이 없어 그대로). `ASSETS.md` 아이콘 행 15개(Author `권 (project owner)`, Source `Original code-drawn SVG…`, Notes 흙손/`.icns`는 이전 용 그림 유지), LICENSE/README 행의 Source는 `Trowel`. `assets/app-icon/README.md`·`LICENSE.txt` 다시 씀. `brand.rs:1` 문서 주석, `README.md` 아이콘 문단(+ 가족 표의 raw URL이 업스트림 용 그림을 계속 보여준다는 한 줄), `docs/target-app-parity.md`의 `dragon app icon` → `app icon`.
- **C. 테마** — `theme.rs`: `Brightness::Plate`(끝에 추가, `#[default]`), `ALL` 5개, label `Quiet Plate`/id `plate`; `Tokens`에 `icon_on`·`selection_fill`·`window_radius` 추가(base: `text_strong`과 같은 값, `accent.gamma_multiply(0.55)`, 6; 각 arm에 `icon_on`만 추가; Plate arm은 명세 표의 44개 토큰 전부 명시, `..base` 없음 — 전부 명시하면 clippy `needless_update`), `Tokens::get` 대체값 `Brightness::default()`, `apply`가 창·메뉴 모서리·`selection.bg_fill`·`widgets.active.fg_stroke`를 토큰에서 읽음. 우물 위 글리프 7곳 → `t.icon_on`(`character.rs`는 `!enabled → text_disabled, on → icon_on, else text_strong` 순서라 기존 프리셋은 픽셀 동일). 기본값 3곳(`theme.rs`, `state.rs:497`, `engine/lib.rs:739` `s("plate")`)과 `prefscmds.rs:262` choice, `menus.rs:277/1161` 문구. `docs/ui-parity.md:158`, `docs/target-app-parity.md:374`와 두 문서의 상태 줄·개정 이력.
- **시험** — `theme.rs` `brightness_roundtrip` 확장 + `old_presets_keep_their_rendering` + `apply_takes_window_radius_selection_fill_and_icon_on_from_tokens`; `prefs_dialog.rs` `fresh_app_resolves_to_plate_and_window_brightness_persists`; `engine/tests_prefs.rs` `set_normalizes_strings_labels_and_colours` 확장 + `ui_brightness_defaults_to_plate_and_accepts_it`; 기댓값 변경: `chrome.rs` 3개, `menus.rs` 창 안 막대 시험, `tests_nativemenu.rs`, `i18n/tests.rs`, `prefs_dialog.rs` ×2, `tests_place.rs`, `save_options.rs`, `tests_placed_document.rs`(모두 제품명·기본 테마 결정에 따른 표시 문자열).

### 검사 (종료 코드; 로그 `/tmp/claude-0/-home-claude/79152f61-3d49-5a1c-acde-dc6cabf2d3d7/scratchpad/factory-logs/J001-build-*.log`)

| 명령 | 종료 |
|---|---|
| `packaging/icons.sh` (stderr: iconutil 경고 한 줄) | 0 |
| `cmp` 브리프 SVG = `vectorcraft.svg` = `vectorcraft-small.svg` = `hicolor/scalable/...svg`; `git diff --quiet HEAD -- assets/app-icon/vectorcraft.icns` | 0 |
| 완료 기준 16 픽셀 검사 (`python3 -I`) | 0 |
| `cargo fmt --all` / `cargo fmt --all -- --check` | 0 / 0 |
| `cargo clippy -p vectorcraft-ui-egui -p vectorcraft-engine -p vectorcraft -p xtask --all-targets -- -D warnings` | 0 (첫 실행은 새 시험의 `ctx.style()` 오류 101 → `global_style()`로 고침) |
| `cargo test -p vectorcraft-ui-egui` | 101: 931 통과, `tests_sysclip_probe::without_its_thread_the_check_runs_in_line_again` 1개 실패(2초 대기 스레드 시험, 이 작업과 무관; 단독 재실행 3/3 통과) |
| `cargo test -p vectorcraft-engine --lib` (tests_prefs·tests_placed_document 포함 1659 통과) | 0 |
| `cargo test -p vectorcraft` (66), `cargo test -p xtask` (35; `the_tree_is_clean`, `packaging_describes_a_vector_app`) | 0 / 0 |
| `cargo xtask assets` (318 자산 귀속), `cargo xtask brands` | 0 / 0 |
| 완료 기준 1·6·11·12·18–20·25–26·29–31·33의 grep | 아래 메모 외 모두 기대값 |

### 열린 것·메모

- `cargo xtask ci`는 테스트 러너가 돌린다(이 상자에서는 엔진 통합 시험 바이너리(각 ~800 MB)를 모두 빌드하면 디스크가 차서 `--lib`만 돌렸다; `target/debug/incremental`과 오래된 통합 시험 바이너리를 지워 공간을 비웠다).
- grep 기대값과 다른 곳(모두 명세의 셈 차이, 내용은 기준대로): `ASSETS.md`의 `trowel blade`는 16이 아니라 **14**(아이콘 17개 파일 중 LICENSE.txt·README.md 행에는 Notes가 없다); `engraved dragon` 1은 `.icns` 행의 Notes를 하이픈 없이 `engraved dragon render`로 써서 맞췄다; `README.md`의 `dragon`은 **2**(한 줄은 `Snapdragon`, 257행, 손대지 않음); `"format": "vectorcraft-shortcuts"`와 `"VectorCraft Templates"`는 원래부터 각각 2줄(변경 없음).
- 카탈로그에 `VectorCraft`가 남은 source 행 중 명세 목록 밖: zh-hans의 `Keeps the document editable: VectorCraft reopens the PDF…`(`crates/pdf/src/presets.rs`, 범위 밖 크레이트).
- `.icns`는 이전 용 그림(macOS에서 `packaging/icons.sh` 필요), DMG 배경·MSI 배너·패키징 이름은 범위 밖 — 사람 검토.
- 화면 확인(`ui.screenshot`)은 하지 않았다(필수 아님).

## 검증 결과 5c659274983aac1da81fe6b45f826465906aea26

테스트 러너, 2026-10-11. 기준 커밋 `431b364` → HEAD `5c65927`(J001 `b190b6e` + J002 `5c65927`), 브랜치 `claude/project-thread-5e8gjw`. 시작 시 `git status` 깨끗함(제품 파일·factory/ 모두). 로그 디렉터리 `/tmp/claude-0/-home-claude/79152f61-3d49-5a1c-acde-dc6cabf2d3d7/scratchpad/factory-logs/`(아래 로그 이름은 그 안의 파일; 각 로그 끝 줄이 `exit=<코드>`). 디스크 한도(~40 GB)로 모든 cargo 명령을 `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only` 환경에서 돌렸다(코드·Cargo.toml·설정 파일 변경 없음).

### 공통 명령 (지정 순서)

| # | 명령 | 종료 | 결과 | 로그 |
|---|---|---|---|---|
| 1 | `cargo fmt --all -- --check` | 0 | | `verify-1-fmt.log` |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 경고 0, 3 m 15 s | `verify-2-clippy.log` |
| 3 | `cargo test -p vectorcraft-ui-egui -p vectorcraft-engine -p vectorcraft -p vectorcraft-tools` | 0 | vectorcraft 66, engine lib 1660 + 통합 15 바이너리 126, tools 290, ui-egui 933; 실패 0, ignored 0 | `verify-3-test.log` |
| 4 | `cargo xtask ci` | 0 | fmt·clippy·test --workspace(117 바이너리, 5080 통과, 실패 0)·assets(318)·brands·layers(27 crates OK)·wasm(전부 ok) — "CI summary: all 7 steps passed" | `verify-4-xtask-ci.log` |

### J001 검증 방법 항목

| 명령 | 종료 | 결과 | 로그 |
|---|---|---|---|
| `packaging/icons.sh` (다른 cargo 없음) | 0 | stderr `warning: iconutil not found…` 한 줄; 재생성물이 커밋된 파일과 바이트 동일(`git status --porcelain -- assets/app-icon` 빈 출력), `.icns` 불변 | `verify-5-j001-icons-sh.log`, `verify-5-j001-icons-sh-git-status.log` |
| `cmp …/brief/trowel-icon.svg assets/app-icon/vectorcraft.svg` | 0 | | `verify-5-j001-cmp-brief-svg.log` |
| `cmp vectorcraft.svg vectorcraft-small.svg` | 0 | | `verify-5-j001-cmp-small-svg.log` |
| `cmp vectorcraft-small.svg hicolor/scalable/apps/ai.storyteller.vectorcraft.svg` (icons.sh 전·후) | 0 / 0 | | `verify-5-j001-cmp-scalable-svg.log`, `verify-5-j001-icons-sh-cmp-scalable.log` |
| `git diff --quiet HEAD -- assets/app-icon/vectorcraft.icns` | 0 | | `verify-5-j001-icns-unchanged.log` |
| `git diff --stat HEAD -- assets/app-icon` | 0 | 빈 출력(모두 커밋됨; 명세의 「16개 파일」은 커밋 전 기준) | `verify-5-j001-icon-diffstat.log` |
| `grep -c 'viewBox="0 0 512 512"' assets/app-icon/vectorcraft.svg` | 0 | 1 | `verify-5-j001-viewbox.log` |
| 완료 기준 16 픽셀 검사 `python3 -I` (icons.sh 전·후) | 0 / 0 | `ok` | `verify-5-j001-pixel-check.log`, `verify-5-j001-pixel-check-after-icons.log` |
| `cargo fmt --all -- --check`, `cargo clippy …` | 0 / 0 | 공통 1·2 | |
| `cargo test -p vectorcraft-ui-egui` | 0 | 933 통과(공통 3과 동일; 단독 실행도 933) | `verify-3-test.log`, `verify-5-j002-test-ui-egui-all.log` |
| `cargo test -p vectorcraft-ui-egui --lib brand::` / `… chrome::tests::system_mode_hides_the_mark` (완료 기준 17) | 0 / 0 | 2 통과 / 1 통과 | `verify-5-j001-test-brand.log`, `verify-5-j001-test-system-mode.log` |
| `cargo test -p vectorcraft-engine -- tests_prefs tests_placed_document` | 0 | lib 45 통과(1615 filtered), 통합 15 바이너리 0 일치 | `verify-5-j001-test-engine-prefs-placed.log` |
| `cargo test -p xtask` | 0 | 35 통과(`the_tree_is_clean`, `packaging_describes_a_vector_app` 포함) | `verify-5-j001-test-xtask.log` |
| `cargo xtask assets` | 0 | 318 자산 귀속 | `verify-5-j001-xtask-assets.log` |
| `cargo xtask brands` | 0 | vendor 이름 없음 | `verify-5-j001-xtask-brands.log` |
| `grep -n $'\t[^\t]*VectorCraft[^\t]*\t' crates/ui-egui/src/i18n/*.tsv` | 0 | 명세 목록(크레딧 2행, 보존 이름 행, 범위 밖 `@msg` 행) + 새 크레딧 행 `Based on VectorCraft…` 10개 + zh-hans `Keeps the document editable: VectorCraft reopens the PDF…`(범위 밖 `crates/pdf`, 구현 요약 메모와 같음). 그 밖의 행 없음 | `verify-5-j001-tsv-vectorcraft-rows.log` |
| `grep -c 'engraved dragon' ASSETS.md` | 0 | 1 | `verify-5-j001-engraved-dragon.log` |
| `cargo xtask ci` | 0 | 공통 4 | `verify-4-xtask-ci.log` |

### 완료 기준 grep (추가 확인, 모두 exit 0)

- 1: `APP_NAME` — `brand.rs:10` 선언, `lib.rs:18` re-export, `native_menu.rs:41` re-export (`verify-5-j001-app-name.log`).
- 11: `Based on VectorCraft by the ArtCraft Team` — `about.rs` 1, complete 카탈로그 10개 모두 1 (`verify-5-j001-about-credit.log`).
- 12: `prefs_path_for` 1, `with_app_id` 1, `id: "vectorcraft"` 1, `id: "template"` 1, `"format": "vectorcraft-shortcuts"` 2(원래 2), `PRESETS` 1, `APP_ID: &str = "vectorcraft"` 1줄, `"VectorCraft Templates"` 2(원래 2), MCP `"name": "vectorcraft"` 1; `git diff 431b364..HEAD --stat -- packaging apps/vectorcraft/build.rs apps/vectorcraft-web crates/{format,eps,svg,color,pdf,mcp}` 빈 출력 (`verify-5-j001-identifiers.log`).
- 18–20: `trowel blade` 14(구현 요약 메모대로 LICENSE/README 행에는 Notes 없음); `assets/app-icon/README.md` `Trowel` 2, `vectorize_tile|ArtCraft drawing|drawn in ArtCraft` 0/0, `iconutil` 2; `dragon` brand.rs 0 · README.md 2(257행 Snapdragon, 348행 업스트림 URL 안내) · target-app-parity.md 0 (`verify-5-j001-readme-license.log`).
- 25·26·29·30·33: `Tokens::get` 대체값 `Brightness::default()`(theme.rs:291), `s("plate")` lib.rs:743, `state.rs:497` `Brightness::Plate`, `#[default] Plate`; `icon_on` toolbar 2 · widgets 2 · blend_options 1 · free_transform 1 · character 1 · theme 11; `prefscmds.rs:262` choice에 plate, `menus.rs:277`·`:1161`; `Quiet Plate` ui-parity 3 · target-app-parity 3 (`verify-5-j001-theme-defaults.log`).

### 판정

- 실패한 명령 없음(35개 모두 exit 0). 새 실패 0, 기존 실패 0 — 기준 `431b364`의 ui-egui 929개가 933개(J001 +3, J002 +1)로 늘었고 모두 통과. 빌더가 본 `tests_sysclip_probe::without_its_thread_the_check_runs_in_line_again`의 간헐 실패는 이번 실행(공통 3, 공통 4, 단독 실행)에서 재현되지 않았다. 기준 커밋 worktree 비교는 필요 없었다.
- 환경 메모: 1차 step 5 실행 중 feature 통합 차이(`-p` 조합마다 다른 빌드)로 `target/`이 세 벌이 되어 디스크가 0 MB가 됐다. 실행 중이던 `cargo test -p vectorcraft-engine -- tests_prefs tests_placed_document`를 중단하고 `target/debug/deps`의 옛 시험 바이너리와 `target/wasm32-unknown-unknown`(빌드 산출물만, 저장소 파일 아님)을 지운 뒤 그 명령부터 다시 돌렸다(위 표의 로그는 모두 완주한 실행의 것). 저장소 트리는 검증 전후 모두 깨끗함(`git status` 빈 출력). push는 하지 않았다(러너 규칙).

## 구현 요약 — 검토 반영 1차

검토(독립 리뷰) 지적 4건(모두 major)을 기준 `3768844`(브랜치 `claude/project-thread-5e8gjw`) 위에서 커밋 1개로 고쳤다. 모두 Quiet Plate에서만 보이던 결함이고, Illustrator 밝기 4개는 토큰 값·렌더링이 그대로다(`old_presets_keep_their_rendering`이 새 토큰 두 개까지 잠근다). 변경은 `crates/ui-egui`에만 있다.

### 바뀐 것

- **지적 2 (눌린/포커스된 egui 버튼이 검은 알약)** — `theme.rs` `Tokens`에 `pressed: Color32`를 더하고 `apply()`의 `widgets.active` 채움을 `tool_active` 대신 `pressed`로. Illustrator 프리셋 4개는 `pressed == tool_active`(base `#1d1d1d`, MediumDark `#303030`, MediumLight `#969696`, Light `#cdcdcd`), Plate는 press `#E8E6DC`. `override_text_color`(ink)가 그 위에서 읽힌다 — 시험이 WCAG 대비 ≥ 4.5:1을 단언. 사용자 도구 우물 7곳(toolbar, widgets, blend_options, free_transform, character)은 그대로 `tool_active` + `icon_on`.
- **지적 1 (`.strong()` 글자가 아이보리로 사라짐)** — 원인은 egui 0.36.2의 `strong_text_color() == widgets.active.fg_stroke.color`. `pressed`가 밝아졌으니 그 위 글리프(체크 표시·라디오 점·크기 조절 손잡이)도 `icon_on`(아이보리)일 수 없다: `apply()`의 `active.fg_stroke`를 `icon_on` → `text_strong`으로(옛 프리셋은 `icon_on == text_strong`이라 픽셀 동일; Plate는 ink). 결합을 새 시험 `strong_text_and_pressed_glyphs_are_text_strong_in_every_level`이 다섯 밝기 모두에서 잠근다(`strong_text_color == text_strong`, 패널 위 대비 ≥ 4.5, pressed 위 대비 ≥ 3). 지적이 제안한 대로 `credits.rs:287`·`:323`, `dialogs/eyedropper.rs:73`에는 `print_presets.rs`처럼 `.color(t.text_strong)`을 명시했다(이제 `.color(` 없는 `.strong()`이 크레이트에 없다). 소스 스캔 시험(선택)은 뿌리 수정으로 불필요해 넣지 않았다.
- **지적 3 (14 px egui 체크 상자가 r8로 동그라미)** — `Tokens::mark_cr()`(= `radius.min(4)`: Plate 4, 옛 프리셋 2 그대로)와 `widgets::check_box(ui, &mut bool, label)`를 더했다. 래퍼는 egui 체크 상자(`&mut bool`, `changed()`, 키보드 포커스, 접근성 역할)를 그대로 쓰되 그 호출 동안만 `ui.set_style`로 위젯 모서리를 mark 반경으로 두고 되돌린다. `ui.checkbox` 11곳(prefs_dialog `bool_row`, panels/properties ×3, dialogs/variables, export_for_screens ×2, save_options, revolve ×2, form)을 그리로 돌렸다. 지적의 첫 선택지(`widgets::check`로 교체)는 egui 체크 상자의 키보드·접근성·`RichText` 레이블(revolve.rs)을 잃어 택하지 않았고, 둘째 선택지(`apply()`에서 egui 반경 전체를 4로 캡)는 버튼·입력까지 r4가 되어 Strata 「control 8, mark 4」와 어긋나 택하지 않았다. 새 프레임 시험 `widgets::check_box_tests::an_egui_check_box_keeps_its_corners`가 Plate·Light에서 14 px 프레임의 반경이 `mark_cr()`이고 크기 절반 − 3 이하이며, 바로 다음 버튼은 다시 `cr()`임을 단언.
- **지적 4 (입력·버튼이 쉼 상태에서 테두리 없음)** — `Tokens`에 `rest_border: Color32`를 더했다(Plate edge `#C2C0B6`, 옛 프리셋 `TRANSPARENT`). `apply()`는 `TRANSPARENT`면 `Stroke::NONE`(폭 0 — 옛 프리셋의 버튼 안쪽 여백이 1 px 달라지지 않게), 아니면 1 px `rest_border`를 `widgets.inactive.bg_stroke`로. hovered = `input_border`, focused = `accent`는 그대로.

### 시험

- 바꾼 시험: `theme::tests::apply_takes_window_radius_selection_fill_and_icon_on_from_tokens` → `apply_takes_window_radius_selection_fill_pressed_and_rest_border_from_tokens`. `active.fg_stroke.color == icon_on` 단언(명세 완료 기준 24)을 지적 1·2가 결함으로 봤으므로 `active.bg_fill == pressed`, `fg_stroke == text_strong`, 라벨 대비, `inactive.bg_stroke == (1, input_border)`, Light는 `tool_active`·`Stroke::NONE` 단언으로 바꿨다. 명세 완료 기준 29의 「`theme.rs:315`는 `icon_on`」도 같은 이유로 더는 성립하지 않는다(`icon_on`은 우물 7곳에서만; `theme.rs`의 `icon_on` 등장은 9로 기준 ≥ 7 유지).
- `brightness_roundtrip`·`old_presets_keep_their_rendering`에 `pressed`·`rest_border`·`mark_cr` 단언 추가. 새 시험 2개(위).

### 검사 (종료 코드; 로그 `/tmp/claude-0/-home-claude/79152f61-3d49-5a1c-acde-dc6cabf2d3d7/scratchpad/factory-logs/J001-build-rework1-*.log`)

| 명령 | 종료 | 로그 |
|---|---|---|
| `cargo fmt --all` / `cargo fmt --all -- --check` | 0 / 0 | `J001-build-rework1-xtask.log` |
| `cargo clippy -p vectorcraft-ui-egui -p vectorcraft-engine -p vectorcraft --all-targets -- -D warnings` | 0 (첫 실행 101: 새 시험의 `show_inside` deprecated·`ui.button` must_use → 고침) | `J001-build-rework1-clippy.log` |
| `cargo test -p vectorcraft-ui-egui` | 0 — 935 통과(933 + 새 시험 2), 실패 0 | `J001-build-rework1-test-ui-egui.log` |
| `cargo xtask assets` / `cargo xtask brands` / `cargo test -p xtask`(35) | 0 / 0 / 0 | `J001-build-rework1-xtask.log` |
| `packaging/icons.sh`(iconutil 경고 한 줄), 뒤이어 `git status --porcelain -- assets/app-icon` 빈 출력, `cmp` scalable SVG | 0 / 0 | 같은 로그 |
| 브리프 SVG `cmp` ×3, `git diff --quiet HEAD -- vectorcraft.icns`, 완료 기준 16 픽셀 검사 | 모두 0 | (즉시 출력) |
| grep: `icon_on` toolbar 2 · widgets 2 · blend_options 1 · free_transform 1 · character 1 · theme 9; `.color(` 없는 `.strong()` 0; 남은 `ui.checkbox(` 0(래퍼 안 제외) | 기대값 | (즉시 출력) |

`cargo xtask ci`는 테스트 러너가 돌린다. 엔진·vectorcraft 크레이트는 손대지 않았다.

### 열린 것·메모

- 검토자의 headless 위젯 덤프(`factory-logs/review/widget_check.rs`)를 `cargo run --example`로 재현하려다 dev 프로필 빌드가 디스크를 채워(동시에 다른 세션이 같은 `target/`에 `cargo build -p vectorcraft`를 돌리고 있었다) 중단했다. 그 대신 같은 사실을 단언하는 단위·프레임 시험을 두었다. 디스크를 비우느라 `target/debug/incremental`과 05:42 이후 쓰인 `target/debug/deps` 산출물을 지웠다 — 그 시각 이후의 다른 세션 빌드 산출물도 함께 지워졌을 수 있다(cargo가 다시 만든다; 저장소 파일은 아니다).
- Plate에서 Strata 「체크 상자: 1 px `#87867F`, on = ink 채움 + 아이보리 틱」까지 맞추려면 egui 체크 상자 11곳을 `widgets::check`(ink 채움·흰 틱)로 바꾸는 후속이 필요하다. 이번에는 반경만 고쳤다(지적의 범위).
- 화면 확인(`ui.screenshot`)은 하지 않았다.
