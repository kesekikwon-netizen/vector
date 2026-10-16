상태: verify

# J020 (디자인 3.0 D0): PC에만 있던 변경을 저장소에 올리기

## 명세 (spec)
- 기준 커밋: `acf4e5d53dd129c2f1057aa37719efc682396c34` (origin/main, J014, 2026-10-15)
- 작업 브랜치: `claude/project-thread-ams9bp` (origin/main에서 새로 시작. 예전 끝 커밋 `dd3cfde`는 main에 다른 형태로 들어간 J003 시안이며 `origin/claude/project-thread-yam40j`에 그대로 남아 있음)
- 근거: 디자인 3.0 (https://claude.ai/artifact/GmVPKvYKq4ADAe5ZbmZLdc) 11장 D0, 사용자 「진행시켜」 2026-10-16 07:54Z

### 목표
사용자 PC(young, `D:\vector`, main 위)의 커밋하지 않은 변경 18개 파일을 그대로 작업 브랜치에 올려, 이후 작업(D1~D7)이 사용자가 실제로 보고 있는 앱과 같은 코드에서 시작하게 한다.

### 완료 기준
1. 저장소의 18개 파일이 PC 파일과 줄바꿈(CRLF→LF)만 빼고 같다. 확인: `git diff --ignore-cr-at-eol --stat`(PC) = `git diff --stat`(브랜치), 863줄 추가 · 322줄 삭제.
2. `cargo build -p vectorcraft`가 성공한다.
3. `cargo fmt --all -- --check`가 통과한다.
4. `cargo test -p vectorcraft-ui-egui trowel`과 `cargo test -p vectorcraft-engine`이 통과한다 (기존 실패가 있으면 그 목록을 따로 적는다).
5. GitHub 「Trowel CI」가 이 커밋에서 통과한다.

### 변경 범위
PC에서 받은 그대로(내용을 고치지 않음): `crates/engine/src/cmd/{buildcmds,elevation,hatch,trowel_plan}.rs`, `crates/engine/src/lib.rs`, `crates/pathops/src/{elevation,hatch}.rs`, `crates/text/src/fontdb.rs`, `crates/tools/src/builder.rs`, `crates/ui-egui/src/{chrome,widgets}.rs`, `crates/ui-egui/src/i18n/ko.tsv`, `crates/ui-egui/src/trowel/{drawings,mod,palette,props,ribbon,tests}.rs`.
PC 폴더에는 쓰지 않는다(읽기만).

### PC 변경의 내용 (읽은 것)
- 단면선·표고 명령이 리본이 보내던 인자 이름(`start/end`, `elevation_m`)도 받음 → 디자인 3.0이 찾은 고장 2건이 엔진 쪽에서 풀림.
- 한 번 눌러 넣는 요소가 화면 가운데 근처에 조금씩 비켜 놓임(`trowel::place_symbol_at`), 고정 좌표에 겹치던 고장이 줄어듦.
- 해발 기준점 모양이 「검은 보조선 + 붉은 막대 + 6pt 글자」로 바뀜, 단면 표시 화살표 모양 바뀜.
- `elevation.create`가 선택이 없을 때도 기준선을 만들고, 단면 윤곽을 한 줄로 정리함(`pathops::elevation`).
- 칠통으로 한 도형을 칠하면 라이브 페인트 묶음 없이 그 도형에 빗금(`tools::builder`, `cmd::hatch`), 세션에 기본 빗금 값.
- 굵기 칩 9개 한 줄(0.1~1.0, 0.4는 파선), 한글 대체 글꼴 목록에 윤고딕 이름 추가(글꼴 파일은 넣지 않음), 속성 크기 mm 표시, 조절줄 X·Y·W·H 좁은 칸.

### 위험·사람 검토 필요
- ◆ 수치 알고리즘: `pathops/src/elevation.rs` 해발선 범위·자르기 변경, `cmd/elevation.rs` 기준선 자동 생성. 사용자가 PC에서 만든 것이라 내용은 그대로 두고, 검토만 한다.
- 글꼴 이름(윤고딕)은 설치된 글꼴을 찾는 이름일 뿐 파일을 넣지 않으므로 자산 규칙에 걸리지 않는다.

### 범위 밖
디자인 3.0의 화면 정리(D2 이후). 여기서는 고치지 않는다.
