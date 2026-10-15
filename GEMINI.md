# Rust Core Rules
1. 소유권(Ownership)·차용(Borrowing) 원칙을 준수하고 불필요한 `clone()` 및 `unsafe` 사용을 지양한다.
2. 프로덕션 코드에서 `unwrap()`/`expect()` 남발을 금지하고 `Result`/`Option`과 `?` 연산자로 에러를 안전하게 처리한다.
3. 빌드는 Cargo 기반으로 구성하며 `cargo check` 및 `cargo clippy` 경고가 없도록 작성한다.
4. 기존 주석과 문서를 보존하고 불필요한 외부 크레이트(crate) 의존성 추가를 자제한다.

# Trowel & Desktop UI/UX Guardrails
1. **실제 실행 파일 빌드 및 스크린샷 육안 검증 필수**: UI/UX 수정 후에는 헤드리스 테스트만 통과했다고 보고하지 말고, 반드시 실행 중인 `vectorcraft.exe`를 먼저 종료한 뒤 `cargo build -p vectorcraft`로 실제 실행 파일을 다시 빌드하고, `--control <port>`로 띄워 `ui.screenshot` 이미지를 직접 눈으로 확인(`view_file`)한 뒤 완료를 보고한다.
2. **조용한 도판(`Brightness::Plate`) 테마 및 `ui.json` 동기화**: Windows 설정 파일은 `%APPDATA%\VectorCraft\ui.json`에 저장되며 종료 시점의 메모리 상태가 다시 기록된다. 또한 `prefs_dialog::apply_runtime`이 매 프레임 `app.session.prefs.ui_brightness` 값을 `app.ui.brightness`에 덮어쓰므로, Trowel 모드에서는 두 값 모두 `"plate"` / `Brightness::Plate`로 유지되어야 한다.
3. **커스텀 페인팅 `egui::Panel` 크기 조절 규칙**: `ui.painter()`로 직접 그리는 좌우 패널(`egui::Panel::left` / `Panel::right`)에 `.resizable(true)`를 켤 때는 내부에서 반드시 `ui.set_min_width(r.width())`를 호출하여 패널이 최소 폭으로 찌부러지지 않게 한다.
4. **작업 레이어 및 `Delete` 키 일관성**:
   - 새 도면(`trowel.newPlan`, `newSection`, `newFinds`, `newSitePlan`) 생성 시 `legend.addGroup`으로 13~15개 레이어를 강제 추가하지 않고, 기본 작업 레이어 1개로만 시작하여 사용자가 필요한 규칙을 직접 추가하게 한다.
   - `Delete` / `Backspace` 키는 잠긴 레이어나 전체 선택된 레이어를 삭제할 때도 동작해야 하며(전체 레이어 삭제 시 빈 레이어 1개를 먼저 생성 후 삭제), 색상 선택기/스와치 창이 열려 있을 때는 색 없음(`paint.none`)으로 동작해야 한다.
   - Windows 데스크톱 바이너리(`apps/vectorcraft/src/main.rs`)는 실행 시 검은 터미널 창이 뜨지 않도록 `#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]`를 유지한다.
5. **`TrowelTheme` 색상 토큰 강제 (`trowel_code_takes_colours_from_tokens`)**:
   - `crates/ui-egui/src/trowel/` 하위 모든 UI 파일(`theme.rs` 및 `#[cfg(test)]` 블록 제외)에서는 `Color32::from_rgb`, `Color32::from_rgba_unmultiplied`, `Color32::BLACK`, `Color32::WHITE` 등 리터럴 색상을 직접 호출하지 않고 반드시 `TrowelTheme` 토큰(`t.ink`, `t.paper`, `t.accent` 등)만 사용한다.
6. **발굴 조사 보고서 실무 표준(소스판 1클릭 도구) 보존**:
   - 사용자가 외부 소스판 파일에서 복사·붙여넣기하지 않도록 실무 7단계 선 굵기(`0.15 / 0.2 / 0.3 / 0.4(파선) / 0.5 / 0.6 / 0.8 pt`)와 현장 실무 표준 12종 프리셋(`legend.addFieldPresets`, 윤고딕 `6p`·`7p` 기준)을 유지한다.
   - 상단 리본 팝오버(`축척` `1/10~1/200`, `단면선` `A–A′`/`B–B′` 적색 기준점 `■` + 시선 화살표 `∠` + `┣ 93.0m` 기준점, `방위` 회전 각도, `표고` 단면 아래 채움 `fillBelowProfile`·우측 주기 `RightEnd`·적색 틱 `redDatumTick`, `범례표` `교란`·`조사경계` 스탬프) 및 로마 숫자(`I~XX`) 토층 기호·`7p` 일렬 토층 설명 블록이 항상 1클릭으로 동작해야 한다.
