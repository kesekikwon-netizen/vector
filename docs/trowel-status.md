# Trowel 발굴 도면 CAD 개발 진행 상태 및 인수인계 문서 (Handover Document)

> **다음 AI 작업자를 위한 즉시 가이드**:
> 1. 현재 작업은 `Trowel 디자인.html` 사양에 따른 고고학 CAD 모드 개발입니다.
> 2. **J003(메인 창 레이아웃 구현 및 빌드/테스트/클리피 통과)**가 방금 완료되었습니다.
> 3. 즉시 검증: `cargo test -p vectorcraft-ui-egui --lib` 및 `cargo clippy -p vectorcraft-ui-egui -- -D warnings`
> 4. 다음으로 진행할 작업은 **J005(범례 = 레이어 연동)**입니다. 아래 세부 계획을 참고하여 바로 이어서 개발을 진행하세요.

---

## 1. 개요 및 설계 원칙

- **목표**: 매장유산 발굴 현장 실측도 및 정식 발굴조사 보고서 도판 제작을 일원화하는 Rust-native 고고학 CAD 드래프팅 도구 (VectorCraft 기반 Trowel 모드)
- **원천 사양서**: `Trowel 디자인.html` (디자인 2.1, 2026-10-11 승인안)
- **디자인 토큰**: Strata 「조용한 도판」 토큰 일치 (`ground`, `desk`, `surface`, `wash`, `press`, `oat`, `hand`, `ink`, `clay`, `action` 등)
- **소프트웨어 룰**: `AGENTS.md`의 Clean-room 준수, No panic, 모든 동작은 Command 기반, Wasm 호환성 유지.

---

## 2. 작업 로드맵 및 단계별 완료 현황 (J001 ~ J019)

