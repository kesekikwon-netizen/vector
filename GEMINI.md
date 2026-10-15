# Rust Core Rules
1. 소유권(Ownership)·차용(Borrowing) 원칙을 준수하고 불필요한 `clone()` 및 `unsafe` 사용을 지양한다.
2. 프로덕션 코드에서 `unwrap()`/`expect()` 남발을 금지하고 `Result`/`Option`과 `?` 연산자로 에러를 안전하게 처리한다.
3. 빌드는 Cargo 기반으로 구성하며 `cargo check` 및 `cargo clippy` 경고가 없도록 작성한다.
4. 기존 주석과 문서를 보존하고 불필요한 외부 크레이트(crate) 의존성 추가를 자제한다.
