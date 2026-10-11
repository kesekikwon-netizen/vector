# J013 — 깃허브 액션 CI: 작업 브랜치마다 `cargo xtask ci`와 윈도우 빌드

상태: 승인 대기 (AI 검토 통과, 사용자 승인과 저장소에서 액션 켜기 필요) · 기준 커밋 `dd6be567123bc4fcde8f81f81a32f876815873cd` · 커밋 1개 (`J013: …`) · **CI 변경이라 사람 검토 필수**

## 목표

사용자 지시(2026-10-11 07:34Z 「깃허브액션을 실행하며 진행하라」). 이 저장소의 워크플로 6개(affinity-corpus, freebsd, packaging-lint, release, windows-arm64, windows7)는 모두 `main`·`release` 푸시, PR, 일정, 손으로 실행할 때만 돌고, 어느 것도 이 저장소의 관문 `cargo xtask ci`를 돌리지 않는다. 그래서 작업 브랜치에 푸시할 때마다 도는 워크플로 하나를 더한다.

- 리눅스: `cargo xtask ci` 전체(fmt, clippy `-D warnings`, `cargo test --workspace`, assets, brands, layers, wasm 검사).
- 윈도우 x64: `vectorcraft`·`vectorcraft-cli` 릴리스 빌드, CLI를 한 번 실행, 두 exe를 14일 동안 내려받을 수 있게 남김(사용자가 윈도우 PC에서 바로 받아 볼 수 있게).

## 완료 기준

1. `.github/workflows/trowel-ci.yml`이 있고 `actionlint`(packaging-lint와 같은 1.7.12)가 이 파일과 기존 워크플로 전부에서 exit 0.
2. 트리거는 `claude/**` 브랜치 푸시, `pull_request`, `workflow_dispatch`뿐이다. `main`·`release`·태그·일정 트리거 없음.
3. 권한은 `contents: read` 하나. `persist-credentials: false`. 비밀값(`secrets.*`)을 쓰지 않는다. 푸시·배포·릴리스·PR 댓글을 하지 않는다.
4. 액션은 기존 워크플로와 같은 판을 쓴다: `actions/checkout@v7`, `dtolnay/rust-toolchain@686976e1… # stable`, `Swatinem/rust-cache@6323deb1… # v2`, `actions/upload-artifact@v7`.
5. 같은 브랜치의 앞선 실행은 새 푸시가 오면 취소된다(`concurrency`, `cancel-in-progress: true`).
6. 제품 코드·`Cargo.toml`·`Cargo.lock`·기존 워크플로 6개는 바뀌지 않는다. 확인: `git diff --stat dd6be56 -- . ':!factory' ':!.github/workflows/trowel-ci.yml'` 출력 없음.
7. (검토 1차에서 추가) 툴체인은 로컬 관문과 같은 `1.97.0`으로 고정한다. 새 stable의 린트만으로 CI가 빨개지지 않게.
8. (검토 1차에서 추가) 윈도우 빌드는 `packaging/windows/package.ps1`과 같다: `--locked`, `--target x86_64-pc-windows-msvc`, 정적 CRT(`+crt-static`, VC++ 재배포 패키지 없이 실행), `VECTORCRAFT_REQUIRE_WINRES=1`(아이콘·버전 정보가 빠지면 실패).
9. (검토 1차에서 추가) 이 저장소의 `claude/` 브랜치에서 연 PR은 push 실행과 겹치므로 pull_request 실행을 건너뛴다. fork PR과 다른 브랜치의 PR은 돈다.

## 변경 범위

- `J013:` 커밋: 새 파일 `.github/workflows/trowel-ci.yml`과 이 기록 파일.
- 백로그 표의 J013 줄은 뒤따르는 `factory:` 커밋(백로그를 디자인 2.0 순서로 재편)에 넣는다. 작업 ID 하나에 커밋 하나.
- **바꾸지 않는 것**: 기존 워크플로 6개, `xtask`, 빌드 설정, 의존성, 패키징.

## 검증 방법

```
cd /home/claude/vector
$SCRATCH/tools/actionlint/actionlint -color .github/workflows/trowel-ci.yml
$SCRATCH/tools/actionlint/actionlint .github/workflows/*.yml
git diff --stat dd6be56 -- . ':!factory' ':!.github/workflows/trowel-ci.yml'
```

실제 실행 결과(러너에서 통과하는지)는 저장소 설정에서 액션이 켜진 뒤 첫 푸시에서만 확인할 수 있다. 그 전까지 「러너에서 통과」는 **미실행**으로 남긴다.

## 범위 밖