| 작업 ID | 작업명 | 설명 | 상태 | 검증 기준 / 결과 |
|:---:|:---|:---|:---:|:---|
| **J001** | 이름·아이콘·색상 토큰 | `Brightness::Trowel`, Strata 조용한 도판 토큰 | **완료** | `Tokens::trowel()` 단위 테스트 통과 |
| **J002** | Z 드래그 확대 / 단축키 | Z키 및 기본 고고학 단축키 매핑 | **완료** | 단축키 테스트 통과 |
| **J013** | CI 및 품질 관문 | cargo xtask ci / clippy lints 무경고 | **완료** | CI 통과 |
| **J003** | **메인 창 레이아웃 (§3 시안)** | 리본 한 줄, 단일 컬럼 도구, 좌측 도면집 판, 우측 3단 스택 독(그리기 팔레트 / 범례·레이어 / 속성), 문서 탭, 컨트롤바, 상태줄 | **완료** | 931개 UI 테스트 통과, Clippy -D warnings 무결점 통과 |
| **J005** | **범례 = 레이어 기능 연동** | 범례 줄 1:1 레이어 매핑, ByLayer 속성, 줄 굵기 수정 시 일괄 변경, 「따로 N」과 「범례대로」 복원 | **대기 (Next)** | 단위 테스트 및 엔진 연동 |
| **J014** | **그리기 팔레트 & 빗금 채우기** | 굵기 칩(pt), 빗금(45°/135°/번갈아/없음), 「잘린 면」 원클릭 테두리+빗금, 맞닿은 조각 자동 엇갈림 | 대기 | 빗금 타일 및 벡터 출력 테스트 |
| **J002b** | Z 스프링 돋보기 | Z키를 쥐고 있는 동안만 임시 돋보기 모드 | 대기 | 키 릴리즈 시 복귀 테스트 |
| **J004** | 시작 화면 (§2 시안) | 최근 도면집 목록, 작업 순서 레일, 새 도면 견본, Strata/desca 연결 상태 | 대기 | 홈 탭 렌더링 |
| **J006** | 도면 축척 및 TM 실제 좌표계 | EPSG:5186 TM 좌표계 표시, 축척 막대, 도곽 눈금자 | 대기 | 좌표 변환 정확도 검증 |
| **J015** | 해발선 자동 생성 및 단면선 클리핑 | 10cm(0.2pt), 50cm(0.5pt, "152.5m"), 단면선 및 잘린 면 안쪽 자동 제외 | 대기 | 기하 교차/절단 알고리즘 테스트 |
| **J007** | 좌표 보존 DXF / desca 단면 가져오기 | DXF 좌표 중심 이동 방지, desca 레이어 매핑, CP949 인코딩 처리 | 대기 | DXF import fuzz 테스트 |
| **J016** | 펜 태블릿 1단계 (Windows Ink) | 톡 칠 때 손떨림 방지(3pt), 닫기 반경 8px, 캔버스 단추줄, 펜 시험 판 | 대기 | 포인터 이벤트 테스트 |
| **J008** | 밑그림 2점 맞추기 | 모눈 2점 거리 기반 축척 보정 및 오차 계산 | 대기 | 오차 수치 산출 테스트 |
| **J009** | 평면 전용 도구 | 단면선, 표고점, 유물번호, 주혈깊이, 경사표시, 폐곡선 면적·장축 계산 | 대기 | 면적 및 기하 계산 테스트 |
| **J010** | 단면·토층 화면 | 토층선, 토층 번호 자동 부여, 라이브페인트 토층칠, 토층설명표 | 대기 | 토층 채우기 연동 |
| **J011** | 유물 실측 도구 | 1:1 실측, 중심선, 반쪽 대칭 복사, 구경/기고/저경 자동 계측 | 대기 | 3점 원 피팅 및 대칭 테스트 |
| **J012** | 도판 조판 및 보고서 내보내기 | 판형(A4/4x6배판/A3), 도면 링크 배치, 표제란, 인쇄/PDF/SVG 출력 | 대기 | PDF 규격 일치 검증 |
| **J017** | 펜 태블릿 2단계 | 사이드 버튼 매핑, 지우개 팁, 팜 리젝션 | 대기 | 태블릿 하드웨어 대응 |
| **J018** | 양식 파일(템플릿) 분리/공유 | 범례 및 도면 스타일 템플릿 파일 저장/불러오기 | 대기 | 템플릿 I/O 테스트 |
| **J019** | Strata GPKG 조사 파일 직접 연동 | GPKG 읽기 전용 연결, 유구 레이어/빗금 각도 프리셋 수신 | 대기 | GPKG 파서 연동 |

---

## 3. 구현된 핵심 파일 맵 (Codebase Map)

