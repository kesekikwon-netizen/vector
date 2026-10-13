---
name: xtask-ci
description: >-
  Runs VectorCraft's quality gates and CI checks (cargo xtask ci) before committing code.
  Includes formatting, clippy with no-panic lints, unit/integration tests, asset attribution,
  brand verification, crate layering, and WASM target checks.
---

# VectorCraft CI & Quality Gates

This skill executes all quality gates required before committing any change to VectorCraft.

## Verification Workflow

1. Execute the unified CI command:
   ```bash
   cargo xtask ci
   ```
2. If any step fails, diagnose and address:
   - **Formatting**: Run `cargo fmt` to apply standard formatting.
   - **Clippy**: Ensure no `unwrap()`, `expect()`, `panic!`, `unreachable!`, `todo!`, or `unimplemented!` are used outside tests.
   - **Layering**: Run `cargo xtask layers` to check crate dependency boundaries (L0 through L6).
   - **Assets**: Run `cargo xtask assets` to verify all asset files are listed in `ASSETS.md`.
   - **Brands**: Run `cargo xtask brands` to verify no prohibited vendor product names exist in user-facing text.
   - **Wasm**: Run `cargo xtask wasm` to ensure `wasm32-unknown-unknown` builds pass.