- 윈도우에서 `cargo test` (첫 실행 결과를 보고 따로 판단), macOS 빌드, 설치 파일(MSI)·서명, 릴리스.
- 기존 워크플로의 일정 실행 끄기(아래 위험 참고): 기존 워크플로를 고치지 않고 사용자에게 알린다.
- 툴체인을 1.99(현재 stable)로 올리기: 로컬도 함께 올리고 새 린트를 고치는 별도 작업.

## 위험·사람 검토 필요

- **CI 변경**이라 사용자 승인이 필요하다(공장 규칙). 승인 전에도 작업 브랜치에서 돌지만 병합·배포와는 상관없다.
- 저장소 설정에서 액션이 꺼져 있다(2026-10-11 07:40Z 확인: 워크플로 목록 0개, `packaging-lint.yml` 실행 요청이 404). 사용자가 https://github.com/kesekikwon-netizen/vector/settings/actions 에서 켜야 돈다.
- 액션을 켜면 기본 브랜치 `claude/project-thread-yam40j`에 있는 일정 워크플로 둘도 함께 켜진다: `windows-arm64.yml`(매주 월 04:41 UTC), `windows7.yml`(매주 월 05:23 UTC). 실패하면 메일이 간다. 원치 않으면 Actions 탭에서 두 워크플로를 「Disable workflow」.
- PR을 열면 경로 필터에 따라 기존 워크플로도 돈다: packaging-lint(J001이 `assets/app-icon`을 고쳤으므로), affinity-corpus(코퍼스 다운로드 필수), `Cargo.toml`이 바뀌면 freebsd(최대 300+150분)·windows-arm64.
- `workflow_dispatch`(손으로 실행)는 이 파일이 기본 브랜치에도 있어야 Actions 탭에 버튼이 생긴다. 지금은 푸시 실행만 쓴다.
- 디버그 정보를 끈 시험 빌드(`CARGO_PROFILE_DEV_DEBUG: 0`, test 프로필에도 적용됨을 검토자가 확인)와 러너 공간 확보 단계는 디스크 부족을 막으려는 것이다. 이 컨테이너의 디버그 target은 23 GB였다.
- 저장소가 공개라 표준 러너(ubuntu-latest, windows-latest)는 무료다. `release.yml`의 macOS job은 큰 러너 `macos-15-xlarge`라 공개 저장소도 유료지만 `release` 푸시나 손 실행 때만 돈다.
- 남은 위험(낮음): `crates/pdf/tests/system_fonts.rs:42-46`의 `expect`는 러너에 설치된 글꼴에 기대므로 리눅스 러너에서 처음 돌 때 실패할 수 있다.

## 기준 커밋

`dd6be567123bc4fcde8f81f81a32f876815873cd` (`git rev-parse HEAD`, 2026-10-11)

## 빌드 (build)

2026-10-11, 기준 `dd6be56`. 손댄 파일: `.github/workflows/trowel-ci.yml`(새로), `factory/jobs/J013-github-actions-ci.md`(새로). 완료 기준 1–6을 위 내용대로 구현했다. 스펙에서 벗어난 점 없음.

## 검증 (verify)

| 명령 | 종료 코드 | 결과 |
|---|---|---|
| `actionlint -color .github/workflows/trowel-ci.yml` | 0 | 지적 없음 |
| `actionlint .github/workflows/*.yml` | 0 | 7개 파일 지적 없음 |
| `git diff --stat dd6be56 -- . ':!factory' ':!.github/workflows/trowel-ci.yml'` | 0 | 출력 없음 (제품 코드·기존 워크플로 불변) |
| 러너에서 실제 실행 | — | **미실행**: 저장소에서 액션이 꺼져 있음 |

## 검토 (review) 1차 — 코드·보안 검토자, 판정 rework

완료 기준 1–6은 모두 충족(검토자가 actionlint를 다시 돌리고 `git ls-remote`로 액션 SHA를 확인). 지적:

1. **major** 툴체인이 고정되지 않았다. 로컬은 1.97.0, 현재 stable은 1.99.0이라 러너의 `clippy -D warnings`가 새 린트로 첫 실행부터 실패할 수 있다.
2. **minor** 윈도우 빌드가 `package.ps1`과 달라 VC++ 재배포 패키지가 없는 PC에서 exe가 안 뜬다(`+crt-static` 없음). `--locked`, `VECTORCRAFT_REQUIRE_WINRES=1`도 없다.
3. **minor** 이 기록은 「백로그 표 한 줄」이라 했지만 백로그 전체를 재편했고(작업 ID 하나에 커밋 하나 위반), 백로그가 없는 파일 `jobs/J003-main-window.md`를 가리킨다.
4. **minor** PR을 열면 push와 pull_request로 같은 커밋이 두 번 돈다.
5. **nit** `workflow_dispatch`는 파일이 기본 브랜치에 있어야 쓸 수 있다.