1. **`crates/ui-egui/src/trowel.rs`**
   - `sheet_index()`: 좌측 「도면집」 판 (평면, 단면, 유물, 도판 계층 목록 및 축척, 하단 도면 메타데이터)
   - `right_dock()`: 우측 기둥 독 (3단 분할: `drawing_palette_panel`, `legend_layers_panel`, `properties_compact_panel`)
   - `doc_tabs()`: 고고학 문서 탭바 (홈, 1호 주거지 평면 1:20, A-A' 단면 1:20, 도면 12, 우측 저장 상태)
   - `status_bar()`: 고고학 상태줄 (작업 가이드 힌트, 도구, 자석, 저장 상태, TM 실좌표 X/Y, 도면 축척, 좌표계)
   - `LegendRow` 구조체: 범례 행 렌더링 매개변수 구조화 (clippy too-many-arguments 방지)

2. **`crates/ui-egui/src/ribbon.rs`**
   - `show()`: 고고학 CAD 전용 1줄 리본 툴바
   - 그룹 구성: 파일, 새 도면, 밑그림, 그리기, 도면 요소, 보기, 내보내기, 명령 찾기
   - 활성/표시/저장필요(`TileStyle::Save`, `TileStyle::Active`, `TileStyle::Shown`) 시각적 피드백 구현

3. **`crates/ui-egui/src/theme.rs`**
   - `Brightness::Trowel`: 테마 밝기 옵션에 추가
   - `Tokens::trowel()`: `Trowel 디자인.html`의 22가지 Strata 토큰 색상 완벽 구현

4. **`crates/ui-egui/src/toolbar.rs`**
   - `TROWEL_SLOTS`: 고고학 전용 단일 컬럼 도구 기둥 레이아웃 (선택, 그리기, 변형, 채색, 탐색, 고고학 묶음)

5. **`crates/ui-egui/src/chrome.rs`**
   - `trowel_control_bar()`: 고고학 컨텍스트 컨트롤 바 (도구명, 범례 선택 팝업, 추정 파선 토글, 폐곡선 계산, 필압 모드, 자석 스냅 및 허용치)
   - `app_bar()`: Trowel 모드 시 Discord 보라색 버튼 숨김 처리

6. **`crates/ui-egui/src/menus.rs`**
   - `trowel_drawing_menu()`: 상단 메뉴바의 "도면" 메뉴 ("Effect"와 "View" 사이에 삽입)
   - 새 도면, 굵기 한 번에, 밑그림 맞추기, 해발선 만들기, 도면 축척, 좌표계 명령 등록

7. **`crates/ui-egui/src/lib.rs`**
   - `app.ui.brightness == Brightness::Trowel` 분기 처리하여 표준 Illustrator UI와 Trowel UI를 매끄럽게 교체

---

## 4. 즉시 검증 및 실행 방법 (How to Verify Instantly)

### 4.1 단위 테스트
```bash
cargo test -p vectorcraft-ui-egui --lib trowel
cargo test -p vectorcraft-ui-egui --lib
```
(모든 테스트가 오류 없이 통과해야 합니다.)

### 4.2 Clippy 무경고 검사
```bash
cargo clippy -p vectorcraft-ui-egui -- -D warnings
```
(컴파일 경고가 0개여야 합니다.)

### 4.3 실제 앱 실행 및 Trowel 모드 확인
```bash
cargo run -p vectorcraft -- --control 7979
```
TCP 7979 포트로 명령을 전송하여 Trowel 테마 활성화 및 스크린샷 검증 가능:
```json
{"id":1,"method":"engine.execute","params":{"command":"theme.brightness","params":{"brightness":"trowel"}}}
{"id":2,"method":"ui.screenshot","params":{"path":"trowel_main.png"}}
```

---

## 5. 다음 AI 작업자(Next Agent)를 위한 J005 작업 가이드

현재 UI의 외형 및 레이아웃(J003)이 완성되어 있으므로, 이제 **엔진 레이어 시스템(`vectorcraft_doc::DocState`)과 범례 판(J005)**을 실제로 연결하는 작업을 진행해야 합니다.

1. **데이터 바인딩**:
   - `crates/ui-egui/src/trowel.rs`의 `legend_layers_panel`에서 하드코딩된 행을 실제 `app.session.active()`의 레이어 목록과 연결.
   - 레이어의 메타데이터(선 굵기, ByLayer 여부, 선종, 빗금 패턴, 레이어 색상, 잠금, 가시성) 조회/수정 연동.
2. **범례 줄 선택 동작**:
   - 범례 줄 클릭 시 해당 레이어를 활성 레이어로 설정 (`app.run("layer.select", ...)`)
   - 도구의 현재 스트로크 굵기 및 선종을 해당 레이어의 설정값으로 자동 변경 (`stroke.set`)
3. **일괄 굵기 변경 (`drawing.weightsAtOnce`)**:
   - 배율(예: ×0.8) 또는 지정 단계로 문서 내 모든 레이어/오브젝트 선 굵기를 일괄 변경하는 엔진 커맨드 연결
4. **테스트 추가**:
   - 범례 줄 선택 시 레이어 및 스트로크 속성이 연동되는 통합 테스트 작성 (`crates/ui-egui/src/trowel.rs`의 `#[cfg(test)]`)