리눅스·윈도우 job의 환경(시스템 라이브러리, DISPLAY, GPU, 네트워크, exe 경로)과 보안(표현식 주입, fork PR, 캐시 격리)은 문제 없음. 범위 밖 기존 문제 하나: CLAUDE.md는 `cargo xtask ci`가 cleanroom 검사를 돈다고 하지만 xtask에 그런 명령이 없다.

## 재작업 (rework) 1차

| 지적 | 반영 |
|---|---|
| 1 | 두 job의 `dtolnay/rust-toolchain`(SHA 그대로)에 `toolchain: "1.97.0"`. affinity-corpus.yml은 MSRV `1.95`를 쓰지만 로컬 관문이 통과한 판에 맞췄다. 완료 기준 7 |
| 2 | 윈도우 빌드를 `cargo build --release --locked -p vectorcraft -p vectorcraft-cli --target x86_64-pc-windows-msvc`로, step env에 `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS: -C target-feature=+crt-static`, `VECTORCRAFT_REQUIRE_WINRES: "1"`. CLI 실행·업로드 경로를 `target/x86_64-pc-windows-msvc/release/`로. 완료 기준 8 |
| 3 | 커밋을 나눴다(변경 범위 참고). 백로그의 없는 파일 참조는 `factory:` 커밋에서 지운다 |
| 4 | 두 job에 `if: github.event_name != 'pull_request' \|\| github.event.pull_request.head.repo.full_name != github.repository \|\| !startsWith(github.head_ref, 'claude/')`. 검토자 제안에 셋째 조건을 더해 같은 저장소의 `claude/`가 아닌 브랜치 PR은 계속 돈다. 완료 기준 9 |
| 5 | 고치지 않고 위험 절에 적었다 |

재검증:

| 명령 | 종료 코드 | 결과 |
|---|---|---|
| `actionlint -color .github/workflows/trowel-ci.yml` | 0 | 지적 없음 |
| `actionlint .github/workflows/*.yml` | 0 | 7개 파일 지적 없음 |
| `git diff --stat dd6be56 -- . ':!factory' ':!.github/workflows/trowel-ci.yml'` | 0 | 출력 없음 |
| shellcheck (`run:` 블록 셸 검사) | — | **미실행**: 이 상자에 shellcheck가 없다. CI의 packaging-lint는 설치해서 돈다 |
| 러너에서 실제 실행 | — | **미실행**: 저장소에서 액션이 꺼져 있음 |

## 재검토 (review) 2차 — 같은 검토자, 판정 pass

- 지적 1: 두 job에 `toolchain: "1.97.0"`. 검토자가 SHA `686976e`의 action.yml을 받아 `toolchain` 입력이 기본값 stable을 덮어쓰고 `rustup default`를 실행함을 확인했다. 저장소에 rust-toolchain 파일이 없어 cargo는 1.97.0을 쓴다.
- 지적 2: 윈도우 빌드가 `package.ps1:60-66`과 같다. 두 액션 모두 전역 `RUSTFLAGS`를 설정하지 않아 대상별 RUSTFLAGS(`+crt-static`)가 먹는다. `cargo tree --locked --offline` exit 0. `vectorcraft.ico`는 PNG 항목 9개가 모두 파일 범위 안이라 `VECTORCRAFT_REQUIRE_WINRES=1`로도 빌드된다(release.yml이 같은 설정으로 빌드하던 구성).
- 지적 4: `if` 식을 다섯 경우로 따져 모두 의도대로(push·손 실행·fork PR·같은 저장소의 다른 브랜치 PR은 실행, 같은 저장소의 `claude/` PR은 push 실행과 겹쳐 건너뜀).
- 새 문제 없음. 남긴 nit 둘(고치지 않음): `claude/` PR에서는 PR 병합 결과(head+base)를 따로 시험하지 않는다. `startsWith`는 대소문자를 가리지 않아 `Claude/…` 브랜치에서만 어긋날 수 있다.
- 재검증: actionlint 1.7.12 exit 0(이 파일, 7개 전부), `git diff --stat` 출력 없음. shellcheck는 미실행.

## 승인 (approval)

- AI 검토: 통과(2차).
- 사용자 승인: **대기**. CI 변경이라 사람 검토가 필요하다. 승인 뒤 코드가 바뀌면 다시 검증하고 다시 승인받는다.
- 러너에서 실제 실행: **미실행**. 저장소 설정에서 액션을 켠 뒤 첫 푸시에서 확인해 이 기록에 덧붙인다.
- 되돌리기: `git revert <J013 커밋>` 또는 `.github/workflows/trowel-ci.yml` 삭제. 제품 코드는 바뀌지 않았다.
